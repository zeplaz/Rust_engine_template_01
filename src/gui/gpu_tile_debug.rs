//! LOD / fire chunk debug: logical **instances** keyed by [`TileDebugViewId`], uploaded to GPU
//! storage and drawn as **one instanced pass** on the [`MainWorldCamera`](super::MainWorldCamera)
//! Core2d subgraph (`crate::render::gpu_tile_debug_draw`).

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;

use super::camera_focus_debug::{CameraFocusDebug, DEBUG_CHUNK_SPACING_WORLD};
use super::map_camera::{in_simulation_or_editor_map, MainWorldCamera, MapCameraDesiredRes};
use super::{ViewAuthoritySystemSet, ViewId, ViewManager};
use super::tile_debug_types::{
    FireDebugOverride, TileDebugDrawGlobals, TileDebugInstance, TileDebugInstanceMap, TileDebugRenderHost,
    TileDebugViewId, TileGpuDebugSettings, tile_flags,
};
use crate::render::{tactical_fire_visual, FireVisualFramesByView};
use crate::systems::fire::ChunkSurfaceFire;
use crate::terrain::generation::{chunk_world_center, Chunk, ChunkCellMatrix};
use crate::render::sim_visual_extract::FIRE_VISUAL_ACTIVE_HEAT_EPS;

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
    let path = std::path::Path::new("assets/shaders/debug/tile_debug_instanced.wgsl");
    path.exists()
        && std::fs::read_to_string(path)
            .map(|s| s.contains("storage, read") && s.contains("tile_instance_color"))
            .unwrap_or(false)
}

pub fn build_tile_debug_instances(
    settings: Res<TileGpuDebugSettings>,
    fire_override: Res<FireDebugOverride>,
    debug: Res<CameraFocusDebug>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesiredRes>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    fire_chunks: Query<(&Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    fire_by_view: Res<FireVisualFramesByView>,
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
    let fire = tactical_fire_visual(fire_by_view.as_ref());
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
    if debug.enabled {
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

    for row in &fire.instances {
        if out.len() >= settings.max_instances {
            break;
        }
        if row.heat() < FIRE_VISUAL_ACTIVE_HEAT_EPS && !fire_override.force_visible {
            continue;
        }
        let pos = Vec2::new(row.world_xyz_radius.x, row.world_xyz_radius.y);
        let heat = row.heat().clamp(0.0, 1.0);
        let mut marker = 3.0 + heat * 5.0;
        if debug.screen_stabilize_lod_overlay {
            marker /= cam_scale;
            marker = marker.clamp(2.0, 12.0);
        }
        out.push(TileDebugInstance {
            world_pos: pos.to_array(),
            size: marker,
            lod: lod_u32,
            flags: tile_flags::FIRE,
        });
    }

    for (chunk, matrix, fire) in &fire_chunks {
        if out.len() >= settings.max_instances {
            break;
        }
        if fire.heat <= 0.02 && !fire_override.force_visible {
            continue;
        }
        let pos = chunk_world_center(chunk.coord, matrix.size);
        let heat = fire.heat.clamp(0.0, 1.0);
        let mut marker = (matrix.size.x.max(matrix.size.y) as f32) * 0.35 + heat * 4.0;
        if debug.screen_stabilize_lod_overlay {
            marker /= cam_scale;
            marker = marker.clamp(2.0, 14.0);
        }
        out.push(TileDebugInstance {
            world_pos: pos.to_array(),
            size: marker,
            lod: lod_u32,
            flags: tile_flags::FIRE,
        });
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
