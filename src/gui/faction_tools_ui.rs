//! Faction tools window — egui (`F4`).
//!
//! Stub UI for the faction editor; populated against `FactionBlueprint` once the data model lands.
//!
//! Designer:
//! - `prompts/designer_questions/factions/faction_editor/02_ui_egui_panels.md`
//! - `prompts/designer_questions/factions/faction_editor/01_data_model.md`
//! - `prompts/designer_questions/factions/implementation_questions_v1.md`
//!
//! Pattern mirrors `crate::systems::production::tools_ui::ProductionToolsUiPlugin`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionToolsPanel {
    Roster,
    Blueprint,
    Diplomacy,
    ImportExport,
}

#[derive(Resource, Debug, Clone)]
pub struct FactionToolsState {
    pub visible: bool,
    pub active: FactionToolsPanel,
}

impl Default for FactionToolsState {
    fn default() -> Self {
        Self { visible: false, active: FactionToolsPanel::Roster }
    }
}

pub struct FactionToolsUiPlugin;

impl Plugin for FactionToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionToolsState>()
            .add_systems(Update, toggle_faction_tools)
            .add_systems(EguiPrimaryContextPass, faction_tools_ui_system);
    }
}

fn toggle_faction_tools(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<FactionToolsState>) {
    if keys.just_pressed(KeyCode::F4) {
        state.visible = !state.visible;
    }
}

pub fn faction_tools_ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<FactionToolsState>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Faction Tools (F4)")
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.active, FactionToolsPanel::Roster, "Roster");
                ui.selectable_value(&mut state.active, FactionToolsPanel::Blueprint, "Blueprint");
                ui.selectable_value(&mut state.active, FactionToolsPanel::Diplomacy, "Diplomacy");
                ui.selectable_value(&mut state.active, FactionToolsPanel::ImportExport, "Import/Export");
            });
            ui.separator();

            match state.active {
                FactionToolsPanel::Roster => {
                    ui.label("Roster — bind to FactionBlueprint store when available.");
                    // TODO: list + add/duplicate/retire (authority-gated).
                }
                FactionToolsPanel::Blueprint => {
                    ui.label("Blueprint inspector — name, HSL color picker, tags, emblem.");
                    // TODO: bind selected FactionBlueprint fields.
                }
                FactionToolsPanel::Diplomacy => {
                    ui.label("Diplomacy matrix — pairwise stances + interlocking modifiers.");
                    // TODO: render N×N stance grid; integrate DiplomaticRelations permission gate.
                }
                FactionToolsPanel::ImportExport => {
                    ui.label("Import/Export — RON blueprints (per assets matrix).");
                    // TODO: file dialog (crate vs native — see implementation_questions §7).
                }
            }
        });

    Ok(())
}
