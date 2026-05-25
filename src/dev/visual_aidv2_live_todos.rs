//! Visual Aid v2 live board (`VISUAL-AID-V2-*`) — parallel to Stage 5, not gated on FULL_APP exit.
//!
//! Spec: [`super::visual_aidv2.md`](super::visual_aidv2.md) · orchestrator: `prompts/guides/visual_aidv2_runbook_v1.md`.

use bevy::log::info;
use bevy::prelude::{App, Resource, World};

use crate::gui::{TileReadabilityWitness};
use crate::gui::hud::HudPanelStateWitness;
use crate::render::{
    stage5_readiness_passes, AppStage5ReadinessReport, Stage5ReadinessProfile,
};

pub use super::stage5_live_todos::TodoStatus;

/// One Visual Aid v2 row.
#[derive(Clone, Copy, Debug)]
pub struct VisualAidV2LiveTodo {
    pub id: &'static str,
    pub status: TodoStatus,
    pub file: &'static str,
    pub goal: &'static str,
    pub runtime_check: &'static str,
}

pub static VISUAL_AID_V2_TODOS: &[VisualAidV2LiveTodo] = &[
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-01",
        status: TodoStatus::Open,
        file: "src/gui/hud/panel_state.rs",
        goal: "HUD cycles Collapsed/Peek/Expanded/Pinned; ESC collapses unpinned panels.",
        runtime_check: "HudPanelStateWitness::cycle_ok + unit tests hud_panel_state.",
    },
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-02",
        status: TodoStatus::Open,
        file: "src/construction/footprint_tile_instances.rs",
        goal: "Building ghost emits GPU tile footprint on sim map (TileDebug WorldMain).",
        runtime_check: "FootprintTileWitness::gpu_path_active && footprint_tile_count > 0.",
    },
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-03",
        status: TodoStatus::Open,
        file: "src/gui/tile_readability.rs",
        goal: "screen_pixels_per_tile >= min under zoom-out (LOD zoom floor).",
        runtime_check: "TileReadabilityWitness::clamp_active.",
    },
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-04",
        status: TodoStatus::Open,
        file: "src/gui/representation_policy.rs",
        goal: "Band change alters building_visual_simplified on RepresentationResult.",
        runtime_check: "VisualAidV2Witness::lod_building_policy_differs_across_bands.",
    },
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-05",
        status: TodoStatus::Open,
        file: "src/gui/map_camera.rs",
        goal: "ZoomVisualBias active; simulation tile size unchanged.",
        runtime_check: "VisualAidV2Witness::zoom_visual_bias_active.",
    },
    VisualAidV2LiveTodo {
        id: "VISUAL-AID-V2-06",
        status: TodoStatus::Open,
        file: "src/gui/strategic_icon_instances.rs",
        goal: "Macro band publishes icon instances via projection graph scaffold.",
        runtime_check: "VisualAidV2Witness::macro_icon_instance_count > 0 (scaffold).",
    },
];

/// Runtime witness written each frame for predicates + `debug_runs/visual_aidv2_live.json`.
#[derive(Resource, Clone, Debug, Default)]
pub struct VisualAidV2Witness {
    pub hud_panel_state_cycle_ok: bool,
    pub footprint_tile_overlay_ok: bool,
    pub footprint_tile_count: u32,
    pub tile_readability_clamp_active: bool,
    pub screen_pixels_per_tile: f32,
    pub lod_building_policy_differs_across_bands: bool,
    pub zoom_visual_bias_active: bool,
    pub macro_icon_instance_count: u32,
}

#[derive(Resource, Debug)]
pub struct VisualAidV2LiveTodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for VisualAidV2LiveTodoBoard {
    fn default() -> Self {
        Self {
            status: vec![TodoStatus::Open; VISUAL_AID_V2_TODOS.len()],
        }
    }
}

impl VisualAidV2LiveTodoBoard {
    pub fn mark(&mut self, id: &str, next: TodoStatus) {
        for (i, row) in VISUAL_AID_V2_TODOS.iter().enumerate() {
            if row.id == id {
                if let Some(s) = self.status.get_mut(i) {
                    *s = next;
                }
                return;
            }
        }
    }
}

/// Inputs for predicate sync (read-only resources).
pub struct VisualAidV2PredicateInputs {
    pub witness: VisualAidV2Witness,
    pub hud: HudPanelStateWitness,
    pub readability: TileReadabilityWitness,
}

fn build_predicate_inputs(world: &World) -> VisualAidV2PredicateInputs {
    VisualAidV2PredicateInputs {
        witness: world
            .get_resource::<VisualAidV2Witness>()
            .cloned()
            .unwrap_or_default(),
        hud: world
            .get_resource::<HudPanelStateWitness>()
            .cloned()
            .unwrap_or_default(),
        readability: world
            .get_resource::<TileReadabilityWitness>()
            .cloned()
            .unwrap_or_default(),
    }
}

fn predicate_done(id: &str, ctx: &VisualAidV2PredicateInputs) -> bool {
    match id {
        "VISUAL-AID-V2-01" => ctx.hud.cycle_ok,
        "VISUAL-AID-V2-02" => {
            ctx.witness.footprint_tile_overlay_ok
                && ctx.witness.footprint_tile_count > 0
        }
        "VISUAL-AID-V2-03" => ctx.readability.clamp_active,
        "VISUAL-AID-V2-04" => ctx.witness.lod_building_policy_differs_across_bands,
        "VISUAL-AID-V2-05" => ctx.witness.zoom_visual_bias_active,
        "VISUAL-AID-V2-06" => ctx.witness.macro_icon_instance_count > 0,
        _ => false,
    }
}

/// Reconcile board from witnesses (after FULL_APP eval; read-only if readiness red).
pub fn sync_visual_aidv2_todo_board_predicates(world: &mut World) {
    let ctx = build_predicate_inputs(world);
    let Some(mut board) = world.get_resource_mut::<VisualAidV2LiveTodoBoard>() else {
        return;
    };
    for (i, row) in VISUAL_AID_V2_TODOS.iter().enumerate() {
        let Some(s) = board.status.get_mut(i) else {
            continue;
        };
        if predicate_done(row.id, &ctx) {
            *s = TodoStatus::Done;
        } else if *s == TodoStatus::Done {
            *s = TodoStatus::InProgress;
        }
    }
}

/// Hook from [`super::stage5_live_todos::hook_post_readiness_evaluate`].
pub fn hook_post_readiness_visual_aidv2(world: &mut World) {
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let report = world.resource::<AppStage5ReadinessReport>().clone();
    if !stage5_readiness_passes(&report) {
        return;
    }
    sync_visual_aidv2_todo_board_predicates(world);
    if let (Some(board), witness) = (
        world.get_resource::<VisualAidV2LiveTodoBoard>(),
        world.get_resource::<VisualAidV2Witness>(),
    ) {
        let done = board
            .status
            .iter()
            .filter(|s| **s == TodoStatus::Done)
            .count();
        if done > 0 {
            info!(
                target: "visual_aidv2_live_todos",
                "VISUAL_AID_V2_BOARD done={done}/{} footprint_ok={} readability={} icons={}",
                VISUAL_AID_V2_TODOS.len(),
                witness.map(|w| w.footprint_tile_overlay_ok).unwrap_or(false),
                witness
                    .map(|w| w.tile_readability_clamp_active)
                    .unwrap_or(false),
                witness.map(|w| w.macro_icon_instance_count).unwrap_or(0),
            );
        }
    }
}

pub fn register_visual_aidv2_runtime_hooks(app: &mut App) {
    app.init_resource::<VisualAidV2LiveTodoBoard>()
        .init_resource::<VisualAidV2Witness>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_aid_v2_registry_has_six_rows() {
        assert_eq!(VISUAL_AID_V2_TODOS.len(), 6);
    }
}
