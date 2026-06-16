//! Gameplay effect stubs (WEATHER-EFFECTS-001) — traction + visibility samples.

use bevy::prelude::*;

use crate::substrate::post_spine::mean_slab_congestion;
use crate::substrate::WorldSubstrateRegistry;
use crate::systems::sim_control::SimControlState;

use super::{ChunkWeather, ClimateState, RegionalWeatherField, WeatherSimDiagnostics, WeatherWitnessInputs};

/// GPU precip authority rollup for witness (WEATHER-GPU-PRECIP-001).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WeatherPrecipAuthority {
    pub gpu_precip_authority: bool,
    pub mesh_precip_demoted: bool,
}

impl WeatherPrecipAuthority {
    #[inline]
    #[must_use]
    pub fn witness_green(&self) -> bool {
        self.gpu_precip_authority && self.mesh_precip_demoted
    }
}

/// Sync witness fields from [`super::weather_visual::WeatherVisualSettings`].
pub fn sync_weather_precip_authority_system(
    settings: Res<super::weather_visual::WeatherVisualSettings>,
    mut auth: ResMut<WeatherPrecipAuthority>,
) {
    auth.gpu_precip_authority = settings.gpu_precip_authority;
    auth.mesh_precip_demoted = settings.mesh_precip_demoted;
}

/// Derived gameplay samples for logistics / sensors (stub — no tile mutation).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WeatherEffectsSample {
    pub traction_stub_active: bool,
    pub traction_factor: f32,
    pub visibility_sample: f32,
}

/// Publish traction + visibility samples from chunk weather + slab congestion mirror.
pub fn weather_effects_tick(
    ctrl: Res<SimControlState>,
    chunks: Query<&ChunkWeather>,
    registry: Option<Res<WorldSubstrateRegistry>>,
    mut sample: ResMut<WeatherEffectsSample>,
    mut diag: ResMut<WeatherSimDiagnostics>,
) {
    if !ctrl.should_tick() {
        return;
    }
    diag.weather_effects_tick_runs = diag.weather_effects_tick_runs.wrapping_add(1);

    let mut vis_sum = 0.0_f32;
    let mut n = 0_u32;
    for w in &chunks {
        n += 1;
        vis_sum += w.visibility_factor;
    }
    sample.visibility_sample = if n > 0 {
        vis_sum / n as f32
    } else {
        1.0
    };

    let congestion = registry.as_deref().map(mean_slab_congestion).unwrap_or(0.0);
    sample.traction_factor = (1.0 - congestion * 0.35).clamp(0.5, 1.0);
    sample.traction_stub_active = n > 0 || congestion > 1e-6;
}

#[must_use]
pub fn gather_weather_witness_inputs(
    diagnostics: &WeatherSimDiagnostics,
    climate: &ClimateState,
    regional: &RegionalWeatherField,
    effects: &WeatherEffectsSample,
    precip: &WeatherPrecipAuthority,
) -> WeatherWitnessInputs {
    WeatherWitnessInputs {
        climate_seed_present: climate.seed_present(),
        climate_state_wired: diagnostics.climate_slow_tick_runs > 0,
        regional_weather_wired: regional.wired_from_clipmap && diagnostics.regional_weather_tick_runs > 0,
        chunk_weather_from_regional: diagnostics.chunk_weather_regional_lerp_runs > 0,
        weather_effects_traction_stub: diagnostics.weather_effects_tick_runs > 0
            && effects.traction_stub_active,
        weather_precip_gpu_authority: precip.witness_green(),
        renewables_from_clipmap: regional.wired_from_clipmap && regional.sample > 1e-5,
        regional_weather_sample: regional.sample,
        ..WeatherWitnessInputs::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn weather_effects_tick_publishes_traction_stub() {
        let mut app = App::new();
        app.init_resource::<WeatherEffectsSample>()
            .init_resource::<WeatherPrecipAuthority>()
            .init_resource::<WeatherSimDiagnostics>()
            .insert_resource(SimControlState::default())
            .add_systems(Update, weather_effects_tick);
        app.world_mut().spawn(ChunkWeather {
            visibility_factor: 0.6,
            ..ChunkWeather::default()
        });
        app.update();
        let sample = app.world().resource::<WeatherEffectsSample>();
        assert!(sample.traction_stub_active);
        assert!(sample.visibility_sample < 1.0);
        let inputs = gather_weather_witness_inputs(
            app.world().resource::<WeatherSimDiagnostics>(),
            &ClimateState::default(),
            &RegionalWeatherField::default(),
            sample,
            app.world().resource::<WeatherPrecipAuthority>(),
        );
        assert!(inputs.weather_effects_traction_stub);
    }
}
