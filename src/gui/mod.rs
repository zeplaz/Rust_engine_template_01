// GUI systems
// Boundary: see prompts/guides/ui_boundary_guide_v1.md
//   splash, app_shell, in_game_hud → Bevy shell / simulation HUD
//   editor/* + selected egui panels  → dev tooling (gated via ui_gates)
//   pressure_tooling.rs             → F2 pressure composer (egui) + Bevy strip
//   ai_explainability_ui.rs         → L7 pipeline + macro explain (default Insert)
//   agent_permissions_ui.rs         → egui tooling

pub mod debug;
pub mod hud;
mod minimap_shell;
mod minimap_egui_dev;
mod minimap_viewport_frame;
pub mod tactical;
/// RGR-P5-002 shim — old `crate::gui::map_camera::*` path kept alive; canonical home is
/// `crate::gui::tactical::map_camera`. Remove once all call sites migrate to the new path.
pub mod map_camera {
    pub use super::tactical::map_camera::*;
}
/// RGR-P5-002 shim — old `crate::gui::sim_map_rtt::*` path kept alive; canonical home is
/// `crate::gui::tactical::sim_map_rtt`. Remove once all call sites migrate to the new path.
pub mod sim_map_rtt {
    pub use super::tactical::sim_map_rtt::*;
}
mod map_zoom_coherence;
mod sim_map_projection;
mod vfx_fire_test_highlight;
mod settlement_block_frame_debug;
mod assembly_snapshot_qc_ui;
mod map_view;
mod map_presentation_controls;
mod map_presentation_diagnostics;
mod map_presentation_fit;
mod map_view_projection;
mod lod_zone_authoring;
mod world_representation;
mod representation_governance;
mod representation_policy;
mod representation_spine_audit;
mod view_representation;
mod view_representation_snapshot;
mod viewport_authority;
mod viewport_layout_solver;
mod view_authority;
mod view_projection_authority;
pub mod map_tile_raster;
pub mod map_tile_atlas_stamp;
pub mod landscape_chunk_atlas_stamp;
pub mod construction_growth_inspector;
pub mod egui_root;
mod egui_window;
mod app_shell;
mod main_menu;
mod splash;
mod ui_gates;
mod world_gen_chrome_contract;
mod input_bindings;
mod input_frame;
mod gameplay_capture;
mod options_keybindings_ui;
mod pressure_tooling;
mod logistics_focus;
mod authoritative_viewport;
mod in_game_hud;
mod in_game_pause_menu;
mod pause_menu_bevy;
mod pause_menu_confirm;
mod logistics_targets_panel;
#[cfg(feature = "legacy_engine")]
mod in_game_ui;
mod gui_assets;
mod gui_sets;
mod ui_windows;
mod agent_permissions_ui;
mod ai_explainability_ui;
mod diagnostics_ui;
mod faction_tools_ui;
mod camera_focus_debug;
mod tile_debug_types;
mod gpu_tile_debug;
mod tile_readability;
mod strategic_icon_instances;
pub mod style;
pub mod components;
pub mod editor;

// Public exports
pub use app_shell::{AppShellPlugin, LoadStubPath};
pub use crate::construction::BuildPlanningPlugin;
pub use assembly_snapshot_qc_ui::{
    aps_bevy_qc_hud_001_witness_green, aps_bevy_qc_hud_001_witness_json,
    aps_bevy_qc_hud_v2_witness_green, load_qc_snapshot, evaluate_p0_readonly,
    placement_grid_coords, AssemblySnapshotQcUiPlugin, AssemblySnapshotQcUiState,
    APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT,
};
pub use vfx_fire_test_highlight::{
    draw_vfx_fire_test_highlight_overlay, refresh_vfx_fire_test_highlight_from_burning,
    vfx_fire_test_highlight_001_witness_green, vfx_fire_test_highlight_001_witness_json,
    VfxFireTestHighlight, VfxFireTestHighlightPlugin,
};
pub use settlement_block_frame_debug::{
    draw_block_frame_debug_overlay, settlement_block_frame_debug_overlay_wired_witness_green,
    BlockFrameDebugUiState, SettlementBlockFrameDebugPlugin,
};
pub use input_bindings::InputBindings;
pub use minimap_shell::{
    minimap_cursor_logical, minimap_uv_to_world_tile, minimap_window_logical_size,
    native_minimap_window_supported, simulation_minimap_bootstrap_rect,
    minimap_overlay_witness_harness, simulation_minimap_overlay_defaults, MinimapCameraBookmark, MinimapEdge, MinimapFollowMode,
    MinimapOverlayMask, MinimapPresentationMode, MinimapPresentationSource, MinimapShellState,
    MINIMAP_EDGE_RAIL_PX, MINIMAP_RESIZE_GRIP_PX, MINIMAP_TITLE_BAR_H_PX,
};
pub use minimap_egui_dev::{minimap_egui_dev_enabled, MinimapEguiDevGate};
pub use minimap_viewport_frame::{
    clamp_tactical_viewport_frame_rect, paint_tactical_viewport_frame_on_minimap,
    tactical_visible_world_rect, world_tile_to_minimap_screen,
};
pub use map_presentation_controls::map_overlay_controls_ui;
pub use map_presentation_diagnostics::{
    sync_map_fit_transform_components, MapPresentationConsumerDiagnostics, MapPresentationDiagnostics,
};
pub use map_presentation_fit::{
    compute_map_fit_strict, default_fit_mode_for, fit_viewport_to_map, map_fit_rect,
    map_fit_zoom_for_panel, MapAspectMode, MapDisplayResult, MapFitMode, MAP_PANEL_INSET_PX,
};
pub use map_view::{
    clear_active_map_view_input_before_map_widgets, map_toolbar, map_toolbar_minimap_zoom,
    map_toolbar_preview_zoom, MapToolbarConfig, ActiveMapViewInput, MapDisplayTransform, MapFitValidation,
    MapFitValidationLog, MapShellPointerGate,
    MapViewInstanceId, MapViewInstances, MapViewInteractionByView, MapViewPlugin, MapViewState,
    resolve_minimap_texture_source, resolve_world_preview_texture_source,
    MinimapInteractionBuffer,
    MapViewPresentationInteractions, MapViewPresentationState, MapViewPresentationStates,
    MapViewReadyStates, MapViewTextureCache, MapTextureSource, ResolvedMapViewFrame,
    ResolvedMapViewFrames, ViewHandle, WorldPreviewInteractionBuffer, minimap, sync_shell_layout_drag_gate,
    world_preview,
};
pub use map_view_projection::{
    ensure_viewport_camera_initialized, map_display_rect, map_presentation_image_rect,
    map_surface_screen_to_world, map_surface_world_to_screen, map_texture_uv_rect,
};
pub use hud::{HudPanelStateWitness, HudPanelState, HudPanelStatePlugin,
    HudCommandShellLayout, HudDockRegistry, HudDockShellPlugin, HudOverlayTrayState,
    TransmissionShellPlugin, TransmissionShellState,
};
pub use style::{
    error_text,
    forbid_raw_colors,
    framed_group,
    CmdUiMonoFont,
    muted_label,
    muted_text,
    neutral_image_tint,
    path_hint,
    primary_label,
    primary_text,
    scenario_execution_badge,
    section_heading,
    status_badge,
    strong_body,
    success_text,
    v_space,
    warning_text,
    weak_body,
    CmdHeadingStyle,
    StatusTone,
    UiPalette,
    UiSpacing,
    UiThemePlugin,
    VertSpace,
};
pub use egui_root::new_root_ui;
pub use egui_window::std_floating;
pub use sim_map_rtt::{
    apply_simulation_map_camera_clear, insert_simulation_map_texture, rtt_diag_camera_mode,
    simulation_map_rtt_clear_color,
    simulation_map_rtt_image, simulation_map_rtt_render_layers, simulation_map_texture_extent,
    spawn_main_world_rtt_camera, spawn_simulation_hud_ui_camera, RttDiagCameraConfig,
    RttDiagCameraMode, SimulationHudUiCamera, SimulationMapFillRect,
    TacticalMapFillRect, SimulationMapTexture, SIMULATION_MAP_RTT_RENDER_LAYER,
};
pub use authoritative_viewport::{
    bootstrap_authoritative_viewport_on_enter_simulation, measure_sim_map_fill_viewport,
    sync_simulation_map_fill_debug_trace, AuthoritativeViewport, CENTER_ROW_HORIZONTAL_PAD_PX,
};
pub use map_camera::{
    default_map_zoom_for_world, in_simulation_or_editor_map, map_camera_viewport_pixels,
    map_zoom_limits_for_world, orthographic_fixed_world_span, primary_cursor_world_xy,
    MainWorldCamera, MainWorldCameraOrthoTrace, MAIN_WORLD_CAMERA_Z, MapCameraDesired, MapCameraDesiredRes,
    MapCameraMode,
    sync_main_world_camera_viewport_and_projection,
    MapCameraPlugin, MapCameraSettings, MapCameraSystemSet,
    derive_map_camera_desired_from_view_authority, mirror_world_main_camera_from_map_desired,
    map_scale_for_zoom_alpha, map_zoom_alpha, map_zoom_alpha_with_limits,
    on_world_main_pose_committed,
    sim_map_cursor_world_xy, sim_map_image_rect,
    sim_map_screen_to_world_xy, sim_map_screen_to_world_xy_with_ortho, sim_map_visible_world_span, sim_map_world_vec3_to_egui,
    sim_map_world_xy_to_egui, sim_map_world_xy_to_egui_with_ortho, TACTICAL_VFX_PROOF_ZOOM_ALPHA,
    trace_map_camera_desired_write_if_full_app, MAP_ZOOM_CLAMP, MAP_ZOOM_AXIS_SNAP_EPS,
};
pub use map_zoom_coherence::{
    map_pick_closure_math_witness_green, map_zoom_coherence_001_witness_green,
    map_zoom_coherence_001_witness_json, map_zoom_axis_snap_applies,
    MAP_ZOOM_DOUBLE_WORLD_FRAMES_MAX, MAP_ZOOM_GHOST_SCREEN_DELTA_PX_MAX,
    MAP_ZOOM_PICK_DELTA_WORLD_MAX,
};
pub use sim_map_projection::{
    camera_map_plane_vec3_to_logical_screen, map_camera_desired_from_presentation,
    map_camera_pose_for_presentation, sim_map_projection_frame,
    sim_map_screen_to_world_xy_in_frame,
    sim_map_world_vec3_to_egui_rendered, MapCameraPresentationPose, SimMapProjectionFrame,
};
pub use camera_focus_debug::{
    fire_chunk_coords_above_visual_eps, CameraFocusDebug, CameraFocusDebugPlugin,
    DEBUG_CHUNK_SPACING_WORLD,
};
pub use gpu_tile_debug::{
    build_tile_debug_instances, triage_gpu_tile_wgsl_001_green, GpuTileDebugPlugin,
};
pub use tile_readability::{
    apply_readability_to_lod_inputs, apply_tile_readability_lod_bias, readability_zoom_floor,
    screen_pixels_per_tile, TileReadabilityConfig, TileReadabilityPlugin,
    TileReadabilityWitness, ZoomVisualBias,
};
pub use strategic_icon_instances::{
    StrategicIconInstanceBuffer, StrategicIconInstancesPlugin, STRATEGIC_ICON_SCAFFOLD,
};
pub use tile_debug_types::{
    construction_phase_on_instanced_path, tile_debug_instanced_has_instances,
    tile_debug_use_gizmos_instead, tile_flags, FireDebugOverride, TileDebugDrawGlobals, TileDebugInstance,
    TileDebugInstanceMap, TileDebugRenderHost, TileDebugViewId, TileGpuDebugSettings,
};
pub use representation_policy::{
    build_representation_inputs, build_representation_result, FireVisualExtractPlan,
    GpuBudgetPolicy, GPU_FIRE_INSTANCE_BUDGET_CEILING, LodZoneClass, RepresentationBand,
    RepresentationInputs, RepresentationResult, WorldRepresentationExtractPlan,
    ComputeBudgetPolicy, OverlayPolicy, representation_band_from_world_lod,
};
pub use world_representation::{
    compute_world_representation_frame, gather_lod_gameplay_signals, CameraLodState,
    GlobalLodState, LodCell, LodGameplaySignals, LodGlobalRules,
    LodInputs, LodZoneId, LodZoneRegistry, LodZoneSource, OperationalLodZone,
    TacticalEscalation, TacticalLodBubble, TacticalLodBubbleRegistry, WorldLodBand, WorldLodBands,
    WorldLodMap, WorldLodPolicyEngine, WorldRepresentationFrame, WorldRepresentationResolver,
    WorldRepresentationSystemSet, WorldResolutionPolicy, WorldVisibilityMask, resolution_for_band,
    visibility_for_band,
};
pub use representation_spine_audit::{
    fire_visual_producer_count, VisualProducerRegistration, REGISTERED_VISUAL_PRODUCERS,
};
pub use representation_governance::{
    RepresentationAuthorityClass, ScaffoldContract, STAGE5_FIX_PRIORITY_ORDER,
    STAGE5_FULL_APP_EXIT_PROOFS, STAGE5_MANDATORY_CLOSURES, TIER1_CONVERGENCE_LANES,
};
pub use view_representation::ViewRepresentationSystemSet;
pub use viewport_layout_solver::{
    commit_authority_from_semantic, semantic_viewport_from_map_fill, stabilize_viewport_floor,
    viewport_rescue_floor, SemanticViewportRect, ViewportAuthorityNode, ViewportSemanticNode,
    ViewportSemanticSource,
};
pub use viewport_authority::{
    clear_viewport_requests, submit_viewport_request, ResolvedViewport, ViewportAuthority,
    ViewportRequest, VIEWPORT_PRIORITY_DEBUG, VIEWPORT_PRIORITY_MINIMAP, VIEWPORT_PRIORITY_PREVIEW,
};
pub use view_authority::{
    commit_map_camera_pose_to_view_authority, commit_world_main_map_focus,
    map_camera_desired_from_view_authority, sync_view_manager_world_main_from_authority,
    tactical_camera_world_pose,
    view_camera_state_from_map_camera_desired,
    DebugFlags, OverlayMask, PerViewLodHints, ViewAuthorityPlugin, ViewAuthoritySystemSet,
    ViewCameraState, ViewCameraTag, ViewFilterMask, ViewId, ViewInstance, ViewInteractionState,
    ViewIsolationDiagnostics, ViewManager, ViewProjection, ViewRenderPolicy, ViewRenderTarget,
    VIEW_NO_ENTITY, vm10_minimap_lockstep_diagnostics_green,
};
pub use view_projection_authority::{
    camera_translation, camera_zoom, view_instance, view_surface_screen_to_world,
    view_surface_world_to_screen, view_visible_world_rect,
};
pub use view_representation_snapshot::{
    build_view_representation_snapshot, validate_view_representation_snapshot,
    SnapshotCameraState, ViewRepresentationSnapshot, WorldBounds,
};
pub use view_representation::{
    apply_camera_visual_from_map_snapshot, apply_minimap_camera_intent,
    camera_owner_label, on_visual_cadence_atmosphere, on_visual_cadence_minimap,
    on_visual_cadence_overlay, on_visual_cadence_preview, preview_partial_min_interval_from_hz,
    preview_partial_min_interval_secs, ActiveCameraOwner, AtmosphereFx, CameraIntent, CameraOwner,
    CameraVisualState, FireLodSelection, FireVisualLod, FxVisibilitySettings, OverlayChannel,
    OverlayFieldFrame, SwapImageBuffers, ViewRepresentationPlugin, VisualBudgetSettings,
    VisualCadence, WorldFireFx,
};
pub use main_menu::*;
pub use splash::*;
pub use input_bindings::*;
pub use input_frame::{InputFrame, InputFramePlugin};
pub use gameplay_capture::*;
pub use options_keybindings_ui::*;
pub use ui_gates::*;
pub use logistics_focus::*;
pub use in_game_hud::*;
pub use in_game_pause_menu::{toggle_pause_menu_on_escape, InGamePauseMenuPlugin};
pub use pause_menu_bevy::{witness_pause_menu_bevy_replay, PauseMenuBevyPlugin};
pub use pause_menu_confirm::PauseMenuPendingAction;
pub use logistics_targets_panel::*;
#[cfg(feature = "legacy_engine")]
pub use in_game_ui::*;
pub use ui_windows::*;
pub use agent_permissions_ui::*;
pub use ai_explainability_ui::AiExplainabilityPlugin;
pub use diagnostics_ui::*;
pub use faction_tools_ui::*;
pub use pressure_tooling::{
    PressureBevyOverlayRoot, PressureComposerPlugin, PressureComposerState, PressureComposerTab,
    PressureOverlayBevyPlugin, StrategicToolingPlugin,
};
pub use editor::*;