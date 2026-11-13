//! Developmental HUD — L0/L1 context and validation copy (`developmental_ux_runbook_v1.md` § UX-1…UX-4 scaffolding).

pub mod dock_shell;
pub mod panel_state;
pub mod live_data;
pub mod shell_diagnostics;
pub mod shell_persistence;
pub mod layout_store;
pub mod pending_hud_layout_commit;
pub mod frame_budget_diagnostics;
pub mod hud_ui_scale;
pub mod hud_chrome;
pub mod sim_play_bevy_chrome;
pub mod shell_framework;
pub mod plant_info_card;
pub mod sim_hud_esc_cascade;
pub mod minimap_topology_legend;
pub mod tool_help;
pub mod validation_feedback;
pub mod viewport_rect_sanity;
pub mod simulation_session;
pub mod simulation_pointer_gate;
pub mod minimap_bevy_interaction;
pub mod viewport_authority_debug;
pub mod icon_atlas;
pub mod power_hud_icon_atlas;
pub mod plant_focus_card;
pub mod simulation_shell_phase2;
pub mod sim_hud_copy;
pub mod sim_build_picker_sheet;
pub mod sim_build_picker_panel;
pub mod sim_road_power_sheets_bevy;
pub mod sim_road_tool_sheet;
pub mod sim_power_tool_sheet;
pub mod power_grid_copy;
pub mod power_node_hover;
pub mod context_tray_build_egui;

pub use dock_shell::{
    HudCommandShellLayout, HudDockShellPlugin, HudOverlayTrayState,
};
pub use panel_state::{HudPanel, HudPanelState, HudPanelStatePlugin};
pub use icon_atlas::{
    tool_context_uses_icon_atlas, IconAtlasManifest, IconAtlasPlugin, IconAtlasUi, IconId,
};
pub use power_hud_icon_atlas::{
    power_hud_atlas_assets_on_disk, power_hud_bevy_draw_path_wired,
    PowerHudDrawnIcon, PowerHudEguiTextureCache,
    PowerHudIconAtlasManifest, PowerHudIconAtlasPlugin, PowerHudIconAtlasUi, PowerHudIconId,
};
pub use plant_focus_card::{
    plant_focus_card_gauges_wired, sync_plant_focus_card_visibility, PlantFocusCardSnapshot,
};
pub use sim_build_picker_sheet::{
    BuildPickerCategory, SimBuildPickerState, AD_HOC_SUBMENU_WINDOWS, BUILD_PICKER_RAIL_GAP_PX,
    BUILD_PICKER_SHEET_W_PX,
};
pub use sim_road_tool_sheet::SimRoadToolSheetState;
pub use sim_power_tool_sheet::{sim_power_tool_sheet_icons_wired, SimPowerToolSheetState};
pub use simulation_shell_phase2::{
    collapse_context_tray_on_escape, format_sim_tick_line, BuildRailRoot, BuildRailToolIcon,
    BuildRailToolLabel, BuildRailToolSlot, ContextTrayBodyLine, ContextTrayBodyRoot,
    ContextTrayRoot, ContextTrayState,
    ContextTrayTab, ContextTrayTabButton, ContextTrayTabLabel, MapViewportFrameInset,
    MinimapChromeRoot, MinimapChromeOverlayHint, MinimapChromeResizeGrip, MinimapChromeTitleBar,
    MinimapChromeTitleText, MinimapGpuImageNode, OpsStripAlertBadge, OpsStripAlertBadgeText,
    OpsStripAlerts, OpsStripIntel, OpsStripPower, OpsStripTime, OpsStripTrayAffordance,
    OpsStripWeather, OpsStripZone, OpsStripOrdersPendingText, LogisticsVehicleChip,
    LogisticsVehicleChipIcon,
    LogisticsVehicleChipLabel, LogisticsVehicleChipRow, PetroleumPanelTabIcon,
    PetroleumPanelTabLabel, PetroleumPanelTabRoot, SimulationShellPhase2Plugin,
    OpsStripZoneLinesSet, BUILD_RAIL_W_PX, COMMAND_LEFT_STACK_COLUMN_GAP_PX,
    LEFT_CONTEXT_STACK_BODY_W_PX, CONTEXT_RAIL_W_PX, CONTEXT_TRAY_BODY_H_PX,
    CONTEXT_TRAY_TAB_H_PX, MAP_FRAME_INSET_PX, OPS_STRIP_TOP_OFFSET_PX,
};
pub use simulation_session::{
    apply_simulation_hud_defaults, suppress_simulation_visual_debug_chrome, SimulationSessionPlugin,
};
pub use shell_framework::{
    HudDockRegistry, HudWidgetDockState, HudWidgetId, ProductShellRegistry, ProductShellWidgetId,
    ShellWidgetRuntime,
};
pub use frame_budget_diagnostics::{
    finalize_frame_budget_diagnostics, FrameBudgetAnomalyKind, FrameBudgetAnomalyReport,
    FrameBudgetBucket, FrameBudgetBucketStats, FrameBudgetDiagnostics, FrameBudgetTimer,
    Stage6VirtualizationBudget, FRAME_HISTORY_LEN, RESIDENCY_CHURN_BOOTSTRAP_FRAMES,
    RESIDENCY_CHURN_CELL_DELTA, RESIDENCY_CHURN_HYSTERESIS_FRAMES,
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
pub use validation_feedback::{ValidationDiagnostic, ValidationSeverity};
pub use shell_diagnostics::ProductShellDiagnostics;
pub use shell_persistence::{
    MinimapBookmarkCollectionR8, MinimapBookmarkEntryR8, OverlayPresetCollectionR8,
    OverlayPresetEntryR8, ProductShellPersistenceBundleR8,
};
pub use layout_store::{
    HudLayoutCollectionR8, HudLayoutStore, HudWidgetFrame, HudWidgetLayoutEntryR8, HudWidgetRectR8,
};
pub use pending_hud_layout_commit::PendingHudLayoutCommit;
pub use live_data::{sync_hud_info_live_data, HudInfoLiveData, HudLiveDataPlugin};
