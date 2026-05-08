//! Weather simulation (scaffold).
//!
//! Design: [`prompts/guides/weather_simulation_runbook_v1.md`](../../../prompts/guides/weather_simulation_runbook_v1.md).
//! Step pack: [`prompts/matrix/simulation_expansion/runbook/s2_steps_v1.md`](../../../prompts/matrix/simulation_expansion/runbook/s2_steps_v1.md).
//!
//! **Precipitation visuals** (screen-space overlay + mesh particles): [`WeatherVisualPlugin`](weather_visual::WeatherVisualPlugin), [`WeatherVisualSettings`](weather_visual::WeatherVisualSettings).

mod chunk_weather;
mod weather_visual;

pub use chunk_weather::{
    ChunkWeather, GlobalRenewableWeatherFactors, WeatherSimDiagnostics,
};
pub use weather_visual::{WeatherPrecipVisualSample, WeatherVisualSettings, WeatherVisualPlugin};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_weather::{
    spawn_chunk_weather_on_new_chunk, update_global_renewable_weather_factors,
    weather_chunk_tick,
};
/// Owns climate → regional → chunk weather systems (chunk layer + diagnostics).
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherSimDiagnostics>()
            .init_resource::<GlobalRenewableWeatherFactors>()
            .add_plugins(WeatherVisualPlugin)
            .add_systems(
                Update,
                (
                    spawn_chunk_weather_on_new_chunk.in_set(ChunkEnvironmentSet::Weather),
                    weather_chunk_tick.in_set(ChunkEnvironmentSet::Weather),
                    update_global_renewable_weather_factors.in_set(ChunkEnvironmentSet::Weather),
                )
                    .chain(),
            );
    }
}
