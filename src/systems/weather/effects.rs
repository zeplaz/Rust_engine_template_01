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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

}
