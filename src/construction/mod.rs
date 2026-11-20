//! **Construction stage** — toolbox, tools, ghost preview, roads/zones/rail, commit funnel.
//!
//! **Public surface (import from `crate::construction` only):** pipeline queue, pending queue,
//! queue intents/panel view, build interaction systems, ghost cursor, blueprint presets, tool hints.

mod blueprint_preset;
mod build_commit;
mod build_footprint_overlay;
mod build_ghost;
mod build_interaction;
mod build_mode;
mod build_overlays;
mod build_tool_authority;
mod build_toolbox;
mod building_catalog;
pub mod building_definitions;
pub mod footprint_tile_instances;
mod map_egui_projection;
mod map_pick_params;
pub mod procedural;
mod supply_chain_role;
mod utility_infrastructure_role;
mod visual_authority;
mod voxel_massing;
mod world_map_frame;
pub use world_map_frame::{
    path_commit_leaves_generated_map, path_samples_leave_generated_map, world_map_tile_center,
};
pub use ghost_visual::{
    corridor_in_progress_color, corridor_planned_color, footprint_place_settle_color,
    footprint_valid_color, road_committed_color, road_segment_color,
};
pub use phase_visual::{phase_map_word, phase_stack_band, phase_stack_band_word, PhaseStackBand};
pub use voxel_massing::{
    derive_massing_floors, massing_uses_instance_batch, pack_massing_instances,
    MassingColumnInstance, StudioMassingOverride, VoxelColumnRequest, MASSING_FLOORS_MAX,
    MASSING_INSTANCE_BATCH_THRESHOLD, STUDIO_MASSING_ENV, STUDIO_MASSING_PREVIEW_REL,
};
pub use map_egui_projection::{world_to_sim_map_egui, world_to_sim_map_screen, ConstructionMapProjection};
mod build_confidence;
mod build_state;
mod build_strip;
mod build_validation;
pub mod building_set;
mod commercial_menu;
mod construction_pipeline;
mod construction_queue_intent;
mod corridor_transport;
mod defense_concrete_stock;
mod defense_staging_stock;
mod blast;
mod blast_present;
mod demolish;
pub use demolish::{
    destructible_catastrophic_collapse, destructible_stage_from_integrity,
    execute_demolish_at_tile, DestructibleDamage, DestructibleDamageStage,
    DESTRUCTIBLE_COLLAPSE_INTEGRITY,
};
pub(crate) mod ghost_visual;
mod grammar_labels;
mod history;
mod hydro_coupling;
mod industrial_menu;
mod iso_draw_scale;
pub(crate) mod build_readability_harness;
mod mock_shapes_menu;
mod path_feedback;
mod pending_construction;
mod pending_construction_panel;
mod phase_fill;
mod phase_visual;
pub mod pilot_catalog;
mod place_feedback;
mod placement_debug;
mod preview_commit_guard;
pub use preview_commit_guard::{
    apply_dam_banks_reject, apply_water_footprint_reject, building_may_occupy_water,
    land_building_blocked_on_water, DAM_BANKS_ERROR, DAM_KIND_SITE_ERROR, WATER_FOOTPRINT_ERROR,
};
mod dam_barrier;
pub mod water_obstacles;
pub mod rubble;
pub use dam_barrier::{
    attach_dam_body_on_commit_system, dam_integrity_progress_system,
    release_barrier_of_removed_dams_system, publish_dam_column_crests_system, dam_face_velocities_m_s,
    link_dam_hydro_site_system,
    dam_column_crest_fraction,
    derate_turbines_from_dam_damage_system, dam_kind_site_suitable, dam_spans_banks,
    erode_dam_columns_by_flow,
    install_barrier_from_operational_dam_system, leak_flow_from_integrity,
    mirror_dam_control_to_barrier_system, DamBody, DamControl, DamKind,
};
mod structural_role;
pub use structural_role::{SiteStructuralRoles, StructuralRole};
mod structural_cascade;
pub use structural_cascade::cascade_mask_from_roles_system;
mod disaster_proxy;
pub use disaster_proxy::{
    apply_structural_collapse_demolish, flood_foundation_integrity_proxy_system,
    StructuralCollapseQueue, FLOOD_FOUNDATION_DEPTH_M,
};
mod round4_corridor;
pub use round4_corridor::CorridorPhaseDrawKind;
mod sessions;
pub mod site_phase_tile_instances;
mod site_stub_overlay;
pub mod site_zone_grid;
mod tile_visual;
mod tool_hints;
pub use round4_corridor::{
    draw_r4_corridor_tray_legend,
    ConstructionRound4ProductGate,
};
mod parametric_commit;
mod procedural_build_spawn;
pub use build_confidence::{confidence_from_validation, BuildConfidence};
pub use iso_draw_scale::{
    ConstructionIsoDrawScale, DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
};
pub use parametric_commit::{
    parametric_placement_snapshot,
    procedural_building_request_from_commit, style_pack_for_site_archetype,
    sync_procedural_assembly_request_from_sites,
};
pub use pilot_catalog::{
    PilotCatalog,
    PilotKind, PILOT_ARCH_DNA_PRESET_WAREHOUSE, PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE,
};

pub use procedural::ProceduralTilePrimaryActive;
pub use procedural_build_spawn::{
    spawn_procedural_build_on_site_operational,
    ProceduralBuildModuleChild, ProceduralBuildSpawned,
};
pub use site_stub_overlay::{
    sync_site_stub_overlay_requests, SiteStubOverlayState,
};
pub mod placement_scaling;
pub use placement_scaling::DEFAULT_SCALE_MAX;
mod power_lines;
mod rail;
mod residential_menu;
mod roads;
mod site_stage;
mod site_stage_tick;
mod site_stage_transitions;
mod snap;
mod staged_ghost_panel;
mod terrain_conform;
mod upgrade;
mod utilities_menu;
pub mod weighted_footprint;
mod zones;

#[cfg(test)]
mod integration_tests;

pub use crate::substrate::hydrology::HydrologyConstructionCouplingStats;
pub use blueprint_preset::{
    blueprint_collection_from_pending, blueprint_preset_entry_from_pending,
    BlueprintImportQueueMode, BlueprintPresetCollectionR8, BlueprintPresetEntryR8,
};
pub use build_commit::queue_commit_construction_site;
pub use build_ghost::{GhostBuildCursor, GhostValid};
pub use build_interaction::{
    build_adjust_blocks_map_wheel, build_adjust_modifiers_system, build_cancel_ghost_system,
    build_clear_pending_queue_system, build_confirm_site_system, build_drag_paint_queue_system,
    build_pick_ghost_tile_system, build_place_on_second_lmb_system,
    build_apply_material_gates_system, build_apply_occupancy_reject_system,
    build_apply_water_reject_system, build_queue_blueprint_on_shift_click_system,
    build_refresh_placement_validation_system,
    build_rotate_mirror_ghost_system, build_sync_ghost_cursor_entity_system,
    construction_map_pick_allowed, two_click_lmb_commit_count, two_click_mode_lock_count, };
pub use build_mode::{BuildEscCancelLatch, BuildMode, BuildModeState};
pub use build_overlays::BuildOverlayVisibility;
pub use build_state::{
    BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementMode, BuildPlacementPreview,
};
pub use build_strip::{BuildStripState, ToolContext};
pub use build_tool_authority::{
    apply_build_rail_tool_selection, ActiveBuildTool, BuildTool, BuildingArchetypeId, DefenseKind,
    RailType, RoadType, ZoneTool,
};
pub use build_toolbox::{register_build_toolbox_bevy_systems, BuildToolboxBevyRoot};
pub use build_validation::{
    evaluate_ghost_site_placement, ghost_valid_flag, validate_planned_site_stubs,
};
pub use building_catalog::{
    default_preview_for_apartment, ApartmentForm, ApartmentUnitKind, BuildingFamily,
    BuildingIntentPreview, DetachedResidenceForm, FootprintMatrix, ResidentialBuildingForm,
};
pub use building_definitions::{
    default_buildings_dir, init_building_definition_registry, intent_from_archetype,
    intent_from_definition, load_building_definitions_from_dir, BuildingDefinition, BuildingDefinitionRegistry,
};
pub use commercial_menu::draw_commercial_submenu;
pub use construction_pipeline::{
    execute_construction_plans_system, validate_construction_plans_system, ConstructionIntent,
    ConstructionPlanQueue, ConstructionType, ConstructionWorldRevision, ExecutedRoadNetwork,
};
pub use construction_queue_intent::{
    apply_construction_queue_intents, sync_construction_queue_panel_view,
    ConstructionBlueprintImportUi, ConstructionQueueIntent, ConstructionQueuePanelEntryView,
    ConstructionQueuePanelView,
};

pub use defense_concrete_stock::{
    INSUFFICIENT_CONCRETE_ERROR,
};
pub use defense_staging_stock::{
    ai_commit_gated_count, apply_deployable_staging_gate, apply_deployable_staging_gate_archetype,
    enqueue_deployable_staging_debit_for_archetype, enqueue_deployable_staging_debit_on_commit,
    note_ai_commit_gated, place_blocked_no_stock_count, place_ok_debit_count,
    INSUFFICIENT_STAGED_ERROR, STOCK_IN_TRANSIT_ERROR,
};
pub use footprint_tile_instances::FootprintTileStats;
pub use grammar_labels::{
    human_age_label, human_archetype_label, human_district_label,
    human_massing_label,
};
pub use blast::{
    BlastCraterBook, BlastIntent, BlastIntentQueue, BlastProfile, ConstructionBlastPlugin,
};
pub use blast_present::BlastCraterDump;
pub use hydro_coupling::{
    construction_hydro_coupling_witness_green, emit_construction_hydro_dirty,
    emit_road_execute_hydro_dirty, emit_site_execute_hydro_dirty, };
pub use mock_shapes_menu::draw_mock_shapes_submenu;
pub use pending_construction::{PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind};
pub use power_lines::{
    commit_power_line_to_utility_graph, cut_power_line_segment, damage_power_line_segment,
    node_key_for_world, power_line_cut_input_wired,
    power_line_demolish_cut_system, power_line_load_fraction,
    power_line_path_input_system, power_line_routing_mode_hotkey_system,
    preview_island_offline_from_cut,
    register_power_segments_from_graph_system, sync_power_damage_to_presentation_system,
    sync_power_line_build_preview, sync_power_line_from_build_tool,
    sync_power_line_preview_overlay_system, tick_power_repair_queue_system,
    update_power_line_path_preview_system, ActivePowerLinePlacement, PowerLineCutToast,
    PowerLineDamageBook, PowerLineRoutingMode, PowerLineSegmentHealth, PowerRepairJob,
    PowerRepairQueue, POWER_REPAIR_PARTS_PER_SEGMENT, POWER_REPAIR_TICKS_PER_JOB,
};
pub use procedural::{
    init_procedural_module_registry, init_style_pack_registry, init_tile_atlas_registry,
    load_procedural_module_registry, load_style_pack_registry, load_tile_atlas_registry,
    ProceduralAssemblyRequest, ProceduralModuleEntry, ProceduralModuleRegistry, StylePackRegistry,
    TileAtlasEntry, TileAtlasRegistry, MODULE_INDEX_RON, TILE_ATLAS_INDEX_RON,
};
pub use rail::{
    rail_path_input_system, sync_rail_path_build_preview, sync_rail_placement_from_tool,
    update_rail_path_preview_system, ActiveRailPlacement,
};
pub use residential_menu::draw_residential_submenu;
pub use roads::{
    commit_road_path_to_queue, cursor_world_on_map, register_road_tool_popup_bevy_systems,
    road_path_input_system, sync_road_path_build_preview, sync_road_placement_width_from_tool,
    update_road_path_preview_system, ActiveRoadPlacement, RoadSegmentPreview,
    RoadToolPopupBevyRoot, RoadToolPopupState,
};
pub use sessions::ActiveToolSession;
pub use site_stage::{ClearingSubstep, SiteStageProgress};
pub use site_stage_tick::{
    advance_site_construction_tick_system, init_site_stage_progress_for_planned_sites,
    SiteStageTickPlugin,
};
pub use snap::RoadSnapSettings;
pub use staged_ghost_panel::{
    commit_approved_staged_rows,
    parametric_active_ghost_hud_line, staged_placements_tray_lines,
    StagedPlacementBook, StagedPlacementMode,
    StagedPlacementRow, StagedValidity, STAGED_PANEL_FLOATING_SIM,
};
pub use supply_chain_role::IndustrialSupplyChainRole;
pub use tile_visual::{
    build_site_tiles_between, toggle_construction_tile_info_labels, ConstructionTileVisualSettings,
};
pub use tool_hints::TOOL_HINTS_DRAW_IN_SIM;
pub use upgrade::enqueue_road_upgrade;
pub use utilities_menu::{
    draw_utilities_submenu, utilities_submenu_power_icons_wired, UtilitiesSubmenuIconUi,
};
pub use utility_infrastructure_role::UtilityInfrastructureRole;
pub use visual_authority::{
    ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest, PathLineRequest,
    VisualPathKind, ZoneTileRequest, sync_zone_visual_requests,
};
pub use zones::{
    admin_zone_drag_input_system, sync_active_zone_from_tool, zone_paint_input_system,
    ActiveZonePaint,
};

pub use history::ConstructionHistory;
pub use path_feedback::ConstructionPathFeedback;
pub use place_feedback::{
    place_feedback_complete_count, place_feedback_start_count,
    PlaceFeedbackPulse, TwoClickCommitParams,
};
pub use rail::RailJunctionAuthority;



use bevy::prelude::*;
use crate::gui::{in_simulation_or_editor, InputBindings};

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
    dock.slot_mut(crate::gui::hud::HudWidgetId::BuildToolbox)
        .visible = true;
}

pub struct BuildPlanningPlugin;

impl Plugin for BuildPlanningPlugin {
    fn build(&self, app: &mut App) {
        let owner = app.world_mut().spawn_empty().id();

        app.insert_resource(BuildCommandActor(owner))
            .add_plugins(SiteStageTickPlugin)
            .add_plugins(ConstructionBlastPlugin)
            .init_resource::<BuildStripState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildModeState>()
            .init_resource::<BuildEscCancelLatch>()
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
            .init_resource::<voxel_massing::StudioMassingOverride>()
            .init_resource::<round4_corridor::ConstructionRound4ProductGate>()
            .init_resource::<footprint_tile_instances::FootprintTileStats>()
            .init_resource::<site_phase_tile_instances::ConstructionPhaseGpuChannel>()
            .init_resource::<rail::RailJunctionAuthority>()
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
                    // One-shot env opt-in — debug / sim_dump only (CQ-09); never polls disk per frame.
                    #[cfg(any(debug_assertions, feature = "sim_dump"))]
                    voxel_massing::load_studio_massing_override_from_env_once,
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
            .init_resource::<place_feedback::PlaceFeedbackPulse>()
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
                    )
                        .chain(),
                    (
                        sessions::tick_tool_session_time,
                    )
                        .chain(),
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(Update, cycle_build_planning_tool_system.run_if(in_simulation_or_editor))
            .add_systems(
                Update,
                (
                    tile_visual::toggle_construction_tile_info_labels,
                    admin_zone_drag_input_system
                        .after(crate::gui::hud::simulation_pointer_gate::PointerGateSet::Publish),
                )
                    .run_if(in_simulation_or_editor),
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
                    place_feedback::sync_place_feedback_visual_requests,
                    site_stub_overlay::sync_site_stub_overlay_requests,
                    round4_corridor::sync_corridor_phase_visual_requests
                        .after(visual_authority::sync_footprint_visual_requests),
                    footprint_tile_instances::push_footprint_tile_instances
                        .after(crate::gui::build_tile_debug_instances),
                    site_phase_tile_instances::push_site_phase_tile_instances
                        .after(footprint_tile_instances::push_footprint_tile_instances),
                )
                    .chain()
                    .after(crate::gui::hud::simulation_pointer_gate::PointerGateSet::Publish)
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    voxel_massing::sync_voxel_massing_visual_requests
                        .after(visual_authority::sync_footprint_visual_requests),
                    world_map_frame::sync_world_map_footprints
                        .after(voxel_massing::sync_voxel_massing_visual_requests),
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    place_feedback::tick_place_feedback_pulse,
                    place_feedback::clear_place_feedback_on_tool_exit,
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
                    crate::strategic::apply_shot_impacts_to_power_lines_system,
                    power_lines::tick_power_repair_queue_system,
                    power_lines::power_line_demolish_cut_system,
                    crate::gui::hud::power_node_hover::sync_power_node_hover_pick,
                )
                    .chain()
                    .after(crate::gui::hud::simulation_pointer_gate::PointerGateSet::Publish)
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
                    build_apply_occupancy_reject_system,
                    build_apply_water_reject_system,
                    build_apply_material_gates_system,
                    staged_ghost_panel::stage_active_ghost_on_lmb_system,
                    build_place_on_second_lmb_system,
                    build_queue_blueprint_on_shift_click_system,
                    build_drag_paint_queue_system,
                    build_rotate_mirror_ghost_system,
                    build_adjust_modifiers_system,
                    build_clear_pending_queue_system,
                    build_confirm_site_system,
                )
                    .chain()
                    .after(crate::gui::hud::simulation_pointer_gate::PointerGateSet::Publish)
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
                Update,
                crate::gui::hud::context_tray_build_egui::ensure_context_tray_peek_when_build_armed
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    attach_dam_body_on_commit_system,
                    install_barrier_from_operational_dam_system
                        .after(attach_dam_body_on_commit_system),
                    dam_integrity_progress_system
                        .after(install_barrier_from_operational_dam_system),
                    link_dam_hydro_site_system.after(attach_dam_body_on_commit_system),
                    derate_turbines_from_dam_damage_system
                        .after(dam_integrity_progress_system)
                        .after(link_dam_hydro_site_system),
                    mirror_dam_control_to_barrier_system
                        .after(derate_turbines_from_dam_damage_system),
                    structural_role::ensure_default_structural_roles_system
                        .before(flood_foundation_integrity_proxy_system),
                    flood_foundation_integrity_proxy_system,
                    apply_structural_collapse_demolish
                        .after(flood_foundation_integrity_proxy_system),
                    release_barrier_of_removed_dams_system
                        .after(apply_structural_collapse_demolish),
                    publish_dam_column_crests_system.after(dam_integrity_progress_system),
                    water_obstacles::publish_water_obstacles_system
                        .after(apply_structural_collapse_demolish),
                    rubble::rubble_from_removed_sites_system
                        .after(apply_structural_collapse_demolish)
                        .before(crate::terrain::ground_deform::GroundDeformSet::Drain),
                    rubble::rubble_from_failed_dam_columns_system
                        .after(dam_integrity_progress_system)
                        .before(crate::terrain::ground_deform::GroundDeformSet::Drain),
                )
                    .run_if(in_simulation_or_editor),
            )
            .init_resource::<StructuralCollapseQueue>()
            .register_type::<DamBody>()
            .register_type::<DamControl>()
            .register_type::<DamKind>()
            .register_type::<SiteStructuralRoles>()
            .register_type::<StructuralRole>();

        register_build_toolbox_bevy_systems(app);
        register_road_tool_popup_bevy_systems(app);
    }
}
