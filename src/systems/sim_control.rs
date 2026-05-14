//! Simulation control loop: pause, single-step, speed multiplier, monotonic tick.
//!
//! Designer doc: `prompts/designer_questions/tools_ui/spec/01_plugin_schedule_patterns.md`.
//! UI driver:   `crate::gui::diagnostics_ui::DiagnosticsUiPlugin` (diagnostics panel).
//!
//! Pause hotkey: [`crate::gui::InputBindings::toggle_simulation_pause`] (Options → key bindings).
//!
//! **Schedule:** [`SimControlSystemSet`] — transport and other gameplay should run **after**
//! [`SimControlSystemSet::AdvanceSimTick`] (see `TransportSimulationPlugin`).

use bevy::prelude::*;

use crate::gui::InputBindings;

/// Ordering hooks for cross-plugin dependencies on `Update`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimControlSystemSet {
    /// Pause / step input before tick advances the same frame.
    ApplyOperatorInput,
    /// `SimTick` + `steps_remaining`; systems that read `SimControlState` for gameplay dt should run after this.
    AdvanceSimTick,
}

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
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimTick(pub u64);

/// Authoritative sim time in microseconds (monotonic while the sim advances).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimTimeMicros(pub u64);

/// Phase **E1** cadence identity for render/compute snapshots (`base_visual_dev01_plan_status` § `phase-e-cadence-scale`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct SimStepStamp {
    pub tick: u64,
    pub sim_time_micros: u64,
}

impl SimStepStamp {
    #[inline]
    #[must_use]
    pub const fn new(tick: u64, sim_time_micros: u64) -> Self {
        Self {
            tick,
            sim_time_micros,
        }
    }

    #[inline]
    #[must_use]
    pub fn from_tick(tick: SimTick, sim_time_micros: SimTimeMicros) -> Self {
        Self::new(tick.0, sim_time_micros.0)
    }
}

pub struct SimControlPlugin;

impl Plugin for SimControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimControlState>()
            .init_resource::<SimTick>()
            .init_resource::<SimTimeMicros>()
            .add_plugins(crate::systems::sim_frame_delta::SimFrameDeltaPlugin)
            .configure_sets(
                Update,
                (
                    SimControlSystemSet::ApplyOperatorInput,
                    SimControlSystemSet::AdvanceSimTick.after(SimControlSystemSet::ApplyOperatorInput),
                ),
            )
            .add_systems(
                Update,
                keyboard_toggle_pause.in_set(SimControlSystemSet::ApplyOperatorInput),
            )
            .add_systems(
                Update,
                advance_sim_tick.in_set(SimControlSystemSet::AdvanceSimTick),
            );
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

fn advance_sim_tick(
    time: Res<Time>,
    mut tick: ResMut<SimTick>,
    mut sim_time: ResMut<SimTimeMicros>,
    mut ctrl: ResMut<SimControlState>,
) {
    if !ctrl.should_tick() {
        return;
    }
    tick.0 = tick.0.wrapping_add(1);
    let delta_micros = (time.delta_secs() * ctrl.speed.max(0.0) * 1_000_000.0) as u64;
    sim_time.0 = sim_time.0.wrapping_add(delta_micros);
    if ctrl.steps_remaining > 0 {
        ctrl.steps_remaining -= 1;
    }
}
