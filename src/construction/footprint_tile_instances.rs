//! Placement footprint fill is the egui ghost in [`super::visual_authority`].
//!
//! This module is not a [`TileDebugInstanceMap`](crate::gui::TileDebugInstanceMap) writer.
//! The old GPU bridge used `tile.y` as the second world axis and stacked quads at the corner.

use bevy::prelude::*;

use super::build_state::BuildGhostState;
use super::build_strip::{BuildStripState, ToolContext};
use super::visual_authority::ConstructionVisualRequests;

#[derive(Resource, Clone, Debug, Default)]
pub struct FootprintTileWitness {
    pub gpu_path_active: bool,
    pub egui_path_active: bool,
    pub instance_count: u32,
}

/// Records that an active ghost has footprint tiles for the egui path.
/// Does not append GPU tile-debug instances.
pub fn push_footprint_tile_instances(
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    requests: Res<ConstructionVisualRequests>,
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
}

pub fn sync_visual_aidv2_footprint_witness(
    footprint: Res<FootprintTileWitness>,
    mut board: ResMut<crate::dev::VisualAidV2Witness>,
) {
    board.footprint_tile_overlay_ok = footprint.gpu_path_active;
    board.footprint_tile_count = footprint.instance_count;
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::construction::visual_authority::{
        ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest,
    };
    use crate::gui::{
        tile_flags, CameraFocusDebug, TileDebugInstance, TileDebugInstanceMap, TileDebugViewId,
        TileGpuDebugSettings,
    };
    use crate::strategic::BuildSiteTile;

    use super::push_footprint_tile_instances;
    use crate::construction::build_state::BuildGhostState;
    use crate::construction::build_strip::{BuildStripState, ToolContext};

    #[test]
    fn armed_ghost_does_not_push_footprint_gpu_quads() {
        let mut app = App::new();
        app.insert_resource(BuildStripState {
            active: ToolContext::Industry,
        });
        let mut ghost = BuildGhostState::default();
        ghost.origin = Some(BuildSiteTile { x: 0, z: 0 });
        app.insert_resource(ghost);
        let mut requests = ConstructionVisualRequests::default();
        requests.footprint_tiles.push(FootprintTileRequest {
            tile: IVec2::ZERO,
            color_kind: FootprintTileColorKind::Valid,
            weight: 1.0,
            fill_override: None,
        });
        app.insert_resource(requests);
        app.insert_resource(CameraFocusDebug {
            enabled: true,
            ..CameraFocusDebug::default()
        });
        app.insert_resource(TileGpuDebugSettings {
            use_batched_mesh_overlay: true,
            ..TileGpuDebugSettings::default()
        });
        let mut map = TileDebugInstanceMap::default();
        map.per_view.insert(
            TileDebugViewId::WorldMain,
            vec![TileDebugInstance {
                world_pos: [0.5, 0.5],
                size: 0.92,
                lod: 0,
                flags: 0,
            }],
        );
        app.insert_resource(map);
        app.init_resource::<super::FootprintTileWitness>();
        app.add_systems(Update, push_footprint_tile_instances);
        app.update();

        let witness = app.world().resource::<super::FootprintTileWitness>();
        assert!(!witness.gpu_path_active);
        assert!(witness.egui_path_active);
        assert_eq!(witness.instance_count, 0);
        let map = app.world().resource::<TileDebugInstanceMap>();
        let rows = map
            .per_view
            .get(&TileDebugViewId::WorldMain)
            .expect("world bucket");
        assert_eq!(rows.len(), 1);
        assert!(rows.iter().all(|row| {
            row.flags
                & (tile_flags::FOOTPRINT_VALID
                    | tile_flags::FOOTPRINT_RISKY
                    | tile_flags::FOOTPRINT_INVALID)
                == 0
        }));
    }
}
