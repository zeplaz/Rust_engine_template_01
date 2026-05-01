//! Central keyboard bindings for tooling / HUD panels (editable from **Options → Key bindings**).
//!
//! Persisted as RON under the path from [`InputBindings::default_input_bindings_ron_path`].
//!
//! **Inventory** (search `KeyCode::` outside this file — should only be defaults, presets, tests,
//! and fallbacks tied to `InputBindings`):
//! - Diagnostics, faction tools, logistics cycle / list, world gen, agent permissions, egui scale
//! - Options window, simulation pause toggle (`SimControlState` via `sim_control`), capture cancel
//!
//! Reserved: capture flow uses [`InputBindings::cancel_keybinding_capture`] instead of a hardcoded key.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Rebindable keys. Defaults match the historical hardcoded shortcuts.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct InputBindings {
    pub toggle_keybindings_options: KeyCode,
    pub toggle_diagnostics: KeyCode,
    pub toggle_faction_tools: KeyCode,
    pub cycle_logistics_focus: KeyCode,
    pub toggle_logistics_targets_panel: KeyCode,
    pub toggle_world_generator: KeyCode,
    pub toggle_agent_permissions: KeyCode,
    /// Toggles egui scale compensation (`ui_windows::update_ui_scale_factor_system`).
    pub toggle_egui_ui_scale: KeyCode,
    /// Toggle `SimControlState.paused` (`systems::sim_control::keyboard_toggle_pause`).
    pub toggle_simulation_pause: KeyCode,
    /// While capturing a binding in the options UI, this key aborts capture (not written into RON as a game action).
    pub cancel_keybinding_capture: KeyCode,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            toggle_keybindings_options: KeyCode::F1,
            toggle_diagnostics: KeyCode::F3,
            toggle_faction_tools: KeyCode::F4,
            cycle_logistics_focus: KeyCode::F9,
            toggle_logistics_targets_panel: KeyCode::F10,
            toggle_world_generator: KeyCode::F8,
            toggle_agent_permissions: KeyCode::F7,
            toggle_egui_ui_scale: KeyCode::Slash,
            toggle_simulation_pause: KeyCode::KeyP,
            cancel_keybinding_capture: KeyCode::Escape,
        }
    }
}

impl InputBindings {
    #[must_use]
    pub fn format_key(code: KeyCode) -> String {
        match code {
            KeyCode::Slash => "/".to_string(),
            _ => format!("{code:?}"),
        }
    }

    /// User-writable config directory + `input_bindings.ron`.
    #[must_use]
    pub fn default_input_bindings_ron_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("proc_A_dine01")
                .join("input_bindings.ron");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local/share/proc_A_dine01/input_bindings.ron");
        }
        PathBuf::from("user_settings/input_bindings.ron")
    }

    /// Read bindings from `path`. Returns `None` if missing or invalid.
    pub fn try_load_from_ron_path(path: &Path) -> Option<Self> {
        let s = fs::read_to_string(path).ok()?;
        ron::from_str(&s).ok()
    }

    /// Pretty RON write; creates parent directories.
    pub fn save_to_ron_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cfg = ron::ser::PrettyConfig::new().depth_limit(4);
        let s = ron::ser::to_string_pretty(self, cfg).map_err(|e| e.to_string())?;
        fs::write(path, s).map_err(|e| e.to_string())
    }
}

/// Keys offered in the keybindings dropdown (subset of `KeyCode` to keep the UI small).
pub fn binding_preset_keys() -> &'static [KeyCode] {
    &[
        KeyCode::Escape,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
        KeyCode::Slash,
        KeyCode::Backslash,
        KeyCode::Comma,
        KeyCode::Period,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_bindings_ron_roundtrip() {
        let original = InputBindings::default();
        let s =
            ron::ser::to_string_pretty(&original, ron::ser::PrettyConfig::new()).unwrap();
        let decoded: InputBindings = ron::from_str(&s).unwrap();
        assert_eq!(original, decoded);
    }
}
