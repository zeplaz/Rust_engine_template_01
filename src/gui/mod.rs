// GUI systems
// Boundary: see prompts/guides/ui_boundary_guide_v1.md
//   splash, app_shell, in_game_hud → Bevy shell / simulation HUD
//   editor/* + selected egui panels  → dev tooling (gated via ui_gates)
//   agent_permissions_ui.rs          → egui tooling

mod map_camera;
mod app_shell;
mod main_menu;
mod splash;
mod ui_gates;
mod input_bindings;
mod options_keybindings_ui;
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
pub mod components;
pub mod editor;

// Public exports
pub use app_shell::{AppShellPlugin, LoadStubPath};
pub use map_camera::{MainWorldCamera, MapCameraPlugin};
pub use main_menu::*;
pub use splash::*;
pub use input_bindings::*;
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
pub use editor::*;