//! Weather simulation.
//!
//! Plan: [`src/dev/plan_weather_wind_v1.md`](../../dev/plan_weather_wind_v1.md).
//!
//! - [`WeatherField`] — the one authority: coarse 1 km grid, seeded climate + moving pressure
//!   systems, solved every `weather_step_secs` of game time; [`WeatherField::sample`] for consumers.
//! - [`ChunkWeather`] — per-chunk projection of the field for existing readers.
//! - **Precipitation visuals** (overlay tint; GPU streaks via `render::weather_vfx`): [`WeatherVisualPlugin`](weather_visual::WeatherVisualPlugin), [`WeatherVisualSettings`](weather_visual::WeatherVisualSettings).

mod air_ops;
mod air_presentation_frame;
mod air_presentation_policy;
mod camera_local;
mod chunk_weather;
mod field;
mod overlay_bake;
mod player_read_hud;
mod polling_stations;
mod sense_propagation;
mod systems;
mod tuning;
mod weather_visual;

pub use air_ops::{
    air_fuel_mul, air_ground_speed_mul, airlift_hold, collect_chunk_smoke_density,
    crosswind_component, evaluate_grounding, evaluate_grounding_with, headwind_component,
    icing_risk, icing_risk_for_sample, link_quality, liquid_water_proxy01, load_air_ops_tuning,
    missile_weather_drift, parse_air_ops_tuning, sensor_range_mul, smoke01_at_chunk_from_frame,
    smoke01_at_tile, smoke01_at_tile_from_frame, smoke01_resolve, sortie_envelopes_ok,
    sync_air_ops_weather_status, temp_at_altitude_c, tile_to_smoke_chunk, AirOpsProfile,
    AirOpsProfileId, AirOpsTuning, AirOpsWeatherStatus, GroundingReason, GroundingVerdict,
    LinkModality, SensorKind, AIR_OPS_TUNING_RON, SMOKE_SAMPLE_CELLS_PER_CHUNK,
};
pub use air_presentation_policy::{
    AirGeometryTier, AirPresentationBand, AirPresentationPolicy, AirPresentationSortieHint,
    FlightPathFidelity,
};
pub use air_presentation_frame::{
    sync_air_presentation_frame, AirPresentationFrame, AirPresentationMark, AirPresentationZoom,
};

pub use camera_local::{
    publish_camera_local_weather_sample, rain_snow01_from_sample, CameraLocalWeatherSample,
};
pub use overlay_bake::{
    bake_weather_overlay_from_field, sample_overlay_bake_probe_at_pos_m,
    sample_overlay_bake_probe_at_uv, sync_weather_projection_node, OverlayBakeProbeSample,
    WeatherOverlayBakeState,
};
pub use player_read_hud::{
    format_ops_power_derate_line, format_ops_wx_line, format_ops_wx_line_with_vis,
    format_tactical_wx_line, WeatherPlayerReadFrame,
};

pub use chunk_weather::{
    project_chunk_weather, ChunkWeather, CHUNK_RAIN_FULL_SCALE_MM_H, CHUNK_VISIBILITY_FULL_SCALE_M,
    CHUNK_WIND_FULL_SCALE_MPS,
};
pub use polling_stations::{
    sync_weather_polling_stations, tile_to_weather_chunk, WeatherPollSiteBook,
    WeatherPollingStations,
};
pub use sense_propagation::{
    build_weather_sense_cell_field, relax_sense_degradation, relax_sense_from_samples,
    sense_seed_mul, WeatherSenseCellField, SENSE_MUL_FLOOR,
};
pub use field::{
    migrate_weather_field_save, PressureKind, PressureSystem, WeatherClimate, WeatherField,
    WeatherFieldSave, WeatherRng, WeatherSample, WeatherSnapshot, BASELINE_PRESSURE_HPA,
    WEATHER_FIELD_SAVE_SCHEMA, WEATHER_FIELD_SAVE_SCHEMA_V1,
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

use crate::systems::atmosphere::{AtmosphereFrame, AtmospherePipelineSet};
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_weather::spawn_chunk_weather_on_new_chunk;

/// Weather field step + chunk projection (no mesh/UI). Pairs with [`WeatherVisualPlugin`] for precipitation VFX.
pub struct WeatherSimulationPlugin;

impl Plugin for WeatherSimulationPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.init_resource::<WeatherTuning>()
            .init_resource::<AirOpsTuning>()
            .init_resource::<AirOpsWeatherStatus>()
            .init_resource::<AirPresentationPolicy>()
            .init_resource::<AirPresentationFrame>()
            .init_resource::<AirPresentationZoom>()
            .init_resource::<WeatherField>()
            // W4: fire→cloud terms (AtmospherePlugin also inits; idempotent).
            .init_resource::<crate::systems::atmosphere::WeatherSourceTerms>()
            // Read path for air-ops / camera-local; built by `AtmospherePlugin` (empty ⇒ not ready).
            .init_resource::<AtmosphereFrame>()
            .init_resource::<WeatherPollSiteBook>()
            .init_resource::<WeatherPollingStations>()
            .init_resource::<WeatherSenseCellField>()
            .init_resource::<CameraLocalWeatherSample>()
            .init_resource::<WeatherOverlayBakeState>()
            .init_resource::<WeatherPresetRequest>()
            .init_resource::<PendingWeatherRestore>()
            .add_systems(
                Startup,
                (
                    systems::load_weather_tuning_on_startup,
                    air_ops::load_air_ops_tuning_on_startup,
                ),
            )
            .add_systems(
                Update,
                (
                    spawn_chunk_weather_on_new_chunk,
                    weather_field_step,
                    project_chunk_weather,
                    sync_weather_polling_stations,
                    build_weather_sense_cell_field,
                    bake_weather_overlay_from_field,
                )
                    .chain()
                    .in_set(ChunkEnvironmentSet::Weather),
            )
            // W2: air-ops reads the derived `AtmosphereFrame`, so it runs after `Frame` (post-fire)
            // instead of inside the Weather chain; stations + sense field are still fresh this tick.
            .add_systems(
                Update,
                sync_air_ops_weather_status
                    .after(AtmospherePipelineSet::Frame)
                    .after(sync_weather_polling_stations)
                    .after(build_weather_sense_cell_field),
            )
            .add_systems(
                Update,
                sync_weather_projection_node.after(bake_weather_overlay_from_field),
            )
            .add_systems(
                Update,
                sync_air_presentation_frame.after(sync_air_ops_weather_status),
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
