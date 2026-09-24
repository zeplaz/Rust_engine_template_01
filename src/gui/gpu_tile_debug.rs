//! LOD / fire chunk debug: logical **instances** keyed by [`TileDebugViewId`], uploaded to GPU
//! storage and drawn as **one instanced pass** on the [`MainWorldCamera`](super::MainWorldCamera)
//! Core2d subgraph (`crate::render::pipelines::gpu_tile_debug_draw`).

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;

use super::camera_focus_debug::{CameraFocusDebug, DEBUG_CHUNK_SPACING_WORLD};
use super::tactical::map_camera::{in_simulation_or_editor_map, MainWorldCamera, MapCameraDesiredRes};
use super::{ViewAuthoritySystemSet, ViewId, ViewManager};
use super::tile_debug_types::{
    FireDebugOverride, TileDebugDrawGlobals, TileDebugInstance, TileDebugInstanceMap, TileDebugRenderHost,
    TileDebugViewId, TileGpuDebugSettings, tile_flags,
};
use crate::terrain::generation::{chunk_world_center, Chunk, ChunkCellMatrix};

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
            Update,
            build_tile_debug_instances
                .after(crate::gui::camera_focus_debug::update_camera_focus_debug)
                .after(ViewAuthoritySystemSet::SyncViewManager)
                .run_if(in_simulation_or_editor_map),
        )
        .add_systems(
            PostUpdate,
            sync_tile_debug_draw_globals
                .after(crate::construction::footprint_tile_instances::push_footprint_tile_instances)
                .after(crate::construction::site_phase_tile_instances::push_site_phase_tile_instances)
                .run_if(in_simulation_or_editor_map),
        );
    }
}

/// **TRIAGE-GPU-TILE-WGSL-001** — storage instanced WGSL present.
#[must_use]
pub fn triage_gpu_tile_wgsl_001_green() -> bool {
    let wgsl = include_str!("../../assets/shaders/debug/tile_debug_instanced.wgsl");
    wgsl.contains("storage, read") && wgsl.contains("tile_instance_color")
}

pub fn build_tile_debug_instances(
    settings: Res<TileGpuDebugSettings>,
    debug: Res<CameraFocusDebug>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesiredRes>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    mut map: ResMut<TileDebugInstanceMap>,
) {
    map.per_view.clear();
    if !settings.use_batched_mesh_overlay {
        return;
    }
    let mut chunk_set = std::collections::HashSet::new();
    let mut chunk_sizes: std::collections::HashMap<bevy::math::IVec2, bevy::math::UVec2> =
        std::collections::HashMap::new();
    for (c, m) in &chunks {
        chunk_set.insert(c.coord);
        chunk_sizes.insert(c.coord, m.size);
    }
    let default_size = chunk_sizes
        .values()
        .next()
        .copied()
        .unwrap_or(bevy::math::UVec2::splat(32));

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
    // LOD yellow/green squares: sole gate is CameraFocusDebug::lod_paint_active
    // (enabled ∧ CAMERA_FOCUS_DEBUG=1). Never emit FOCUS/TERRAIN into the playfield otherwise.
    if debug.lod_paint_active() {
        let r = debug.overlay_radius_chunks.clamp(1, 12);
        let center = debug.focus_chunk;
        'outer: for dy in -r..=r {
            for dx in -r..=r {
                if out.len() >= settings.max_instances {
                    break 'outer;
                }
                let tile = center + IVec2::new(dx, dy);
                let size = chunk_sizes.get(&tile).copied().unwrap_or(default_size);
                let pos = chunk_world_center(tile, size);
                let is_focus = tile == center;
                let mut extent = (size.x.max(size.y) as f32) * lod_size_mul;
                if is_focus {
                    extent *= 1.08;
                }
                if debug.screen_stabilize_lod_overlay {
                    extent /= cam_scale;
                    extent = extent.clamp(6.0, DEBUG_CHUNK_SPACING_WORLD * 1.25);
                }
                let mut flags = 0u32;
                if is_focus {
                    flags |= tile_flags::FOCUS;
                }
                if chunk_set.contains(&tile) {
                    flags |= tile_flags::TERRAIN;
                }
                out.push(TileDebugInstance {
                    world_pos: pos.to_array(),
                    size: extent,
                    lod: lod_u32,
                    flags,
                });
            }
        }
    }

    map.per_view.insert(TileDebugViewId::WorldMain, out);
}

pub fn sync_tile_debug_draw_globals(
    mut globals: ResMut<TileDebugDrawGlobals>,
    settings: Res<TileGpuDebugSettings>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    map: Res<TileDebugInstanceMap>,
) {
    *globals = TileDebugDrawGlobals::default();
    if !settings.use_batched_mesh_overlay {
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
