//! EFFECTS-SYSTEM ES-5 — weather precip GPU frontend + Core2d draw.
//!
//! **Authority:** `ClimateVisualAggregate` → zoom-banded streaks →
//! `WEATHER_PRECIP_INSTANCES_BUFFER` → [`draw`] vertex-pull raster.
//!
//! ES-5-001 fill · ES-5-002 upload/draw · ES-5-3 mesh retired · ES-5-4 provisional zoom.
//! Reject: Hanabi as world-scale rain (ES-7 event puffs remain parallel-OK).

pub mod draw;
pub mod extract;
pub mod frame;
pub mod witness;

pub use draw::WeatherPrecipDrawStatus;
pub use extract::{
    apply_weather_precip_frame_from_climate, apply_weather_precip_frame_from_climate_at,
    build_precip_streak_instances, build_precip_streak_instances_at, WeatherPrecipZoomBand,
    WEATHER_PRECIP_STREAK_CAP,
};
pub use frame::WeatherPrecipFrame;
pub use witness::{
    cpu_weather_mesh_precip_enabled, cpu_weather_precip_retired,
    cpu_weather_precip_retired_with_gate, weather_precip_mesh_spawn_present,
    weather_vfx_witness_json, weather_vfx_witness_json_with_gate,
    weather_vfx_witness_json_with_status,
};

use bevy::prelude::*;

use crate::render::particle_domains::ParticleDomainRegistry;
use crate::render::ClimateVisualAggregate;

/// Registers [`WeatherPrecipFrame`] + climate fill + GPU draw.
pub struct WeatherVfxPlugin;

impl Plugin for WeatherVfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherPrecipFrame>();
        app.init_resource::<ClimateVisualAggregate>();
        app.init_resource::<ParticleDomainRegistry>();
        extract::register_weather_precip_extract(app);
        draw::register_weather_precip_draw(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::particle_domains::ParticleDomainRegistry;

    #[test]
    fn default_frame_not_retired_until_authority() {
        let frame = WeatherPrecipFrame::default();
        let reg = ParticleDomainRegistry::default();
        assert!(!cpu_weather_precip_retired(&frame, Some(&reg)));
        assert!(!weather_precip_mesh_spawn_present());
        assert!(!cpu_weather_mesh_precip_enabled());
    }
}
