//! Developmental HUD — L0/L1 context and validation copy (`developmental_ux_runbook_v1.md` § UX-1…UX-4 scaffolding).

pub mod dock_shell;
pub mod panel_state;
pub mod explainability_viewer;
pub mod frame_budget_diagnostics;
pub mod retained_widget_cache;
pub mod hud_interaction_budget;
pub mod virtualized_list;
pub mod shell_surface;
pub mod interaction_latency;
pub mod hud_shell_stress_harness;
pub mod cached_egui_texture;
pub mod hud_async_task_queue;
pub mod hud_dev_overlay;
pub mod hud_root_tick;
pub mod hud_chrome;
pub mod hud_side_status_panel;
pub mod layout_store;
pub mod pending_hud_layout_commit;
pub mod overlay_framework;
pub mod player_intent_panel;
pub mod shell_cohesion;
pub mod shell_diagnostics;
pub mod shell_framework;
pub mod shell_persistence;
pub mod shell_update_budget;
pub mod shell_widget_timing;
pub mod stage5_spine_consumer;
pub mod stage6_consumer;
pub mod info_tabs;
pub mod overlay_shell;
pub mod stage6_telemetry;
pub mod stage7_ui_shell;
pub mod transmission_media;
pub mod widget_presentation;
pub mod world_interaction_diagnostics;
pub mod cause_chain;
pub mod contextual_tip;
pub mod strategic_preview;
pub mod tool_help;
pub mod transmission;
pub mod validation_feedback;
pub mod viewport_rect_sanity;
pub mod sim_view_sync_debug;
pub mod simulation_session;
pub mod simulation_pointer_gate;
pub mod minimap_bevy_interaction;
pub mod layout_debug;
pub mod viewport_authority_debug;
pub mod icon_atlas;
pub mod simulation_shell_phase2;
pub mod sim_hud_l5_polish;
pub mod ui_stress_state;
pub mod ui_shell_migration;

pub use cause_chain::{
    update_developmental_cause_strip_system, DevelopmentalCauseStripLine, DevelopmentalCauseStripRoot,
};
pub use contextual_tip::{update_developmental_context_strip_system, DevelopmentalContextStripLine};
pub use dock_shell::{
    HudCommandShellLayout, HudDockShellPlugin, HudOverlayTrayState,
};
pub use hud_side_status_panel::{
    draw_hud_side_status_panel_egui, hud_status_side_panel_toggle_system,
};
pub use panel_state::{HudPanel, HudPanelState, HudPanelStatePlugin, HudPanelStateWitness};
pub use icon_atlas::{
    tool_context_uses_icon_atlas, IconAtlasManifest, IconAtlasPlugin, IconAtlasUi, IconId,
};
pub use sim_hud_l5_polish::{
    sim_hud_info_panel_tokens_ok, sim_hud_l5_polish_rollup_green, sim_hud_slice_build_green,
    sim_hud_slice_dock_green, sim_hud_slice_minimap_green, sim_hud_slice_ops_polish_green,
    INFO_PANEL_BODY_FONT_MIN_PX, OPS_STRIP_BODY_FONT_PT, OPS_STRIP_FONT_MIN_PX,
};
pub use simulation_shell_phase2::{
    collapse_context_tray_on_escape, format_sim_tick_line, BuildRailRoot, BuildRailToolIcon,
    BuildRailToolLabel, BuildRailToolSlot, ContextTrayBodyLine, ContextTrayBodyRoot,
    ContextTrayRoot, ContextTrayState,
    ContextTrayTab, ContextTrayTabButton, ContextTrayTabLabel, MapViewportFrameInset,
    MinimapChromeRoot, MinimapGpuImageNode, OpsStripAlertBadge, OpsStripAlertBadgeText,
    OpsStripAlerts, OpsStripIntel, OpsStripPower, OpsStripTime, OpsStripTrayAffordance,
    OpsStripWeather, OpsStripZone, OpsStripOrdersPendingText, LogisticsVehicleChip,
    LogisticsVehicleChipIcon,
    LogisticsVehicleChipLabel, LogisticsVehicleChipRow, PetroleumPanelTabIcon,
    PetroleumPanelTabLabel, PetroleumPanelTabRoot, SimulationShellPhase2Plugin,
    OpsStripZoneLinesSet, UiShellMigrationLiveProofState,
    UiShellMigrationPlugin, UiShellMigrationWitness, CONTEXT_RAIL_W_PX, CONTEXT_TRAY_BODY_H_PX,
    CONTEXT_TRAY_TAB_H_PX, MAP_FRAME_INSET_PX, OPS_STRIP_TOP_OFFSET_PX,
};
pub use ui_shell_migration::*;
pub use simulation_session::{apply_simulation_hud_defaults, SimulationSessionPlugin};
pub use hud_dev_overlay::{HudDevOverlayPlugin, HudDevOverlayState};
pub use explainability_viewer::{
    draw_explainability_viewer, mock_explainability_events, ExplainabilityFeedEvent,
    ExplainabilityViewerState,
};
pub use overlay_framework::{
    default_overlay_channel_runtimes, draw_overlay_legend, OverlayChannelRuntime,
    OverlayFrameworkState,
};
pub use shell_framework::{
    capture_shell_layout, draw_minimized_shell_chip, draw_shell_window_chrome,
    floating_unanchored_default_pos,
    shell_anchored_default_pos, shell_default_window_pos, shell_widget_runs_egui,
    shell_widget_runs_egui_with_budget, show_product_shell_window, sync_shell_slot_from_outcome,
    HudDockRegistry, HudWidgetDockState, HudWidgetId, ProductShellRegistry, ProductShellWidgetId,
    ShellWidgetRuntime, ShellWindowHost, ShellWindowOutcome,
};
pub use info_tabs::{
    draw_info_tab_bar, draw_info_tab_body, HudInfoLiveData, HudInfoTab, HudInfoTabPlugin,
    HudInfoTabState, sync_hud_info_live_data,
};
pub use shell_diagnostics::{product_shell_diagnostics_tick, ProductShellDiagnostics};
pub use shell_update_budget::{
    advance_product_shell_update_budget, ProductShellUpdateBudget, ShellRefreshPolicy,
    ShellRefreshTier, ShellWidgetRuntimeState, BACKGROUND_PANEL_HZ, DETACHED_UNFOCUSED_HZ,
};
pub use shell_widget_timing::{
    ShellWidgetDiagnostics, ShellWidgetTimingRow, WidgetFrameCost, WidgetRebuildReason,
};
pub use widget_presentation::{
    HudShellInteractionRouter, WidgetPresentationBackend, WidgetPresentationBackendKind,
    WidgetPresentationPolicy, WidgetShellState,
};
pub use hud_async_task_queue::{
    drain_hud_async_task_queue, HudAsyncResultCache, HudAsyncTask, HudAsyncTaskKind,
    HudAsyncTaskQueue, TelemetrySnapshot,
};
pub use retained_widget_cache::{draw_retained_lines_or_build, RetainedWidgetCache, RetainedWidgetFrame};
pub use hud_interaction_budget::{
    apply_hud_interaction_frame_budget, HudFrameBudget, DEFAULT_ASYNC_BUDGET_MS, DEFAULT_UI_BUDGET_MS,
};
pub use virtualized_list::draw_virtualized_rows;
pub use shell_surface::{ShellSurfaceMode, ShellSurfacePolicy};
pub use interaction_latency::{refresh_interaction_latency_metrics, InteractionLatencyMetrics};
pub use hud_shell_stress_harness::{HudShellStressHarness, HudShellStressReport};
pub use cached_egui_texture::{
    reset_hud_egui_texture_frame, CachedEguiTextureBinding, HudEguiTextureCache,
};
pub use pending_hud_layout_commit::{
    finalize_pending_hud_layout_commits, flush_pending_hud_layout_commits,
    flush_pending_hud_layout_on_pointer_release, PendingHudLayoutCommit,
};
pub use world_interaction_diagnostics::{
    refresh_world_interaction_diagnostics, WorldInteractionDiagnostics,
};
pub use frame_budget_diagnostics::{
    finalize_frame_budget_diagnostics, FrameBudgetAnomalyKind, FrameBudgetAnomalyReport,
    FrameBudgetBucket, FrameBudgetBucketStats, FrameBudgetDiagnostics, FrameBudgetTimer,
    Stage6VirtualizationBudget, FRAME_HISTORY_LEN, RESIDENCY_CHURN_BOOTSTRAP_FRAMES,
    RESIDENCY_CHURN_CELL_DELTA, RESIDENCY_CHURN_HYSTERESIS_FRAMES,
};
pub use sim_view_sync_debug::{sim_view_sync_debug_enabled, SimViewSyncDebugPlugin};
pub use layout_debug::{
    ui_layout_debug_enabled, DebugLayoutTag, HudLayoutDebugPlugin, UI_LAYOUT_DEBUG_TARGET,
};
pub use crate::gui::debug::{
    ui_layout_tree_debug_enabled, UiLayoutTreeDebugPlugin, UI_LAYOUT_TREE_TARGET,
};
pub use viewport_authority_debug::{
    assert_viewport_integrity, stroke_viewport_debug_rect, trace_viewport_authority,
    trace_viewport_chain_integrity, trace_viewport_drift, viewport_authority_debug_enabled,
    viewport_debug_overlay_enabled, CameraProjectionInfo, RenderViewportRect, UiViewportRect,
    ViewportAuthoritySource, ViewportIntegrityAssertPlugin,
    VIEWPORT_AUTHORITY_TARGET,
};
pub use viewport_rect_sanity::{
    ViewportRectIssue, ViewportRectIssueKind, ViewportRectSanity, ViewportRectSource,
    VIEWPORT_RECT_COLLAPSED_MAX, VIEWPORT_RECT_SAFE_MIN, VIEWPORT_SIM_MAP_LAYOUT_MIN_H,
    VIEWPORT_SIM_MAP_LAYOUT_MIN_W, VIEWPORT_SIM_MAP_SAFE_MIN_H, VIEWPORT_SIM_MAP_SAFE_MIN_W,
};
pub use shell_persistence::{
    MinimapBookmarkCollectionR8, MinimapBookmarkEntryR8, OverlayPresetCollectionR8,
    OverlayPresetEntryR8, ProductShellPersistenceBundleR8,
};
pub use stage5_spine_consumer::draw_stage5_spine_consumer_panel;
pub use stage6_consumer::{
    draw_stage6_residency_consumer_panel, mock_residency_overlay_consumer,
    ResidencyOverlayConsumerDto,
};
pub use stage6_telemetry::{
    gpu_shell_resource_stats_from, refresh_stage6_hud_telemetry, residency_overlay_consumer_from_frame,
    GpuShellResourceStats, Stage6HudTelemetry,
};
pub use player_intent_panel::{PlayerIntentDraft, PlayerIntentPanelState};
pub use layout_store::{
    HudLayoutCollectionR8, HudLayoutStore, HudWidgetFrame, HudWidgetLayoutEntryR8, HudWidgetRectR8,
};
pub use overlay_shell::{mock_overlay_channel_descriptors, OverlayShellPlugin, OverlayShellState};
pub use stage7_ui_shell::{mock_belief_snapshots, mock_dispatch_envelopes, Stage7UiShellPlugin};
pub use transmission::{TransmissionShellPlugin, TransmissionShellState};
pub use transmission_media::{
    seed_ux_e03_transmission_media_registry, ux_e03_coder_a_green,
    TransmissionMediaProviderKind, TransmissionMediaProviderRegistry,
};
pub use validation_feedback::{ValidationDiagnostic, ValidationSeverity};
