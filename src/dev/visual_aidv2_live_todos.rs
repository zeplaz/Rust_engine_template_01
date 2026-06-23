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

#[derive(Resource, Clone, Debug)]
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
    try_commit_visual_aidv2_live_proof_in_capture(world);
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

/// Consecutive readiness evals at 6/6 before writing live witness (not lib fixture).
pub const VA2_LIVE_PROOF_GREEN_STREAK: u32 = 5;

/// Write `visual_aidv2_live.json` when board is green during `--test visual` — not gated on FINISH-UX-06.
fn try_commit_visual_aidv2_live_proof_in_capture(world: &mut World) {
    let full_capture = world
        .get_resource::<crate::engine::EngineLaunchArgs>()
        .is_some_and(|l| l.full_capture_active());
    if !full_capture {
        return;
    }
    let visual_auto_exit = world
        .get_resource::<crate::engine::EngineLaunchArgs>()
        .is_some_and(|l| l.visual_auto_exit);
    let (done, board_clone) = {
        let Some(board) = world.get_resource::<VisualAidV2LiveTodoBoard>() else {
            return;
        };
        let done = board
            .status
            .iter()
            .filter(|s| **s == TodoStatus::Done)
            .count();
        (done, board.clone())
    };

    let Some(mut harness) = world.get_resource_mut::<VisualAidV2HarnessState>() else {
        return;
    };
    if harness.live_proof_written {
        return;
    }
    if done == VISUAL_AID_V2_TODOS.len() {
        harness.green_streak = harness.green_streak.saturating_add(1);
    } else {
        harness.green_streak = 0;
        return;
    }
    if harness.green_streak < VA2_LIVE_PROOF_GREEN_STREAK {
        return;
    }
    drop(harness);

    let Some(witness) = world.get_resource::<VisualAidV2Witness>().cloned() else {
        return;
    };
    let Some(hud) = world.get_resource::<HudPanelStateWitness>().cloned() else {
        return;
    };
    if write_visual_aidv2_live_proof(&board_clone, &witness, &hud) {
        let Some(mut harness) = world.get_resource_mut::<VisualAidV2HarnessState>() else {
            return;
        };
        harness.live_proof_written = true;
        harness.request_visual_exit = visual_auto_exit;
        info!(
            target: "visual_aidv2_live_todos",
            path = VISUAL_AID_V2_LIVE_JSON,
            done,
            total = VISUAL_AID_V2_TODOS.len(),
            "wrote visual aid v2 live proof (board green, independent of FINISH-UX-06)"
        );
    }
}

/// VA2-HARNESS-CLOSE-001 — drives ESC + build ghost + macro icon probe during `--test visual`.
#[derive(Resource, Clone, Debug, Default)]
pub struct VisualAidV2HarnessState {
    pub esc_armed: bool,
    pub esc_injected: bool,
    pub build_seeded: bool,
    pub macro_icon_probe: bool,
    /// Live `--test visual` witness written (`lib_fixture: false`).
    pub live_proof_written: bool,
    /// Consecutive evals at 6/6 before [`try_commit_visual_aidv2_live_proof_in_capture`].
    pub green_streak: u32,
    /// Arm graceful GPU exit after live proof (when `visual_auto_exit`).
    pub request_visual_exit: bool,
}

pub const VISUAL_AID_V2_LIVE_JSON: &str = "debug_runs/visual_aidv2_live.json";

fn todo_status_label(st: TodoStatus) -> &'static str {
    match st {
        TodoStatus::Open => "Open",
        TodoStatus::InProgress => "InProgress",
        TodoStatus::Done => "Done",
    }
}

/// JSON snapshot for `debug_runs/visual_aidv2_live.json`.
pub fn visual_aidv2_live_board_json(
    board: &VisualAidV2LiveTodoBoard,
    witness: &VisualAidV2Witness,
    hud: &HudPanelStateWitness,
) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = VISUAL_AID_V2_TODOS
        .iter()
        .zip(board.status.iter())
        .map(|(row, st)| {
            serde_json::json!({
                "id": row.id,
                "status": todo_status_label(*st),
                "goal": row.goal,
                "runtime_check": row.runtime_check,
            })
        })
        .collect();
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    serde_json::json!({
        "board": "VISUAL_AID_V2",
        "done": done,
        "total": VISUAL_AID_V2_TODOS.len(),
        "green": done == VISUAL_AID_V2_TODOS.len(),
        "rows": rows,
        "witness": {
            "hud_panel_state_cycle_ok": hud.cycle_ok,
            "footprint_tile_overlay_ok": witness.footprint_tile_overlay_ok,
            "footprint_tile_count": witness.footprint_tile_count,
            "tile_readability_clamp_active": witness.tile_readability_clamp_active,
            "lod_building_policy_differs_across_bands": witness.lod_building_policy_differs_across_bands,
            "zoom_visual_bias_active": witness.zoom_visual_bias_active,
            "macro_icon_instance_count": witness.macro_icon_instance_count,
        },
    })
}

/// Write VA2 board proof (called from `--test visual` proof commit).
pub fn write_visual_aidv2_live_proof(
    board: &VisualAidV2LiveTodoBoard,
    witness: &VisualAidV2Witness,
    hud: &HudPanelStateWitness,
) -> bool {
    write_visual_aidv2_live_proof_with_grade(board, witness, hud, false)
}

/// Lib fixture: harness-target witnesses → board 6/6 (CI without display).
pub fn refresh_visual_aidv2_harness_lib_witness() -> bool {
    let mut board = VisualAidV2LiveTodoBoard::default();
    let witness = VisualAidV2Witness {
        hud_panel_state_cycle_ok: true,
        footprint_tile_overlay_ok: true,
        footprint_tile_count: 4,
        tile_readability_clamp_active: true,
        lod_building_policy_differs_across_bands: true,
        zoom_visual_bias_active: true,
        macro_icon_instance_count: 1,
        ..Default::default()
    };
    let hud = HudPanelStateWitness {
        cycle_ok: true,
        last_esc_collapsed: true,
    };
    let readability = TileReadabilityWitness {
        clamp_active: true,
        ..Default::default()
    };
    let ctx = VisualAidV2PredicateInputs {
        witness: witness.clone(),
        hud: hud.clone(),
        readability,
    };
    for (i, row) in VISUAL_AID_V2_TODOS.iter().enumerate() {
        if predicate_done(row.id, &ctx) {
            if let Some(s) = board.status.get_mut(i) {
                *s = TodoStatus::Done;
            }
        }
    }
    write_visual_aidv2_live_proof_with_grade(&board, &witness, &hud, true)
}

fn write_visual_aidv2_live_proof_with_grade(
    board: &VisualAidV2LiveTodoBoard,
    witness: &VisualAidV2Witness,
    hud: &HudPanelStateWitness,
    lib_fixture: bool,
) -> bool {
    let mut body = visual_aidv2_live_board_json(board, witness, hud);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("lib_fixture".into(), serde_json::json!(lib_fixture));
        obj.insert(
            "proof_grade".into(),
            serde_json::json!(if lib_fixture {
                "lib_harness_fixture"
            } else {
                "visual_capture"
            }),
        );
    }
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "VISUAL_AID_V2",
        "visual_aidv2_live_todos",
        VISUAL_AID_V2_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VISUAL_AID_V2_LIVE_JSON, payload)
}

pub fn register_visual_aidv2_runtime_hooks(app: &mut App) {
    app.init_resource::<VisualAidV2LiveTodoBoard>()
        .init_resource::<VisualAidV2Witness>()
        .init_resource::<VisualAidV2HarnessState>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_aid_v2_registry_has_six_rows() {
        assert_eq!(VISUAL_AID_V2_TODOS.len(), 6);
    }

    #[test]
    fn va2_harness_predicates_close_all_six_rows() {
        let ctx = VisualAidV2PredicateInputs {
            witness: VisualAidV2Witness {
                footprint_tile_overlay_ok: true,
                footprint_tile_count: 4,
                tile_readability_clamp_active: true,
                lod_building_policy_differs_across_bands: true,
                zoom_visual_bias_active: true,
                macro_icon_instance_count: 1,
                ..Default::default()
            },
            hud: HudPanelStateWitness {
                cycle_ok: true,
                ..Default::default()
            },
            readability: TileReadabilityWitness {
                clamp_active: true,
                ..Default::default()
            },
        };
        for row in VISUAL_AID_V2_TODOS {
            assert!(
                predicate_done(row.id, &ctx),
                "expected Done for {}",
                row.id
            );
        }
    }

    #[test]
    fn va2_harness_lib_witness_writes_fixture_json() {
        assert!(super::refresh_visual_aidv2_harness_lib_witness());
        let raw = std::fs::read_to_string(super::VISUAL_AID_V2_LIVE_JSON).expect("witness path");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
        assert_eq!(v["green"], serde_json::json!(true));
        assert_eq!(v["done"], serde_json::json!(6));
        assert_eq!(v["lib_fixture"], serde_json::json!(true));
    }
}
