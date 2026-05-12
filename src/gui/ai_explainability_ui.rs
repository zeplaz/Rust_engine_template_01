//! L7 egui **AI explainability** window — macro resolution + micro pipeline (`simulation_explainability_runbook_v1.md`).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::input_bindings::InputBindings;
use crate::gui::style::{muted_label, primary_label, section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::strategic::{
    format_hybrid_telemetry_explain, DecisionExplainabilitySnapshot, HybridSimLastResolved,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct ExplainabilityUiState {
    pub visible: bool,
}

pub struct AiExplainabilityPlugin;

impl Plugin for AiExplainabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplainabilityUiState>()
            .add_systems(
                Update,
                toggle_explainability_ui.run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                ai_explainability_egui_system.run_if(in_simulation_or_editor),
            );
    }
}

fn toggle_explainability_ui(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<ExplainabilityUiState>,
) {
    if keys.just_pressed(bindings.toggle_ai_explainability) {
        state.visible = !state.visible;
    }
}

fn ai_explainability_egui_system(
    mut contexts: EguiContexts,
    ui_state: Res<ExplainabilityUiState>,
    bindings: Res<InputBindings>,
    palette: Res<UiPalette>,
    snap: Option<Res<DecisionExplainabilitySnapshot>>,
    hybrid_last: Option<Res<HybridSimLastResolved>>,
) -> Result {
    if !ui_state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;

    crate::gui::std_floating(
        egui::Window::new(format!(
            "AI explainability ({})",
            InputBindings::format_key(bindings.toggle_ai_explainability)
        ))
        .default_size(egui::vec2(460.0, 520.0)),
    )
    .show(ctx, |ui| {
        muted_label(
            ui,
            &palette,
            "Interpreted telemetry only — no raw weights. Balancing / missions / mods.",
        );
        ui.separator();

        section_heading(
            ui,
            &palette,
            CmdHeadingStyle::Gt,
            "Macro resolution (last hybrid tick)",
        );
        if let Some(last) = hybrid_last.as_ref() {
            if let Some(ref tel) = last.telemetry {
                for line in format_hybrid_telemetry_explain(tel) {
                    primary_label(ui, &palette, line);
                }
            } else {
                muted_label(ui, &palette, "No telemetry yet — advance simulation.");
            }
            if let Some(ev) = last.event {
                muted_label(ui, &palette, format!("Last event enum: {ev:?}"));
            }
        } else {
            muted_label(ui, &palette, "HybridSimLastResolved not loaded in this app.");
        }

        ui.separator();
        section_heading(
            ui,
            &palette,
            CmdHeadingStyle::Gt,
            "Micro pipeline (strongest agent pulse)",
        );
        if let Some(s) = snap.as_ref() {
            primary_label(
                ui,
                &palette,
                format!(
                    "SimTick {} | sample {:?} | composed {:.4}",
                    s.sim_tick, s.sample_entity, s.composed
                ),
            );
            for line in &s.pipeline_contributors {
                primary_label(ui, &palette, line.as_str());
            }
        } else {
            muted_label(ui, &palette, "DecisionExplainabilitySnapshot missing — load BehaviorPlugin stack.");
        }
    });

    Ok(())
}
