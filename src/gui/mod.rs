// GUI systems
// Boundary: see prompts/guides/ui_boundary_guide_v1.md
//   splash, app_shell, in_game_hud → Bevy shell / simulation HUD
//   editor/* + selected egui panels  → dev tooling (gated via ui_gates)
//   pressure_tooling.rs             → F2 pressure composer (egui) + Bevy strip
//   ai_explainability_ui.rs         → L7 pipeline + macro explain (default Insert)
//   agent_permissions_ui.rs         → egui tooling

pub mod build;
pub mod hud;
mod map_camera;
mod lod_zone_authoring;
mod world_representation;
mod representation_policy;
mod representation_spine_audit;
mod view_representation;
pub mod map_tile_raster;
pub mod egui_window;
mod app_shell;
mod main_menu;
mod splash;
mod ui_gates;
mod input_bindings;
mod input_frame;
mod gameplay_capture;
mod options_keybindings_ui;
mod pressure_tooling;
mod logistics_focus;
mod in_game_hud;
mod logistics_targets_panel;
mod in_game_ui;     // LEGACY MODULE — kept for migration trace
mod gui_assets;
mod gui_sets;
mod ui_windows;
mod agent_permissions_ui;
mod ai_explainability_ui;
mod diagnostics_ui;
mod faction_tools_ui;
pub mod style;
pub mod components;
pub mod editor;

// Public exports
pub use app_shell::{AppShellPlugin, LoadStubPath};
pub use build::BuildPlanningPlugin;
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
pub use egui_window::std_floating;
pub use map_camera::{
    default_map_zoom_for_world, MainWorldCamera, MapCameraDesired, MapCameraMode, MapCameraPlugin,
    MapCameraSettings, MapCameraSystemSet,
};
pub use representation_policy::{
    build_representation_inputs, build_representation_result, FireVisualExtractPlan,
    LodZoneClass, RepresentationBand,
    RepresentationInputs, RepresentationResult, WorldRepresentationExtractPlan,
    ComputeBudgetPolicy, GpuBudgetPolicy, OverlayPolicy, representation_band_from_world_lod,
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
pub use view_representation::ViewRepresentationSystemSet;
pub use view_representation::{
    apply_camera_visual_from_map_snapshot,
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
pub use logistics_targets_panel::*;
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