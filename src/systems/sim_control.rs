//! Simulation control loop: pause, single-step, speed multiplier, monotonic tick.
//!
//! Ref-books **DBG-06** (gea4v1 10.5) · LEGACY keyword `debug_cameras_pause` (gea_v1_e4 p582) —
//! **V4-DBG06** formal ADOPT of D2 pause/step: freeze the **gameplay timeline** while the
//! **render and UI loop stay live** (Bevy `Time` / input / extract). Gameplay systems gate on
//! [`SimControlState::should_tick`] and scale sim dt with [`SimControlState::dt_scale`]; they
//! must **not** gate presentation or HUD on those flags.
//!
//! Ref-books **LOOP-03** · LEGACY `game_loop` / `time_measurement` (GEA 4e I §8.5.5, gea_v1_e4
//! p522 / p514): [`advance_sim_tick`] clamps wall-clock dt via
//! [`crate::systems::sim_frame_delta::clamp_sim_frame_delta_secs`] so a hitch/load never leaps
//! sim time past the catch-up budget.
//!
//! Ref-books **LOOP-04** · LEGACY `time_measurement` (GEA 4e I §8.5.4.4 / §3.3.1.4, p522 / p138):
//! [`SimTick`] and [`SimTimeMicros`] (**u64**) are sim-time truth — never cast to f32 for gameplay.
//!
//! **Studio / playcheck stills:** set `RUST_ENGINE_SIM_PAUSE=1` before launch so the sim starts
//! paused with gameplay dt frozen; the window still paints frames for capture (`tools/sim_review/`).
//!
//! Designer doc: `prompts/designer_questions/tools_ui/spec/01_plugin_schedule_patterns.md`.
//! UI driver:   `crate::gui::diagnostics_ui::DiagnosticsUiPlugin` (diagnostics panel).
//!
//! Pause hotkey: [`crate::gui::InputBindings::toggle_simulation_pause`] (Options → key bindings).
//!
//! **Schedule:** [`SimControlSystemSet`] — transport and other gameplay should run **after**
//! [`SimControlSystemSet::AdvanceSimTick`] (see `TransportSimulationPlugin`).

/// Latched once at startup: start with [`SimControlState::paused`] so playcheck / Studio can grab
/// stills before the timeline advances.
pub const SIM_PAUSE_ENV: &str = "RUST_ENGINE_SIM_PAUSE";

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gui::InputBindings;
use crate::systems::sim_frame_delta::clamp_sim_frame_delta_secs;

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
    /// Whether the gameplay timeline should advance this frame.
    ///
    /// When `false`, [`Self::dt_scale`] is 0 — sim systems skip work, but the render/UI frame loop
    /// continues (DBG-06).
    #[inline]
    pub fn should_tick(&self) -> bool {
        !self.paused || self.steps_remaining > 0
    }

    /// Effective dt scaler for **gameplay** systems only. Returns 0.0 when fully paused.
    #[inline]
    pub fn dt_scale(&self) -> f32 {
        if self.should_tick() { self.speed.max(0.0) } else { 0.0 }
    }

    /// Queue single-step ticks consumed while paused (scenario `SimAdvance`, diagnostics step).
    #[inline]
    pub fn queue_steps(&mut self, ticks: u32) {
        self.steps_remaining = self.steps_remaining.saturating_add(ticks);
    }
}

/// Monotonic simulation tick counter; incremented when `SimControlState::should_tick()`.
///
/// **LOOP-04:** this `u64` (with [`SimTimeMicros`]) is sim-time truth — not f32 seconds.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimTick(pub u64);

/// Authoritative sim time in microseconds (monotonic while the sim advances).
///
/// **LOOP-04:** integer micros stay exact across multi-hour sessions; never cast to f32 for gameplay paths.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimTimeMicros(pub u64);

/// Wall-clock frame → sim micros for one Update, after LOOP-03 hitch clamp.
#[inline]
#[must_use]
pub fn sim_delta_micros_from_frame(delta_secs: f32, speed: f32) -> u64 {
    let clamped = clamp_sim_frame_delta_secs(delta_secs);
    let scaled = clamped * speed.max(0.0);
    if !scaled.is_finite() || scaled <= 0.0 {
        return 0;
    }
    (scaled * 1_000_000.0) as u64
}

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
            .add_systems(PreStartup, apply_env_sim_pause_latch)
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

/// CQ-22: pure parse for pause-env truthy values (tests must not `set_var`).
#[must_use]
pub fn sim_pause_env_flag(raw: Option<&str>) -> bool {
    raw.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn sim_pause_env_on() -> bool {
    sim_pause_env_flag(std::env::var(SIM_PAUSE_ENV).ok().as_deref())
}

fn apply_env_sim_pause_latch(mut ctrl: ResMut<SimControlState>) {
    if sim_pause_env_on() {
        ctrl.paused = true;
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
    // Running path clamps wall-clock dt (LOOP-03) so stalls/loads cannot leap sim time.
    let delta_micros = if ctrl.paused {
        SIM_STEP_MICROS
    } else {
        sim_delta_micros_from_frame(time.delta_secs(), ctrl.speed)
    };
    sim_time.0 = sim_time.0.wrapping_add(delta_micros);
    if ctrl.steps_remaining > 0 {
        ctrl.steps_remaining -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, MinimalPlugins};

    #[test]
    fn paused_freezes_gameplay_dt_without_steps() {
        let ctrl = SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 2.0,
        };
        assert!(!ctrl.should_tick());
        assert_eq!(ctrl.dt_scale(), 0.0);
    }

    #[test]
    fn single_step_while_paused_advances_once() {
        let mut ctrl = SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        };
        ctrl.queue_steps(1);
        assert!(ctrl.should_tick());
        assert_eq!(ctrl.dt_scale(), 1.0);
    }

    #[test]
    fn sim_pause_env_recognizes_truthy_values() {
        assert!(sim_pause_env_flag(Some("1")));
        assert!(sim_pause_env_flag(Some("true")));
        assert!(sim_pause_env_flag(Some("TRUE")));
        assert!(!sim_pause_env_flag(Some("0")));
        assert!(!sim_pause_env_flag(Some("false")));
        assert!(!sim_pause_env_flag(None));
    }

    #[test]
    fn advance_sim_tick_skips_while_paused() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.insert_resource(SimTick::default());
        app.insert_resource(SimTimeMicros::default());
        app.add_systems(Update, advance_sim_tick);
        app.update();
        assert_eq!(app.world().resource::<SimTick>().0, 0);
    }

    #[test]
    fn hitch_frame_delta_clamps_before_sim_micros() {
        let max_micros = u64::from(SIM_STEP_MAX_PER_FRAME) * 1_000_000;
        assert_eq!(sim_delta_micros_from_frame(0.016, 1.0), 16_000);
        assert_eq!(sim_delta_micros_from_frame(120.0, 1.0), max_micros);
        assert_eq!(sim_delta_micros_from_frame(120.0, 2.0), max_micros * 2);
        assert_eq!(sim_delta_micros_from_frame(-1.0, 1.0), 0);
    }

    #[test]
    fn u64_tick_and_micros_remain_exact_when_f32_display_collapses() {
        // ~1 hour of sim: consecutive µs are distinct as u64, but identical as f32 seconds
        // (LOOP-04 / GEA 8.5.4.4 floating-point clock limits).
        let hour_micros = 3_600_000_000u64;
        assert_ne!(hour_micros, hour_micros + 1);
        let as_f32 = |micros: u64| (micros as f64 / 1_000_000.0) as f32;
        assert_eq!(as_f32(hour_micros), as_f32(hour_micros + 1));
        let tick = SimTick(1_000_000);
        assert_eq!(tick.0.wrapping_add(1), 1_000_001);
        assert_eq!(SimTimeMicros(hour_micros).0 + 1, hour_micros + 1);
    }

    /// DBG-06 / V4-DBG06 tripwire: paused gameplay clock stays frozen while Bevy `Time`
    /// (render/UI frame loop) keeps advancing.
    #[test]
    fn dbg06_pause_freezes_sim_tick_while_bevy_time_advances() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.insert_resource(SimTick::default());
        app.insert_resource(SimTimeMicros::default());
        app.add_systems(Update, advance_sim_tick);

        app.update();
        let elapsed_a = app.world().resource::<Time>().elapsed_secs_f64();
        assert_eq!(app.world().resource::<SimTick>().0, 0);
        assert_eq!(app.world().resource::<SimTimeMicros>().0, 0);

        std::thread::sleep(std::time::Duration::from_millis(25));
        app.update();
        let elapsed_b = app.world().resource::<Time>().elapsed_secs_f64();
        assert!(
            elapsed_b > elapsed_a,
            "Bevy Time must keep advancing while paused (DBG-06 render/UI live)"
        );
        assert_eq!(app.world().resource::<SimTick>().0, 0);
        assert_eq!(app.world().resource::<SimTimeMicros>().0, 0);
        assert!(!app.world().resource::<SimControlState>().should_tick());
        assert_eq!(app.world().resource::<SimControlState>().dt_scale(), 0.0);
    }

    /// DBG-06 / V4-DBG06 tripwire: single-step while paused advances exactly one sim second,
    /// then refreezes (no further ticks without another queued step).
    #[test]
    fn dbg06_single_step_advances_one_sim_second_then_refreezes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        });
        app.insert_resource(SimTick::default());
        app.insert_resource(SimTimeMicros::default());
        app.add_systems(Update, advance_sim_tick);

        app.world_mut().resource_mut::<SimControlState>().queue_steps(1);
        app.update();
        assert_eq!(app.world().resource::<SimTick>().0, 1);
        assert_eq!(app.world().resource::<SimTimeMicros>().0, SIM_STEP_MICROS);
        assert_eq!(app.world().resource::<SimControlState>().steps_remaining, 0);
        assert!(!app.world().resource::<SimControlState>().should_tick());

        app.update();
        assert_eq!(app.world().resource::<SimTick>().0, 1);
        assert_eq!(app.world().resource::<SimTimeMicros>().0, SIM_STEP_MICROS);
    }

}
