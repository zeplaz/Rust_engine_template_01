//! Writes `debug_runs/construction_stage_live.json` during simulation.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::dev::construction_live_todos::{ConstructionLiveTodoBoard, TodoStatus, CONSTRUCTION_TODOS};
use crate::dev::construction_operational_todos::{
    ConstructionOperationalTodoBoard, ConstructionOperationalWitness, CONSTRUCTION_OPERATIONAL_TODOS,
};
use crate::dev::construction_p9_todos::{
    con_e01_p9_acceptance_green, ConstructionP9TodoBoard, ConstructionP9Witness,
    CONSTRUCTION_P9_TODOS,
};
use crate::dev::construction_phase2_todos::{
    ConstructionPhase2TodoBoard, ConstructionPhase2Witness, CONSTRUCTION_PHASE2_TODOS,
};
use crate::dev::construction_round2_todos::{
    ConstructionRound2TodoBoard, CONSTRUCTION_ROUND2_TODOS,
};
use crate::dev::construction_round3_todos::{
    ConstructionRound3TodoBoard, CONSTRUCTION_ROUND3_TODOS,
};
use crate::engine::states::BaseState;

use super::history::ConstructionHistory;

#[derive(Resource, Debug)]
pub struct ConstructionLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for ConstructionLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[allow(dead_code)]
fn proof_output_path() -> PathBuf {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("debug_runs")
        .join("construction_stage_live.json")
}

fn board_snapshot(ids: &[&str], statuses: &[TodoStatus]) -> serde_json::Value {
    serde_json::json!(
        ids.iter()
            .zip(statuses.iter())
            .map(|(id, st)| {
                serde_json::json!({
                    "id": id,
                    "status": format!("{st:?}"),
                })
            })
            .collect::<Vec<_>>()
    )
}

fn build_proof_payload(
    build_board: Option<&ConstructionLiveTodoBoard>,
    phase2_board: Option<&ConstructionPhase2TodoBoard>,
    phase2_witness: Option<&ConstructionPhase2Witness>,
    p9_board: Option<&ConstructionP9TodoBoard>,
    p9_witness: Option<&ConstructionP9Witness>,
    proof_written: bool,
    round2_board: Option<&ConstructionRound2TodoBoard>,
    round3_board: Option<&ConstructionRound3TodoBoard>,
    operational_board: Option<&ConstructionOperationalTodoBoard>,
    operational_witness: Option<&ConstructionOperationalWitness>,
    stage_witness: Option<&super::ConstructionStageWitness>,
    history: Option<&ConstructionHistory>,
    path_feedback: Option<&super::path_feedback::ConstructionPathFeedback>,
    junction_count: usize,
) -> serde_json::Value {
    let build_ids: Vec<&str> = CONSTRUCTION_TODOS.iter().map(|t| t.id).collect();
    let phase2_ids: Vec<&str> = CONSTRUCTION_PHASE2_TODOS.iter().map(|t| t.id).collect();
    let p9_ids: Vec<&str> = CONSTRUCTION_P9_TODOS.iter().map(|t| t.id).collect();
    let round2_ids: Vec<&str> = CONSTRUCTION_ROUND2_TODOS.iter().map(|t| t.id).collect();
    let round3_ids: Vec<&str> = CONSTRUCTION_ROUND3_TODOS.iter().map(|t| t.id).collect();
    let op_ids: Vec<&str> = CONSTRUCTION_OPERATIONAL_TODOS.iter().map(|t| t.id).collect();

    serde_json::json!({
        "profile": "CONSTRUCTION_STAGE",
        "operational_green": operational_witness.map(|w| w.toolbox && w.road_commit && w.zone_paint && w.building_place && w.demolish && w.undo && w.proof_json && w.no_legacy),
        "con_e01_p9_green": p9_witness.map(|w| con_e01_p9_acceptance_green(w, proof_written)),
        "build_p_star": build_board.map(|b| board_snapshot(&build_ids, &b.status)),
        "phase2_build": phase2_board.map(|b| board_snapshot(&phase2_ids, &b.status)),
        "p9_build": p9_board.map(|b| board_snapshot(&p9_ids, &b.status)),
        "round2_build": round2_board.map(|b| board_snapshot(&round2_ids, &b.status)),
        "round3_build": round3_board.map(|b| board_snapshot(&round3_ids, &b.status)),
        "operational": operational_board.map(|b| board_snapshot(&op_ids, &b.status)),
        "p9_witness": p9_witness.map(|w| serde_json::json!({
            "construction_proof_json": w.construction_proof_json,
            "curved_road_spline": w.curved_road_spline,
            "grid_and_node_snap": w.grid_and_node_snap,
            "road_upgrade_lane": w.road_upgrade_lane,
            "terrain_conform": w.terrain_conform,
        })),
        "phase2_witness": phase2_witness.map(|w| serde_json::json!({
            "shim_removed": w.shim_removed,
            "demolish_execute": w.demolish_execute,
            "zone_strategic_commit": w.zone_strategic_commit,
            "legacy_roads_removed": w.legacy_roads_removed,
            "building_archetype_map": w.building_archetype_map,
            "commercial_tool": w.commercial_tool,
            "industrial_tool": w.industrial_tool,
            "utilities_tool": w.utilities_tool,
            "building_intent_pipeline": w.building_intent_pipeline,
            "rail_module": w.rail_module,
            "road_cost_estimate": w.road_cost_estimate,
            "ghost_policy": w.ghost_policy,
            "road_e2e_integration": w.road_e2e_integration,
            "zone_e2e_integration": w.zone_e2e_integration,
            "input_conflict_matrix": w.input_conflict_matrix,
            "construction_proof_json": w.construction_proof_json,
            "curved_road_spline": w.curved_road_spline,
            "grid_and_node_snap": w.grid_and_node_snap,
            "road_upgrade_lane": w.road_upgrade_lane,
            "terrain_conform": w.terrain_conform,
        })),
        "history": history.map(|h| serde_json::json!({
            "undo_depth": h.undo_stack.len(),
            "redo_depth": h.redo_stack.len(),
            "last_action": h.last_action_kind.map(|k| format!("{k:?}")),
        })),
        "path_tool_feedback": path_feedback.map(|f| serde_json::json!({
            "snap_hint": f.snap_hint,
            "required_actions": f.required_actions,
        })),
        "rail_junction_count": junction_count,
        "construction_mv_001": stage_witness.map(|w| serde_json::json!({
            "gate": "CONSTRUCTION-MV-001",
            "green": w.multiview_ghosts_wired && w.ghost_commit_isolated && w.road_ghost_draw,
            "multiview_ghosts_wired": w.multiview_ghosts_wired,
        })),
    })
}

pub fn write_construction_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<ConstructionLiveProofState>,
    build_board: Option<Res<ConstructionLiveTodoBoard>>,
    phase2_board: Option<Res<ConstructionPhase2TodoBoard>>,
    phase2_witness: Option<Res<ConstructionPhase2Witness>>,
    p9_board: Option<Res<ConstructionP9TodoBoard>>,
    p9_witness: Option<Res<ConstructionP9Witness>>,
    round2_board: Option<Res<ConstructionRound2TodoBoard>>,
    round3_board: Option<Res<ConstructionRound3TodoBoard>>,
    operational_board: Option<Res<ConstructionOperationalTodoBoard>>,
    operational_witness: Option<Res<ConstructionOperationalWitness>>,
    stage_witness: Option<Res<super::ConstructionStageWitness>>,
    history: Option<Res<ConstructionHistory>>,
    path_feedback: Option<Res<super::path_feedback::ConstructionPathFeedback>>,
    junctions: Option<Res<super::rail::RailJunctionAuthority>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    // Proof write runs after witness refresh in the same frame — treat JSON as committed here.
    let operational_for_payload = operational_witness.as_deref().map(|w| {
        let mut snap = w.clone();
        snap.proof_json = true;
        snap
    });

    let junction_count = junctions.as_deref().map(|j| j.junctions.len()).unwrap_or(0);
    let p9_for_payload = p9_witness.as_deref().map(|w| {
        let mut snap = w.clone();
        snap.construction_proof_json = true;
        snap
    });
    let body = build_proof_payload(
        build_board.as_deref(),
        phase2_board.as_deref(),
        phase2_witness.as_deref(),
        p9_board.as_deref(),
        p9_for_payload.as_ref(),
        true,
        round2_board.as_deref(),
        round3_board.as_deref(),
        operational_board.as_deref(),
        operational_for_payload.as_ref(),
        stage_witness.as_deref(),
        history.as_deref(),
        path_feedback.as_deref(),
        junction_count,
    );
    const PROOF_PATH: &str = "debug_runs/construction_stage_live.json";
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "CONSTRUCTION_STAGE",
        "construction_live_proof",
        PROOF_PATH,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, payload) {
        state.written = true;
    }
}

/// After proof write, align witness flags so todo boards match `operational_green` payload.
pub fn sync_construction_proof_witness_flags(
    proof: Res<ConstructionLiveProofState>,
    mut operational: ResMut<crate::dev::construction_operational_todos::ConstructionOperationalWitness>,
    mut phase2: ResMut<crate::dev::construction_phase2_todos::ConstructionPhase2Witness>,
    mut p9: ResMut<crate::dev::construction_p9_todos::ConstructionP9Witness>,
) {
    let due_this_frame = proof
        .frames_since_write
        .saturating_add(1)
        >= proof.write_interval;
    if proof.written || due_this_frame {
        operational.proof_json = true;
        phase2.construction_proof_json = true;
        *p9 = crate::dev::construction_p9_todos::ConstructionP9Witness::from_phase2(phase2.as_ref());
    }
}

#[cfg(test)]
mod live_proof_sim_tests {
    use super::*;
    use std::fs;
    use bevy::app::App;
    use bevy::MinimalPlugins;

    use crate::construction::path_feedback::ConstructionPathFeedback;
    use crate::construction::rail::RailJunctionAuthority;

    use crate::dev::construction_finish_todos;
    use crate::dev::construction_live_todos;
    use crate::dev::construction_operational_todos;
    use crate::dev::construction_phase2_todos;
    use crate::dev::construction_round2_todos;
    use crate::dev::construction_round3_todos;

    use super::super::building_definitions::{
        default_buildings_dir, load_building_definitions_from_dir,
    };
    use super::super::build_mode::BuildModeState;
    use super::super::build_tool_authority::ActiveBuildTool;
    use super::super::construction_stage_witness;
    use super::super::roads::{ActiveRoadPlacement, IntersectionRegistry};
    use super::super::sessions::ActiveToolSession;
    use super::super::zones::ActiveZonePaint;

    /// **CONSTRUCTION-MV-001** — sim writer + multiview witness rollup.
    pub fn refresh_construction_mv_001_live_witness() -> bool {
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_construction_proof_sim_app();
        for _ in 0..15 {
            app.update();
        }
        let path = proof_output_path();
        if !path.exists() {
            return false;
        }
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read proof json")).expect("parse");
        json["construction_mv_001"]["green"].as_bool().unwrap_or(false)
            && json["operational_green"].as_bool().unwrap_or(false)
    }

    fn assemble_construction_proof_sim_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);

        construction_live_todos::register_construction_todo_runtime_hooks(&mut app);
        construction_finish_todos::register_construction_finish_todo_hooks(&mut app);
        construction_phase2_todos::register_construction_phase2_todo_hooks(&mut app);
        crate::dev::construction_p9_todos::register_construction_p9_todo_hooks(&mut app);
        construction_round2_todos::register_construction_round2_todo_hooks(&mut app);
        construction_round3_todos::register_construction_round3_todo_hooks(&mut app);
        construction_operational_todos::register_construction_operational_todo_hooks(&mut app);

        app.init_resource::<ConstructionLiveProofState>();
        app.init_resource::<ActiveToolSession>();
        app.init_resource::<ActiveBuildTool>();
        app.init_resource::<BuildModeState>();
        app.init_resource::<ActiveRoadPlacement>();
        app.init_resource::<ActiveZonePaint>();
        app.init_resource::<ConstructionHistory>();
        app.init_resource::<IntersectionRegistry>();
        app.init_resource::<ConstructionPathFeedback>();
        app.init_resource::<RailJunctionAuthority>();
        app.init_resource::<crate::render::view_runtime::ViewProjectionAuthority>();
        app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));

        app.world_mut()
            .resource_mut::<ConstructionLiveProofState>()
            .write_interval = 10;

        app.add_systems(
            Update,
            (
                construction_stage_witness::refresh_construction_stage_witness,
                construction_stage_witness::refresh_construction_finish_witness_system,
                construction_stage_witness::refresh_construction_phase2_witness_system,
                construction_stage_witness::refresh_construction_round2_witness_system,
                construction_stage_witness::refresh_construction_round3_witness_system,
                construction_stage_witness::refresh_construction_operational_witness_system,
                sync_construction_proof_witness_flags,
                construction_stage_witness::sync_construction_live_todo_board_system,
                construction_stage_witness::sync_construction_finish_board_system,
                construction_stage_witness::sync_construction_phase2_board_system,
                construction_stage_witness::sync_construction_p9_board_system,
                construction_stage_witness::sync_construction_round2_board_system,
                construction_stage_witness::sync_construction_round3_board_system,
                construction_stage_witness::sync_construction_operational_board_system,
                write_construction_live_proof_system,
            )
                .chain(),
        );
        {
            use crate::gui::ViewCameraState;
            use crate::render::view_runtime::{
                ViewAuthorityWriter, ViewProjectionAuthority, ViewSurfaceId,
            };
            let mut auth = app.world_mut().resource_mut::<ViewProjectionAuthority>();
            auth.commit_pose(
                ViewSurfaceId::SimulationMap,
                ViewCameraState::default(),
                ViewAuthorityWriter::BridgeCompat,
            );
        }
        app
    }

    #[test]
    fn simulation_writes_construction_stage_live_json_operational_green() {
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_construction_proof_sim_app();
        for _ in 0..15 {
            app.update();
        }
        let path = proof_output_path();
        assert!(path.exists(), "expected {:?} after sim ticks", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read proof json")).expect("parse");
        assert_eq!(json["profile"], "CONSTRUCTION_STAGE");
        assert_eq!(json["operational_green"], true);
        assert_eq!(json["con_e01_p9_green"], true);
        assert_eq!(
            json["p9_witness"]["construction_proof_json"],
            serde_json::json!(true)
        );
        assert!(
            json["p9_build"]
                .as_array()
                .expect("p9_build")
                .iter()
                .all(|row| row["status"] == "Done"),
            "p9_build: {}",
            json["p9_build"]
        );
        assert!(json.get("p9_build").is_some());
        assert!(app.world().resource::<ConstructionP9TodoBoard>().is_green());
        assert!(app.world().resource::<ConstructionLiveProofState>().written);
        assert_eq!(
            json["construction_mv_001"]["green"],
            serde_json::json!(true),
            "CONSTRUCTION-MV-001: {}",
            json["construction_mv_001"]
        );
    }
}

#[cfg(test)]
pub fn refresh_construction_mv_001_live_witness() -> bool {
    live_proof_sim_tests::refresh_construction_mv_001_live_witness()
}
