//! Weather simulation.
//!
//! Plan: [`src/dev/plan_weather_wind_v1.md`](../../dev/plan_weather_wind_v1.md).
//!
//! - [`WeatherField`] — the one authority: coarse 1 km grid, seeded climate + moving pressure
//!   systems, solved every `weather_step_secs` of game time; [`WeatherField::sample`] for consumers.
//! - [`ChunkWeather`] — per-chunk projection of the field for existing readers.
//! - **Precipitation visuals** (overlay tint; GPU streaks via `render::weather_vfx`): [`WeatherVisualPlugin`](weather_visual::WeatherVisualPlugin), [`WeatherVisualSettings`](weather_visual::WeatherVisualSettings).

mod camera_local;
mod chunk_weather;
mod field;
mod overlay_bake;
mod player_read_hud;
mod systems;
mod tuning;
mod weather_visual;

pub use camera_local::{
    publish_camera_local_weather_sample, rain_snow01_from_sample, CameraLocalWeatherSample,
};
pub use overlay_bake::{
    bake_weather_overlay_from_field, sync_weather_projection_node, WeatherOverlayBakeState,
};
pub use player_read_hud::{
    format_ops_power_derate_line, format_ops_wx_line, format_ops_wx_line_with_vis,
    format_tactical_wx_line, WeatherPlayerReadFrame,
};

pub use chunk_weather::{
    project_chunk_weather, ChunkWeather, CHUNK_RAIN_FULL_SCALE_MM_H, CHUNK_VISIBILITY_FULL_SCALE_M,
    CHUNK_WIND_FULL_SCALE_MPS,
};
pub use field::{
    PressureKind, PressureSystem, WeatherClimate, WeatherField, WeatherFieldSave, WeatherRng,
    WeatherSample, WeatherSnapshot, BASELINE_PRESSURE_HPA, WEATHER_FIELD_SAVE_SCHEMA,
};
pub use systems::{weather_field_step, PendingWeatherRestore, WeatherPresetRequest};
pub use tuning::{
    load_weather_tuning, parse_weather_tuning, WeatherPreset, WeatherPresetId, WeatherTuning,
    WEATHER_TUNING_RON,
};
pub use weather_visual::{
    weather_precip_show_background, weather_precip_show_tactical, weather_precip_tactical_band,
    WeatherPrecipVisualSample, WeatherVisualSettings, WeatherVisualPlugin, WeatherVfxCameraChild,
    WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA,
};


use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_weather::spawn_chunk_weather_on_new_chunk;

/// Weather field step + chunk projection (no mesh/UI). Pairs with [`WeatherVisualPlugin`] for precipitation VFX.
pub struct WeatherSimulationPlugin;

impl Plugin for WeatherSimulationPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.init_resource::<WeatherTuning>()
            .init_resource::<WeatherField>()
            .init_resource::<CameraLocalWeatherSample>()
            .init_resource::<WeatherOverlayBakeState>()
            .init_resource::<WeatherPresetRequest>()
            .init_resource::<PendingWeatherRestore>()
            .add_systems(Startup, systems::load_weather_tuning_on_startup)
            .add_systems(
                Update,
                (
                    spawn_chunk_weather_on_new_chunk,
                    weather_field_step,
                    project_chunk_weather,
                    bake_weather_overlay_from_field,
                )
                    .chain()
                    .in_set(ChunkEnvironmentSet::Weather),
            )
            .add_systems(
                Update,
                sync_weather_projection_node.after(bake_weather_overlay_from_field),
            );
    }
}

/// Owns the weather field, chunk projection and precipitation visuals.
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WeatherSimulationPlugin,
            WeatherVisualPlugin,
            crate::render::WeatherVfxPlugin,
        ));
        player_read_hud::register_weather_player_read(app);
    }
}
