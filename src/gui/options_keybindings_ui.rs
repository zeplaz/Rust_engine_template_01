//! **Options → Key bindings** egui window (`InputBindings` defaults: F1 opens this panel).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::input_bindings::{binding_preset_keys, InputBindings};

#[derive(Resource, Debug, Clone, Default)]
pub struct KeybindingsUiState {
    pub visible: bool,
    pub capture_slot: Option<BindingSlot>,
    pub last_io_message: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingSlot {
    ToggleOptions,
    Diagnostics,
    FactionTools,
    CycleLogistics,
    LogisticsTargets,
    WorldGenerator,
    AgentPermissions,
    EguiUiScale,
    SimulationPause,
    CancelCapture,
    MapPanNorth,
    MapPanSouth,
    MapPanWest,
    MapPanEast,
    MapPanFast,
    MapMouseGrip,
    MapZoomIn,
    MapZoomOut,
    MapRotateCcw,
    MapRotateCw,
}

pub struct KeybindingsOptionsPlugin;

impl Plugin for KeybindingsOptionsPlugin {
    fn build(&self, app: &mut App) {
        let bindings = InputBindings::try_load_from_ron_path(&InputBindings::default_input_bindings_ron_path())
            .unwrap_or_default();
        app.insert_resource(bindings)
            .init_resource::<KeybindingsUiState>()
            .add_systems(
                Update,
                (toggle_keybindings_options_window, apply_binding_capture),
            )
            .add_systems(EguiPrimaryContextPass, keybindings_options_ui);
    }
}

fn toggle_keybindings_options_window(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<KeybindingsUiState>,
) {
    if state.capture_slot.is_some() {
        return;
    }
    if keys.just_pressed(bindings.toggle_keybindings_options) {
        state.visible = !state.visible;
    }
}

fn apply_binding_capture(
    keys: Res<ButtonInput<KeyCode>>,
    mut bindings: ResMut<InputBindings>,
    mut state: ResMut<KeybindingsUiState>,
) {
    let Some(slot) = state.capture_slot else {
        return;
    };

    let cancel = bindings.cancel_keybinding_capture;
    if keys.just_pressed(cancel) {
        state.capture_slot = None;
        return;
    }

    for k in keys.get_just_pressed() {
        let k = *k;
        if k == cancel {
            continue;
        }
        match slot {
            BindingSlot::ToggleOptions => bindings.toggle_keybindings_options = k,
            BindingSlot::Diagnostics => bindings.toggle_diagnostics = k,
            BindingSlot::FactionTools => bindings.toggle_faction_tools = k,
            BindingSlot::CycleLogistics => bindings.cycle_logistics_focus = k,
            BindingSlot::LogisticsTargets => bindings.toggle_logistics_targets_panel = k,
            BindingSlot::WorldGenerator => bindings.toggle_world_generator = k,
            BindingSlot::AgentPermissions => bindings.toggle_agent_permissions = k,
            BindingSlot::EguiUiScale => bindings.toggle_egui_ui_scale = k,
            BindingSlot::SimulationPause => bindings.toggle_simulation_pause = k,
            BindingSlot::CancelCapture => bindings.cancel_keybinding_capture = k,
            BindingSlot::MapPanNorth => bindings.map_pan_north = k,
            BindingSlot::MapPanSouth => bindings.map_pan_south = k,
            BindingSlot::MapPanWest => bindings.map_pan_west = k,
            BindingSlot::MapPanEast => bindings.map_pan_east = k,
            BindingSlot::MapPanFast => bindings.map_pan_fast_modifier = k,
            BindingSlot::MapMouseGrip => bindings.map_mouse_grip = k,
            BindingSlot::MapZoomIn => bindings.map_zoom_in = k,
            BindingSlot::MapZoomOut => bindings.map_zoom_out = k,
            BindingSlot::MapRotateCcw => bindings.map_rotate_ccw = k,
            BindingSlot::MapRotateCw => bindings.map_rotate_cw = k,
        }
        state.capture_slot = None;
        return;
    }
}

fn slot_label_id(slot: BindingSlot) -> &'static str {
    match slot {
        BindingSlot::ToggleOptions => "bind_toggle_opts",
        BindingSlot::Diagnostics => "bind_diag",
        BindingSlot::FactionTools => "bind_faction",
        BindingSlot::CycleLogistics => "bind_logi_cycle",
        BindingSlot::LogisticsTargets => "bind_logi_panel",
        BindingSlot::WorldGenerator => "bind_worldgen",
        BindingSlot::AgentPermissions => "bind_agents",
        BindingSlot::EguiUiScale => "bind_egui_scale",
        BindingSlot::SimulationPause => "bind_sim_pause",
        BindingSlot::CancelCapture => "bind_cancel_cap",
        BindingSlot::MapPanNorth => "bind_map_n",
        BindingSlot::MapPanSouth => "bind_map_s",
        BindingSlot::MapPanWest => "bind_map_w",
        BindingSlot::MapPanEast => "bind_map_e",
        BindingSlot::MapPanFast => "bind_map_fast",
        BindingSlot::MapMouseGrip => "bind_map_grip",
        BindingSlot::MapZoomIn => "bind_map_zoom_in",
        BindingSlot::MapZoomOut => "bind_map_zoom_out",
        BindingSlot::MapRotateCcw => "bind_map_rot_ccw",
        BindingSlot::MapRotateCw => "bind_map_rot_cw",
    }
}

fn keybindings_options_ui(
    mut contexts: EguiContexts,
    mut bindings: ResMut<InputBindings>,
    mut state: ResMut<KeybindingsUiState>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let presets = binding_preset_keys();
    let toggle_opts = bindings.toggle_keybindings_options;
    let path = InputBindings::default_input_bindings_ron_path();

    egui::Window::new(format!(
        "Options — key bindings ({})",
        InputBindings::format_key(toggle_opts)
    ))
    .resizable(true)
    .default_width(420.0)
    .show(ctx, |ui| {
        ui.label(format!(
            "Change shortcuts below. Use Capture… or the dropdown. {} cancels capture (configurable below).",
            InputBindings::format_key(bindings.cancel_keybinding_capture)
        ));
        ui.small(format!("RON file: {}", path.display()));
        if let Some(msg) = &state.last_io_message {
            ui.colored_label(egui::Color32::LIGHT_RED, msg);
        }
        if state.capture_slot.is_some() {
            ui.colored_label(egui::Color32::LIGHT_BLUE, "Capturing…");
        }
        ui.separator();

        macro_rules! row_combo {
            ($label:expr, $hint:expr, $field:ident, $slot:expr) => {{
                ui.horizontal(|ui| {
                    ui.label(concat!($label, ":"));
                    let capturing = state.capture_slot == Some($slot);
                    if capturing {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            format!(
                                "Press a key… ({} cancels)",
                                InputBindings::format_key(bindings.cancel_keybinding_capture)
                            ),
                        );
                    } else {
                        let mut chosen = bindings.$field;
                        egui::ComboBox::from_id_salt(slot_label_id($slot))
                            .selected_text(InputBindings::format_key(chosen))
                            .show_ui(ui, |ui| {
                                for &k in presets {
                                    ui.selectable_value(&mut chosen, k, InputBindings::format_key(k));
                                }
                            });
                        if chosen != bindings.$field {
                            bindings.$field = chosen;
                        }
                    }
                    if ui.small_button(if capturing { "Cancel" } else { "Capture…" }).clicked() {
                        state.capture_slot = if capturing { None } else { Some($slot) };
                    }
                });
                ui.small($hint);
                ui.add_space(4.0);
            }};
        }

        row_combo!(
            "Open this options window",
            "Keep at least one key you remember.",
            toggle_keybindings_options,
            BindingSlot::ToggleOptions
        );
        row_combo!(
            "Diagnostics",
            "FPS, sim control, entity counts.",
            toggle_diagnostics,
            BindingSlot::Diagnostics
        );
        row_combo!(
            "Faction tools",
            "Faction editor tooling.",
            toggle_faction_tools,
            BindingSlot::FactionTools
        );
        row_combo!(
            "Cycle logistics HUD focus",
            "Cycles entities with storage (dev-style).",
            cycle_logistics_focus,
            BindingSlot::CycleLogistics
        );
        row_combo!(
            "Logistics target list",
            "Pick inventory focus from a list.",
            toggle_logistics_targets_panel,
            BindingSlot::LogisticsTargets
        );
        row_combo!(
            "World generator",
            "Terrain / world tooling UI.",
            toggle_world_generator,
            BindingSlot::WorldGenerator
        );
        row_combo!(
            "Agent permissions",
            "Only if AgentPermissionsUiPlugin is loaded.",
            toggle_agent_permissions,
            BindingSlot::AgentPermissions
        );
        row_combo!(
            "Toggle egui UI scale",
            "Compensates for display scaling (see ui_windows).",
            toggle_egui_ui_scale,
            BindingSlot::EguiUiScale
        );
        row_combo!(
            "Toggle simulation pause / resume",
            "Flips SimControlState.paused (same as Diagnostics Play/Pause).",
            toggle_simulation_pause,
            BindingSlot::SimulationPause
        );
        ui.separator();
        ui.heading("Map camera (simulation + editor)");
        ui.small("WASD, edge scroll, mouse wheel zoom; Space or middle mouse + drag to pan. Respects egui focus.");
        row_combo!(
            "Map pan north",
            "",
            map_pan_north,
            BindingSlot::MapPanNorth
        );
        row_combo!(
            "Map pan south",
            "",
            map_pan_south,
            BindingSlot::MapPanSouth
        );
        row_combo!(
            "Map pan west",
            "",
            map_pan_west,
            BindingSlot::MapPanWest
        );
        row_combo!(
            "Map pan east",
            "",
            map_pan_east,
            BindingSlot::MapPanEast
        );
        row_combo!(
            "Map pan speed boost (hold)",
            "Multiplies keyboard and edge pan.",
            map_pan_fast_modifier,
            BindingSlot::MapPanFast
        );
        row_combo!(
            "Map mouse pan grip (hold)",
            "Hold and move mouse to pan; middle mouse also pans without this.",
            map_mouse_grip,
            BindingSlot::MapMouseGrip
        );
        row_combo!(
            "Map zoom in (hold)",
            "",
            map_zoom_in,
            BindingSlot::MapZoomIn
        );
        row_combo!(
            "Map zoom out (hold)",
            "",
            map_zoom_out,
            BindingSlot::MapZoomOut
        );
        row_combo!(
            "Map rotate counter-clockwise",
            "",
            map_rotate_ccw,
            BindingSlot::MapRotateCcw
        );
        row_combo!(
            "Map rotate clockwise",
            "",
            map_rotate_cw,
            BindingSlot::MapRotateCw
        );
        ui.separator();
        row_combo!(
            "Cancel keybinding capture",
            "Abort \"Press a key…\" in this panel without assigning.",
            cancel_keybinding_capture,
            BindingSlot::CancelCapture
        );

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Save to disk (RON)").clicked() {
                match bindings.save_to_ron_path(&path) {
                    Ok(()) => {
                        state.last_io_message = Some(format!("Saved {}", path.display()));
                    }
                    Err(e) => {
                        state.last_io_message = Some(format!("Save failed: {e}"));
                    }
                }
            }
            if ui.button("Reload from disk").clicked() {
                match InputBindings::try_load_from_ron_path(&path) {
                    Some(b) => {
                        *bindings = b;
                        state.last_io_message =
                            Some(format!("Reloaded {}", path.display()));
                    }
                    None => {
                        state.last_io_message = Some(format!(
                            "No valid file at {}",
                            path.display()
                        ));
                    }
                }
            }
        });
        if ui.button("Reset defaults").clicked() {
            *bindings = InputBindings::default();
            state.last_io_message = Some("Reset to defaults (not saved until you Save).".into());
        }
    });

    Ok(())
}
