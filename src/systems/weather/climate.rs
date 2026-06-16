//! L3 slow climate resource (WEATHER-CLIMATE-001).
//!
//! Design: [`plan_weather_parallel_lane_v1.md`](../../../dev/plan_weather_parallel_lane_v1.md) · runbook L3 tier.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::systems::sim_control::{SimControlState, SimTick};

use super::WeatherSimDiagnostics;

/// Sim ticks between L3 climate mutations (slow tier vs per-tick chunk weather).
pub const CLIMATE_SLOW_INTERVAL: u64 = 1024;

/// World-scale climate envelope — seed-driven season phase and base temperature.
#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClimateState {
    pub seed: u64,
    /// 0..1 annual phase (deterministic from seed + slow ticks).
    pub season_phase: f32,
    /// Baseline °C bias for regional sampling (future L2).
    pub base_temperature_c: f32,
    /// Count of slow-tier mutations applied.
    pub slow_ticks: u64,
}

impl Default for ClimateState {
    fn default() -> Self {
        Self {
            seed: 0x5eed_042_u64,
            season_phase: 0.25,
            base_temperature_c: 12.0,
            slow_ticks: 0,
        }
    }
}

impl ClimateState {
    #[inline]
    #[must_use]
    pub fn seed_present(&self) -> bool {
        self.seed != 0
    }
}

/// Runs every sim frame under [`SimControlState::should_tick`]; mutates on [`CLIMATE_SLOW_INTERVAL`].
pub fn climate_slow_tick(
    ctrl: Res<SimControlState>,
    tick: Res<SimTick>,
    mut climate: ResMut<ClimateState>,
    mut diag: ResMut<WeatherSimDiagnostics>,
) {
    if !ctrl.should_tick() {
        return;
    }
    diag.climate_slow_tick_runs = diag.climate_slow_tick_runs.wrapping_add(1);

    if tick.0 == 0 || tick.0 % CLIMATE_SLOW_INTERVAL != 0 {
        return;
    }

    climate.slow_ticks = climate.slow_ticks.wrapping_add(1);
    let mix = tick
        .0
        .wrapping_mul(climate.seed)
        .wrapping_add(climate.slow_ticks)
        .wrapping_mul(0x9E37_79B9);
    let delta = ((mix % 10_000) as f32) / 10_000_000.0;
    climate.season_phase = (climate.season_phase + 0.002 + delta).fract();
    climate.base_temperature_c = 8.0
        + 14.0 * (climate.season_phase * std::f32::consts::TAU).sin().mul_add(0.5, 0.5);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn climate_slow_tick_runs_under_should_tick() {
        let mut app = App::new();
        app.init_resource::<ClimateState>()
            .init_resource::<WeatherSimDiagnostics>()
            .insert_resource(SimControlState::default())
            .insert_resource(SimTick(0))
            .add_systems(Update, climate_slow_tick);

        app.update();
        let diag = app.world().resource::<WeatherSimDiagnostics>();
        assert!(diag.climate_slow_tick_runs >= 1);
        let climate = app.world().resource::<ClimateState>();
        assert!(climate.seed_present());
    }

    #[test]
    fn climate_slow_interval_mutates_season() {
        let mut app = App::new();
        app.init_resource::<ClimateState>()
            .init_resource::<WeatherSimDiagnostics>()
            .insert_resource(SimControlState::default())
            .insert_resource(SimTick(CLIMATE_SLOW_INTERVAL))
            .add_systems(Update, climate_slow_tick);

        let before = app.world().resource::<ClimateState>().season_phase;
        app.update();
        let after = app.world().resource::<ClimateState>();
        assert_eq!(after.slow_ticks, 1);
        assert_ne!(after.season_phase, before);
    }
}
