//! Simulation control loop: pause, single-step, speed multiplier, monotonic tick.
//!
//! Designer doc: `prompts/designer_questions/tools_ui/spec/01_plugin_schedule_patterns.md`.
//! UI driver:   `crate::gui::diagnostics_ui::DiagnosticsUiPlugin` (diagnostics panel).
//!
//! Pause hotkey: [`crate::gui::InputBindings::toggle_simulation_pause`] (Options → key bindings).

use bevy::prelude::*;

use crate::gui::InputBindings;

/// Operator controls; mutated by tools UI, read by gameplay systems.
#[derive(Resource, Debug, Clone)]
pub struct SimControlState {
    pub paused: bool,
    /// One-shot ticks consumed even while `paused`.
    pub steps_remaining: u32,
    /// 1.0 = real-time. Read by sim systems that want to scale `Time::delta_secs()`.
    pub speed: f32,
}

impl Default for SimControlState {
    fn default() -> Self {
        Self { paused: false, steps_remaining: 0, speed: 1.0 }
    }
}

impl SimControlState {
    /// Whether sim should advance this frame.
    #[inline]
    pub fn should_tick(&self) -> bool {
        !self.paused || self.steps_remaining > 0
    }

    /// Effective dt scaler for sim systems. Returns 0.0 when fully paused.
    #[inline]
    pub fn dt_scale(&self) -> f32 {
        if self.should_tick() { self.speed.max(0.0) } else { 0.0 }
    }
}

/// Monotonic simulation tick counter; incremented when `SimControlState::should_tick()`.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct SimTick(pub u64);

pub struct SimControlPlugin;

impl Plugin for SimControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimControlState>()
            .init_resource::<SimTick>()
            .add_systems(Update, (advance_sim_tick, keyboard_toggle_pause));
    }
}

fn keyboard_toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut ctrl: ResMut<SimControlState>,
) {
    if keys.just_pressed(bindings.toggle_simulation_pause) {
        ctrl.paused = !ctrl.paused;
    }
}

fn advance_sim_tick(mut tick: ResMut<SimTick>, mut ctrl: ResMut<SimControlState>) {
    if ctrl.should_tick() {
        tick.0 = tick.0.wrapping_add(1);
        if ctrl.steps_remaining > 0 {
            ctrl.steps_remaining -= 1;
        }
    }
}
