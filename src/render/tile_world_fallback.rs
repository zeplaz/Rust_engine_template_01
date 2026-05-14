//! Raster [`TileMarker`] worlds to a single sprite when chunk tilemaps are absent (default build has no `bevy_tilemap_adapter`).
//!
//! Without this, generated tiles have no mesh/material and the main camera shows nothing.
//!
//! **Performance:** `tile_world_fallback_rasterize` rewrites the CPU RGBA buffer when
//! [`TileWorldFallbackRasterDirty`] bumps (new sprite, new/changed tiles or roads), not only when
//! `tile_count` changes — avoids stale maps when terrain edits keep the same count (`base_visual_dev01_plan_status` P0-A).
//! **Fire tint:** after terrain + roads, applies [`crate::gui::map_tile_raster::apply_shared_fire_heat_to_rgba`]
//! from [`crate::render::SharedOverlayFieldBuffers`] (same source as world preview); raster systems run in
//! [`TileWorldFallbackAfterFireExtract`] **after** [`crate::render::FireVisualFrameSet::BuildProfiles`].
//!
//! **Camera:** [`MainWorldCamera`] is centered on `(params.width/2, params.height/2)` in tile space; CLI `--test`
//! modes apply extra orthographic scale so the overworld fills more of the window.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::{ActiveTestScene, BaseState};
use crate::gui::{default_map_zoom_for_world, MapCameraDesired};
use crate::gui::MainWorldCamera;
use crate::gui::map_tile_raster::{apply_shared_fire_heat_to_rgba, raster_tiles_and_roads_to_rgba};
use crate::gui::preview_partial_min_interval_from_hz;
use crate::render::FireAtmosphereAggregate;
use crate::render::{FireVisualFrameSet, SharedOverlayFieldBuffers};
use crate::gui::std_floating;
use crate::gui::editor::map_editor::MapEditorRoadMarkerV1;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::{TerrainType, TileMarker, WorldGenParams};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::MaterialRegistry;

/// Marks the full-map fallback sprite entity.
#[derive(Component)]
pub struct TileWorldFallbackSprite;

#[derive(Resource, Default)]
pub struct TileWorldFallbackState {
    pub sprite_entity: Option<Entity>,
    pub image: Handle<Image>,
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

/// Runs **after** [`FireVisualFrameSet::BuildProfiles`](crate::render::FireVisualFrameSet) so minimap RGBA sees fresh [`SharedOverlayFieldBuffers`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileWorldFallbackAfterFireExtract;

pub struct TileWorldFallbackPlugin;

impl Plugin for TileWorldFallbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileWorldFallbackState>()
            .init_resource::<SimMinimapUiState>()
            .init_resource::<TileWorldFallbackRasterDirty>()
            .configure_sets(
                Update,
                TileWorldFallbackAfterFireExtract.after(FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                focus_main_camera_on_world_params,
            )
            .add_systems(
                OnEnter(BaseState::Editor),
                focus_main_camera_on_world_params,
            )
            .add_systems(
                Update,
                (
                    tile_world_fallback_sync_spawner,
                    mark_tile_world_fallback_dirty_on_changes,
                    tile_world_fallback_rasterize,
                )
                    .chain()
                    .in_set(TileWorldFallbackAfterFireExtract),
            )
            .add_systems(EguiPrimaryContextPass, simulation_minimap_egui_window);
    }
}

fn focus_main_camera_on_world_params(
    mut cam: Query<&mut Transform, With<MainWorldCamera>>,
    params: Res<WorldGenParams>,
    test_scene: Option<Res<ActiveTestScene>>,
    mut desired: ResMut<MapCameraDesired>,
) {
    if params.width == 0 || params.height == 0 {
        return;
    }
    let cx = params.width as f32 * 0.5;
    let cy = params.height as f32 * 0.5;
    let zoom = default_map_zoom_for_world(test_scene);
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
        t.scale = Vec3::splat(zoom);
        t.rotation = Quat::IDENTITY;
    }
    desired.translation = Vec3::new(cx, cy, 0.0);
    desired.scale = Vec3::splat(zoom);
    desired.rotation = Quat::IDENTITY;
}

fn make_rgba_image(w: u32, h: u32) -> Image {
    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("tile_world_fallback"),
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
    image
}

fn tile_world_fallback_sync_spawner(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    tiles: Query<(), With<TileMarker>>,
    #[cfg(feature = "bevy_tilemap_adapter")] chunk_maps: Query<(), With<crate::render::ChunkTilemaps>>,
    mut state: ResMut<TileWorldFallbackState>,
    mut raster_dirty: ResMut<TileWorldFallbackRasterDirty>,
) {
    #[cfg(feature = "bevy_tilemap_adapter")]
    let active = !tiles.is_empty() && chunk_maps.is_empty();
    #[cfg(not(feature = "bevy_tilemap_adapter"))]
    let active = !tiles.is_empty();

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
        let image = images.add(make_rgba_image(w, h));
        let e = commands
            .spawn((
                TileWorldFallbackSprite,
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
        state.last_w = w;
        state.last_h = h;
        raster_dirty.bump();
    }
}

fn mark_tile_world_fallback_dirty_on_changes(
    mut dirty: ResMut<TileWorldFallbackRasterDirty>,
    added_tiles: Query<(), Added<TileMarker>>,
    changed_terrain: Query<(), (With<TileMarker>, Changed<TerrainType>)>,
    added_roads: Query<(), Added<MapEditorRoadMarkerV1>>,
    changed_roads: Query<(), Changed<MapEditorRoadMarkerV1>>,
    handles: Res<TerrainRegistriesHandles>,
    overlay: Res<SharedOverlayFieldBuffers>,
    mut last_overlay_revision: Local<u64>,
) {
    if added_tiles.iter().next().is_some()
        || changed_terrain.iter().next().is_some()
        || added_roads.iter().next().is_some()
        || changed_roads.iter().next().is_some()
        || handles.is_changed()
    {
        dirty.bump();
    }
    if overlay.revision != *last_overlay_revision {
        *last_overlay_revision = overlay.revision;
        dirty.bump();
    }
}

fn tile_world_fallback_rasterize(
    mut images: ResMut<Assets<Image>>,
    state: Res<TileWorldFallbackState>,
    base: Res<State<BaseState>>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tile_q: Query<(&Transform, &TerrainType), With<TileMarker>>,
    road_q: Query<&MapEditorRoadMarkerV1>,
    chunk_geom_q: Query<(&Chunk, &ChunkCellMatrix)>,
    overlay: Res<SharedOverlayFieldBuffers>,
    raster_dirty: Res<TileWorldFallbackRasterDirty>,
    time: Res<Time>,
    cadence: Option<Res<crate::gui::VisualCadence>>,
    mut last_applied_revision: Local<Option<u64>>,
    mut cadence_acc: Local<f32>,
) {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        *last_applied_revision = None;
        return;
    }
    if state.sprite_entity.is_none() || state.image == Handle::default() {
        *last_applied_revision = None;
        return;
    }
    let tile_count = tile_q.iter().count();
    if tile_count == 0 {
        return;
    }
    let rev = raster_dirty.revision();
    if *last_applied_revision == Some(rev) {
        return;
    }

    if let Some(c) = cadence.as_deref() {
        let hz = if c.minimap_hz.is_finite() && c.minimap_hz > 0.25 {
            c.minimap_hz
        } else {
            10.0
        };
        let interval = preview_partial_min_interval_from_hz(hz);
        let force_immediate = last_applied_revision.is_none();
        if !force_immediate {
            *cadence_acc += time.delta_secs();
            if *cadence_acc < interval {
                return;
            }
            *cadence_acc -= interval;
        }
    }

    let Some(image) = images.get_mut(&state.image) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let tex_w = state.last_w as usize;
    let tex_h = state.last_h as usize;
    if tex_w == 0 || tex_h == 0 {
        return;
    }

    let mat_slices: Vec<(bevy::math::IVec2, bevy::math::UVec2, &[crate::terrain::material::MaterialId])> =
        vec![];
    let reg_opt = materials.get(&handles.material_registry);
    let fam_opt = Some(crate::terrain::default_terrain_families());

    let tile_iter = tile_q.iter().filter_map(|(tf, terrain)| {
        let x = tf.translation.x.round() as isize;
        let y = tf.translation.z.round() as isize;
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= tex_w || y >= tex_h {
            return None;
        }
        Some((x, y, terrain.0))
    });
    let road_iter = road_q.iter().map(|m| (m.tile_x as usize, m.tile_z as usize));

    raster_tiles_and_roads_to_rgba(
        data,
        tex_w,
        tex_h,
        tile_iter,
        road_iter,
        &mat_slices,
        reg_opt,
        fam_opt,
    );

    let chunk_geom: Vec<(bevy::math::IVec2, bevy::math::UVec2)> =
        chunk_geom_q.iter().map(|(c, m)| (c.coord, m.size)).collect();
    apply_shared_fire_heat_to_rgba(
        data.as_mut_slice(),
        tex_w,
        tex_h,
        &chunk_geom,
        &overlay.chunk_fire_heat,
    );
    *last_applied_revision = Some(rev);
}

fn simulation_minimap_egui_window(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    mut minimap_ui: ResMut<SimMinimapUiState>,
    fallback: Res<TileWorldFallbackState>,
    fire_atm: Option<Res<FireAtmosphereAggregate>>,
    mut tex_cache: Local<Option<(Handle<Image>, egui::TextureId)>>,
    mut fire_label_cache: Local<Option<(f32, f32, f32, String)>>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if !minimap_ui.open || fallback.sprite_entity.is_none() || fallback.image == Handle::default() {
        return Ok(());
    }

    let handle = fallback.image.clone();
    let tex_id = match tex_cache.as_ref() {
        Some((h, id)) if *h == handle => *id,
        _ => {
            let id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(handle.clone()));
            *tex_cache = Some((handle, id));
            id
        }
    };
    let w = fallback.last_w as f32;
    let h = fallback.last_h as f32;

    std_floating(egui::Window::new("Minimap"))
        .id(egui::Id::new("sim_minimap_fallback"))
        .default_size([320.0, 280.0])
        .min_size([180.0, 160.0])
        .open(&mut minimap_ui.open)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(egui::RichText::new("Overworld preview · close to hide").small().weak());
            if let Some(agg) = fire_atm.as_ref() {
                let s = agg.smoke_density.clamp(0.0, 1.0);
                let vl = agg.visibility_loss.clamp(0.0, 1.0);
                let heat = agg.heat_energy;
                let need_label = match fire_label_cache.as_ref() {
                    Some((ps, pvl, ph, _)) => {
                        (*ps - s).abs() > 0.01
                            || (*pvl - vl).abs() > 0.01
                            || (*ph - heat).abs() > 0.05
                            || agg.is_changed()
                    }
                    None => true,
                };
                let label = if need_label {
                    let t = format!(
                        "Fire extract: smoke {:.0}% · vis loss {:.0}% · heat {:.1}",
                        s * 100.0,
                        vl * 100.0,
                        heat
                    );
                    *fire_label_cache = Some((s, vl, heat, t.clone()));
                    t
                } else {
                    fire_label_cache.as_ref().unwrap().3.clone()
                };
                ui.label(egui::RichText::new(label).small());
                ui.add(
                    egui::ProgressBar::new(s)
                        .fill(egui::Color32::from_rgb(200, 90, 40))
                        .desired_width(ui.available_width())
                        .show_percentage(),
                );
            }
            let max_side = 280.0;
            let scale = (max_side / w.max(h).max(1.0)).min(1.0);
            let dw = (w * scale).max(64.0);
            let dh = (h * scale).max(64.0);
            ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, [dw, dh])));
        });
    Ok(())
}
