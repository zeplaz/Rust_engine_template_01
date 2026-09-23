//! ES-5 weather precip witness keys.

use super::draw::WeatherPrecipDrawStatus;
use super::frame::WeatherPrecipFrame;
use crate::render::particle_domains::{ParticleDomainId, ParticleDomainRegistry};

/// Mesh precip retired (ES-5-3). Env `RUST_ENGINE_CPU_WEATHER_PRECIP=1` is ignored — no mesh path.
#[inline]
#[must_use]
pub fn cpu_weather_mesh_precip_enabled() -> bool {
    false
}

/// Mesh precip spawn path deleted in ES-5-3 (`weather_visual` overlay-only).
#[inline]
#[must_use]
pub fn weather_precip_mesh_spawn_present() -> bool {
    false
}

#[inline]
#[must_use]
pub fn cpu_weather_precip_retired(
    frame: &WeatherPrecipFrame,
    domains: Option<&ParticleDomainRegistry>,
) -> bool {
    cpu_weather_precip_retired_with_gate(frame, domains, cpu_weather_mesh_precip_enabled())
}

#[inline]
#[must_use]
pub fn cpu_weather_precip_retired_with_gate(
    frame: &WeatherPrecipFrame,
    domains: Option<&ParticleDomainRegistry>,
    cpu_mesh_enabled: bool,
) -> bool {
    let domain_active = domains
        .map(|r| {
            r.entries
                .iter()
                .any(|e| e.id == ParticleDomainId::WeatherPrecip && e.active)
        })
        .unwrap_or(false);
    frame.gpu_authority
        && domain_active
        && frame.instance_count > 0
        && !cpu_mesh_enabled
        && !weather_precip_mesh_spawn_present()
}

#[must_use]
pub fn weather_vfx_witness_json(
    frame: &WeatherPrecipFrame,
    domains: Option<&ParticleDomainRegistry>,
) -> serde_json::Value {
    weather_vfx_witness_json_with_status(frame, domains, None, cpu_weather_mesh_precip_enabled())
}

#[must_use]
pub fn weather_vfx_witness_json_with_gate(
    frame: &WeatherPrecipFrame,
    domains: Option<&ParticleDomainRegistry>,
    cpu_mesh_enabled: bool,
) -> serde_json::Value {
    weather_vfx_witness_json_with_status(frame, domains, None, cpu_mesh_enabled)
}

#[must_use]
pub fn weather_vfx_witness_json_with_status(
    frame: &WeatherPrecipFrame,
    domains: Option<&ParticleDomainRegistry>,
    status: Option<&WeatherPrecipDrawStatus>,
    cpu_mesh_enabled: bool,
) -> serde_json::Value {
    let domain_active = domains
        .map(|r| {
            r.entries
                .iter()
                .any(|e| e.id == ParticleDomainId::WeatherPrecip && e.active)
        })
        .unwrap_or(false);
    let upload_active = status.map(|s| s.upload_active).unwrap_or(frame.gpu_authority);
    let draw_instances = status
        .map(|s| s.draw_instances)
        .unwrap_or(frame.instance_count as u32);
    serde_json::json!({
        "weather_vfx_frontend_stub": false,
        "weather_vfx_frontend_filled": frame.climate_fed,
        "cpu_weather_precip_retired": cpu_weather_precip_retired_with_gate(frame, domains, cpu_mesh_enabled),
        "cpu_weather_mesh_precip_enabled": cpu_mesh_enabled,
        "weather_precip_domain_active": domain_active,
        "weather_precip_gpu_authority": frame.gpu_authority,
        "weather_precip_gpu_upload_active": upload_active,
        "weather_precip_draw_instances": draw_instances,
        "weather_precip_camera_centered": frame.camera_centered,
        "weather_precip_mesh_spawn_present": weather_precip_mesh_spawn_present(),
        "weather_precip_zoom_alpha": frame.zoom_alpha,
        "weather_precip_tactical_band": frame.tactical_band,
        "weather_precip_instance_count": frame.instance_count,
        "weather_precip_mean_rain": frame.mean_rain,
        "weather_precip_mean_snow": frame.mean_snow,
        "weather_precip_chunk_count": frame.weather_chunk_count,
        "plan_note": "ES-5-3 mesh retired + ES-5-4 provisional zoom density (0.45 tactical threshold). Designer may refine. Reject Hanabi world rain. Hanabi L6 embellishment parallel OK.",
    })
}
