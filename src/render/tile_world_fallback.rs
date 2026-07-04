//! Raster [`TileMarker`] worlds to a single sprite when chunk tilemaps are absent (default build has no `bevy_tilemap_adapter`).
//!
//! Without this, generated tiles have no mesh/material and the main camera shows nothing.
//!
//! **Performance:** `tile_world_fallback_rasterize` repaints **128×128 dirty chunks** per frame
//! ([`TileWorldFallbackChunkGrid`], `RASTER_CHUNK_TILES`) when [`TileWorldFallbackRasterDirty`]
//! bumps — `O(changed_chunks × chunk_area)` not `O(world)` collect + full texture clear.
//! **Fire tint:** after terrain + roads, applies [`crate::gui::map_tile_raster::apply_shared_fire_heat_to_rgba`]
//! from [`crate::render::SharedOverlayFieldBuffers`] (same source as world preview); raster systems run in
//! [`TileWorldFallbackAfterFireExtract`] **after** [`crate::render::FireVisualFrameSet::BuildProfiles`].
//!
//! **Camera:** [`MainWorldCamera`] is centered on `(params.width/2, params.height/2)` in tile space; CLI `--test`
//! modes apply extra orthographic scale so the overworld fills more of the window.

use std::collections::HashMap;

use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use bevy_egui::{egui, EguiContexts};

use crate::engine::{ActiveTestScene, BaseState, EngineLaunchArgs, TestScene};
use crate::gui::{MapViewInstances, MapViewPresentationStates, MapViewState};
use crate::gui::{map_toolbar, map_toolbar_minimap_zoom, MapToolbarConfig};
use crate::gui::{
    camera_translation, compute_map_fit_strict, resolve_minimap_texture_source,
    ActiveMapViewInput, InputBindings, MapCameraDesired, MapCameraDesiredRes, MapPresentationDiagnostics,
    MapTextureSource, MapViewInstanceId, MinimapInteractionBuffer, MinimapPresentationSource,
    MinimapShellState, ResolvedMapViewFrames, SimulationMapViewport, ViewAuthoritySystemSet,
    ViewId, ViewManager, native_minimap_window_supported, paint_tactical_viewport_frame_on_minimap,
    MAP_PANEL_INSET_PX, view_surface_screen_to_world,
};
use crate::render::MinimapRenderTargetRegistry;
use crate::gui::std_floating;
use crate::gui::hud::cached_egui_texture::HudEguiTextureCache;
use crate::gui::hud::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics, FrameBudgetTimer};
use crate::gui::hud::layout_store::HudLayoutStore;
use crate::gui::hud::pending_hud_layout_commit::PendingHudLayoutCommit;
use crate::gui::hud::shell_framework::{capture_shell_layout, HudWidgetId};
use crate::gui::hud::{ProductShellDiagnostics, ViewportRectSanity, ViewportRectSource};
use crate::gui::MainWorldCamera;
use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::{ensure_viewport_camera_initialized, fit_viewport_to_map};
use crate::gui::preview_partial_min_interval_from_hz;
use crate::render::{FireVisualFrameSet, SharedOverlayFieldBuffers};
use crate::render::FireAtmosphereAggregate;
use crate::gui::style::UiPalette;
use crate::gui::editor::map_editor::MapEditorRoadMarkerV1;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::{
    Height, Moisture, Temperature, TerrainType, TileMarker, WorldGenParams,
};
use crate::render::terrain_render_authority::TerrainRenderAuthority;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WorldGenDenseTerrainCache};
use crate::terrain::material::MaterialRegistry;

/// Marks the full-map fallback sprite entity.
#[derive(Component)]
pub struct TileWorldFallbackSprite;

#[derive(SystemParam)]
struct TileFallbackGpuStampInputs<'w> {
    authority: Res<'w, TerrainRenderAuthority>,
    gpu_stamps: ResMut<'w, crate::gui::map_tile_atlas_stamp::TerrainGpuStampIndices>,
}

#[derive(Resource, Default)]
pub struct TileWorldFallbackState {
    pub sprite_entity: Option<Entity>,
    /// Main world / simulation map sprite (layer mask from [`crate::gui::MapViewPresentationStates::simulation_map`]).
    pub image: Handle<Image>,
    /// Minimap egui texture only (layer mask from [`crate::gui::MapViewInstances::minimap`]).
    pub minimap_image: Handle<Image>,
    pub last_w: u32,
    pub last_h: u32,
    #[cfg(feature = "bevy_tilemap_adapter")]
    pub suppressed: bool,
}

#[derive(Resource)]
pub struct SimMinimapUiState {
    pub open: bool,
}

impl Default for SimMinimapUiState {
    fn default() -> Self {
        Self { open: true }
    }
}

/// Bump this whenever overworld fallback pixels must be recomputed (see `mark_tile_world_fallback_dirty_on_changes`).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TileWorldFallbackRasterDirty {
    revision: u64,
}

impl TileWorldFallbackRasterDirty {
    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Public hook for systems that mutate tiles/roads without triggering `Changed<>` filters the same frame.
    pub fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

/// Dirty-region tile size for overworld CPU raster (tiles per axis).
pub const RASTER_CHUNK_TILES: u32 = 128;

/// Tracks which 128×128 tile regions need CPU repaint (see [`RASTER_CHUNK_TILES`]).
#[derive(Resource, Debug, Clone)]
pub struct TileWorldFallbackChunkGrid {
    chunks_x: u32,
    chunks_z: u32,
    dirty: Vec<bool>,
}

impl Default for TileWorldFallbackChunkGrid {
    fn default() -> Self {
        Self {
            chunks_x: 0,
            chunks_z: 0,
            dirty: Vec::new(),
        }
    }
}

impl TileWorldFallbackChunkGrid {
    pub fn resize_for_world(&mut self, tex_w: u32, tex_h: u32) {
        let chunks_x = tex_w.div_ceil(RASTER_CHUNK_TILES).max(1);
        let chunks_z = tex_h.div_ceil(RASTER_CHUNK_TILES).max(1);
        let n = (chunks_x * chunks_z) as usize;
        if self.chunks_x != chunks_x || self.chunks_z != chunks_z || self.dirty.len() != n {
            self.chunks_x = chunks_x;
            self.chunks_z = chunks_z;
            self.dirty.resize(n, false);
        }
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty.fill(true);
    }

    pub fn mark_tile(&mut self, tx: u32, tz: u32) {
        self.mark_chunk(tx / RASTER_CHUNK_TILES, tz / RASTER_CHUNK_TILES);
    }

    pub fn mark_chunk(&mut self, cx: u32, cz: u32) {
        if cx >= self.chunks_x || cz >= self.chunks_z {
            return;
        }
        let i = (cz * self.chunks_x + cx) as usize;
        if let Some(slot) = self.dirty.get_mut(i) {
            *slot = true;
        }
    }

    pub fn has_dirty(&self) -> bool {
        self.dirty.iter().any(|&d| d)
    }

    /// Take up to `budget` dirty chunk coords and clear their dirty flags.
    pub fn take_dirty_chunks(&mut self, budget: usize) -> Vec<(u32, u32)> {
        let mut out = Vec::with_capacity(budget.min(self.dirty.len()));
        for cz in 0..self.chunks_z {
            for cx in 0..self.chunks_x {
                if out.len() >= budget {
                    return out;
                }
                let i = (cz * self.chunks_x + cx) as usize;
                if self.dirty.get(i) == Some(&true) {
                    self.dirty[i] = false;
                    out.push((cx, cz));
                }
            }
        }
        out
    }

    #[must_use]
    pub fn chunk_pixel_bounds(
        cx: u32,
        cz: u32,
        tex_w: u32,
        tex_h: u32,
    ) -> (usize, usize, usize, usize) {
        let x0 = (cx * RASTER_CHUNK_TILES) as usize;
        let y0 = (cz * RASTER_CHUNK_TILES) as usize;
        let x1 = ((cx + 1) * RASTER_CHUNK_TILES).min(tex_w) as usize;
        let y1 = ((cz + 1) * RASTER_CHUNK_TILES).min(tex_h) as usize;
        (x0, y0, x1, y1)
    }
}

/// CPU repaint of `minimap_image` for the **effects lane** (`SharedCpuRaster` opt-in) or when GPU compositor is off.
/// Skipped on the default simulation HUD path (GPU compositor + `SharedRenderTargetImage`).
/// See [`plan_visual_perf_production_v1.md`](../../dev/plan_visual_perf_production_v1.md) P1-A.
#[inline]
fn tile_fallback_cpu_minimap_raster_needed(
    shell: Option<&MinimapShellState>,
    registry: Option<&MinimapRenderTargetRegistry>,
    fallback: &TileWorldFallbackState,
) -> bool {
    let (Some(shell), Some(registry)) = (shell, registry) else {
        return true;
    };
    !matches!(
        resolve_minimap_texture_source(shell, fallback, registry),
        MapTextureSource::GpuRenderTarget(_)
    )
}

fn refresh_tile_raster_budget(
    base: Res<State<crate::engine::states::BaseState>>,
    launch: Option<Res<crate::engine::launch_args::EngineLaunchArgs>>,
    test_scene: Option<Res<crate::engine::ActiveTestScene>>,
    params: Res<WorldGenParams>,
    budgets: Res<crate::gui::VisualBudgetSettings>,
    mut raster_budget: ResMut<crate::render::TileRasterBudget>,
    mut fire_cadence: ResMut<crate::render::FireExtractCadence>,
) {
    *raster_budget =
        crate::render::TileRasterBudget::from_world_and_settings(params.width, params.height, &budgets);
    *fire_cadence = crate::render::FireExtractCadence::from(&*budgets);
    let harness = launch.as_deref().is_some_and(|l| l.test_mode()) || test_scene.is_some();
    if harness || matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        crate::render::FireExtractCadence::clamp_for_world(
            &mut fire_cadence,
            params.width.max(1),
            params.height.max(1),
            harness,
        );
    }
}

fn rebuild_tile_world_fallback_index(
    index: &mut TileWorldFallbackTileIndex,
    revision: u64,
    tex_w: usize,
    tex_h: usize,
    queries: &mut ParamSet<(
        Query<(
            &Transform,
            &TerrainType,
            &Height,
            &Moisture,
            &Temperature,
        ), With<TileMarker>>,
        Query<&MapEditorRoadMarkerV1>,
        Query<(&Chunk, &ChunkCellMatrix, Option<&crate::systems::fire::ChunkFireOverlay>)>,
    )>,
) {
    if index.revision == Some(revision) {
        return;
    }
    index.revision = Some(revision);
    index.tiles_by_chunk.clear();
    index.roads_by_chunk.clear();
    let chunk_tiles = RASTER_CHUNK_TILES as usize;
    if queries.p0().is_empty() {
        for (chunk, matrix, _) in queries.p2().iter() {
            let sx = matrix.size.x as usize;
            let sy = matrix.size.y as usize;
            if sx == 0 || sy == 0 {
                continue;
            }
            for y in 0..sy {
                for x in 0..sx {
                    let wx = chunk.coord.x as isize * sx as isize + x as isize;
                    let wy = chunk.coord.y as isize * sy as isize + y as isize;
                    if wx < 0 || wy < 0 {
                        continue;
                    }
                    let xu = wx as usize;
                    let yu = wy as usize;
                    if xu >= tex_w || yu >= tex_h {
                        continue;
                    }
                    let i = matrix.idx(x as u32, y as u32);
                    let cx = (xu / chunk_tiles) as u32;
                    let cz = (yu / chunk_tiles) as u32;
                    index.tiles_by_chunk.entry((cx, cz)).or_default().push((
                        xu,
                        yu,
                        matrix.family[i],
                        matrix.elevation[i],
                        matrix.moisture[i],
                        matrix.temperature[i],
                    ));
                }
            }
        }
    } else {
        for (tf, terrain, height, moisture, temperature) in queries.p0().iter() {
            let x = tf.translation.x.round() as isize;
            let y = tf.translation.z.round() as isize;
            if x < 0 || y < 0 {
                continue;
            }
            let x = x as usize;
            let y = y as usize;
            if x >= tex_w || y >= tex_h {
                continue;
            }
            let cx = (x / chunk_tiles) as u32;
            let cz = (y / chunk_tiles) as u32;
            index.tiles_by_chunk.entry((cx, cz)).or_default().push((
                x,
                y,
                terrain.0,
                height.0,
                moisture.0,
                temperature.0,
            ));
        }
    }
    for road in queries.p1().iter() {
        let x = road.tile_x as usize;
        let y = road.tile_z as usize;
        if x >= tex_w || y >= tex_h {
            continue;
        }
        let cx = (x / chunk_tiles) as u32;
        let cz = (y / chunk_tiles) as u32;
        index
            .roads_by_chunk
            .entry((cx, cz))
            .or_default()
            .push((x, y));
    }
}

type TileRasterSample = (
    usize,
    usize,
    crate::terrain::family::TerrainFamilyId,
    f32,
    f32,
    f32,
);

/// Spatial index for [`tile_world_fallback_rasterize`] — rebuilt when [`TileWorldFallbackRasterDirty::revision`] changes.
#[derive(Debug, Default)]
struct TileWorldFallbackTileIndex {
    revision: Option<u64>,
    tiles_by_chunk: HashMap<(u32, u32), Vec<TileRasterSample>>,
    roads_by_chunk: HashMap<(u32, u32), Vec<(usize, usize)>>,
}

/// Chunk grid + cadence/revision bookkeeping (keeps raster system under Bevy's param limit).
#[derive(Resource, Debug, Default)]
pub struct TileWorldFallbackRasterCtrl {
    pub chunk_grid: TileWorldFallbackChunkGrid,
    last_applied_revision: Option<u64>,
    cadence_acc: f32,
    last_ms: Option<f32>,
    tile_index: TileWorldFallbackTileIndex,
    /// Throttle fire-overlay-driven chunk marks (overlay rev can tick every frame).
    last_fire_overlay_mark_frame: u32,
    /// Re-raster on zoom band change only (CPU tint uses zoom α).
    last_raster_zoom_band: Option<u8>,
    /// Building iso stamps for tactical map material swap (from [`TileAtlasRegistry`]).
    pub atlas_stamps: Vec<crate::gui::map_tile_atlas_stamp::TileAtlasStampRequest>,
}

/// Whether [`tile_world_fallback_rasterize`] should repaint `minimap_image` this frame.
#[derive(Resource, Debug, Clone, Copy)]
pub struct TileFallbackRasterPolicy {
    pub cpu_minimap_pass: bool,
    pub chunks_per_frame: usize,
    pub fire_overlay_mark_interval_frames: u32,
    pub defer_zoom_dirty: bool,
    pub minimap_cadence_hz: f32,
}

impl Default for TileFallbackRasterPolicy {
    fn default() -> Self {
        let budget = crate::render::TileRasterBudget::default();
        Self {
            cpu_minimap_pass: true,
            chunks_per_frame: budget.chunks_per_frame,
            fire_overlay_mark_interval_frames: budget.fire_overlay_mark_interval_frames,
            defer_zoom_dirty: false,
            minimap_cadence_hz: 10.0,
        }
    }
}

/// Zoom α bands for overworld CPU raster (match interest quantum 0.1).
const RASTER_ZOOM_BANDS: u32 = 10;

#[inline]
fn raster_zoom_band(zoom_alpha: f32) -> u8 {
    (zoom_alpha.clamp(0.0, 1.0) * RASTER_ZOOM_BANDS as f32).floor() as u8
}

/// MAP-ZOOM-001 — zoom-band crossing always schedules tile dirty (Option B partial; lib witness).
#[must_use]
pub fn tile_raster_dirty_on_zoom_band_change_enabled() -> bool {
    true
}

/// Any main-camera zoom delta — full tile re-raster (prevents RTT zoom "ghost" from stale chunks).
fn bump_tile_raster_on_main_camera_zoom(
    desired: Res<MapCameraDesiredRes>,
    mut dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    mut last_zoom: Local<f32>,
) {
    let z = desired.scale.x;
    if *last_zoom > 0.0 && (z - *last_zoom).abs() > 1e-4 {
        ctrl.chunk_grid.mark_all_dirty();
        dirty.bump();
    }
    *last_zoom = z;
}

/// Runs **after** [`FireVisualFrameSet::BuildProfiles`](crate::render::FireVisualFrameSet) so minimap RGBA sees fresh [`SharedOverlayFieldBuffers`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileWorldFallbackAfterFireExtract;

pub struct TileWorldFallbackPlugin;

impl Plugin for TileWorldFallbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileWorldFallbackState>()
            .init_resource::<SimMinimapUiState>()
            .init_resource::<MinimapShellState>()
            .init_resource::<TileWorldFallbackRasterDirty>()
            .init_resource::<TileWorldFallbackRasterCtrl>()
            .init_resource::<TileFallbackRasterPolicy>()
            .init_resource::<crate::render::TileRasterBudget>()
            .init_resource::<crate::render::FireExtractCadence>()
            .init_resource::<crate::render::FireExtractClock>()
            .init_resource::<crate::render::TileRasterSpikeFeedback>()
            .init_resource::<crate::gui::map_tile_atlas_stamp::TileAtlasGpuCache>()
            .init_resource::<crate::gui::map_tile_atlas_stamp::TerrainGpuStampIndices>()
            .configure_sets(
                Update,
                TileWorldFallbackAfterFireExtract.after(FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                (
                    focus_main_camera_on_world_params,
                    sync_minimap_follow_camera_on_sim_enter.after(focus_main_camera_on_world_params),
                    refresh_tile_raster_budget,
                ),
            )
            .add_systems(
                OnEnter(BaseState::Editor),
                (focus_main_camera_on_world_params, refresh_tile_raster_budget),
            )
            .add_systems(
                Update,
                bump_tile_raster_on_main_camera_zoom
                    .after(crate::gui::MapCameraSystemSet::Smooth)
                    .run_if(crate::gui::in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                refresh_tile_raster_budget.run_if(|params: Res<WorldGenParams>| params.is_changed()),
            )
            .add_systems(
                Last,
                crate::render::sync_tile_raster_spike_feedback_system
                    .after(crate::gui::hud::finalize_frame_budget_diagnostics),
            )
            .add_systems(
                Update,
                (
                    tile_world_fallback_sync_spawner,
                    mark_tile_world_fallback_dirty_on_changes
                        .after(tile_world_fallback_sync_spawner),
                    mark_tile_world_fallback_dirty_on_map_overlay_controls
                        .after(mark_tile_world_fallback_dirty_on_changes),
                    sync_tile_fallback_raster_policy
                        .after(mark_tile_world_fallback_dirty_on_map_overlay_controls),
                    tile_world_fallback_rasterize
                        .after(sync_tile_fallback_raster_policy)
                        .after(crate::render::WaterSurfaceVisualSet)
                        .run_if(in_state(BaseState::Simulation).or_else(in_state(BaseState::Editor))),
                    tile_world_fallback_rasterize_perf
                        .after(tile_world_fallback_rasterize),
                )
                    .in_set(TileWorldFallbackAfterFireExtract),
            )
            .add_systems(
                Update,
                (
                    minimap_shell_keyboard_toggle,
                    minimap_shell_smooth_zoom_system,
                    sync_map_follow_from_game_camera.after(ViewAuthoritySystemSet::SyncViewManager),
                )
                    .run_if(|base: Res<State<BaseState>>| {
                        matches!(base.get(), BaseState::Simulation)
                    }),
            );
    }
}

fn focus_main_camera_on_world_params(
    mut cam: Query<&mut Transform, With<MainWorldCamera>>,
    params: Res<WorldGenParams>,
    test_scene: Option<Res<ActiveTestScene>>,
    launch: Option<Res<EngineLaunchArgs>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sim_viewport: Res<crate::gui::SimulationMapViewport>,
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    mut trace: ResMut<crate::render::view_runtime::ViewRuntimeTrace>,
) {
    if params.width == 0 || params.height == 0 {
        return;
    }
    let cx = params.width as f32 * 0.5;
    let cy = params.height as f32 * 0.5;
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let window_px = windows
        .single()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let viewport = crate::gui::map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
    let (zoom_lo, zoom_hi) = crate::gui::map_zoom_limits_for_world(world_w, world_h, viewport);
    let tactical_proof = test_scene
        .as_ref()
        .map(|s| s.0)
        .is_some_and(|scene| match scene {
            TestScene::VfxSandbox => true,
            TestScene::Visual => launch.as_ref().is_some_and(|l| l.full_capture_active()),
            _ => false,
        });
    let zoom = if tactical_proof {
        crate::gui::map_scale_for_zoom_alpha(
            crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA,
            zoom_lo,
            zoom_hi,
        )
    } else {
        match test_scene.as_ref().map(|s| s.0) {
            Some(TestScene::Fire) | Some(TestScene::Atmosphere) => {
                let margin = 0.9;
                let fit: f32 = margin * (viewport.x / world_w).min(viewport.y / world_h);
                fit.clamp(zoom_lo, zoom_hi)
            }
            _ => {
                let margin = 0.9;
                let fit = margin * (viewport.x / world_w.max(1.0)).min(viewport.y / world_h.max(1.0));
                fit.clamp(zoom_lo, zoom_hi)
            }
        }
    };
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
        t.translation.z = 999.0;
        t.scale = Vec3::ONE;
        t.rotation = Quat::IDENTITY;
    }
    let pose = MapCameraDesired {
        translation: Vec3::new(cx, cy, 999.0),
        scale: Vec3::splat(zoom),
        rotation: Quat::IDENTITY,
    };
    crate::gui::commit_map_camera_pose_to_view_authority(
        authority.as_mut(),
        trace.as_mut(),
        &pose,
    );
}

/// After [`focus_main_camera_on_world_params`], align minimap follow state with the committed main map pose.
fn sync_minimap_follow_camera_on_sim_enter(
    params: Res<WorldGenParams>,
    desired: Res<MapCameraDesiredRes>,
    mut map_views: ResMut<crate::gui::MapViewInstances>,
    mut shell: ResMut<MinimapShellState>,
) {
    if map_views.minimap.follow_mode != crate::gui::MinimapFollowMode::FollowCamera {
        return;
    }
    let center = if params.width > 0 && params.height > 0 {
        Vec2::new(desired.translation.x, desired.translation.y)
    } else {
        map_views.minimap.camera_center
    };
    map_views.minimap.camera_center = center;
    shell.world_center = center;
    // GPU compositor shows the full-world RT in FollowCamera — keep panel zoom neutral.
    map_views.minimap.zoom = 1.0;
    map_views.minimap.zoom_target = 1.0;
    shell.zoom = 1.0;
    shell.zoom_target = 1.0;
    shell.clamp_zoom();
}

fn make_rgba_image(w: u32, h: u32, label: &'static str) -> Image {
    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(label),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    let len = 4 * w as usize * h as usize;
    image.data = Some(vec![0u8; len]);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

fn tile_world_fallback_sync_spawner(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    authority: Res<TerrainRenderAuthority>,
    tiles: Query<(), With<TileMarker>>,
    terrain_chunks: Query<(), With<ChunkCellMatrix>>,
    dense_cache: Option<Res<WorldGenDenseTerrainCache>>,
    #[cfg(feature = "bevy_tilemap_adapter")] chunk_maps: Query<(), With<crate::render::ChunkTilemaps>>,
    mut state: ResMut<TileWorldFallbackState>,
    mut raster_dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
) {
    if authority.uses_gpu_sprite_display() {
        // GPU path: one sprite + dirty-gated texture bake (no per-frame CPU paint).
    } else if authority.is_gpu() {
        if let Some(e) = state.sprite_entity.take() {
            commands.entity(e).despawn();
        }
        state.last_w = 0;
        state.last_h = 0;
        state.image = Handle::default();
        return;
    }

    let has_terrain = !tiles.is_empty()
        || dense_cache.is_some()
        || !terrain_chunks.is_empty();
    #[cfg(feature = "bevy_tilemap_adapter")]
    let active = has_terrain && chunk_maps.is_empty();
    #[cfg(not(feature = "bevy_tilemap_adapter"))]
    let active = has_terrain;

    if !active {
        if let Some(e) = state.sprite_entity.take() {
            commands.entity(e).despawn();
        }
        state.last_w = 0;
        state.last_h = 0;
        state.image = Handle::default();
        return;
    }

    let w = params.width;
    let h = params.height;

    let need_new = state.sprite_entity.is_none() || state.last_w != w || state.last_h != h;
    if need_new {
        if let Some(e) = state.sprite_entity.take() {
            commands.entity(e).despawn();
        }
        let image = images.add(make_rgba_image(w, h, "tile_world_fallback_sim"));
        let minimap_image = images.add(make_rgba_image(w, h, "tile_world_fallback_minimap"));
        let e = commands
            .spawn((
                TileWorldFallbackSprite,
                crate::gui::simulation_map_rtt_render_layers(),
                crate::render::mig_a_static_bulk_bundle(),
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::new(w as f32, h as f32)),
                    ..default()
                },
                Transform::from_xyz(w as f32 * 0.5, h as f32 * 0.5, 0.0),
            ))
            .id();
        state.sprite_entity = Some(e);
        state.image = image;
        state.minimap_image = minimap_image;
        state.last_w = w;
        state.last_h = h;
        raster_ctrl.chunk_grid.resize_for_world(w, h);
        raster_ctrl.tile_index.revision = None;
        raster_ctrl.chunk_grid.mark_all_dirty();
        raster_dirty.bump();
    }
}

#[derive(Default)]
struct MapOverlayRasterWatch {
    sim_layers: PreviewLayers,
    sim_fire: bool,
    minimap_layers: PreviewLayers,
    minimap_fire: bool,
}

fn mark_tile_world_fallback_dirty_on_map_overlay_controls(
    map_views: Res<MapViewInstances>,
    presentation: Res<MapViewPresentationStates>,
    mut dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    mut last: Local<MapOverlayRasterWatch>,
) {
    let sim = presentation.get(MapViewInstanceId::SimulationMap);
    let minimap = &map_views.minimap;
    if sim.layers != last.sim_layers
        || sim.overlays.fire_heat != last.sim_fire
        || minimap.layers != last.minimap_layers
        || minimap.overlays.fire_heat != last.minimap_fire
    {
        last.sim_layers = sim.layers;
        last.sim_fire = sim.overlays.fire_heat;
        last.minimap_layers = minimap.layers;
        last.minimap_fire = minimap.overlays.fire_heat;
        raster_ctrl.chunk_grid.mark_all_dirty();
        dirty.bump();
    }
}

fn sync_tile_fallback_raster_policy(
    minimap_shell: Option<Res<MinimapShellState>>,
    minimap_registry: Option<Res<MinimapRenderTargetRegistry>>,
    fallback: Res<TileWorldFallbackState>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
    raster_spike_feedback: Option<Res<crate::render::TileRasterSpikeFeedback>>,
    mut raster_budget: ResMut<crate::render::TileRasterBudget>,
    tile_registry: Option<Res<crate::construction::procedural::TileAtlasRegistry>>,
    asset_server: Res<AssetServer>,
    sites: Query<(
        &crate::strategic::PlannedSite,
        &crate::strategic::ConstructionSite,
        &crate::strategic::SiteFootprint,
        Option<&crate::strategic::ProceduralBuildingSpec>,
    )>,
    catalog: Option<Res<crate::construction::procedural::VariantCatalog>>,
    sim_tick: Option<Res<crate::systems::sim_control::SimTick>>,
    overlay: Option<Res<crate::render::SharedOverlayFieldBuffers>>,
    mut policy: ResMut<TileFallbackRasterPolicy>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    mut atlas_cache: ResMut<crate::gui::map_tile_atlas_stamp::TileAtlasGpuCache>,
    mut gpu_stamp_inputs: TileFallbackGpuStampInputs,
) {
    let authority = gpu_stamp_inputs.authority.as_ref();
    let spike_active = spike_guard
        .as_deref()
        .is_some_and(|g| g.spike_active)
        || raster_spike_feedback
            .as_deref()
            .is_some_and(|f| f.defer_zoom_dirty);
    policy.cpu_minimap_pass = tile_fallback_cpu_minimap_raster_needed(
        minimap_shell.as_deref(),
        minimap_registry.as_deref(),
        fallback.as_ref(),
    );
    policy.chunks_per_frame = raster_budget.effective_chunks_per_frame(spike_active);
    policy.fire_overlay_mark_interval_frames = raster_budget.fire_overlay_mark_interval_frames;
    policy.defer_zoom_dirty = spike_active;
    policy.minimap_cadence_hz = 10.0;
    raster_budget.minimap_cpu_allowed = policy.cpu_minimap_pass;

    crate::gui::map_tile_atlas_stamp::preload_tile_atlas_gpu_cache(
        tile_registry.as_deref(),
        asset_server.as_ref(),
        atlas_cache.as_mut(),
    );
    raster_ctrl.atlas_stamps.clear();
    if let Some(registry) = tile_registry.as_deref() {
        let tick = sim_tick.as_deref().map(|t| t.0).unwrap_or(0);
        let overlay_heat = overlay
            .as_deref()
            .map(|o| {
                o.chunk_fire_heat
                    .values()
                    .copied()
                    .fold(0.0f32, f32::max)
            })
            .unwrap_or(0.0);
        for (planned, site, footprint, spec) in &sites {
            if let Some(stamp) = crate::gui::map_tile_atlas_stamp::stamp_request_for_site(
                registry,
                catalog.as_deref(),
                tick,
                overlay_heat,
                planned,
                site,
                footprint,
                spec,
            ) {
                raster_ctrl.atlas_stamps.push(stamp);
            }
        }
    }
    if authority.is_gpu() {
        crate::gui::map_tile_atlas_stamp::queue_gpu_terrain_stamp_indices(
            &raster_ctrl.atlas_stamps,
            gpu_stamp_inputs.gpu_stamps.as_mut(),
        );
    } else {
        gpu_stamp_inputs.gpu_stamps.tiles.clear();
    }
}

fn mark_tile_world_fallback_dirty_on_changes(
    frame: Res<FrameCount>,
    raster_policy: Res<TileFallbackRasterPolicy>,
    mut dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    added_tiles: Query<&Transform, Added<TileMarker>>,
    added_site_footprints: Query<&crate::strategic::SiteFootprint, Added<crate::strategic::SiteFootprint>>,
    changed_terrain: Query<&Transform, (With<TileMarker>, Changed<TerrainType>)>,
    added_roads: Query<&MapEditorRoadMarkerV1, Added<MapEditorRoadMarkerV1>>,
    changed_roads: Query<&MapEditorRoadMarkerV1, Changed<MapEditorRoadMarkerV1>>,
    handles: Res<TerrainRegistriesHandles>,
    overlay: Res<SharedOverlayFieldBuffers>,
    presentation: Res<MapViewPresentationStates>,
    map_views: Res<MapViewInstances>,
    sites: Query<&crate::strategic::SiteFootprint>,
    mut last_overlay_revision: Local<u64>,
) {
    let mut structural_bump = false;
    for tf in added_tiles.iter().chain(changed_terrain.iter()) {
        let tx = tf.translation.x.round().max(0.0) as u32;
        let tz = tf.translation.z.round().max(0.0) as u32;
        raster_ctrl.chunk_grid.mark_tile(tx, tz);
        structural_bump = true;
    }
    for fp in &added_site_footprints {
        for tile in &fp.tiles {
            raster_ctrl
                .chunk_grid
                .mark_tile(tile.x.max(0) as u32, tile.y.max(0) as u32);
            structural_bump = true;
        }
    }
    for road in added_roads.iter().chain(changed_roads.iter()) {
        raster_ctrl.chunk_grid.mark_tile(road.tile_x, road.tile_z);
        structural_bump = true;
    }
    if handles.is_changed() {
        raster_ctrl.chunk_grid.mark_all_dirty();
        structural_bump = true;
    }
    if overlay.revision != *last_overlay_revision {
        *last_overlay_revision = overlay.revision;
        let sim = presentation.get(MapViewInstanceId::SimulationMap);
        if sim.overlays.fire_heat || map_views.minimap.overlays.fire_heat {
            let fire_mark_interval = raster_policy.fire_overlay_mark_interval_frames;
            if frame
                .0
                .saturating_sub(raster_ctrl.last_fire_overlay_mark_frame)
                >= fire_mark_interval
            {
                raster_ctrl.last_fire_overlay_mark_frame = frame.0;
                let chunk_tiles = RASTER_CHUNK_TILES as i32;
                for chunk_coord in overlay.chunk_fire_heat.keys() {
                    let tx = chunk_coord.x.saturating_mul(chunk_tiles as i32).max(0) as u32;
                    let tz = chunk_coord.y.saturating_mul(chunk_tiles as i32).max(0) as u32;
                    raster_ctrl.chunk_grid.mark_tile(tx, tz);
                }
                // PT-5-003 — refresh building iso stamp footprints when fire band changes.
                for footprint in &sites {
                    for tile in &footprint.tiles {
                        raster_ctrl.chunk_grid.mark_tile(
                            tile.x.max(0) as u32,
                            tile.y.max(0) as u32,
                        );
                    }
                }
            }
        }
        // PERF-PLAY-001: overlay-only heat changes repaint dirty chunks — do not bump
        // global raster revision (that rebuilds the full tile spatial index every frame).
    }
    if structural_bump {
        dirty.bump();
    }
}

fn raster_tile_fallback_subregion(
    images: &mut Assets<Image>,
    handle: &Handle<Image>,
    tex_w: usize,
    tex_h: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    layers: PreviewLayers,
    fire_heat_overlay: bool,
    tile_iter: impl Iterator<Item = (usize, usize, crate::terrain::family::TerrainFamilyId, f32, f32, f32)>,
    road_iter: impl Iterator<Item = (usize, usize)>,
    mat_slices: &[(bevy::math::IVec2, bevy::math::UVec2, &[crate::terrain::material::MaterialId])],
    reg_opt: Option<&MaterialRegistry>,
    fam_opt: Option<&crate::terrain::TerrainFamilyRegistry>,
    chunk_geom: &[(bevy::math::IVec2, bevy::math::UVec2)],
    cell_fire_heat: &[(bevy::math::IVec2, bevy::math::UVec2, &[f32])],
    overlay: &SharedOverlayFieldBuffers,
    fire_heat_visibility_boost: f32,
    water_catalog: Option<&crate::render::WaterSurfaceVisualCatalog>,
    time_secs: f32,
    zoom_alpha: f32,
) {
    let Some(mut image) = images.get_mut(handle) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    crate::gui::map_tile_raster::raster_sim_minimap_layered_to_subregion(
        data,
        tex_w,
        tex_h,
        x0,
        y0,
        x1,
        y1,
        tile_iter,
        road_iter,
        layers,
        mat_slices,
        reg_opt,
        fam_opt,
    );
    if fire_heat_overlay {
        if !cell_fire_heat.is_empty() {
            crate::gui::map_tile_raster::apply_cell_fire_heat_to_rgba_subregion(
                data.as_mut_slice(),
                tex_w,
                x0,
                y0,
                x1,
                y1,
                cell_fire_heat,
                fire_heat_visibility_boost,
            );
        } else {
            crate::gui::map_tile_raster::apply_shared_fire_heat_to_rgba_subregion(
                data.as_mut_slice(),
                tex_w,
                x0,
                y0,
                x1,
                y1,
                chunk_geom,
                &overlay.chunk_fire_heat,
                fire_heat_visibility_boost,
            );
        }
    }
    if let Some(catalog) = water_catalog {
        crate::render::apply_water_surface_overlay_subregion(
            data.as_mut_slice(),
            tex_w,
            x0,
            y0,
            x1,
            y1,
            catalog,
            time_secs,
            zoom_alpha,
        );
    }
}

fn tile_world_fallback_rasterize(
    authority: Res<TerrainRenderAuthority>,
    mut images: ResMut<Assets<Image>>,
    state: Res<TileWorldFallbackState>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    map_views: Res<MapViewInstances>,
    presentation: Res<MapViewPresentationStates>,
    mut queries: ParamSet<(
        Query<(
            &Transform,
            &TerrainType,
            &Height,
            &Moisture,
            &Temperature,
        ), With<TileMarker>>,
        Query<&MapEditorRoadMarkerV1>,
        Query<(&Chunk, &ChunkCellMatrix, Option<&crate::systems::fire::ChunkFireOverlay>)>,
    )>,
    overlay: Res<SharedOverlayFieldBuffers>,
    water_catalog: Option<Res<crate::render::WaterSurfaceVisualCatalog>>,
    camera: Res<MapCameraDesiredRes>,
    raster_dirty: Res<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    time: Res<Time>,
    raster_policy: Res<TileFallbackRasterPolicy>,
    atlas_cache: Res<crate::gui::map_tile_atlas_stamp::TileAtlasGpuCache>,
) {
    if !authority.uses_cpu_fallback_raster() && !authority.uses_gpu_sprite_display() {
        raster_ctrl.last_applied_revision = Some(raster_dirty.revision());
        return;
    }
    if state.sprite_entity.is_none()
        || state.image == Handle::default()
        || state.minimap_image == Handle::default()
    {
        raster_ctrl.last_applied_revision = None;
        return;
    }
    // Chunk-authoritative world gen has no TileMarker entities; raster from ChunkCellMatrix.
    if queries.p0().is_empty() && queries.p2().is_empty() {
        return;
    }
    let tex_w_u = state.last_w;
    let tex_h_u = state.last_h;
    let tex_w = tex_w_u as usize;
    let tex_h = tex_h_u as usize;
    if tex_w == 0 || tex_h == 0 {
        return;
    }
    raster_ctrl.chunk_grid.resize_for_world(tex_w_u, tex_h_u);

    let rev = raster_dirty.revision();
    let zoom_alpha = crate::gui::map_zoom_alpha(camera.scale.x);
    let zoom_band = raster_zoom_band(zoom_alpha);
    let zoom_band_changed = raster_ctrl.last_raster_zoom_band != Some(zoom_band);
    let spike_active = raster_policy.defer_zoom_dirty;
    if zoom_band_changed {
        raster_ctrl.last_raster_zoom_band = Some(zoom_band);
        if !raster_ctrl.chunk_grid.has_dirty() {
            raster_ctrl.chunk_grid.mark_all_dirty();
        }
    }

    let work_pending =
        raster_ctrl.last_applied_revision != Some(rev) || raster_ctrl.chunk_grid.has_dirty();
    if !work_pending {
        return;
    }

    let hz = if raster_policy.minimap_cadence_hz.is_finite() && raster_policy.minimap_cadence_hz > 0.25 {
        raster_policy.minimap_cadence_hz
    } else {
        10.0
    };
    let interval = preview_partial_min_interval_from_hz(hz);
    let force_immediate =
        raster_ctrl.last_applied_revision.is_none() || raster_ctrl.chunk_grid.has_dirty();
    if !force_immediate {
        raster_ctrl.cadence_acc += time.delta_secs();
        if raster_ctrl.cadence_acc < interval {
            return;
        }
        raster_ctrl.cadence_acc -= interval;
    }

    if raster_ctrl.last_applied_revision != Some(rev) && !raster_ctrl.chunk_grid.has_dirty() {
        raster_ctrl.chunk_grid.mark_all_dirty();
    }

    // First paint must not inherit spike cap — GPU minimap compositor reads `fallback.image` and
    // stays black until terrain RGBA is populated at least once.
    let chunk_budget = if raster_ctrl.last_applied_revision.is_none() {
        usize::MAX
    } else if spike_active {
        raster_policy.chunks_per_frame.min(2)
    } else {
        raster_policy.chunks_per_frame
    };
    let dirty_chunks = raster_ctrl.chunk_grid.take_dirty_chunks(chunk_budget);
    if dirty_chunks.is_empty() {
        if !raster_ctrl.chunk_grid.has_dirty() {
            raster_ctrl.last_applied_revision = Some(rev);
        }
        return;
    }

    let mut atlas_owned: HashMap<String, (Vec<u8>, usize, usize)> = HashMap::new();
    if !raster_ctrl.atlas_stamps.is_empty() {
        for req in &raster_ctrl.atlas_stamps {
            if atlas_owned.contains_key(&req.atlas_id) {
                continue;
            }
            let Some(handle) = atlas_cache.handles.get(&req.atlas_id) else {
                continue;
            };
            let Some(image) = images.get(handle) else {
                continue;
            };
            let Some(data) = image.data.as_ref() else {
                continue;
            };
            let w = image.width() as usize;
            let h = image.height() as usize;
            if w > 0 && h > 0 {
                atlas_owned.insert(req.atlas_id.clone(), (data.to_vec(), w, h));
            }
        }
    }
    let atlas_slices: HashMap<String, (&[u8], usize, usize)> = atlas_owned
        .iter()
        .map(|(k, (v, w, h))| (k.clone(), (v.as_slice(), *w, *h)))
        .collect();

    let raster_started = FrameBudgetTimer::start();
    // VX-P0-01: strategic zoom boost only on the main overworld raster — minimap stays 1.0 so
    // optional fire-heat toggle does not wash the whole panel when zoomed out.
    let fire_boost_main = if zoom_alpha < crate::render::gpu_particles::FIRE_SPARK_STRATEGIC_ZOOM_ALPHA {
        1.0
    } else {
        (1.0 + 0.85 / camera.scale.x.max(0.5)).clamp(1.0, 2.0)
    };
    let fire_boost_minimap = 1.0;
    let time_secs = time.elapsed_secs();
    let water_catalog = water_catalog.as_deref();

    let mat_slices: Vec<(bevy::math::IVec2, bevy::math::UVec2, &[crate::terrain::material::MaterialId])> =
        vec![];
    let reg_opt = materials.get(&handles.material_registry);
    let fam_opt = Some(crate::terrain::default_terrain_families());
    let chunk_geom: Vec<(bevy::math::IVec2, bevy::math::UVec2)> = queries
        .p2()
        .iter()
        .map(|(c, m, _)| (c.coord, m.size))
        .collect();
    let cell_fire_owned: Vec<(bevy::math::IVec2, bevy::math::UVec2, Vec<f32>)> = queries
        .p2()
        .iter()
        .filter_map(|(c, m, ovl)| {
            let ovl = ovl?;
            if ovl.heat.is_empty() {
                return None;
            }
            Some((c.coord, m.size, ovl.heat.clone()))
        })
        .collect();
    let cell_fire_heat: Vec<(bevy::math::IVec2, bevy::math::UVec2, &[f32])> = cell_fire_owned
        .iter()
        .map(|(c, s, h)| (*c, *s, h.as_slice()))
        .collect();

    let sim = presentation.get(MapViewInstanceId::SimulationMap);
    let mut sim_layers = sim.layers;
    if sim_layers.base_bits().is_empty() {
        sim_layers.replace_base(PreviewLayers::BIOME);
    }
    let minimap = &map_views.minimap;

    rebuild_tile_world_fallback_index(
        &mut raster_ctrl.tile_index,
        rev,
        tex_w,
        tex_h,
        &mut queries,
    );

    for (cx, cz) in &dirty_chunks {
        let (x0, y0, x1, y1) =
            TileWorldFallbackChunkGrid::chunk_pixel_bounds(*cx, *cz, tex_w_u, tex_h_u);

        let chunk_tiles = raster_ctrl
            .tile_index
            .tiles_by_chunk
            .get(&(*cx, *cz))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let tile_iter = chunk_tiles.iter().copied().filter(|(x, y, ..)| {
            *x >= x0 && *x < x1 && *y >= y0 && *y < y1
        });
        let chunk_roads = raster_ctrl
            .tile_index
            .roads_by_chunk
            .get(&(*cx, *cz))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let road_iter = chunk_roads
            .iter()
            .copied()
            .filter(|(x, y)| *x >= x0 && *x < x1 && *y >= y0 && *y < y1);

        raster_tile_fallback_subregion(
            images.as_mut(),
            &state.image,
            tex_w,
            tex_h,
            x0,
            y0,
            x1,
            y1,
            sim_layers,
            sim.overlays.fire_heat,
            tile_iter,
            road_iter,
            &mat_slices,
            reg_opt,
            fam_opt,
            &chunk_geom,
            &cell_fire_heat,
            overlay.as_ref(),
            fire_boost_main,
            water_catalog,
            time_secs,
            zoom_alpha,
        );

        if !raster_ctrl.atlas_stamps.is_empty()
            && !atlas_slices.is_empty()
            && crate::gui::map_tile_atlas_stamp::stamp_cpu_rgba_blit_enabled(*authority)
        {
            if let Some(mut dest_image) = images.get_mut(&state.image) {
                if let Some(data) = dest_image.data.as_mut() {
                    crate::gui::map_tile_atlas_stamp::apply_atlas_stamps_to_rgba_subregion(
                        data,
                        tex_w,
                        x0,
                        y0,
                        x1,
                        y1,
                        &raster_ctrl.atlas_stamps,
                        &atlas_slices,
                    );
                }
            }
        }

        let tile_iter = chunk_tiles.iter().copied().filter(|(x, y, ..)| {
            *x >= x0 && *x < x1 && *y >= y0 && *y < y1
        });
        let road_iter = chunk_roads
            .iter()
            .copied()
            .filter(|(x, y)| *x >= x0 && *x < x1 && *y >= y0 && *y < y1);

        if raster_policy.cpu_minimap_pass && !spike_active {
            raster_tile_fallback_subregion(
                images.as_mut(),
                &state.minimap_image,
                tex_w,
                tex_h,
                x0,
                y0,
                x1,
                y1,
                minimap.layers,
                minimap.overlays.fire_heat,
                tile_iter,
                road_iter,
                &mat_slices,
                reg_opt,
                fam_opt,
                &chunk_geom,
                &cell_fire_heat,
                overlay.as_ref(),
                fire_boost_minimap,
                water_catalog,
                time_secs,
                zoom_alpha,
            );
        }
    }

    if !raster_ctrl.chunk_grid.has_dirty() {
        raster_ctrl.last_applied_revision = Some(rev);
    }
    raster_ctrl.last_ms = Some(raster_started.elapsed_ms());
}

fn tile_world_fallback_rasterize_perf(
    authority: Res<crate::render::terrain_render_authority::TerrainRenderAuthority>,
    mut raster_ctrl: ResMut<TileWorldFallbackRasterCtrl>,
    mut budget: Option<ResMut<FrameBudgetDiagnostics>>,
    mut perf: Option<ResMut<crate::render::FramePerf>>,
) {
    let Some(raster_ms) = raster_ctrl.last_ms.take() else {
        return;
    };
    // GPU sprite bakes are dirty-gated; §10 gate counts only CPU fallback raster.
    if !authority.uses_cpu_fallback_raster() {
        return;
    }
    if let Some(budget) = budget.as_mut() {
        budget.record_bucket_ms(FrameBudgetBucket::MinimapRaster, raster_ms);
    }
    if let Some(perf) = perf.as_mut() {
        perf.tile_raster_ran = true;
        crate::render::record_frame_perf_ms(
            perf,
            raster_ms,
            crate::render::FramePerfSlot::TileRaster,
        );
    }
}

fn minimap_shell_smooth_zoom_system(
    time: Res<Time>,
    mut map_views: ResMut<MapViewInstances>,
    mut shell: ResMut<MinimapShellState>,
) {
    map_views.minimap.tick_smooth_zoom(time.delta_secs());
    shell.zoom = map_views.minimap.zoom;
    shell.zoom_target = map_views.minimap.zoom_target;
}

fn sync_map_follow_from_game_camera(
    mut map_views: ResMut<MapViewInstances>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesiredRes>,
    mut shell: ResMut<MinimapShellState>,
) {
    if map_views.minimap.follow_mode != crate::gui::MinimapFollowMode::FollowCamera {
        return;
    }
    let center = camera_translation(&view_manager, ViewId::WorldMain)
        .unwrap_or_else(|| Vec2::new(desired.translation.x, desired.translation.y));
    map_views.minimap.camera_center = center;
    shell.world_center = map_views.minimap.camera_center;
    shell.diagnostic_camera_drove_ui = true;
}

fn minimap_shell_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut shell: ResMut<MinimapShellState>,
    mut legacy: ResMut<SimMinimapUiState>,
) {
    if keys.just_pressed(bindings.toggle_sim_minimap) {
        shell.visible = !shell.visible;
        legacy.open = shell.visible;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn simulation_minimap_egui_texture(
    contexts: &mut EguiContexts,
    shell: &mut MinimapShellState,
    legacy: &mut SimMinimapUiState,
    dock: &mut crate::gui::hud::HudDockRegistry,
    fallback: &TileWorldFallbackState,
    frames: &crate::gui::ResolvedMapViewFrames,
    tex_cache: &mut HudEguiTextureCache,
    ready: &mut crate::gui::MapViewReadyStates,
    interaction_frozen: bool,
) -> Option<egui::TextureId> {
    crate::gui::minimap::resolve_minimap_egui_texture(
        contexts,
        shell,
        legacy,
        dock,
        fallback,
        frames,
        tex_cache,
        ready,
        interaction_frozen,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn draw_simulation_minimap_egui(
    ctx: &mut egui::Context,
    tex_id: egui::TextureId,
    shell: &mut MinimapShellState,
    legacy: &mut SimMinimapUiState,
    manager: &ViewManager,
    desired: &MapCameraDesired,
    sim_map_viewport: &SimulationMapViewport,
    presentation: &mut MapViewState,
    dock: &mut crate::gui::hud::HudDockRegistry,
    layout: &mut HudLayoutStore,
    palette: &UiPalette,
    shell_diag: &mut ProductShellDiagnostics,
    viewport_rect_sanity: &mut ViewportRectSanity,
    fallback: &TileWorldFallbackState,
    raster_dirty: &TileWorldFallbackRasterDirty,
    fire_atm: Option<&FireAtmosphereAggregate>,
    map_frames: &ResolvedMapViewFrames,
    map_presentation_diag: &mut MapPresentationDiagnostics,
    defer_heavy_chrome: bool,
    pending_layout: &mut PendingHudLayoutCommit,
    interaction: &mut MinimapInteractionBuffer,
    active_input: &mut ActiveMapViewInput,
    base: BaseState,
    overlays: &crate::gui::MinimapOverlayMask,
    ecology_rows: u32,
    veg_burn_rows: u32,
    infra_settings: Option<&crate::render::InfrastructureOverlaySettings>,
    infra_overlays: Option<&crate::render::InfrastructureOverlayDrawRequests>,
    power_presentation: Option<&crate::render::PowerMapOverlayPresentation>,
) {
    let rev = raster_dirty.revision();
    let w = fallback.last_w as f32;
    let h = fallback.last_h as f32;
    let mut open = shell.visible;
    let default_viewport = viewport_rect_sanity.inspect_logical_size(
        shell.panel_viewport_suggestion_logical_size,
        ViewportRectSource::MinimapPanelSliders,
        Vec2::new(260.0, 220.0),
        Some(shell_diag),
    );
    let default_size = [default_viewport.x, default_viewport.y];
    let default_pos = crate::gui::hud::floating_unanchored_default_pos(
        ctx,
        HudWidgetId::Minimap,
        default_size,
    );

    let window = |ui: &mut egui::Ui| {
        map_toolbar(
            ui,
            presentation,
            &palette,
            "sim_minimap",
            MapToolbarConfig {
                show_follow: true,
                show_bookmarks: true,
                show_generation_tools: false,
                show_render_mode: false,
                show_zoom_reset: false,
            },
        );
        map_toolbar_minimap_zoom(ui, shell, presentation);
        ui.checkbox(
            &mut shell.show_tactical_viewport_frame,
            "Show map view frame",
        );
        ui.horizontal(|ui| {
            ui.checkbox(&mut shell.detached, "Detached");
        });
        if !pending_layout.drag_active {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(
                        &mut shell.panel_viewport_suggestion_logical_size.x,
                        180.0..=720.0,
                    )
                    .text("Width"),
                );
                ui.add(
                    egui::Slider::new(
                        &mut shell.panel_viewport_suggestion_logical_size.y,
                        160.0..=720.0,
                    )
                    .text("Height"),
                );
            });
            shell.panel_viewport_suggestion_active = true;
        }
        if native_minimap_window_supported() {
            ui.checkbox(&mut shell.native_window_requested, "Detached native window (stub)");
        }
        ui.horizontal(|ui| {
            ui.label("Source");
            egui::ComboBox::from_id_salt("minimap_presentation_source")
                .selected_text(match shell.presentation_source {
                    MinimapPresentationSource::SharedCpuRaster => "CPU raster",
                    MinimapPresentationSource::SharedRenderTargetImage => "GPU RT (stub)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut shell.presentation_source,
                        MinimapPresentationSource::SharedCpuRaster,
                        "CPU raster",
                    );
                    ui.selectable_value(
                        &mut shell.presentation_source,
                        MinimapPresentationSource::SharedRenderTargetImage,
                        "GPU RT (stub)",
                    );
                });
        });
        let source = match shell.presentation_source {
            MinimapPresentationSource::SharedCpuRaster => "CPU shared raster",
            MinimapPresentationSource::SharedRenderTargetImage => "GPU RT consumer (stub)",
        };
        ui.label(
            egui::RichText::new(format!(
                "{source} · rev {rev} · click/double-click to focus camera"
            ))
            .small()
            .weak(),
        );
        if let Some(agg) = fire_atm.as_ref() {
            if !defer_heavy_chrome {
                let s = agg.smoke_density.clamp(0.0, 1.0);
                let vl = agg.visibility_loss.clamp(0.0, 1.0);
                let heat = agg.heat_energy;
                egui::CollapsingHeader::new("Fire extract")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(format!(
                            "smoke {:.0}% · vis loss {:.0}% · heat {:.1}",
                            s * 100.0,
                            vl * 100.0,
                            heat
                        ))
                        .small());
                    });
            }
        }
        const MAP_PANEL_PADDING: f32 = MAP_PANEL_INSET_PX;
        let available = ui.available_size();
        let (panel_rect, response) = ui.allocate_exact_size(
            egui::vec2(available.x.max(120.0), available.y.max(96.0)),
            egui::Sense::click(),
        );
        let inner = panel_rect.shrink(MAP_PANEL_PADDING);
        let tex_size = UVec2::new(w.max(1.0) as u32, h.max(1.0) as u32);
        ensure_viewport_camera_initialized(presentation, w.max(1.0), h.max(1.0));
        let panel_size = Vec2::new(inner.width(), inner.height());
        // MINIMAP-RESIZE-FEEDBACK-FIX: do NOT write egui content size back into the panel-size
        // authority (`panel_viewport_suggestion_*`) during layout. That authority is owned by the
        // sliders + `resolve_minimap_panel_viewport`; feeding content size here (and the outer
        // window rect below) ratcheted the window ~+2px/frame and re-bumped the resolve revision
        // every frame. The minimap still renders at its allocated size because the painted image
        // (`compute_map_fit_strict` over `inner`/`tex_size`) and the local fit camera follow the
        // egui-allocated panel — we only stop the write-back loop.
        if panel_size.x > 1.0
            && panel_size.y > 1.0
            && (presentation.viewport_size - panel_size).length_squared() > 4.0
        {
            presentation.viewport_size = panel_size;
            // PERF-INSTR-VFX-001: egui-allocated panel size written into the minimap presentation
            // camera viewport. Trace to confirm whether this still ratchets vs the suggestion path.
            crate::render::trace_minimap_size_writer(
                "presentation.viewport_size",
                panel_size.x,
                panel_size.y,
            );
            fit_viewport_to_map(presentation, panel_size, w.max(1.0), h.max(1.0));
        }
        let fit = compute_map_fit_strict(inner, tex_size, presentation.fit_mode);
        let image_rect = fit.image_rect;
        let sample_uv = fit.uv_rect;
        let map_center = Vec2::new(w.max(1.0) * 0.5, h.max(1.0) * 0.5);
        let hit_zoom = fit.scale.max(1e-6);
        let painter = ui.painter().with_clip_rect(panel_rect);
        painter.image(tex_id, image_rect, sample_uv, egui::Color32::WHITE);
        if let (Some(settings), Some(edges), Some(power)) =
            (infra_settings, infra_overlays, power_presentation)
        {
            super::power_map_overlay_draw::draw_power_strokes_on_minimap(
                &painter,
                edges,
                settings,
                power,
                w,
                h,
                image_rect,
                sample_uv,
            );
        }
        if shell.show_tactical_viewport_frame {
            paint_tactical_viewport_frame_on_minimap(
                &painter,
                palette,
                None,
                manager,
                desired,
                sim_map_viewport,
                w,
                h,
                image_rect,
                sample_uv,
            );
        }
        shell.last_image_rect = Some(image_rect);
        map_presentation_diag.record_fit_truth(
            MapViewInstanceId::Minimap,
            panel_rect,
            tex_size,
            presentation.fit_mode,
            MAP_PANEL_PADDING,
            image_rect,
            sample_uv,
            map_frames.get(MapViewInstanceId::Minimap).viewport_extent,
            map_center,
            hit_zoom,
            Some(map_frames.get(MapViewInstanceId::Minimap).world_bounds),
            true,
        );
        if response.hovered() {
            active_input.0 = Some(MapViewInstanceId::Minimap);
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                interaction.queue_scroll_zoom(scroll * 0.035);
            }
        }
        if response.clicked() || response.double_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                shell.last_image_rect = Some(image_rect);
                let tex_w = w.max(1.0);
                let tex_h = h.max(1.0);
                if let Some(world) = view_surface_screen_to_world(
                    manager,
                    ViewId::Minimap,
                    pos,
                    image_rect,
                    tex_w,
                    tex_h,
                ) {
                    interaction.queue_focus(world, response.double_clicked());
                }
            }
        }
        if !defer_heavy_chrome {
            crate::gui::hud::minimap_topology_legend::draw_minimap_topology_legend_ui(
                ui,
                shell,
                overlays,
                ecology_rows,
                veg_burn_rows,
                base,
            );
        }
    };

    let window_builder = layout.apply_window(
        HudWidgetId::Minimap,
        std_floating(egui::Window::new("Minimap")).id(HudWidgetId::Minimap.egui_window_id()),
        default_pos,
        [default_viewport.x, default_viewport.y],
    )
    .min_size([150.0, 130.0]);
    let response = window_builder.open(&mut open).show(ctx, window);
    if let Some(inner) = response {
        if pending_layout.can_emit_layout_capture() {
            capture_shell_layout(
                layout,
                HudWidgetId::Minimap,
                &inner.response,
                Some(pending_layout),
            );
            // MINIMAP-RESIZE-FEEDBACK-FIX: do NOT requeue the outer window rect (content + toolbar
            // + sliders + legend chrome) as the next panel extent. That fed a value larger than the
            // panel back into `panel_viewport_suggestion_logical_size` (via the interaction buffer →
            // `update_minimap_view` → `commit_map_view_viewport_suggestions`), which then became the
            // `default_viewport` that sized the window next frame — a ~+2px/frame ratchet that never
            // settled and re-bumped the resolve revision every frame. The panel size authority is now
            // the sliders + `resolve_minimap_panel_viewport` only. `viewport_rect_sanity` is still
            // consulted for `default_viewport` above (sole sizing read).
        }
        shell.last_window_rect = Some(inner.response.rect);
        // PERF-INSTR-VFX-001: outer egui window rect (content + toolbar + sliders + legend chrome).
        // This feeds `apply_window_rect_layout` → body → suggestion. Prime ratchet suspect: trace it.
        crate::render::trace_minimap_size_writer(
            "shell.last_window_rect",
            inner.response.rect.width(),
            inner.response.rect.height(),
        );
    }
    shell.visible = open;
    legacy.open = open;
    dock.slot_mut(HudWidgetId::Minimap).visible = open;
    dock.slot_mut(HudWidgetId::Minimap).detached = shell.detached;
    dock.slot_mut(HudWidgetId::Minimap).minimized = shell.minimized;
    shell_diag.set_widget_visible(HudWidgetId::Minimap, open && !shell.minimized);
}

#[cfg(test)]
mod chunk_grid_tests {
    use super::*;
    #[test]
    fn cpu_minimap_raster_skipped_on_main_sim_gpu_path() {
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        let fallback = TileWorldFallbackState::default();
        if crate::render::minimap_gpu_compositor_env_enabled() {
            assert!(
                !tile_fallback_cpu_minimap_raster_needed(Some(&shell), Some(&registry), &fallback),
                "main sim GPU path must not run duplicate CPU minimap pass"
            );
        }

        let mut images = Assets::<Image>::default();
        registry.committed_image = images.add(super::make_rgba_image(8, 8, "test_minimap_rt"));
        assert!(!tile_fallback_cpu_minimap_raster_needed(
            Some(&shell),
            Some(&registry),
            &fallback,
        ));
    }

    #[test]
    fn cpu_minimap_raster_runs_for_effects_opt_in() {
        if !crate::render::minimap_gpu_compositor_env_enabled() {
            return;
        }
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedCpuRaster,
            ..Default::default()
        };
        let registry = MinimapRenderTargetRegistry::default();
        let fallback = TileWorldFallbackState::default();
        assert!(
            tile_fallback_cpu_minimap_raster_needed(Some(&shell), Some(&registry), &fallback),
            "effects SharedCpuRaster must keep CPU minimap raster alive"
        );
    }

    #[test]
    fn cpu_minimap_raster_runs_when_gpu_compositor_disabled() {
        if crate::render::minimap_gpu_compositor_env_enabled() {
            return;
        }
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedCpuRaster,
            ..Default::default()
        };
        let registry = MinimapRenderTargetRegistry::default();
        let fallback = TileWorldFallbackState::default();
        assert!(tile_fallback_cpu_minimap_raster_needed(
            Some(&shell),
            Some(&registry),
            &fallback,
        ));
    }

    #[test]
    fn chunk_bounds_clamp_to_world() {
        let (x0, y0, x1, y1) = TileWorldFallbackChunkGrid::chunk_pixel_bounds(1, 1, 200, 200);
        assert_eq!((x0, y0, x1, y1), (128, 128, 200, 200));
    }

    #[test]
    fn take_dirty_respects_budget() {
        let mut grid = TileWorldFallbackChunkGrid::default();
        grid.resize_for_world(256, 256);
        grid.mark_all_dirty();
        let batch = grid.take_dirty_chunks(3);
        assert_eq!(batch.len(), 3);
        assert!(grid.has_dirty());
        let rest = grid.take_dirty_chunks(16);
        assert!(!rest.is_empty());
        assert!(!grid.has_dirty());
    }
}
