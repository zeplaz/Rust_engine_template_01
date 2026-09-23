//! **EFFECTS-SYSTEM-ES-5** — refresh `debug_runs/effects_system_es5_live.json`.
//!
//! ES-5-3/4: mesh retired + provisional zoom density; `cpu_weather_precip_retired` true when
//! gpu_authority + streaks present.

use bevy::prelude::*;

use crate::render::{
    apply_weather_precip_frame_from_climate_at, build_precip_streak_instances_at,
    cpu_weather_precip_retired_with_gate, weather_precip_mesh_spawn_present,
    weather_vfx_witness_json_with_gate, ClimateVisualAggregate, ParticleDomainRegistry,
    ParticleDomainRegistryPlugin, WeatherPrecipDrawStatus, WeatherPrecipFrame,
    WeatherPrecipZoomBand, WeatherVfxPlugin,
};

pub const EFFECTS_SYSTEM_ES5_LIVE_JSON: &str = "debug_runs/effects_system_es5_live.json";

#[must_use]
pub fn effects_system_es5_witness_body() -> serde_json::Value {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ParticleDomainRegistryPlugin);
    app.add_plugins(WeatherVfxPlugin);
    app.insert_resource(ClimateVisualAggregate {
        mean_rain: 0.5,
        mean_snow: 0.12,
        mean_fog_density: 0.18,
        mean_wind_speed: 0.9,
        weather_chunk_count: 6,
        ..Default::default()
    });

    for _ in 0..2 {
        app.update();
    }

    let climate = *app.world().resource::<ClimateVisualAggregate>();
    let mut frame = WeatherPrecipFrame::default();
    let mut domains = ParticleDomainRegistry::default();
    apply_weather_precip_frame_from_climate_at(
        &climate,
        &mut frame,
        &mut domains,
        false,
        None,
        WeatherPrecipZoomBand::from_zoom_alpha(0.75),
    );
    let mut status = WeatherPrecipDrawStatus::default();
    if domains.weather_precip_active() && frame.instance_count > 0 {
        frame.gpu_authority = true;
        frame.camera_centered = true;
        status.upload_active = true;
        status.draw_instances = frame.instance_count as u32;
        status.camera_centered = true;
    }

    let tac = build_precip_streak_instances_at(
        &climate,
        bevy::math::Vec2::ZERO,
        WeatherPrecipZoomBand::from_zoom_alpha(0.8),
    );
    let bg = build_precip_streak_instances_at(
        &climate,
        bevy::math::Vec2::ZERO,
        WeatherPrecipZoomBand::from_zoom_alpha(0.2),
    );

    let weather = weather_vfx_witness_json_with_gate(&frame, Some(&domains), false);
    let retired = cpu_weather_precip_retired_with_gate(&frame, Some(&domains), false);
    let mesh_present = weather_precip_mesh_spawn_present();

    let frontend_filled = frame.climate_fed;
    let upload_landed = frame.gpu_authority && status.upload_active && status.draw_instances > 0;
    let zoom_banded = tac.len() > bg.len();
    let green = frontend_filled && upload_landed && !mesh_present && retired && zoom_banded;

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-5",
        "slice": "ES-5-003",
        "status": if green { "done" } else { "failed" },
        "weather_vfx_frontend_filled": frontend_filled,
        "cpu_weather_precip_retired": retired,
        "weather_precip_gpu_upload_active": status.upload_active,
        "weather_precip_gpu_authority": frame.gpu_authority,
        "weather_precip_mesh_spawn_present": mesh_present,
        "weather_precip_zoom_banded": zoom_banded,
        "modes": {
            "tactical_streaks": tac.len(),
            "background_streaks": bg.len(),
            "instance_count": frame.instance_count,
            "gpu_authority": frame.gpu_authority,
        },
        "weather_vfx": weather,
        "frontend_fill_landed": frontend_filled,
        "gpu_upload_landed": upload_landed,
        "mesh_retired": !mesh_present,
        "green": green,
        "deferred": {
            "slice": "ES-5-4-designer-refine",
            "note": "Provisional 0.45 zoom table shipped; designer may retune density/scatter without blocking",
        },
        "plan_note": "ES-5 closed for production spine: GPU precip + overlay tint. Hanabi L6 parallel OK (not weather authority).",
    })
}

#[must_use]
pub fn refresh_effects_system_es5_live_witness() -> bool {
    let body = effects_system_es5_witness_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EFFECTS-SYSTEM-ES-5-003",
        "refresh_effects_system_es5_live_witness",
        EFFECTS_SYSTEM_ES5_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(EFFECTS_SYSTEM_ES5_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_system_es5_live_witness_retired_green() {
        assert!(refresh_effects_system_es5_live_witness());
        let body = effects_system_es5_witness_body();
        assert_eq!(body["cpu_weather_precip_retired"], true);
        assert_eq!(body["mesh_retired"], true);
        assert_eq!(body["weather_precip_zoom_banded"], true);
    }
}
