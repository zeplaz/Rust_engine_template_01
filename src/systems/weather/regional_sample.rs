//! L2 regional weather sample (WEATHER-REGIONAL-001) — clipmap → chunk targets.

use bevy::prelude::*;

use crate::substrate::atmosphere::{clipmap_l2_mean_scalar, AtmosphereClipmapStack};
use crate::systems::sim_control::SimControlState;

use super::{ClimateState, WeatherSimDiagnostics};

/// Regional L2 envelope sampled from [`AtmosphereClipmapStack`] when present.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct RegionalWeatherField {
    pub sample: f32,
    pub rain_target: f32,
    pub fog_target: f32,
    pub wind_target: f32,
    pub wired_from_clipmap: bool,
    pub tick_runs: u64,
}

impl RegionalWeatherField {
    #[must_use]
    pub fn active(&self) -> bool {
        self.wired_from_clipmap && self.sample > 1e-5
    }
}

/// Sample L2 clipmap (+ climate bias) under sim tick — not witness-only.
pub fn regional_weather_tick(
    ctrl: Res<SimControlState>,
    climate: Res<ClimateState>,
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    mut regional: ResMut<RegionalWeatherField>,
    mut diag: ResMut<WeatherSimDiagnostics>,
) {
    if !ctrl.should_tick() {
        return;
    }
    diag.regional_weather_tick_runs = diag.regional_weather_tick_runs.wrapping_add(1);
    regional.tick_runs = regional.tick_runs.wrapping_add(1);

    let Some(clipmap) = clipmap else {
        regional.wired_from_clipmap = false;
        return;
    };

    let raw = clipmap_l2_mean_scalar(&clipmap);
    let season = climate.season_phase.clamp(0.0, 1.0);
    let sample = (raw * 0.85 + season * 0.12).clamp(0.0, 1.0);
    regional.sample = sample;
    regional.wired_from_clipmap = true;
    regional.rain_target = (sample * 1.12 + season * 0.08).clamp(0.0, 1.0);
    regional.fog_target = (sample * 0.78).clamp(0.0, 1.0);
    regional.wind_target = (sample * 0.35 + season * 0.15).clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::sim_control::SimControlState;
    use bevy::app::App;

    #[test]
    fn regional_weather_tick_samples_clipmap_l2() {
        let mut app = App::new();
        let mut stack = AtmosphereClipmapStack::default();
        if let Some(l2) = stack.levels.get_mut(2) {
            for v in &mut l2.smoke_density {
                *v = 0.42;
            }
        }
        app.insert_resource(stack)
            .init_resource::<ClimateState>()
            .init_resource::<RegionalWeatherField>()
            .init_resource::<WeatherSimDiagnostics>()
            .insert_resource(SimControlState::default())
            .add_systems(Update, regional_weather_tick);

        app.update();
        let regional = app.world().resource::<RegionalWeatherField>();
        assert!(regional.wired_from_clipmap);
        assert!(regional.sample > 0.1);
        use crate::systems::weather::{
            gather_weather_witness_inputs, WeatherEffectsSample, WeatherPrecipAuthority,
        };
        let inputs = gather_weather_witness_inputs(
            app.world().resource::<WeatherSimDiagnostics>(),
            app.world().resource::<ClimateState>(),
            regional,
            &WeatherEffectsSample::default(),
            &WeatherPrecipAuthority::default(),
        );
        assert!(inputs.regional_weather_wired);
    }
}
