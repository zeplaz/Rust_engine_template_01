//! Bottom-left tool hint overlay (Round 2).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::InputBindings;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::path_feedback::ConstructionPathFeedback;

/// Sim session routes hints through tray Build peek — no LEFT_BOTTOM floater.
pub const TOOL_HINTS_DRAW_IN_SIM: bool = false;

pub fn draw_tool_hints_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<crate::engine::states::BaseState>>,
    tool: Res<ActiveBuildTool>,
    bindings: Res<InputBindings>,
    path_feedback: Res<ConstructionPathFeedback>,
) -> Result {
    if matches!(tool.tool, BuildTool::None) {
        return Ok(());
    }
    if matches!(base.get(), crate::engine::states::BaseState::Simulation) {
        return Ok(());
    }

    let confirm_key = InputBindings::format_key(bindings.confirm_build_placement);
    let hints: Vec<&str> = match tool.tool {
        BuildTool::None => vec![],
        BuildTool::Zone(_) => vec![
            "LMB: paint tile",
            "Alt+LMB drag: paint area",
            "RMB: undo last",
            "Shift+LMB: queue zone batch",
            "Esc: clear paint (keep tool)",
        ],
        BuildTool::Building(_) => vec![
            "Pick a building in the picker first",
            "LMB: lock ghost on map",
            "Adjust: Ctrl+scroll rotate · Shift+scroll size",
            "LMB again: place (Enter shortcut)",
            "RMB / Esc: unlock ghost",
            "Esc again: cancel building tool",
            "Backspace: clear unapproved queue",
        ],
        BuildTool::Defense(kind) => {
            let lead = match kind {
                crate::construction::DefenseKind::DefensiveWall => {
                    crate::gui::hud::sim_hud_copy::HINT_DEFENSIVE_WALL
                }
                crate::construction::DefenseKind::DragonTeeth => {
                    crate::gui::hud::sim_hud_copy::HINT_DRAGON_TEETH
                }
                crate::construction::DefenseKind::Minefield => {
                    crate::gui::hud::sim_hud_copy::HINT_MINEFIELD
                }
                crate::construction::DefenseKind::TrenchLine => "Trench line — two clicks on map",
                crate::construction::DefenseKind::Bunker => "Bunker — two clicks on map",
            };
            vec![
                lead,
                "LMB: lock ghost on map",
                "Adjust: Ctrl+scroll rotate · Shift+scroll size",
                "LMB again: place (Enter shortcut)",
                "RMB / Esc: unlock ghost",
                "Esc again: cancel defense tool",
            ]
        },
        BuildTool::Road(_) => vec![
            "LMB: add point",
            "RMB: undo point",
            "Shift+LMB: commit segment",
            "Esc: clear path (keep tool)",
        ],
        BuildTool::Rail(_) => vec![
            "LMB: add rail point",
            "RMB: undo point",
            "Shift+LMB: commit track",
            "Esc: clear path (keep tool)",
            "Rail: grade + curve limits apply",
        ],
        BuildTool::PowerLine(_) => vec![
            "LMB: add point",
            "RMB: undo point",
            "Shift+LMB: commit line",
            "O / [ / ]: routing mode",
            "Esc: clear path (keep tool)",
        ],
        BuildTool::Demolish => vec![
            "LMB: pick target",
            "Confirm key: demolish approved",
            "Esc: clear pick (keep tool)",
        ],
    };

    egui::Area::new(egui::Id::new("construction_tool_hints"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(egui::RichText::new(format!("Tool: {}", tool.tool.label())).strong());
            for line in &hints {
                ui.label(egui::RichText::new(*line).small().weak());
            }
            if tool.tool.uses_two_click_place() {
                ui.label(
                    egui::RichText::new(format!("{confirm_key}: place"))
                        .small()
                        .strong(),
                );
            }
            if let Some(hint) = &path_feedback.snap_hint {
                ui.label(egui::RichText::new(hint).small().color(egui::Color32::from_rgb(120, 200, 160)));
            }
            if !path_feedback.required_actions.is_empty() {
                ui.label(
                    egui::RichText::new(format!(
                        "Fix: {}",
                        path_feedback.required_actions.join(" · ")
                    ))
                    .small()
                    .color(egui::Color32::from_rgb(220, 180, 96)),
                );
            }
            ui.label(egui::RichText::new("Ctrl+Z: undo last commit").small().weak());
        });
    Ok(())
}
