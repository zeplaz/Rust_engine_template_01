// GUI systems
// Boundary: see prompts/guides/ui_boundary_guide_v1.md
//   splash, app_shell, in_game_hud → Bevy shell / simulation HUD
//   editor/* + selected egui panels  → dev tooling (gated via ui_gates)
//   pressure_tooling.rs             → F2 pressure composer (egui) + Bevy strip
//   agent_permissions_ui.rs         → egui tooling

mod map_camera;
pub mod map_tile_raster;
pub mod egui_window;
mod app_shell;
mod main_menu;
mod splash;
mod ui_gates;
mod input_bindings;
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
mod diagnostics_ui;
mod faction_tools_ui;
pub mod style;
pub mod components;
pub mod editor;

// Public exports
pub use app_shell::{AppShellPlugin, LoadStubPath};
pub use style::{
    error_text,
    forbid_raw_colors,
    framed_group,
    muted_text,
    neutral_image_tint,
    path_hint,
    primary_text,
    scenario_execution_badge,
    section_heading,
    status_badge,
    success_text,
    warning_text,
    CmdHeadingStyle,
    StatusTone,
    UiPalette,
    UiSpacing,
    UiThemePlugin,
};
pub use egui_window::std_floating;
pub use map_camera::{MainWorldCamera, MapCameraPlugin};
pub use main_menu::*;
pub use splash::*;
pub use input_bindings::*;
pub use gameplay_capture::*;
pub use options_keybindings_ui::*;
pub use ui_gates::*;
pub use logistics_focus::*;
pub use in_game_hud::*;
pub use logistics_targets_panel::*;
pub use in_game_ui::*;
pub use ui_windows::*;
pub use agent_permissions_ui::*;
pub use diagnostics_ui::*;
pub use faction_tools_ui::*;
pub use pressure_tooling::{
    PressureBevyOverlayRoot, PressureComposerPlugin, PressureComposerState, PressureComposerTab,
    PressureOverlayBevyPlugin, StrategicToolingPlugin,
};
pub use editor::*;