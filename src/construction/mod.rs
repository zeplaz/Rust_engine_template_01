//! **Construction stage** — toolbox, tools, ghost preview, roads/zones/rail, commit funnel.
//!
//! **Public surface (import from `crate::construction` only):** pipeline queue, pending queue,
//! queue intents/panel view, build interaction systems, ghost cursor, blueprint presets, tool hints.

mod blueprint_preset;
mod build_commit;
mod build_ghost;
mod build_footprint_overlay;
pub mod footprint_tile_instances;
mod build_interaction;
mod build_mode;
mod build_overlays;
mod building_catalog;
pub mod building_definitions;
pub mod procedural;
mod supply_chain_role;
mod utility_infrastructure_role;
mod visual_authority;
mod build_tool_authority;
mod build_toolbox;
mod map_egui_projection;
pub use map_egui_projection::{world_to_sim_map_egui, ConstructionMapProjection};
mod commercial_menu;
mod corridor_transport;
mod construction_pipeline;
mod construction_queue_intent;
mod construction_stage_witness;
mod demolish;
mod industrial_menu;
mod mock_shapes_menu;
mod pending_construction;
mod pending_construction_panel;
mod path_feedback;
mod build_state;
mod build_strip;
mod build_validation;
mod build_confidence;
pub mod building_set;
pub mod site_zone_grid;
mod ghost_visual;
mod grammar_labels;
mod iso_draw_scale;
pub mod pilot_catalog;
mod placement_debug;
mod site_stub_overlay;
mod tile_visual;
mod history;
mod hydro_coupling;
mod phase_visual;
pub mod site_phase_tile_instances;
mod sessions;
mod tool_hints;
mod witness_collectors;
mod round4_corridor;
pub use round4_corridor::{
    construction_r4_corridor_legend_wired_witness_green, draw_r4_corridor_tray_legend,
    ConstructionRound4ProductGate,
};
mod parametric_commit;
mod procedural_build_spawn;
pub use parametric_commit::{
    build_read_visual_001_witness_body, build_read_visual_001_witness_green,
    construction_procedural_build_001_witness_green, procedural_building_request_from_commit,
    style_pack_for_site_archetype, sync_procedural_assembly_request_from_sites,
};
pub use iso_draw_scale::{
    build_read_world_002_witness_body, build_read_world_002_witness_green,
    ConstructionIsoDrawScale, DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
};
pub use site_stub_overlay::{
    build_read_site_v0_002_witness_green, sync_site_stub_overlay_requests, SiteStubOverlayState,
};
pub use pilot_catalog::{
    build_read_shape_002_witness_green, pilot_catalog_parity_witness_green, PilotCatalog,
    PilotKind,
};
pub use placement_debug::{
    build_read_debug_001_witness_green, build_verify_debug_001_witness_json,
    construction_placement_001_witness_green, construction_placement_001_witness_json,
    map_pick_closure_lib_witness_green,
};
pub use build_confidence::{confidence_from_validation, BuildConfidence};
pub use procedural_build_spawn::{
    spawn_procedural_build_on_site_operational, procedural_pg2_spawn_witness_green,
    ProceduralBuildModuleChild, ProceduralBuildSpawned,
};
pub use procedural::ProceduralTilePrimaryActive;
pub mod placement_scaling;
pub use placement_scaling::DEFAULT_SCALE_MAX;
mod scaling_audit;
mod staged_ghost_panel;
pub mod weighted_footprint;
mod residential_menu;
mod roads;
mod power_lines;
mod rail;
mod site_stage;
mod site_stage_tick;
mod site_stage_transitions;
mod snap;
mod terrain_conform;
mod upgrade;
mod utilities_menu;
mod zones;

#[cfg(test)]
mod integration_tests;

pub use visual_authority::{
    ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest,
};
pub use build_commit::queue_commit_construction_site;
pub use hydro_coupling::{
    emit_construction_hydro_dirty, emit_road_execute_hydro_dirty, emit_site_execute_hydro_dirty,
    seed_hydro_coupling_lib_witness,
    construction_hydro_coupling_witness_green,
};
pub use crate::substrate::hydrology::HydrologyConstructionCouplingWitness;
pub use build_ghost::GhostBuildCursor;
pub use blueprint_preset::{
    blueprint_collection_from_pending, blueprint_preset_entry_from_pending,
    BlueprintImportQueueMode, BlueprintPresetCollectionR8, BlueprintPresetEntryR8,
};
pub use build_interaction::{
    build_cancel_ghost_system, build_clear_pending_queue_system, build_confirm_site_system,
    build_drag_paint_queue_system, build_pick_ghost_tile_system,
    build_queue_blueprint_on_shift_click_system, build_refresh_placement_validation_system,
    build_rotate_mirror_ghost_system, build_sync_ghost_cursor_entity_system,
};
pub use construction_pipeline::{
    execute_construction_plans_system, validate_construction_plans_system,
    ConstructionPlanQueue, ConstructionWorldRevision, ExecutedRoadNetwork,
};
pub use construction_queue_intent::{
    apply_construction_queue_intents, sync_construction_queue_panel_view,
    ConstructionBlueprintImportUi, ConstructionQueueIntent, ConstructionQueuePanelEntryView,
    ConstructionQueuePanelView,
};
pub use pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
pub use staged_ghost_panel::{
    build_approved_drains_staged_witness_green, commit_approved_staged_rows,
    draw_staged_placements_panel_body, parametric_active_ghost_hud_line,
    staging_toggle_wired_witness_green, StagedPlacementBook, StagedPlacementMode,
    StagedPlacementRow, StagedValidity, STAGED_PANEL_FLOATING_SIM,
};
pub use tool_hints::TOOL_HINTS_DRAW_IN_SIM;
pub use pending_construction_panel::draw_pending_construction_queue_egui;
pub use build_overlays::BuildOverlayVisibility;
pub use tile_visual::{
    build_site_tiles_between, toggle_construction_tile_info_labels, ConstructionTileVisualSettings,
};
pub use footprint_tile_instances::FootprintTileWitness;
pub use build_state::{
    BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementMode, BuildPlacementPreview,
};
pub use build_mode::{BuildMode, BuildModeState};
pub use build_strip::{BuildStripState, ToolContext};
pub use supply_chain_role::IndustrialSupplyChainRole;
pub use utility_infrastructure_role::UtilityInfrastructureRole;
pub use building_definitions::{
    default_buildings_dir, init_building_definition_registry, load_building_definitions_from_dir,
    mock_shapes_parity_green, BuildingDefinition, BuildingDefinitionRegistry,
};
pub use procedural::{
    init_procedural_module_registry, init_style_pack_registry, init_tile_atlas_registry,
    load_procedural_module_registry, load_style_pack_registry, load_tile_atlas_registry,
    ProceduralAssemblyRequest, ProceduralModuleEntry, ProceduralModuleRegistry, StylePackRegistry,
    TileAtlasEntry, TileAtlasRegistry, MODULE_INDEX_RON, TILE_ATLAS_INDEX_RON,
};
pub use building_catalog::{
    ApartmentForm, ApartmentUnitKind, BuildingFamily, BuildingIntentPreview, DetachedResidenceForm,
    FootprintMatrix, ResidentialBuildingForm, default_preview_for_apartment,
};
pub use grammar_labels::{
    grammar_labels_loaded_green, human_age_label, human_archetype_label, human_district_label,
    human_massing_label,
};
pub use build_tool_authority::{
    apply_build_rail_tool_selection, ActiveBuildTool, BuildTool, BuildingArchetypeId, RailType,
    RoadType, ZoneTool,
};
pub use commercial_menu::draw_commercial_submenu;
pub use residential_menu::draw_residential_submenu;
pub use utilities_menu::{
    draw_utilities_submenu, utilities_submenu_power_icons_wired, UtilitiesSubmenuIconUi,
};
pub use mock_shapes_menu::draw_mock_shapes_submenu;
pub use build_toolbox::draw_build_toolbox_egui;
pub use crate::gui::hud::sim_build_picker_sheet::draw_sim_build_rail_submenus_egui;
pub use construction_stage_witness::{
    refresh_construction_stage_witness, ConstructionStageWitness, CONSTRUCTION_TODO_COUNT,
};
pub use build_validation::validate_planned_site_stubs;
pub use site_stage::{ClearingSubstep, SiteStageProgress};
pub use site_stage_tick::{
    advance_site_construction_tick_system, init_site_stage_progress_for_planned_sites,
    SiteStageTickPlugin,
};
pub use roads::{
    commit_road_path_to_queue, cursor_world_on_map, draw_road_path_ghost_egui,
    draw_road_tool_popup_egui, road_path_input_system, sync_road_path_build_preview,
    sync_road_placement_width_from_tool, update_road_path_preview_system, ActiveRoadPlacement,
    RoadSegmentPreview, RoadToolPopupState,
};
pub use sessions::ActiveToolSession;
pub use snap::RoadSnapSettings;
pub use upgrade::enqueue_road_upgrade;
pub use power_lines::{
    commit_power_line_to_utility_graph, cut_power_line_segment, damage_power_line_segment,
    draw_power_line_path_ghost_egui, node_key_for_world, power_damage_segment_witness_green,
    power_line_cut_input_wired, power_line_demolish_cut_system, power_line_draw_witness_green,
    power_line_ghost_preview_dashed_witness_green, power_line_path_input_system,
    power_repair_queue_witness_green, power_line_routing_mode_hotkey_system,
    preview_island_offline_from_cut, register_power_segments_from_graph_system,
    sync_power_damage_to_presentation_system, sync_power_line_build_preview,
    sync_power_line_from_build_tool, sync_power_line_preview_overlay_system,
    tick_power_repair_queue_system, update_power_line_path_preview_system,
    ActivePowerLinePlacement, PowerLineCutToast, PowerLineDamageBook, PowerLineRoutingMode,
    PowerLineSegmentHealth, PowerRepairJob, PowerRepairQueue, POWER_REPAIR_PARTS_PER_SEGMENT,
    POWER_REPAIR_TICKS_PER_JOB,
};
pub use rail::{
    draw_rail_path_ghost_egui, rail_path_input_system, sync_rail_path_build_preview,
    sync_rail_placement_from_tool, update_rail_path_preview_system, ActiveRailPlacement,
};
pub use zones::{
    draw_zone_paint_ghost_egui, sync_active_zone_from_tool, zone_paint_input_system,
    ActiveZonePaint,
};

pub use witness_collectors::build_construction_stage_proof_payload;
#[cfg(test)]
pub use witness_collectors::{refresh_bq128_apply_live_witnesses, refresh_construction_mv_001_live_witness};
pub use history::ConstructionHistory;
pub use path_feedback::ConstructionPathFeedback;
pub use rail::RailJunctionAuthority;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::gui::{in_simulation_or_editor, product_egui_shell_active, InputBindings};

pub fn cycle_build_planning_tool_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut strip: ResMut<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    mut tool: ResMut<ActiveBuildTool>,
) {
    if !keyboard.just_pressed(bindings.cycle_build_planning_tool) {
        return;
    }
    strip.active = strip.active.next();
    tool.tool = BuildTool::from_tool_context(strip.active);
    tool.close_submenus();
    ghost.footprint = strip.active.footprint_for_tool();
    if strip.active == ToolContext::None {
        ghost.origin = None;
    }
}

fn ensure_build_toolbox_shell_visible(mut dock: ResMut<crate::gui::hud::HudDockRegistry>) {
    dock.slot_mut(crate::gui::hud::HudWidgetId::BuildToolbox).visible = true;
}

pub struct BuildPlanningPlugin;

impl Plugin for BuildPlanningPlugin {
    fn build(&self, app: &mut App) {
        let owner = app.world_mut().spawn_empty().id();

        app.insert_resource(BuildCommandActor(owner))
            .add_plugins(SiteStageTickPlugin)
            .init_resource::<BuildStripState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildModeState>()
            .init_resource::<ActiveRoadPlacement>()
            .init_resource::<ActiveRailPlacement>()
            .init_resource::<ActivePowerLinePlacement>()
            .init_resource::<crate::construction::PowerLineDamageBook>()
            .init_resource::<crate::construction::PowerRepairQueue>()
            .init_resource::<crate::construction::PowerLineCutToast>()
            .init_resource::<ActiveZonePaint>()
            .init_resource::<history::ConstructionHistory>()
            .init_resource::<roads::RoadToolPopupState>()
            .init_resource::<snap::RoadSnapSettings>()
            .init_resource::<sessions::ActiveToolSession>()
            .init_resource::<roads::IntersectionRegistry>()
            .init_resource::<visual_authority::ConstructionVisualRequests>()
            .init_resource::<round4_corridor::ConstructionRound4ProductGate>()
            .init_resource::<footprint_tile_instances::FootprintTileWitness>()
            .init_resource::<site_phase_tile_instances::ConstructionPhaseGpuChannel>()
            .init_resource::<rail::RailJunctionAuthority>()
            .init_resource::<witness_collectors::ConstructionLiveProofState>()
            .add_systems(
                Startup,
                (
                    procedural::init_procedural_module_registry,
                    procedural::init_tile_atlas_registry,
                    procedural::init_style_pack_registry,
                    procedural::init_palette_catalog_registry,
                    procedural::init_variant_catalog,
                    building_definitions::init_building_definition_registry,
                    ensure_build_toolbox_shell_visible,
                )
                    .chain(),
            )
            .init_resource::<BuildGhostState>()
            .init_resource::<BuildPlacementPreview>()
            .init_resource::<staged_ghost_panel::StagedPlacementMode>()
            .init_resource::<staged_ghost_panel::StagedPlacementBook>()
            .init_resource::<BuildOverlayVisibility>()
            .init_resource::<ConstructionIsoDrawScale>()
            .init_resource::<SiteStubOverlayState>()
            .init_resource::<placement_debug::ConstructionPlacementDebugOverlay>()
            .init_resource::<placement_debug::ConstructionPlacementDebugProbe>()
            .init_resource::<tile_visual::ConstructionTileVisualSettings>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<ConstructionBlueprintImportUi>()
            .init_resource::<ConstructionQueuePanelView>()
            .init_resource::<path_feedback::ConstructionPathFeedback>()
            .init_resource::<ConstructionPlanQueue>()
            .init_resource::<ConstructionWorldRevision>()
            .init_resource::<ExecutedRoadNetwork>()
            .init_resource::<crate::gui::hud::power_node_hover::PowerNodeHoverState>()
            .add_systems(Startup, hydro_coupling::register_construction_hydro_coupling_bridge)
            .add_message::<ConstructionQueueIntent>()
            .add_systems(
                Update,
                (
                    (
                        build_tool_authority::apply_active_build_tool_to_strip,
                        build_tool_authority::sync_active_build_tool_from_strip,
                        build_mode::sync_build_mode_state,
                        build_mode::build_escape_cancel_system,
                        sync_active_zone_from_tool,
                        refresh_construction_stage_witness,
                        construction_stage_witness::refresh_construction_finish_witness_system,
                        construction_stage_witness::refresh_construction_phase2_witness_system,
                        construction_stage_witness::refresh_construction_round2_witness_system,
                        construction_stage_witness::refresh_construction_round3_witness_system,
                        construction_stage_witness::refresh_construction_operational_witness_system,
                    )
                        .chain(),
                    (
                        sessions::tick_tool_session_time,
                        witness_collectors::sync_construction_proof_witness_flags,
                        construction_stage_witness::sync_construction_live_todo_board_system,
                        construction_stage_witness::sync_construction_finish_board_system,
                        construction_stage_witness::sync_construction_phase2_board_system,
                        construction_stage_witness::sync_construction_p9_board_system,
                        construction_stage_witness::sync_construction_round2_board_system,
                        construction_stage_witness::sync_construction_round3_board_system,
                        construction_stage_witness::sync_construction_operational_board_system,
                        witness_collectors::write_construction_live_proof_system
                            .run_if(crate::dev::runtime_witness::construction_live_proof_due),
                    )
                        .chain(),
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(Update, cycle_build_planning_tool_system.run_if(in_simulation_or_editor))
            .add_systems(
                Update,
                tile_visual::toggle_construction_tile_info_labels.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    sync_road_placement_width_from_tool,
                    update_road_path_preview_system,
                    sync_road_path_build_preview.after(update_road_path_preview_system),
                    road_path_input_system.after(sync_road_path_build_preview),
                    sync_rail_placement_from_tool,
                    update_rail_path_preview_system,
                    sync_rail_path_build_preview.after(update_rail_path_preview_system),
                    rail_path_input_system.after(sync_rail_path_build_preview),
                    zone_paint_input_system,
                    visual_authority::clear_construction_visual_requests,
                    visual_authority::sync_road_visual_requests,
                    visual_authority::sync_rail_visual_requests,
                    visual_authority::sync_zone_visual_requests,
                    visual_authority::sync_footprint_visual_requests,
                    site_stub_overlay::sync_site_stub_overlay_requests,
                    round4_corridor::sync_corridor_phase_visual_requests
                        .after(visual_authority::sync_footprint_visual_requests),
                    footprint_tile_instances::push_footprint_tile_instances
                        .after(crate::gui::build_tile_debug_instances),
                    site_phase_tile_instances::push_site_phase_tile_instances
                        .after(footprint_tile_instances::push_footprint_tile_instances),
                    footprint_tile_instances::sync_visual_aidv2_footprint_witness,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    sync_power_line_from_build_tool,
                    update_power_line_path_preview_system,
                    sync_power_line_build_preview.after(update_power_line_path_preview_system),
                    sync_power_line_preview_overlay_system.after(sync_power_line_build_preview),
                    power_line_path_input_system.after(sync_power_line_build_preview),
                    power_line_routing_mode_hotkey_system,
                    power_lines::register_power_segments_from_graph_system,
                    power_lines::tick_power_repair_queue_system,
                    power_lines::power_line_demolish_cut_system,
                    crate::gui::hud::power_node_hover::sync_power_node_hover_pick,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    history::construction_undo_input_system,
                    history::construction_redo_input_system,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                history::finalize_site_history_records.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    apply_construction_queue_intents,
                    sync_construction_queue_panel_view,
                    build_pick_ghost_tile_system,
                    demolish::demolish_pick_queue_system,
                    build_refresh_placement_validation_system,
                    staged_ghost_panel::stage_active_ghost_on_lmb_system
                        .after(build_refresh_placement_validation_system),
                    build_queue_blueprint_on_shift_click_system,
                    build_drag_paint_queue_system,
                    build_rotate_mirror_ghost_system,
                    build_clear_pending_queue_system,
                    build_confirm_site_system,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    staged_ghost_panel::commit_approved_staged_placements_system,
                    build_cancel_ghost_system,
                    build_sync_ghost_cursor_entity_system,
                    validate_construction_plans_system,
                    execute_construction_plans_system,
                )
                    .chain()
                    .after(build_confirm_site_system)
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                (
                    placement_debug::sync_construction_placement_debug_probe,
                    sync_site_stub_overlay_requests.after(visual_authority::sync_footprint_visual_requests),
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_build_toolbox_egui.run_if(product_egui_shell_active),
                    staged_ghost_panel::draw_staged_placements_panel_egui_system
                        .run_if(in_simulation_or_editor),
                    crate::gui::hud::sim_build_picker_sheet::draw_sim_build_picker_sheet_egui
                        .run_if(in_simulation_or_editor),
                    crate::gui::hud::context_tray_build_egui::draw_context_tray_build_body_egui
                        .run_if(in_simulation_or_editor),
                    crate::gui::hud::context_tray_power_repair_egui::draw_context_tray_power_repair_egui
                        .run_if(in_simulation_or_editor),
                    placement_debug::draw_construction_placement_debug_overlay
                        .run_if(in_simulation_or_editor),
                    tool_hints::draw_tool_hints_egui,
                    crate::gui::hud::sim_road_tool_sheet::draw_sim_road_tool_sheet_egui,
                    crate::gui::hud::sim_power_tool_sheet::draw_sim_power_tool_sheet_egui,
                    crate::gui::hud::plant_focus_card::draw_plant_focus_card_egui,
                    crate::gui::hud::power_node_hover_egui::draw_power_node_hover_egui,
                    draw_power_line_path_ghost_egui,
                    visual_authority::draw_construction_visual_requests_egui,
                    phase_visual::draw_construction_phase_labels_egui,
                    build_footprint_overlay::build_footprint_validity_overlay_egui,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}
