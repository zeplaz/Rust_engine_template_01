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
use serde::{Deserialize, Serialize};

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

/// Simulated microseconds per gameplay step: one step = one simulated second.
///
/// Every gameplay rate (`rate_per_tick`, order-sim ticks, construction phase steps, AI intervals)
/// is authored per step, so production, deliveries and combat run at the same sim speed at any
/// frame rate.
pub const SIM_STEP_MICROS: u64 = 1_000_000;

/// Catch-up cap: steps one frame may run (high sim speed at a low frame rate). Backlog beyond it
/// is dropped rather than spiralling.
pub const SIM_STEP_MAX_PER_FRAME: u32 = 30;

/// Fixed-rate gameplay clock, quantised from [`SimTimeMicros`] (the one time authority: speed
/// scaled, stops on pause). It adds no time source of its own.
///
/// Sole writer: [`advance_sim_step_clock`]. Gameplay systems either gate on [`sim_step_due`] and
/// run [`SimStepClock::due`] steps, or read [`SimStepClock::steps`] as their tick.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct SimStepClock {
    pub step_micros: u64,
    pub max_steps_per_frame: u32,
    /// Steps completed since the clock started (monotonic).
    pub steps: u64,
    /// Steps completed this frame (0 on most frames at 1x and 60 fps).
    pub due: u32,
    last_micros: Option<u64>,
}

impl Default for SimStepClock {
    fn default() -> Self {
        Self {
            step_micros: SIM_STEP_MICROS,
            max_steps_per_frame: SIM_STEP_MAX_PER_FRAME,
            steps: 0,
            due: 0,
            last_micros: None,
        }
    }
}

impl SimStepClock {
    /// Seconds of sim time one step covers.
    #[must_use]
    pub fn step_secs(&self) -> f32 {
        self.step_micros as f32 / 1_000_000.0
    }
}

/// Number of whole steps elapsed between two sim-time readings, and the new step boundary.
#[must_use]
pub fn due_steps(last: Option<u64>, now: u64, step_micros: u64, cap: u32) -> (u32, u64) {
    let step = step_micros.max(1);
    let Some(last) = last else {
        return (0, now);
    };
    if now <= last {
        return (0, now.min(last));
    }
    let n = (now - last) / step;
    let steps = n.min(u64::from(cap)) as u32;
    // Drop the backlog beyond the cap rather than spiralling.
    let new_last = if n > u64::from(cap) { now } else { last + n * step };
    (steps, new_last)
}

/// Quantise [`SimTimeMicros`] into whole gameplay steps (runs in [`SimControlSystemSet::AdvanceSimTick`]).
pub fn advance_sim_step_clock(time: Option<Res<SimTimeMicros>>, mut clock: ResMut<SimStepClock>) {
    let now = time.map_or(0, |t| t.0);
    let (due, last) = due_steps(clock.last_micros, now, clock.step_micros, clock.max_steps_per_frame);
    clock.last_micros = Some(last);
    clock.due = due;
    clock.steps = clock.steps.wrapping_add(u64::from(due));
}

/// Run condition: at least one gameplay step completed this frame.
#[must_use]
pub fn sim_step_due(clock: Option<Res<SimStepClock>>) -> bool {
    clock.is_some_and(|c| c.due > 0)
}

/// The step clock alone, for headless apps that drive [`SimTimeMicros`] themselves.
/// [`SimControlPlugin`] orders it after the time advance. Add it through
/// [`ensure_sim_step_clock`] from any plugin whose systems step on it.
pub struct SimStepClockPlugin;

impl Plugin for SimStepClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimTimeMicros>()
            .init_resource::<SimStepClock>()
            .add_systems(
                Update,
                advance_sim_step_clock.in_set(SimControlSystemSet::AdvanceSimTick),
            );
    }
}

/// Add [`SimStepClockPlugin`] unless the app already has it (gameplay plugins call this, so the
/// clock exists in every app that runs fixed-step systems).
pub fn ensure_sim_step_clock(app: &mut App) {
    if !app.is_plugin_added::<SimStepClockPlugin>() {
        app.add_plugins(SimStepClockPlugin);
    }
}

/// Phase **E1** cadence identity for render/compute snapshots (`base_visual_dev01_plan_status` § `phase-e-cadence-scale`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
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
            .add_plugins(crate::systems::sim_frame_delta::SimFrameDeltaPlugin);
        ensure_sim_step_clock(app);
        app
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
                advance_sim_tick
                    .in_set(SimControlSystemSet::AdvanceSimTick)
                    .before(advance_sim_step_clock),
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

pub(crate) fn advance_sim_tick(
    time: Res<Time>,
    mut tick: ResMut<SimTick>,
    mut sim_time: ResMut<SimTimeMicros>,
    mut ctrl: ResMut<SimControlState>,
) {
    if !ctrl.should_tick() {
        return;
    }
    tick.0 = tick.0.wrapping_add(1);
    // An operator single-step while paused advances exactly one gameplay step.
    let delta_micros = if ctrl.paused {
        SIM_STEP_MICROS
    } else {
        (time.delta_secs() * ctrl.speed.max(0.0) * 1_000_000.0) as u64
    };
    sim_time.0 = sim_time.0.wrapping_add(delta_micros);
    if ctrl.steps_remaining > 0 {
        ctrl.steps_remaining -= 1;
    }
}
