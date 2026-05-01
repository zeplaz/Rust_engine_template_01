//! Devtools diagnostics window — egui (`F3`).
//!
//! Purpose: minimal **iteration-loop UX** — see FPS, drive sim (pause/step/speed),
//! count entities. Future tabs: streaming stats, ECS counters, channel drops.
//!
//! Designer:
//! - `prompts/designer_questions/tools_ui/spec/04_metrics_diagnostics.md`
//! - `prompts/designer_questions/tools_ui/implementation_questions_v1.md` §5–10
//!
//! Pattern mirrors `crate::gui::agent_permissions_ui::permissions_ui_system`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use super::input_bindings::InputBindings;
use crate::systems::sim_control::{SimControlState, SimTick};

/// UI visibility + cheap rolling FPS estimate.
#[derive(Resource, Debug, Clone)]
pub struct DiagnosticsUiState {
    pub visible: bool,
    /// Exponential-moving-average FPS; populated each frame from `Time::delta_secs()`.
    pub fps_smoothed: f32,
}

impl Default for DiagnosticsUiState {
    fn default() -> Self {
        Self { visible: true, fps_smoothed: 0.0 }
    }
}

pub struct DiagnosticsUiPlugin;

impl Plugin for DiagnosticsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsUiState>()
            .add_systems(Update, (toggle_diagnostics_ui, sample_fps))
            .add_systems(EguiPrimaryContextPass, diagnostics_ui_system);
    }
}

fn toggle_diagnostics_ui(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<DiagnosticsUiState>,
) {
    if keys.just_pressed(bindings.toggle_diagnostics) {
        state.visible = !state.visible;
    }
}

fn sample_fps(time: Res<Time>, mut state: ResMut<DiagnosticsUiState>) {
    let dt = time.delta_secs();
    if dt > f32::EPSILON {
        let inst = 1.0 / dt;
        state.fps_smoothed = if state.fps_smoothed <= 0.0 {
            inst
        } else {
            state.fps_smoothed * 0.9 + inst * 0.1
        };
    }
}

/// Renders the panel; consumers add tabs by extending this system or chaining own systems
/// in `EguiPrimaryContextPass` after this one.
pub fn diagnostics_ui_system(
    mut contexts: EguiContexts,
    state: Res<DiagnosticsUiState>,
    bindings: Res<InputBindings>,
    mut ctrl: ResMut<SimControlState>,
    tick: Res<SimTick>,
    entities: Query<Entity>,
) -> Result {
    if !state.visible {
        return Ok(());
    }

    let entity_count = entities.iter().count();
    let ctx = contexts.ctx_mut()?;

    egui::Window::new(format!(
        "Diagnostics ({})",
        InputBindings::format_key(bindings.toggle_diagnostics)
    ))
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(format!("FPS (EMA): {:.1}", state.fps_smoothed));
            ui.label(format!("Sim tick:  {}", tick.0));
            ui.label(format!("Entities:  {entity_count}"));

            ui.separator();
            ui.heading("Sim control");
            ui.horizontal(|ui| {
                if ui.button(if ctrl.paused { "Play" } else { "Pause" }).clicked() {
                    ctrl.paused = !ctrl.paused;
                }
                if ui.button("Step").clicked() {
                    ctrl.steps_remaining = ctrl.steps_remaining.saturating_add(1);
                    ctrl.paused = true;
                }
            });
            ui.add(egui::Slider::new(&mut ctrl.speed, 0.0..=8.0).text("speed"));

            // TODO: tabs — chunk streamer, production manifest summary, faction roster.
        });

    Ok(())
}
