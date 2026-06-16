//! Weather program witness — `debug_runs/weather_sim_live.json` (PLAN-WEATHER-WITNESS-002 / WEATHER-WITNESS-001).

use serde_json::{json, Value};

use super::{ClimateState, RegionalWeatherField, WeatherEffectsSample, WeatherSimDiagnostics};
use super::effects::{gather_weather_witness_inputs, WeatherPrecipAuthority};

pub const WEATHER_SIM_LIVE_GATE: &str = "WEATHER-SIM-LIVE-001";
pub const WEATHER_SIM_LIVE_JSON: &str = "debug_runs/weather_sim_live.json";

/// Roll-up inputs for weather program closure (v1 schema).
#[derive(Clone, Debug, Default)]
pub struct WeatherWitnessInputs {
    pub climate_seed_present: bool,
    pub climate_state_wired: bool,
    pub regional_weather_wired: bool,
    pub chunk_weather_from_regional: bool,
    pub weather_effects_traction_stub: bool,
    pub weather_precip_gpu_authority: bool,
    pub renewables_from_clipmap: bool,
    pub renewable_factors_read: bool,
    pub construction_penalty_published: bool,
    pub regional_weather_sample: f32,
}

#[must_use]
pub fn weather_sim_live_green(inputs: &WeatherWitnessInputs) -> bool {
    inputs.climate_state_wired
        && inputs.regional_weather_wired
        && inputs.chunk_weather_from_regional
        && inputs.renewables_from_clipmap
}

#[must_use]
pub fn build_weather_sim_live_payload(
    diagnostics: &WeatherSimDiagnostics,
    inputs: &WeatherWitnessInputs,
) -> Value {
    let green = weather_sim_live_green(inputs);
    json!({
        "gate": WEATHER_SIM_LIVE_GATE,
        "green": green,
        "climate_seed_present": inputs.climate_seed_present,
        "climate_state_wired": inputs.climate_state_wired,
        "regional_weather_wired": inputs.regional_weather_wired,
        "chunk_weather_from_regional": inputs.chunk_weather_from_regional,
        "weather_effects_traction_stub": inputs.weather_effects_traction_stub,
        "weather_precip_gpu_authority": inputs.weather_precip_gpu_authority,
        "renewables_from_clipmap": inputs.renewables_from_clipmap,
        "weather_sim_ticks": diagnostics.ticks_advanced,
        "regional_weather_sample": inputs.regional_weather_sample,
        "cross_system_hooks": {
            "renewable_factors_read": inputs.renewable_factors_read,
            "visual_extract_only": true,
            "construction_penalty_published": inputs.construction_penalty_published,
            "tile_coupling_forbidden": true,
        },
    })
}

#[must_use]
pub fn weather_witness_001_schema_keys_present(body: &Value) -> bool {
    let required = [
        "gate",
        "green",
        "climate_seed_present",
        "climate_state_wired",
        "regional_weather_wired",
        "chunk_weather_from_regional",
        "weather_effects_traction_stub",
        "weather_precip_gpu_authority",
        "renewables_from_clipmap",
        "weather_sim_ticks",
        "regional_weather_sample",
    ];
    if !required.iter().all(|key| body.get(key).is_some()) {
        return false;
    }
    let hooks = match body.get("cross_system_hooks").and_then(Value::as_object) {
        Some(h) => h,
        None => return false,
    };
    [
        "renewable_factors_read",
        "visual_extract_only",
        "construction_penalty_published",
        "tile_coupling_forbidden",
    ]
    .iter()
    .all(|key| hooks.get(*key).is_some())
}

/// **WEATHER-WITNESS-001** — writer + schema present; program `green` stays false until W-SIM train lands.
#[must_use]
pub fn weather_witness_inputs_from_sim(
    diagnostics: &WeatherSimDiagnostics,
    climate: &ClimateState,
    regional: &RegionalWeatherField,
    effects: &WeatherEffectsSample,
    precip: &WeatherPrecipAuthority,
) -> WeatherWitnessInputs {
    gather_weather_witness_inputs(diagnostics, climate, regional, effects, precip)
}

#[must_use]
pub fn weather_witness_001_witness_green() -> bool {
    let diagnostics = WeatherSimDiagnostics::default();
    let body = build_weather_sim_live_payload(&diagnostics, &WeatherWitnessInputs::default());
    weather_witness_001_schema_keys_present(&body)
        && body["gate"].as_str() == Some(WEATHER_SIM_LIVE_GATE)
        && body["green"].as_bool() == Some(false)
        && body["cross_system_hooks"]["visual_extract_only"].as_bool() == Some(true)
        && body["cross_system_hooks"]["tile_coupling_forbidden"].as_bool() == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_witness_climate_inputs_from_sim() {
        let climate = ClimateState::default();
        let regional = RegionalWeatherField::default();
        let effects = WeatherEffectsSample::default();
        let mut diag = WeatherSimDiagnostics::default();
        let precip = WeatherPrecipAuthority::default();
        assert!(
            !weather_witness_inputs_from_sim(&diag, &climate, &regional, &effects, &precip)
                .climate_state_wired
        );
        diag.climate_slow_tick_runs = 1;
        let inputs = weather_witness_inputs_from_sim(&diag, &climate, &regional, &effects, &precip);
        assert!(inputs.climate_seed_present);
        assert!(inputs.climate_state_wired);
    }

    #[test]
    fn weather_witness_001_schema_and_rollup() {
        assert!(weather_witness_001_witness_green());
        let mut inputs = WeatherWitnessInputs::default();
        assert!(!weather_sim_live_green(&inputs));
        inputs.climate_state_wired = true;
        inputs.regional_weather_wired = true;
        inputs.chunk_weather_from_regional = true;
        inputs.renewables_from_clipmap = true;
        assert!(weather_sim_live_green(&inputs));
    }
}
