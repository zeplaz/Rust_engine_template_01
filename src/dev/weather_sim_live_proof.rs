//! WEATHER-WITNESS-001 — `debug_runs/weather_sim_live.json` lib refresh.

use bevy::prelude::{App, IntoScheduleConfigs, MinimalPlugins, Update};
use serde_json::Value;

use crate::dev::debug_run_envelope;
use crate::substrate::atmosphere::AtmosphereClipmapStack;
use crate::systems::sim_control::SimControlState;
use crate::systems::sim_control::SimTick;
use crate::systems::weather::{
    build_weather_sim_live_payload, climate_slow_tick, gather_weather_witness_inputs,
    regional_weather_tick, sync_weather_precip_authority_system, weather_chunk_tick,
    weather_effects_tick, weather_sim_live_green, weather_witness_001_schema_keys_present,
    weather_witness_001_witness_green, ClimateState, RegionalWeatherField, WeatherEffectsSample,
    WeatherPrecipAuthority, WeatherSimDiagnostics, WeatherVisualSettings, WeatherWitnessInputs,
    WEATHER_SIM_LIVE_JSON,
};
use crate::terrain::generation::Chunk;

#[must_use]
pub fn build_weather_sim_live_proof_payload(
    diagnostics: &WeatherSimDiagnostics,
    inputs: &WeatherWitnessInputs,
) -> Value {
    build_weather_sim_live_payload(diagnostics, inputs)
}

/// Mini sim: clipmap → regional → chunk → effects (W-SIM train proof inputs).
#[must_use]
pub fn weather_sim_train_proof_state() -> (WeatherSimDiagnostics, WeatherWitnessInputs) {
    let mut app = App::new();
    let mut stack = AtmosphereClipmapStack::default();
    if let Some(l2) = stack.levels.get_mut(2) {
        for v in &mut l2.smoke_density {
            *v = 0.55;
        }
    }
    app.add_plugins(MinimalPlugins)
        .insert_resource(stack)
        .init_resource::<ClimateState>()
        .init_resource::<RegionalWeatherField>()
        .init_resource::<WeatherEffectsSample>()
        .init_resource::<WeatherPrecipAuthority>()
        .init_resource::<WeatherVisualSettings>()
        .init_resource::<WeatherSimDiagnostics>()
        .insert_resource(SimControlState::default())
        .init_resource::<SimTick>()
        .add_systems(
            Update,
            (
                climate_slow_tick,
                regional_weather_tick,
                weather_chunk_tick,
                weather_effects_tick,
                sync_weather_precip_authority_system,
            )
                .chain(),
        );
    app.world_mut().spawn((
        Chunk {
            coord: bevy::math::IVec2::ZERO,
        },
        crate::systems::weather::ChunkWeather {
            visibility_factor: 0.72,
            ..crate::systems::weather::ChunkWeather::default()
        },
    ));
    for _ in 0..4 {
        app.update();
    }
    let world = app.world();
    let diagnostics = world.resource::<WeatherSimDiagnostics>().clone();
    let inputs = gather_weather_witness_inputs(
        world.resource::<WeatherSimDiagnostics>(),
        world.resource::<ClimateState>(),
        world.resource::<RegionalWeatherField>(),
        world.resource::<WeatherEffectsSample>(),
        world.resource::<WeatherPrecipAuthority>(),
    );
    (diagnostics, inputs)
}

#[must_use]
pub fn refresh_weather_sim_live_witness() -> bool {
    if !weather_witness_001_witness_green() {
        return false;
    }
    let (diagnostics, inputs) = weather_sim_train_proof_state();
    if !weather_sim_live_green(&inputs)
        || !inputs.weather_effects_traction_stub
        || !inputs.weather_precip_gpu_authority
    {
        return false;
    }
    let body = build_weather_sim_live_proof_payload(&diagnostics, &inputs);
    if !weather_witness_001_schema_keys_present(&body) {
        return false;
    }
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "WEATHER_WITNESS_001",
        "refresh_weather_sim_live_witness",
        WEATHER_SIM_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(WEATHER_SIM_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_effects_001_proof_inputs_green() {
        let (_, inputs) = weather_sim_train_proof_state();
        assert!(inputs.climate_state_wired);
        assert!(inputs.regional_weather_wired);
        assert!(inputs.chunk_weather_from_regional);
        assert!(inputs.weather_effects_traction_stub);
        assert!(inputs.weather_precip_gpu_authority);
        assert!(inputs.renewables_from_clipmap);
        assert!(weather_sim_live_green(&inputs));
    }

    #[test]
    fn weather_sim_live_witness_refresh_green() {
        assert!(refresh_weather_sim_live_witness());
    }
}
