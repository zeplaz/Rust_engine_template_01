// GUI systems
// Boundary: see prompts/guides/ui_boundary_guide_v1.md
//   in_game_hud.rs, splash.rs, main_menu.rs  → native Bevy UI (Node)
//   editor/*                                  → egui tooling
//   agent_permissions_ui.rs                   → egui tooling

mod main_menu;
mod splash;
mod input_bindings;
mod options_keybindings_ui;
mod hud_quick_menu;
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
pub use main_menu::*;
pub use splash::*;
pub use input_bindings::*;
pub use options_keybindings_ui::*;
pub use hud_quick_menu::*;
pub use logistics_focus::*;
pub use in_game_hud::*;
pub use logistics_targets_panel::*;
pub use in_game_ui::*;
pub use gui_assets::*;
pub use gui_sets::*;
pub use ui_windows::*;
pub use agent_permissions_ui::*;
pub use diagnostics_ui::*;
pub use faction_tools_ui::*;
pub use editor::*;