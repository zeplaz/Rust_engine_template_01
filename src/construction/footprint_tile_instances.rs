//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner visual_aidv2_agent
//! @orchestrator-do-not-cleanup
//! @orchestrator-note VA2 footprint tile bridge until overlay channel owns placement fill
//!
//! Pushes construction footprint tiles into [`TileDebugInstanceMap`] (WorldMain).

use bevy::prelude::*;

use crate::gui::{
    tile_flags, CameraFocusDebug, ScaffoldContract, TileDebugInstance, TileDebugInstanceMap,
    TileDebugViewId, TileGpuDebugSettings,
};
use super::build_state::BuildGhostState;
use super::build_strip::{BuildStripState, ToolContext};
use super::visual_authority::{FootprintTileColorKind, ConstructionVisualRequests};

/// Transitional scaffold — exit when overlay channel owns placement fill.
pub const FOOTPRINT_TILE_SCAFFOLD: ScaffoldContract = ScaffoldContract {
    owner: "construction/footprint_tile_instances",
    intended_replacement: "RepresentationResult overlay channel",
    exit_condition: "Footprint tiles published via SharedOverlayFieldBuffers or projection graph slice",
    removal_trigger: "duplicate TileDebug producer for placement",
};

#[derive(Resource, Clone, Debug, Default)]
pub struct FootprintTileWitness {
    pub gpu_path_active: bool,
    pub egui_path_active: bool,
    pub instance_count: u32,
}

fn footprint_flag_for_kind(kind: FootprintTileColorKind) -> u32 {
    match kind {
        FootprintTileColorKind::Valid => tile_flags::FOOTPRINT_VALID,
        FootprintTileColorKind::Risky => tile_flags::FOOTPRINT_RISKY,
        FootprintTileColorKind::Invalid => tile_flags::FOOTPRINT_INVALID,
    }
}

/// Append footprint instances after LOD debug builder (same view bucket).
pub fn push_footprint_tile_instances(
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    requests: Res<ConstructionVisualRequests>,
    debug: Res<CameraFocusDebug>,
    tile_settings: Res<TileGpuDebugSettings>,
    mut map: ResMut<TileDebugInstanceMap>,
    mut witness: ResMut<FootprintTileWitness>,
) {
    witness.gpu_path_active = false;
    witness.egui_path_active = false;
    witness.instance_count = 0;
    if strip.active == ToolContext::None || ghost.origin.is_none() {
        return;
    }
    if requests.footprint_tiles.is_empty() {
        return;
    }
    witness.egui_path_active = true;
    if !tile_settings.use_batched_mesh_overlay || !debug.enabled {
        return;
    }
    const FOOTPRINT_TILE_WORLD: f32 = 0.92;
    let rows = map
        .per_view
        .entry(TileDebugViewId::WorldMain)
        .or_default();
    for tile in &requests.footprint_tiles {
        rows.push(TileDebugInstance {
            world_pos: [
                tile.tile.x as f32 + 0.5,
                tile.tile.y as f32 + 0.5,
            ],
            size: FOOTPRINT_TILE_WORLD,
            lod: 0,
            flags: footprint_flag_for_kind(tile.color_kind),
        });
    }
    witness.gpu_path_active = true;
    witness.egui_path_active = false;
    witness.instance_count = requests.footprint_tiles.len() as u32;
    let _ = FOOTPRINT_TILE_SCAFFOLD.is_declared();
}

pub fn sync_visual_aidv2_footprint_witness(
    footprint: Res<FootprintTileWitness>,
    mut board: ResMut<crate::dev::VisualAidV2Witness>,
) {
    board.footprint_tile_overlay_ok = footprint.gpu_path_active;
    board.footprint_tile_count = footprint.instance_count;
}
