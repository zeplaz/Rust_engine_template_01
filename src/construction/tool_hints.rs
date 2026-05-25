//! Bottom-left tool hint overlay (Round 2).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::InputBindings;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::path_feedback::ConstructionPathFeedback;

pub fn draw_tool_hints_egui(
    mut contexts: bevy_egui::EguiContexts,
    tool: Res<ActiveBuildTool>,
    bindings: Res<InputBindings>,
    path_feedback: Res<ConstructionPathFeedback>,
) -> Result {
    if matches!(tool.tool, BuildTool::None) {
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
            "LMB: move ghost on map",
            "Shift+LMB: queue blueprint",
            "RMB: clear ghost",
            "Esc: clear ghost (keep tool)",
        ],
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
            if matches!(tool.tool, BuildTool::Building(_)) {
                ui.label(
                    egui::RichText::new(format!("{confirm_key}: place building"))
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
