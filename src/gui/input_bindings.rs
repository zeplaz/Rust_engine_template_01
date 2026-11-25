//! Central keyboard bindings for play HUD panels (editable from **Options → Key bindings**).
//!
//! # Config authority split (IO-11)
//!
//! Ref-books **IO-11** · LEGACY keyword `engine_configuration` (GEA 4e I §6.5 / §6.5.1–6.5.3.1,
//! gea4v1 p455–458 · gea_v1_e4 p454): load/save options, **per-user** prefs, and Quake-style
//! **cvars** for per-game engine tuning must not share one blob.
//!
//! | Store | Type | Path helper | Who edits |
//! |:---|:---|:---|:---|
//! | **Per-user options** | [`InputBindings`] | [`user_options_root`] / [`InputBindings::default_input_bindings_ron_path`] | Play Options → Key bindings |
//! | **Per-game tuning** | [`EngineGameTuning`] (cvars) | [`game_config_root`] / [`EngineGameTuning::default_ron_path`] | Install / ship defaults; not the keybindings panel |
//!
//! Ref-books **INPUT-03** · Input remapping (GEA 4e I §9.5.7, gea4v1 p562–564): [`InputBindings`]
//! is the single action table between devices and gameplay — remaps stay in the per-user store.
//!
//! **Inventory** (search `KeyCode::` outside this file — should only be defaults, presets, tests,
//! and fallbacks tied to `InputBindings`):
//! - Diagnostics, AI explainability (L7), logistics cycle / list, world gen
//! - Options window, simulation pause toggle (`SimControlState` via `sim_control`), capture cancel
//! - Gameplay capture: window screenshots → PNG sequence + GIF (`gameplay_capture`)
//! - Map camera: WASD / edge scroll / wheel+keys zoom / grip pan / rotate (`map_camera`)
//!
//! Reserved: capture flow uses [`InputBindings::cancel_keybinding_capture`] instead of a hardcoded key.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Which config store owns a blob (GEA 6.5.1–6.5.2). Never mix the two RON files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConfigStoreKind {
    /// Per-user preferences (key bindings). Under APPDATA / XDG; never the sole install-adjacent path.
    PerUser,
    /// Per-game / engine tuning (cvars). Under `game_config/`; not written by the keybindings panel.
    PerGame,
}

/// Per-user options root (`%APPDATA%/proc_A_dine01` · `~/.local/share/proc_A_dine01` · `user_settings/`).
#[must_use]
pub fn user_options_root() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("proc_A_dine01");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/proc_A_dine01");
    }
    PathBuf::from("user_settings")
}

/// Per-game / engine config root (install-adjacent cwd). Distinct from [`user_options_root`].
#[must_use]
pub fn game_config_root() -> PathBuf {
    PathBuf::from("game_config")
}

/// One Quake-style cvar entry (GEA 6.5.3.1). Archived cvars may persist in the per-game RON.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EngineCvar {
    pub name: String,
    pub value: String,
    /// When true, included when writing [`EngineGameTuning`] to disk.
    pub archive: bool,
}

/// Per-game engine tuning (cvars). **Not** mixed into the [`InputBindings`] user-options RON.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EngineGameTuning {
    pub cvars: Vec<EngineCvar>,
}

impl Default for EngineGameTuning {
    fn default() -> Self {
        Self {
            cvars: vec![
                EngineCvar {
                    name: "sim_speed_default".into(),
                    value: "1.0".into(),
                    archive: true,
                },
                EngineCvar {
                    name: "capture_frame_stride".into(),
                    value: "1".into(),
                    archive: true,
                },
                EngineCvar {
                    name: "capture_max_frames".into(),
                    value: "3600".into(),
                    archive: true,
                },
            ],
        }
    }
}

impl EngineGameTuning {
    #[must_use]
    pub const fn store_kind() -> ConfigStoreKind {
        ConfigStoreKind::PerGame
    }

    /// Install-adjacent `game_config/engine_tuning.ron` (not under the per-user options root).
    #[must_use]
    pub fn default_ron_path() -> PathBuf {
        game_config_root().join("engine_tuning.ron")
    }

    #[must_use]
    pub fn cvar(&self, name: &str) -> Option<&str> {
        self.cvars
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.value.as_str())
    }

    /// Read tuning from `path`. Returns `None` if missing or invalid.
    pub fn try_load_from_ron_path(path: &Path) -> Option<Self> {
        let s = fs::read_to_string(path).ok()?;
        ron::from_str(&s).ok()
    }

    /// Pretty RON write of archived cvars only; creates parent directories.
    pub fn save_to_ron_path(&self, path: &Path) -> Result<(), String> {
        let archived = EngineGameTuning {
            cvars: self
                .cvars
                .iter()
                .filter(|c| c.archive)
                .cloned()
                .collect(),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cfg = ron::ser::PrettyConfig::new().depth_limit(4);
        let s = ron::ser::to_string_pretty(&archived, cfg).map_err(|e| e.to_string())?;
        fs::write(path, s).map_err(|e| e.to_string())
    }
}

/// Rebindable keys — **per-user options** ([`ConfigStoreKind::PerUser`]). Defaults match historical shortcuts.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct InputBindings {
    pub toggle_keybindings_options: KeyCode,
    /// Toggle the AI explainability panel (faction AI decisions + macro pipeline; default Insert).
    pub toggle_ai_explainability: KeyCode,
    pub cycle_logistics_focus: KeyCode,
    pub toggle_logistics_targets_panel: KeyCode,
    /// Scenario script runner (`scenario_script_panel`, Editor). Default **F10**; logistics targets panel uses **F6** to avoid clash.
    pub toggle_scenario_script_panel: KeyCode,
    /// Collapse / expand the left command context stack (Bevy HUD panel under the operations strip).
    pub toggle_command_left_stack: KeyCode,
    /// Cycle operational **build / planning** tool context (`construction::BuildStripState`; Bevy HUD shows active mode).
    pub cycle_build_planning_tool: KeyCode,
    /// Toggle construction site / ghost **info labels** on the tactical map (tile outlines stay on).
    pub toggle_construction_tile_labels: KeyCode,
    /// Confirm ghost site placement (commits [`CommitConstructionSiteEvent`](crate::strategic::CommitConstructionSiteEvent) when valid).
    pub confirm_build_placement: KeyCode,
    /// Toggle simulation minimap egui shell.
    pub toggle_sim_minimap: KeyCode,
    /// Rotate build ghost footprint (quarter turns).
    pub rotate_build_ghost: KeyCode,
    /// Mirror build ghost on X axis.
    pub mirror_build_ghost: KeyCode,
    /// Clear pending blueprint queue while planning.
    pub clear_pending_blueprints: KeyCode,
    pub toggle_world_generator: KeyCode,
    /// Toggle `SimControlState.paused` (`systems::sim_control::keyboard_toggle_pause`).
    pub toggle_simulation_pause: KeyCode,
    /// While capturing a binding in the options UI, this key aborts capture (not written into RON as a game action).
    pub cancel_keybinding_capture: KeyCode,
    /// Start full-window gameplay capture (PNG stream; stop builds `clip.gif`).
    pub start_gameplay_recording: KeyCode,
    pub stop_gameplay_recording: KeyCode,

    // --- Map camera (`map_camera::MapCameraPlugin`) ---
    pub map_pan_north: KeyCode,
    pub map_pan_south: KeyCode,
    pub map_pan_west: KeyCode,
    pub map_pan_east: KeyCode,
    /// Hold with WASD / edge pan for faster movement.
    pub map_pan_fast_modifier: KeyCode,
    /// Hold + mouse drag pans (middle mouse also pans without this).
    pub map_mouse_grip: KeyCode,
    pub map_zoom_in: KeyCode,
    pub map_zoom_out: KeyCode,
    pub map_rotate_ccw: KeyCode,
    pub map_rotate_cw: KeyCode,
    /// Toggle RTS edge scroll (default Scroll Lock).
    pub map_toggle_edge_scroll: KeyCode,
    /// Recenter on world bounds center (default Home).
    pub map_recenter_world: KeyCode,
    /// Reset orthographic zoom to scenario default (default Z).
    pub map_reset_zoom: KeyCode,
    /// Fit map to window in tile space (default B).
    pub map_frame_world: KeyCode,
    /// Cycle map camera mode label: Strategic → Tactical → Cinematic (default M).
    pub map_cycle_camera_mode: KeyCode,
    /// Toggle world presentation: Tactical (top-down) ↔ World (PixelWorld3D / voxel massing) (default V).
    pub toggle_world_presentation: KeyCode,
    /// Voxel world view: turn the camera a quarter left (default Q). PixelWorld3D only.
    pub pixel_world_rotate_left: KeyCode,
    /// Voxel world view: turn the camera a quarter right (default E). PixelWorld3D only.
    pub pixel_world_rotate_right: KeyCode,
    /// Cycle what clicks / box-drags select: all → buildings → civilian → military (default F4).
    pub selection_filter_cycle: KeyCode,

    // --- Front controls (`gui::front_controls`): fronts, formations, orders ---
    /// Hold the selected formation(s) in place (front controls).
    pub front_hold: KeyCode,
    /// Order the selection to dig in on its theater (Fortify).
    pub front_fortify: KeyCode,
    /// Cycle the selection through your own formations.
    pub front_cycle_selection: KeyCode,
    /// Show / hide every front overlay layer.
    pub front_toggle_overlay: KeyCode,
    /// Toggle the front-line layer.
    pub front_toggle_fronts: KeyCode,
    /// Toggle the theater-outline layer.
    pub front_toggle_theaters: KeyCode,
    /// Toggle the pressure-point layer.
    pub front_toggle_pressure: KeyCode,
    /// Toggle the formation-symbol layer.
    pub front_toggle_formations: KeyCode,
    /// Toggle front overlay labels.
    pub front_toggle_labels: KeyCode,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            toggle_keybindings_options: KeyCode::F1,
            toggle_ai_explainability: KeyCode::Insert,
            cycle_logistics_focus: KeyCode::F9,
            toggle_logistics_targets_panel: KeyCode::F6,
            toggle_scenario_script_panel: KeyCode::F10,
            toggle_command_left_stack: KeyCode::Backquote,
            cycle_build_planning_tool: KeyCode::Semicolon,
            toggle_construction_tile_labels: KeyCode::F7,
            confirm_build_placement: KeyCode::Enter,
            toggle_sim_minimap: KeyCode::KeyN,
            rotate_build_ghost: KeyCode::KeyR,
            mirror_build_ghost: KeyCode::KeyX,
            clear_pending_blueprints: KeyCode::Backspace,
            toggle_world_generator: KeyCode::F8,
            toggle_simulation_pause: KeyCode::KeyP,
            cancel_keybinding_capture: KeyCode::Escape,
            start_gameplay_recording: KeyCode::F11,
            stop_gameplay_recording: KeyCode::F12,
            map_pan_north: KeyCode::KeyW,
            map_pan_south: KeyCode::KeyS,
            map_pan_west: KeyCode::KeyA,
            map_pan_east: KeyCode::KeyD,
            map_pan_fast_modifier: KeyCode::ShiftLeft,
            map_mouse_grip: KeyCode::Space,
            map_zoom_in: KeyCode::Equal,
            map_zoom_out: KeyCode::Minus,
            map_rotate_ccw: KeyCode::Comma,
            map_rotate_cw: KeyCode::Period,
            map_toggle_edge_scroll: KeyCode::ScrollLock,
            map_recenter_world: KeyCode::Home,
            map_reset_zoom: KeyCode::KeyZ,
            map_frame_world: KeyCode::KeyB,
            map_cycle_camera_mode: KeyCode::KeyM,
            toggle_world_presentation: KeyCode::KeyV,
            pixel_world_rotate_left: KeyCode::KeyQ,
            pixel_world_rotate_right: KeyCode::KeyE,
            selection_filter_cycle: KeyCode::F4,
            front_hold: KeyCode::KeyH,
            front_fortify: KeyCode::KeyF,
            front_cycle_selection: KeyCode::Tab,
            front_toggle_overlay: KeyCode::KeyK,
            front_toggle_fronts: KeyCode::Digit1,
            front_toggle_theaters: KeyCode::Digit2,
            front_toggle_pressure: KeyCode::Digit3,
            front_toggle_formations: KeyCode::Digit4,
            front_toggle_labels: KeyCode::Digit5,
        }
    }
}

impl InputBindings {
    #[must_use]
    pub const fn store_kind() -> ConfigStoreKind {
        ConfigStoreKind::PerUser
    }

    #[must_use]
    pub fn format_key(code: KeyCode) -> String {
        match code {
            KeyCode::Slash => "/".to_string(),
            KeyCode::Backquote => "`".to_string(),
            _ => format!("{code:?}"),
        }
    }

    /// Per-user `input_bindings.ron` under [`user_options_root`].
    #[must_use]
    pub fn default_input_bindings_ron_path() -> PathBuf {
        user_options_root().join("input_bindings.ron")
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
        KeyCode::Tab,
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
        KeyCode::Insert,
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
        KeyCode::Semicolon,
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
        KeyCode::Backquote,
        KeyCode::Backslash,
        KeyCode::Comma,
        KeyCode::Period,
        KeyCode::Enter,
        KeyCode::Space,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::Equal,
        KeyCode::Minus,
        KeyCode::Home,
        KeyCode::ScrollLock,
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

    /// Old saved files without the front-control keys still load, with the defaults.
    #[test]
    fn missing_front_keys_fall_back_to_defaults() {
        let decoded: InputBindings = ron::from_str("(toggle_world_generator: F9)").unwrap();
        assert_eq!(decoded.toggle_world_generator, KeyCode::F9);
        assert_eq!(decoded.front_fortify, InputBindings::default().front_fortify);
    }

    /// The front-control defaults must not shadow any other default binding.
    #[test]
    fn front_defaults_do_not_collide_with_other_bindings() {
        let text = ron::ser::to_string_pretty(&InputBindings::default(), ron::ser::PrettyConfig::new()).unwrap();
        let pairs: Vec<(String, String)> = text
            .lines()
            .filter_map(|l| l.trim().trim_end_matches(',').split_once(": "))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let front: Vec<&(String, String)> = pairs.iter().filter(|(k, _)| k.starts_with("front_")).collect();
        assert_eq!(front.len(), 9);
        for (k, v) in &front {
            for (k2, v2) in &pairs {
                if k != k2 {
                    assert_ne!(v, v2, "{k} collides with {k2}");
                }
            }
        }
    }

    #[test]
    fn per_user_and_per_game_stores_are_distinct() {
        assert_eq!(InputBindings::store_kind(), ConfigStoreKind::PerUser);
        assert_eq!(EngineGameTuning::store_kind(), ConfigStoreKind::PerGame);
        let user = InputBindings::default_input_bindings_ron_path();
        let game = EngineGameTuning::default_ron_path();
        assert_ne!(user, game);
        assert_ne!(user_options_root(), game_config_root());
        assert_eq!(user.file_name().and_then(|s| s.to_str()), Some("input_bindings.ron"));
        assert_eq!(game.file_name().and_then(|s| s.to_str()), Some("engine_tuning.ron"));
        assert_eq!(user.parent(), Some(user_options_root().as_path()));
        assert_eq!(game.parent(), Some(game_config_root().as_path()));
    }

    #[test]
    fn user_options_ron_does_not_embed_game_tuning_cvars() {
        let text =
            ron::ser::to_string_pretty(&InputBindings::default(), ron::ser::PrettyConfig::new())
                .unwrap();
        assert!(
            !text.contains("sim_speed_default"),
            "per-user bindings must not serialize per-game cvars"
        );
        assert!(!text.contains("capture_frame_stride"));
        assert!(!text.contains("cvars"));
    }

    #[test]
    fn engine_game_tuning_ron_roundtrip_archives_only() {
        let mut tuning = EngineGameTuning::default();
        tuning.cvars.push(EngineCvar {
            name: "dbg_scratch".into(),
            value: "1".into(),
            archive: false,
        });
        let dir = std::env::temp_dir().join(format!(
            "proc_a_dine01_io11_tuning_{}",
            std::process::id()
        ));
        let path = dir.join("engine_tuning.ron");
        tuning.save_to_ron_path(&path).expect("save tuning");
        let loaded = EngineGameTuning::try_load_from_ron_path(&path).expect("load tuning");
        assert!(loaded.cvar("sim_speed_default").is_some());
        assert!(loaded.cvar("dbg_scratch").is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn per_user_bindings_save_load_roundtrip_on_disk() {
        let dir = std::env::temp_dir().join(format!(
            "proc_a_dine01_io11_user_{}",
            std::process::id()
        ));
        let path = dir.join("input_bindings.ron");
        let mut bindings = InputBindings::default();
        bindings.toggle_simulation_pause = KeyCode::KeyO;
        bindings.save_to_ron_path(&path).expect("save user options");
        let loaded = InputBindings::try_load_from_ron_path(&path).expect("load user options");
        assert_eq!(loaded.toggle_simulation_pause, KeyCode::KeyO);
        let _ = fs::remove_dir_all(&dir);
    }
}
