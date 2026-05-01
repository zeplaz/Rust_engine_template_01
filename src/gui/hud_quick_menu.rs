//! Floating egui **Panels** strip (complement to Bevy HUD toolbar buttons).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent;

use super::input_bindings::InputBindings;
use super::options_keybindings_ui::KeybindingsUiState;
use super::DiagnosticsUiState;
use super::FactionToolsState;
use super::LogisticsTargetsPanelState;

pub struct HudQuickMenuPlugin;

impl Plugin for HudQuickMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, hud_quick_panel_buttons);
    }
}

fn hud_quick_panel_buttons(
    mut contexts: EguiContexts,
    bindings: Res<InputBindings>,
    mut diag: ResMut<DiagnosticsUiState>,
    mut faction: ResMut<FactionToolsState>,
    mut logistics: ResMut<LogisticsTargetsPanelState>,
    mut keys_ui: ResMut<KeybindingsUiState>,
    mut worldgen_ev: MessageWriter<ToggleWorldGenUiEvent>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Area::new(egui::Id::new("hud_quick_panel_picker"))
        .fixed_pos(egui::pos2(8.0, 118.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(40))
                .inner_margin(egui::Margin::same(6))
                .show(ui, |ui| {
                    ui.label("Panels");
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .small_button(format!(
                                "Diag ({})",
                                InputBindings::format_key(bindings.toggle_diagnostics)
                            ))
                            .clicked()
                        {
                            diag.visible = !diag.visible;
                        }
                        if ui
                            .small_button(format!(
                                "Faction ({})",
                                InputBindings::format_key(bindings.toggle_faction_tools)
                            ))
                            .clicked()
                        {
                            faction.visible = !faction.visible;
                        }
                        if ui
                            .small_button(format!(
                                "Logi ({})",
                                InputBindings::format_key(bindings.toggle_logistics_targets_panel)
                            ))
                            .clicked()
                        {
                            logistics.visible = !logistics.visible;
                        }
                        if ui
                            .small_button(format!(
                                "World ({})",
                                InputBindings::format_key(bindings.toggle_world_generator)
                            ))
                            .clicked()
                        {
                            worldgen_ev.write(ToggleWorldGenUiEvent);
                        }
                        if ui
                            .small_button(format!(
                                "Keys ({})",
                                InputBindings::format_key(bindings.toggle_keybindings_options)
                            ))
                            .clicked()
                        {
                            keys_ui.visible = !keys_ui.visible;
                        }
                    });
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_salt("hud_open_panel_once")
                        .selected_text("Open panel (shows once)…")
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(false, "Diagnostics")
                                .clicked()
                            {
                                diag.visible = true;
                            }
                            if ui.selectable_label(false, "Faction tools").clicked() {
                                faction.visible = true;
                            }
                            if ui.selectable_label(false, "Logistics list").clicked() {
                                logistics.visible = true;
                            }
                            if ui.selectable_label(false, "World generator").clicked() {
                                worldgen_ev.write(ToggleWorldGenUiEvent);
                            }
                            if ui.selectable_label(false, "Key bindings").clicked() {
                                keys_ui.visible = true;
                            }
                        });
                });
        });

    Ok(())
}
