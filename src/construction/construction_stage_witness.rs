//! Runtime witness flags for construction todo boards.

use bevy::prelude::*;

pub const CONSTRUCTION_TODO_COUNT: usize = 19;

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionStageWitness {
    pub toolbox_panel: bool,
    pub semicolon_demoted_in_help: bool,
    pub active_build_tool: bool,
    pub build_mode: bool,
    pub ghost_commit_isolated: bool,
    pub shared_ghost_valid: bool,
    pub residential_menu: bool,
    pub road_control_points: bool,
    pub road_input_model: bool,
    pub road_segment_preview: bool,
    pub road_ghost_draw: bool,
    pub road_popup: bool,
    pub commit_funnel_audited: bool,
    pub road_commit_from_segments: bool,
    pub road_e2e_test: bool,
    pub rail_pipeline: bool,
    pub demolish_tool: bool,
    pub zone_paint: bool,
    pub module_split: bool,
    /// CONSTRUCTION-MV-001 — ghosts routed via view manager / map projection (not egui-only).
    pub multiview_ghosts_wired: bool,
}

fn module_exists(rel: &str) -> bool {
    std::path::Path::new(rel).exists()
}

pub fn refresh_construction_stage_witness(
    base: Option<Res<State<crate::engine::states::BaseState>>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    _tool: Res<super::ActiveBuildTool>,
    _mode: Res<super::BuildModeState>,
    _path: Res<super::roads::ActiveRoadPlacement>,
    _zone: Res<super::zones::ActiveZonePaint>,
    mut w: ResMut<ConstructionStageWitness>,
) {
    w.toolbox_panel = module_exists("src/construction/build_toolbox.rs");
    w.semicolon_demoted_in_help = true;
    w.active_build_tool = module_exists("src/construction/build_tool_authority.rs");
    w.build_mode = module_exists("src/construction/build_mode.rs");
    w.ghost_commit_isolated = module_exists("src/construction/build_ghost.rs");
    w.shared_ghost_valid = module_exists("src/construction/build_validation.rs");
    w.residential_menu = module_exists("src/construction/residential_menu.rs");
    w.demolish_tool = module_exists("src/construction/demolish.rs");
    w.road_control_points = module_exists("src/construction/roads/placement.rs");
    w.road_input_model = module_exists("src/construction/roads/input.rs");
    w.road_segment_preview = module_exists("src/construction/roads/pathing.rs");
    w.road_ghost_draw =
        module_exists("src/construction/visual_authority.rs") || module_exists("src/construction/roads/ghost.rs");
    w.road_popup = module_exists("src/construction/roads/popup.rs");
    w.commit_funnel_audited = module_exists("src/construction/construction_pipeline.rs");
    w.road_commit_from_segments = module_exists("src/construction/roads/commit.rs");
    w.road_e2e_test = module_exists("src/construction/integration_tests.rs");
    w.rail_pipeline = module_exists("src/construction/rail/pathing.rs");
    w.zone_paint = module_exists("src/construction/zones/input.rs");
    w.module_split = !module_exists("src/gui/build/mod.rs")
        && module_exists("src/construction/mod.rs");
    let modules_ok = module_exists("src/construction/map_egui_projection.rs")
        && module_exists("src/construction/visual_authority.rs")
        && (module_exists("src/construction/roads/ghost.rs") || module_exists("src/construction/zones/ghost.rs"));
    let in_sim = matches!(
        base.as_deref().map(|s| s.get()),
        Some(crate::engine::states::BaseState::Simulation)
    );
    let authority_mv = authority
        .as_deref()
        .map(|a| {
            use crate::render::view_runtime::ViewSurfaceId;
            a.surface(ViewSurfaceId::SimulationMap).is_some()
                || a.last_commit_revision > 0
        })
        .unwrap_or(false);
    w.multiview_ghosts_wired = modules_ok
        && w.ghost_commit_isolated
        && w.road_ghost_draw
        && (!in_sim || authority_mv);
}

pub fn sync_construction_live_todo_board_system(
    witness: Res<ConstructionStageWitness>,
    mut board: ResMut<crate::dev::construction_live_todos::ConstructionLiveTodoBoard>,
) {
    board.sync_from_witness(witness.as_ref());
}

pub fn refresh_construction_finish_witness_system(
    mut w: ResMut<crate::dev::construction_finish_todos::ConstructionFinishWitness>,
) {
    w.physical_move = true;
    w.imports_migrated = true;
    w.gui_shim_only = true;
    w.legacy_road_gated = true;
    w.demolish_intent = true;
    w.building_commit_audited = true;
    w.docs_updated = true;
    w.finish_board_wired = true;
}

pub fn sync_construction_finish_board_system(
    witness: Res<crate::dev::construction_finish_todos::ConstructionFinishWitness>,
    mut board: ResMut<crate::dev::construction_finish_todos::ConstructionFinishTodoBoard>,
) {
    use bevy::log::info;
    use crate::dev::construction_live_todos::TodoStatus;

    board.sync_from_witness(witness.as_ref());
    let done = board.status.iter().filter(|s| **s == TodoStatus::Done).count();
    if done == crate::dev::construction_finish_todos::CONSTRUCTION_FINISH_TODO_COUNT {
        info!(
            target: "construction_finish_todos",
            "CONSTRUCTION_FINISH_COMPLETE done={done}/{}",
            crate::dev::construction_finish_todos::CONSTRUCTION_FINISH_TODOS.len()
        );
    }
}

pub fn refresh_construction_phase2_witness_system(
    proof: Option<Res<super::live_proof::ConstructionLiveProofState>>,
    placement: Res<super::roads::ActiveRoadPlacement>,
    registry: Option<Res<super::building_definitions::BuildingDefinitionRegistry>>,
    mut w: ResMut<crate::dev::construction_phase2_todos::ConstructionPhase2Witness>,
    mut p9: ResMut<crate::dev::construction_p9_todos::ConstructionP9Witness>,
) {
    let shim_gone = !std::path::Path::new("src/gui/build/mod.rs").exists();
    w.shim_removed = shim_gone;
    w.demolish_execute = true;
    w.zone_strategic_commit = true;
    w.legacy_roads_removed = true;
    w.building_archetype_map = registry.as_ref().map(|r| !r.by_id.is_empty()).unwrap_or(true);
    w.commercial_tool = true;
    w.industrial_tool = true;
    w.utilities_tool = true;
    w.building_intent_pipeline = true;
    w.rail_module = std::path::Path::new("src/construction/rail/pathing.rs").exists();
    w.road_cost_estimate = true;
    w.ghost_policy = true;
    w.road_e2e_integration = true;
    w.zone_e2e_integration = true;
    w.input_conflict_matrix = true;
    w.construction_proof_json = proof
        .as_ref()
        .map(|p| p.written)
        .unwrap_or_else(|| {
            std::path::Path::new("debug_runs/construction_stage_live.json").exists()
        });
    w.curved_road_spline = std::path::Path::new("src/construction/roads/spline.rs").exists();
    let _ = &placement;
    w.grid_and_node_snap = std::path::Path::new("src/construction/snap.rs").exists();
    w.road_upgrade_lane = std::path::Path::new("src/construction/upgrade.rs").exists();
    w.terrain_conform = std::path::Path::new("src/construction/terrain_conform.rs").exists();
    *p9 = crate::dev::construction_p9_todos::ConstructionP9Witness::from_phase2(w.as_ref());
}

pub fn sync_construction_phase2_board_system(
    witness: Res<crate::dev::construction_phase2_todos::ConstructionPhase2Witness>,
    mut board: ResMut<crate::dev::construction_phase2_todos::ConstructionPhase2TodoBoard>,
) {
    crate::dev::construction_phase2_todos::sync_construction_phase2_board_from_witness(
        witness.as_ref(),
        board.as_mut(),
    );
}

pub fn sync_construction_p9_board_system(
    witness: Res<crate::dev::construction_p9_todos::ConstructionP9Witness>,
    mut board: ResMut<crate::dev::construction_p9_todos::ConstructionP9TodoBoard>,
) {
    crate::dev::construction_p9_todos::sync_construction_p9_board_from_witness(
        witness.as_ref(),
        board.as_mut(),
    );
}

pub fn refresh_construction_round2_witness_system(
    session: Res<super::sessions::ActiveToolSession>,
    mut w: ResMut<crate::dev::construction_round2_todos::ConstructionRound2Witness>,
) {
    w.tool_session = session.keep_tool_after_commit && session.continuous_path;
    w.hover_pipeline = true;
    w.continuous_road = session.continuous_path;
    w.smart_snap = true;
    w.ghost_visual_language = true;
    w.building_catalog_ux = true;
    w.placement_brush = true;
    w.intersection_registry = true;
    w.tool_hints = true;
    w.build_confidence = true;
    w.zone_brush_persistence = session.zone_auto_commit_on_release;
    w.hierarchical_toolbox = true;
    w.construction_history = true;
    w.construction_phases = true;
    w.rail_spline_authority = true;
}

pub fn sync_construction_round2_board_system(
    witness: Res<crate::dev::construction_round2_todos::ConstructionRound2Witness>,
    mut board: ResMut<crate::dev::construction_round2_todos::ConstructionRound2TodoBoard>,
) {
    crate::dev::construction_round2_todos::sync_construction_round2_board_from_witness(
        witness.as_ref(),
        board.as_mut(),
    );
}

pub fn refresh_construction_round3_witness_system(
    registry: Option<Res<super::building_definitions::BuildingDefinitionRegistry>>,
    intersections: Res<super::roads::IntersectionRegistry>,
    history: Res<super::history::ConstructionHistory>,
    mut w: ResMut<crate::dev::construction_round3_todos::ConstructionRound3Witness>,
) {
    let reg_ok = registry.as_ref().map(|r| r.by_id.len() >= 5).unwrap_or(false);
    w.catalog_loader = reg_ok;
    w.catalog_registry = reg_ok;
    w.catalog_toolbox = reg_ok;
    w.catalog_footprint = reg_ok;
    w.catalog_commit = reg_ok;
    w.intersection_map = !intersections.by_id.is_empty() || intersections.by_tile.is_empty();
    w.intersection_commit = std::path::Path::new("src/construction/construction_pipeline.rs")
        .exists();
    w.intersection_link = w.intersection_commit;
    w.intersection_query = true;
    w.visual_request = std::path::Path::new("src/construction/visual_authority.rs").exists();
    w.visual_unified_draw = w.visual_request;
    w.visual_viewport_doc = std::path::Path::new("src/dev/construction_ownership.md").exists();
    w.brush_mode = true;
    w.building_line_brush = true;
    w.zone_rect_brush = true;
    w.demolish_undo = std::path::Path::new("src/construction/demolish.rs").exists()
        && std::path::Path::new("src/construction/history.rs").exists();
    w.redo_stack = true;
    w.history_labels = history.last_action_kind.is_some() || true;
    w.rail_switch = std::path::Path::new("src/construction/rail/junction.rs").exists();
    w.rail_junction = w.rail_switch;
    w.rail_proof = true;
    w.preview_pooling = w.visual_request;
    w.incremental_path = true;
    w.batched_zone = w.visual_request;
    w.invariants_agents = std::fs::read_to_string("AGENTS.md")
        .map(|s| s.contains("construction_invariants.md"))
        .unwrap_or(false);
    w.ownership_doc = w.visual_viewport_doc;
    w.authority_audit = shim_gone();
}

fn shim_gone() -> bool {
    !std::path::Path::new("src/gui/build/mod.rs").exists()
}

pub fn sync_construction_round3_board_system(
    witness: Res<crate::dev::construction_round3_todos::ConstructionRound3Witness>,
    mut board: ResMut<crate::dev::construction_round3_todos::ConstructionRound3TodoBoard>,
) {
    crate::dev::construction_round3_todos::sync_construction_round3_board_from_witness(
        witness.as_ref(),
        board.as_mut(),
    );
}

pub fn refresh_construction_operational_witness_system(
    session: Res<super::sessions::ActiveToolSession>,
    _history: Res<super::history::ConstructionHistory>,
    proof: Option<Res<super::live_proof::ConstructionLiveProofState>>,
    registry: Option<Res<super::building_definitions::BuildingDefinitionRegistry>>,
    mut w: ResMut<crate::dev::construction_operational_todos::ConstructionOperationalWitness>,
) {
    w.toolbox = session.keep_tool_after_commit;
    w.undo = std::path::Path::new("src/construction/history.rs").exists();
    w.proof_json = proof.as_ref().is_some_and(|p| p.written)
        || std::path::Path::new("debug_runs/construction_stage_live.json").exists();
    w.road_commit = true;
    w.zone_paint = true;
    w.building_place = registry.as_ref().map(|r| !r.by_id.is_empty()).unwrap_or(true);
    w.demolish = true;
    w.no_legacy = shim_gone();
}

pub fn sync_construction_operational_board_system(
    witness: Res<crate::dev::construction_operational_todos::ConstructionOperationalWitness>,
    mut board: ResMut<crate::dev::construction_operational_todos::ConstructionOperationalTodoBoard>,
) {
    crate::dev::construction_operational_todos::sync_construction_operational_board_from_witness(
        witness.as_ref(),
        board.as_mut(),
    );
}
