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
mod building_definitions;
pub mod procedural;
mod supply_chain_role;
mod utility_infrastructure_role;
mod visual_authority;
mod build_tool_authority;
mod build_toolbox;
mod map_egui_projection;
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
mod ghost_visual;
mod tile_visual;
mod history;
mod hydro_coupling;
mod phase_visual;
pub mod site_phase_tile_instances;
mod sessions;
mod tool_hints;
mod witness_collectors;
mod round4_corridor;
mod parametric_commit;
mod procedural_build_spawn;
pub use parametric_commit::{
    construction_procedural_build_001_witness_green, procedural_building_request_from_commit,
    style_pack_for_site_archetype, sync_procedural_assembly_request_from_sites,
};
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
    staging_toggle_wired_witness_green, StagedPlacementBook, StagedPlacementMode,
    StagedPlacementRow, StagedValidity,
};
pub use pending_construction_panel::draw_pending_construction_queue_egui;
pub use build_overlays::BuildOverlayVisibility;
pub use tile_visual::{
    build_site_tiles_between, toggle_construction_tile_info_labels, ConstructionTileVisualSettings,
};
pub use footprint_tile_instances::FootprintTileWitness;
pub use build_state::{
    BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementPreview,
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
pub use build_tool_authority::{
    apply_build_rail_tool_selection, ActiveBuildTool, BuildTool, BuildingArchetypeId, RailType,
    RoadType, ZoneTool,
};
pub use build_toolbox::{draw_build_toolbox_egui, draw_sim_build_rail_submenus_egui};
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
    draw_road_path_ghost_egui, draw_road_tool_popup_egui, road_path_input_system,
    sync_road_path_build_preview, sync_road_placement_width_from_tool,
    update_road_path_preview_system, ActiveRoadPlacement, RoadSegmentPreview, RoadToolPopupState,
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
            .init_resource::<tile_visual::ConstructionTileVisualSettings>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<ConstructionBlueprintImportUi>()
            .init_resource::<ConstructionQueuePanelView>()
            .init_resource::<path_feedback::ConstructionPathFeedback>()
            .init_resource::<ConstructionPlanQueue>()
            .init_resource::<ConstructionWorldRevision>()
            .init_resource::<ExecutedRoadNetwork>()
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
                        witness_collectors::write_construction_live_proof_system,
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
                    staged_ghost_panel::commit_approved_staged_placements_system
                        .after(build_confirm_site_system),
                    build_cancel_ghost_system,
                    build_sync_ghost_cursor_entity_system,
                    validate_construction_plans_system,
                    execute_construction_plans_system,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_build_toolbox_egui.run_if(product_egui_shell_active),
                    staged_ghost_panel::draw_staged_placements_panel_egui_system
                        .run_if(in_simulation_or_editor),
                    draw_sim_build_rail_submenus_egui.run_if(in_simulation_or_editor),
                    tool_hints::draw_tool_hints_egui,
                    draw_road_tool_popup_egui,
                    visual_authority::draw_construction_visual_requests_egui,
                    phase_visual::draw_construction_phase_labels_egui,
                    build_footprint_overlay::build_footprint_validity_overlay_egui,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}
