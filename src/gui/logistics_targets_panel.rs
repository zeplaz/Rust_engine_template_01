//! Pick logistics HUD focus from a list (hotkey, **Options** keybindings, this egui panel).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::entities::production::core::{
    LogisticsSiteMember, LogisticsSiteRoot, ResourceStorage,
};

use crate::gui::ui_gates::in_simulation_or_editor;
use crate::gui::input_bindings::InputBindings;
use crate::gui::logistics_focus::HudLogisticsFocus;

#[derive(Resource, Debug, Clone)]
pub struct LogisticsTargetsPanelState {
    pub visible: bool,
}

impl Default for LogisticsTargetsPanelState {
    fn default() -> Self {
        Self { visible: false }
    }
}

pub struct LogisticsTargetsPanelPlugin;

impl Plugin for LogisticsTargetsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogisticsTargetsPanelState>()
            .add_systems(
                Update,
                toggle_logistics_targets_panel.run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                logistics_targets_panel_ui.run_if(in_simulation_or_editor),
            );
    }
}

fn toggle_logistics_targets_panel(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<LogisticsTargetsPanelState>,
) {
    if keys.just_pressed(bindings.toggle_logistics_targets_panel) {
        state.visible = !state.visible;
    }
}

fn logistics_targets_panel_ui(
    mut contexts: EguiContexts,
    state: Res<LogisticsTargetsPanelState>,
    bindings: Res<InputBindings>,
    mut focus: ResMut<HudLogisticsFocus>,
    roots: Query<Entity, With<LogisticsSiteRoot>>,
    storages: Query<Entity, With<ResourceStorage>>,
    members: Query<&LogisticsSiteMember>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;

    egui::Window::new(format!(
        "Logistics targets ({})",
        InputBindings::format_key(bindings.toggle_logistics_targets_panel)
    ))
        .resizable(true)
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.label("Set HUD inventory focus (physical storage / site hub).");
            ui.separator();
            ui.heading("Site hubs");
            for hub in roots.iter() {
                if ui.button(format!("Hub {:?}", hub)).clicked() {
                    focus.tracked_entity = Some(hub);
                }
            }
            ui.separator();
            ui.heading("Storage entities");
            for e in storages.iter() {
                let label = if members.get(e).is_ok() {
                    format!("{:?} (site member → uses hub focus)", e)
                } else {
                    format!("{:?}", e)
                };
                if ui.button(label).clicked() {
                    let is_hub = roots.get(e).is_ok();
                    let m = members.get(e).ok();
                    focus.tracked_entity = Some(
                        crate::entities::production::core::resolve_logistics_focus_entity(
                            e, m, is_hub,
                        ),
                    );
                }
            }
            ui.separator();
            if ui.button("Clear focus").clicked() {
                focus.tracked_entity = None;
            }
            if let Some(t) = focus.tracked_entity {
                ui.label(format!("Current focus: {:?}", t));
            } else {
                ui.label("Current focus: none");
            }
        });

    Ok(())
}
