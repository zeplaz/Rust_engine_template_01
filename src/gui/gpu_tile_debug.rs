//! LOD / fire chunk debug: logical **instances** keyed by [`TileDebugViewId`], uploaded to GPU
//! storage and drawn as **one instanced pass** on the [`MainWorldCamera`](super::MainWorldCamera)
//! Core2d subgraph (`crate::render::gpu_tile_debug_draw`).

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;

use super::camera_focus_debug::{fire_chunk_coords_above_visual_eps, CameraFocusDebug, DEBUG_CHUNK_SPACING_WORLD};
use super::map_camera::{in_simulation_or_editor_map, MainWorldCamera, MapCameraDesired};
use super::{ViewAuthoritySystemSet, ViewId, ViewManager};
use super::tile_debug_types::{
    FireDebugOverride, TileDebugDrawGlobals, TileDebugInstance, TileDebugInstanceMap, TileDebugRenderHost,
    TileDebugViewId, TileGpuDebugSettings, tile_flags,
};
use crate::engine::BaseState;
use crate::render::{tactical_fire_visual, FireVisualFramesByView};
use crate::terrain::generation::Chunk;

pub struct GpuTileDebugPlugin;

impl Plugin for GpuTileDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileDebugInstanceMap>()
            .init_resource::<TileGpuDebugSettings>()
            .init_resource::<FireDebugOverride>()
            .init_resource::<TileDebugDrawGlobals>()
            .add_plugins((
                ExtractResourcePlugin::<TileDebugInstanceMap>::default(),
                ExtractResourcePlugin::<TileDebugDrawGlobals>::default(),
                ExtractComponentPlugin::<TileDebugRenderHost>::default(),
            ));
        crate::render::register_tile_debug_instance_storage_upload(app);
        crate::render::register_tile_debug_instanced_draw(app);
        app.add_systems(
            OnEnter(BaseState::Simulation),
            enable_tile_gpu_instanced_authoritative,
        )
        .add_systems(
            Update,
            (
                build_tile_debug_instances
                    .after(crate::gui::camera_focus_debug::update_camera_focus_debug)
                    .after(ViewAuthoritySystemSet::SyncViewManager),
                sync_tile_debug_draw_globals.after(build_tile_debug_instances),
            )
                .chain()
                .run_if(in_simulation_or_editor_map),
        );
    }
}

/// **TRIAGE-GPU-TILE-WGSL-001** — storage instanced WGSL present.
#[must_use]
pub fn triage_gpu_tile_wgsl_001_green() -> bool {
    let path = std::path::Path::new("assets/shaders/debug/tile_debug_instanced.wgsl");
    path.exists()
        && std::fs::read_to_string(path)
            .map(|s| s.contains("storage, read") && s.contains("tile_instance_color"))
            .unwrap_or(false)
}

/// IN-C06: simulation uses GPU instanced tile debug; gizmo path only when explicitly disabled.
fn enable_tile_gpu_instanced_authoritative(mut settings: ResMut<TileGpuDebugSettings>) {
    settings.use_batched_mesh_overlay = true;
}

pub fn build_tile_debug_instances(
    settings: Res<TileGpuDebugSettings>,
    fire_override: Res<FireDebugOverride>,
    debug: Res<CameraFocusDebug>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesired>,
    chunks: Query<&Chunk>,
    fire_by_view: Res<FireVisualFramesByView>,
    mut map: ResMut<TileDebugInstanceMap>,
) {
    map.per_view.clear();
    if !settings.use_batched_mesh_overlay || !debug.enabled {
        return;
    }
    let mut chunk_set = std::collections::HashSet::new();
    for c in &chunks {
        chunk_set.insert(c.coord);
    }
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    let fire_chunks = fire_chunk_coords_above_visual_eps(fire);

    let r = debug.overlay_radius_chunks.clamp(1, 12);
    let center = debug.focus_chunk;
    let half = DEBUG_CHUNK_SPACING_WORLD * 0.45;
    let base = half * 2.0;
    let cam_scale = view_manager
        .view(ViewId::WorldMain)
        .map(|v| v.camera.zoom.abs().max(0.001))
        .unwrap_or_else(|| desired.scale.x.abs().max(0.001));
    let lod_size_mul = match debug.lod_band {
        crate::gui::world_representation::WorldLodBand::LocalTactical => 0.85,
        crate::gui::world_representation::WorldLodBand::Operational => 0.95,
        crate::gui::world_representation::WorldLodBand::Strategic => 1.0,
        crate::gui::world_representation::WorldLodBand::Macro => 1.05,
    };
    let lod_u32 = match debug.lod_band {
        crate::gui::world_representation::WorldLodBand::LocalTactical => 0,
        crate::gui::world_representation::WorldLodBand::Operational => 1,
        crate::gui::world_representation::WorldLodBand::Strategic => 2,
        crate::gui::world_representation::WorldLodBand::Macro => 3,
    };

    let mut out = Vec::new();
    'outer: for dy in -r..=r {
        for dx in -r..=r {
            if out.len() >= settings.max_instances {
                break 'outer;
            }
            let tile = center + IVec2::new(dx, dy);
            let pos = Vec2::new(
                tile.x as f32 * DEBUG_CHUNK_SPACING_WORLD + DEBUG_CHUNK_SPACING_WORLD * 0.5,
                tile.y as f32 * DEBUG_CHUNK_SPACING_WORLD + DEBUG_CHUNK_SPACING_WORLD * 0.5,
            );
            let is_focus = tile == center;
            let mut size = base * lod_size_mul * if is_focus { 1.12 } else { 1.0 };
            if debug.screen_stabilize_lod_overlay {
                size /= cam_scale;
                size = size.clamp(6.0, DEBUG_CHUNK_SPACING_WORLD * 1.25);
            }
            let mut flags = 0u32;
            if is_focus {
                flags |= tile_flags::FOCUS;
            }
            if fire_chunks.contains(&tile) || fire_override.force_visible {
                flags |= tile_flags::FIRE;
            }
            if chunk_set.contains(&tile) {
                flags |= tile_flags::TERRAIN;
            }
            out.push(TileDebugInstance {
                world_pos: pos.to_array(),
                size,
                lod: lod_u32,
                flags,
            });
        }
    }

    map.per_view.insert(TileDebugViewId::WorldMain, out);
}

fn sync_tile_debug_draw_globals(
    mut globals: ResMut<TileDebugDrawGlobals>,
    settings: Res<TileGpuDebugSettings>,
    debug: Res<CameraFocusDebug>,
    footprint: Option<Res<crate::construction::FootprintTileWitness>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    map: Res<TileDebugInstanceMap>,
) {
    *globals = TileDebugDrawGlobals::default();
    let footprint_active = footprint.as_deref().is_some_and(|w| w.gpu_path_active);
    if !footprint_active && (!settings.use_batched_mesh_overlay || !debug.enabled) {
        return;
    }
    let Ok((camera, gt)) = cam_q.single() else {
        return;
    };
    let Some(rows) = map.per_view.get(&TileDebugViewId::WorldMain) else {
        return;
    };
    if rows.is_empty() {
        return;
    }
    let view_from_world = Mat4::from(gt.affine().inverse());
    globals.view_proj = camera.clip_from_view() * view_from_world;
    globals.instance_count = rows.len() as u32;
}
