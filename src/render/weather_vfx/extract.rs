//! Climate / camera-local → [`WeatherPrecipFrame`] frontend extract (EFFECTS-SYSTEM ES-5).
//!
//! **W3:** density and per-streak rain/snow from [`WeatherField::sample`] at camera / streak XY
//! when the field is ready; [`ClimateVisualAggregate`] remains the fallback for tests / dry boot.

use bevy::math::Vec4;
use bevy::prelude::*;

use crate::gui::ZoomFrame;
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;
use crate::render::particle_domains::ParticleDomainRegistry;
use crate::render::ClimateVisualAggregate;
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::systems::sim_control::SimTimeMicros;
use crate::systems::weather::{
    rain_snow01_from_sample, CameraLocalWeatherSample, WeatherField,
    WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA,
};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::frame::WeatherPrecipFrame;

/// Cap for prepared streak rows (tactical band) — fill the view, not a tiny patch.
pub const WEATHER_PRECIP_STREAK_CAP: usize = 220;
/// Background (zoomed-out) density fraction of tactical cap.
pub const WEATHER_PRECIP_BACKGROUND_DENSITY: f32 = 0.85;
const PRECIP_STREAK_THRESHOLD: f32 = 0.04;
const PRECIP_CLASS_ID: f32 = 1.0;
const PRECIP_Z: f32 = 360.0;

/// Zoom-band parameters for GPU streak layout (provisional ES-5-4 — designer may refine).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherPrecipZoomBand {
    pub zoom_alpha: f32,
    pub tactical: bool,
    /// Multiplier on instance count (1.0 tactical, ~0.35 background).
    pub density_scale: f32,
    /// Local scatter radius in world units.
    pub scatter_radius: f32,
    /// Billboard half-edge scale.
    pub half_scale: f32,
}

impl WeatherPrecipZoomBand {
    #[must_use]
    pub fn from_zoom_alpha(zoom_alpha: f32) -> Self {
        Self::from_zoom_and_view(zoom_alpha, 96.0)
    }

    /// Prefer this when camera metrics are available — scatter covers most of the view.
    #[must_use]
    pub fn from_zoom_and_view(zoom_alpha: f32, view_half_diag: f32) -> Self {
        let tactical = zoom_alpha > WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;
        let radius = (view_half_diag * if tactical { 0.72 } else { 0.95 }).max(64.0);
        // Base half — draw shader scales by 4/camera_zoom for screen-stable size.
        let half_scale = 1.15;
        if tactical {
            Self {
                zoom_alpha,
                tactical: true,
                density_scale: 1.0,
                scatter_radius: radius,
                half_scale,
            }
        } else {
            Self {
                zoom_alpha,
                tactical: false,
                density_scale: WEATHER_PRECIP_BACKGROUND_DENSITY,
                scatter_radius: radius,
                half_scale: half_scale * 1.25,
            }
        }
    }
}

#[inline]
#[allow(dead_code)] // kept for designer half_scale curves
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Pure apply — used by scheduled system and unit/live proofs (climate means fallback).
pub fn apply_weather_precip_frame_from_climate(
    climate: &ClimateVisualAggregate,
    frame: &mut WeatherPrecipFrame,
    domains: &mut ParticleDomainRegistry,
    _cpu_mesh_enabled: bool,
) {
    apply_weather_precip_frame_from_climate_at(
        climate,
        frame,
        domains,
        false,
        None,
        WeatherPrecipZoomBand::from_zoom_alpha(0.5),
        None,
        None,
        SimTimeMicros(0),
    );
}

/// Camera-centered apply with explicit zoom band.
pub fn apply_weather_precip_frame_from_climate_at(
    climate: &ClimateVisualAggregate,
    frame: &mut WeatherPrecipFrame,
    domains: &mut ParticleDomainRegistry,
    _cpu_mesh_enabled: bool,
    camera: Option<&ExtractedCameraMetrics>,
    zoom: WeatherPrecipZoomBand,
    local: Option<&CameraLocalWeatherSample>,
    field: Option<&WeatherField>,
    sim_time: SimTimeMicros,
) {
    let (rain, snow, fog, wind_speed, wind_mps, local_ok) = if let Some(loc) = local.filter(|l| l.ready)
    {
        (
            loc.rain01,
            loc.snow01,
            loc.fog,
            loc.sample.wind_mps.length() / 20.0,
            loc.sample.wind_mps,
            true,
        )
    } else {
        (
            climate.mean_rain,
            climate.mean_snow,
            climate.mean_fog_density,
            climate.mean_wind_speed,
            climate.mean_wind_mps,
            false,
        )
    };

    frame.mean_rain = rain;
    frame.mean_snow = snow;
    frame.mean_fog = fog;
    frame.mean_wind = wind_speed;
    frame.wind_mps = wind_mps;
    frame.weather_chunk_count = climate.weather_chunk_count.max(if local_ok { 1 } else { 0 });
    frame.climate_fed = true;
    frame.local_sample = local_ok;
    frame.gpu_authority = false;
    frame.zoom_alpha = zoom.zoom_alpha;
    frame.tactical_band = zoom.tactical;

    domains.set_weather_precip_active(true);
    let origin = camera.map(|c| c.translation).unwrap_or(bevy::math::Vec2::ZERO);
    frame.streaks = build_precip_streak_instances_at(
        rain,
        snow,
        fog,
        wind_speed,
        wind_mps,
        climate.weather_chunk_count.max(if local_ok { 1 } else { 0 }),
        origin,
        zoom,
        field.filter(|f| f.is_ready()),
        sim_time,
    );
    frame.instance_count = frame.streaks.len();
    frame.camera_centered = camera.is_some();
}

#[must_use]
pub fn build_precip_streak_instances(climate: &ClimateVisualAggregate) -> Vec<GpuInstancedQuadInstance> {
    build_precip_streak_instances_at(
        climate.mean_rain,
        climate.mean_snow,
        climate.mean_fog_density,
        climate.mean_wind_speed,
        climate.mean_wind_mps,
        climate.weather_chunk_count,
        bevy::math::Vec2::ZERO,
        WeatherPrecipZoomBand::from_zoom_alpha(0.5),
        None,
        SimTimeMicros(0),
    )
}

#[must_use]
pub fn build_precip_streak_instances_at(
    rain: f32,
    snow: f32,
    fog: f32,
    wind_speed: f32,
    wind_mps: Vec2,
    weather_chunk_count: u32,
    origin: bevy::math::Vec2,
    zoom: WeatherPrecipZoomBand,
    field: Option<&WeatherField>,
    sim_time: SimTimeMicros,
) -> Vec<GpuInstancedQuadInstance> {
    let precip = (rain * 0.85 + snow * 0.65).clamp(0.0, 1.0);
    if precip < PRECIP_STREAK_THRESHOLD || weather_chunk_count == 0 {
        return Vec::new();
    }
    let raw = ((precip * WEATHER_PRECIP_STREAK_CAP as f32 * zoom.density_scale) as usize)
        .clamp(1, WEATHER_PRECIP_STREAK_CAP);
    let n = raw;
    let wind = wind_speed.max(wind_mps.length() / 20.0);
    let half = (0.35 + precip * 0.45) * zoom.half_scale;
    let radius = zoom.scatter_radius;
    let drift = wind_mps * 0.35;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let fi = i as f32;
        let x = origin.x + (fi * 17.13 + wind * 3.1).sin() * radius + drift.x;
        let y = origin.y + (fi * 9.71 + wind * 1.7).cos() * radius + drift.y;
        let (streak_rain, streak_snow) = if let Some(f) = field {
            let s = f.sample(Vec2::new(x, y) * TILE_METRES_GAMEPLAY, sim_time);
            rain_snow01_from_sample(&s)
        } else {
            (rain, snow)
        };
        if streak_rain + streak_snow < PRECIP_STREAK_THRESHOLD * 0.5 {
            continue;
        }
        out.push(GpuInstancedQuadInstance {
            position_xyz_custom0: Vec4::new(x, y, PRECIP_Z, streak_rain),
            custom1_custom2_custom3_custom4: Vec4::new(
                streak_snow,
                PRECIP_CLASS_ID,
                half,
                fog,
            ),
        });
    }
    out
}

pub(crate) fn fill_weather_precip_frame_from_climate(
    climate: Res<ClimateVisualAggregate>,
    cam: Res<ExtractedCameraMetrics>,
    zoom_frame: Option<Res<ZoomFrame>>,
    local: Option<Res<CameraLocalWeatherSample>>,
    field: Option<Res<WeatherField>>,
    sim_time: Option<Res<SimTimeMicros>>,
    mut frame: ResMut<WeatherPrecipFrame>,
    mut domains: ResMut<ParticleDomainRegistry>,
) {
    let zoom_alpha = zoom_frame
        .map(|z| z.zoom_alpha)
        .unwrap_or(cam.zoom_alpha);
    // Approximate visible half-diagonal from view pixels / zoom (px-per-tile).
    let zoom = cam.zoom_level.max(0.08);
    let half_w = cam.view_pixels.x.max(1.0) * 0.5 / zoom;
    let half_h = cam.view_pixels.y.max(1.0) * 0.5 / zoom;
    let half_diag = (half_w * half_w + half_h * half_h).sqrt();
    let t = sim_time
        .as_deref()
        .copied()
        .unwrap_or_else(|| {
            field
                .as_ref()
                .map(|f| SimTimeMicros(f.prev_micros))
                .unwrap_or(SimTimeMicros(0))
        });
    apply_weather_precip_frame_from_climate_at(
        &climate,
        &mut frame,
        &mut domains,
        false,
        Some(&cam),
        WeatherPrecipZoomBand::from_zoom_and_view(zoom_alpha, half_diag),
        local.as_deref(),
        field.as_deref(),
        t,
    );
}

pub(super) fn register_weather_precip_extract(app: &mut App) {
    app.init_resource::<ExtractedCameraMetrics>();
    app.configure_sets(Update, AtmospherePipelineSet::VisualExtract);
    app.configure_sets(Update, ExtractedCameraMetricsSet::Sync);
    app.add_systems(
        Update,
        fill_weather_precip_frame_from_climate
            .after(AtmospherePipelineSet::VisualExtract)
            .after(ExtractedCameraMetricsSet::Sync)
            .after(crate::systems::weather::publish_camera_local_weather_sample),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::particle_domains::ParticleDomainRegistry;

    fn rainy_climate() -> ClimateVisualAggregate {
        ClimateVisualAggregate {
            mean_rain: 0.55,
            mean_snow: 0.1,
            mean_fog_density: 0.2,
            mean_wind_speed: 1.2,
            weather_chunk_count: 8,
            ..Default::default()
        }
    }

    #[test]
    fn activates_domain_and_prepares_streaks() {
        let climate = rainy_climate();
        let mut frame = WeatherPrecipFrame::default();
        let mut domains = ParticleDomainRegistry::default();
        apply_weather_precip_frame_from_climate(&climate, &mut frame, &mut domains, false);
        assert!(frame.climate_fed);
        assert!(domains.weather_precip_active());
        assert!(!frame.streaks.is_empty());
    }

    #[test]
    fn background_band_fewer_streaks_than_tactical() {
        let climate = rainy_climate();
        let tac = build_precip_streak_instances_at(
            climate.mean_rain,
            climate.mean_snow,
            climate.mean_fog_density,
            climate.mean_wind_speed,
            climate.mean_wind_mps,
            climate.weather_chunk_count,
            bevy::math::Vec2::ZERO,
            WeatherPrecipZoomBand::from_zoom_alpha(0.8),
            None,
            SimTimeMicros(0),
        );
        let bg = build_precip_streak_instances_at(
            climate.mean_rain,
            climate.mean_snow,
            climate.mean_fog_density,
            climate.mean_wind_speed,
            climate.mean_wind_mps,
            climate.weather_chunk_count,
            bevy::math::Vec2::ZERO,
            WeatherPrecipZoomBand::from_zoom_alpha(0.2),
            None,
            SimTimeMicros(0),
        );
        assert!(tac.len() > bg.len());
        assert!(WeatherPrecipZoomBand::from_zoom_alpha(0.8).tactical);
        assert!(!WeatherPrecipZoomBand::from_zoom_alpha(0.2).tactical);
    }

    #[test]
    fn camera_origin_offsets_streaks() {
        let climate = rainy_climate();
        let cam = ExtractedCameraMetrics {
            translation: bevy::math::Vec2::new(100.0, -50.0),
            ..Default::default()
        };
        let mut frame = WeatherPrecipFrame::default();
        let mut domains = ParticleDomainRegistry::default();
        apply_weather_precip_frame_from_climate_at(
            &climate,
            &mut frame,
            &mut domains,
            false,
            Some(&cam),
            WeatherPrecipZoomBand::from_zoom_alpha(0.7),
            None,
            None,
            SimTimeMicros(0),
        );
        assert!(frame.camera_centered);
        assert!(frame.tactical_band);
        let first = frame.streaks[0].position_xyz_custom0;
        assert!((first.x - 100.0).abs() < 80.0);
    }

    #[test]
    fn dry_climate_activates_domain_with_empty_streaks() {
        let climate = ClimateVisualAggregate {
            weather_chunk_count: 2,
            mean_rain: 0.01,
            ..Default::default()
        };
        let mut frame = WeatherPrecipFrame::default();
        let mut domains = ParticleDomainRegistry::default();
        apply_weather_precip_frame_from_climate(&climate, &mut frame, &mut domains, false);
        assert!(domains.weather_precip_active());
        assert!(frame.streaks.is_empty());
    }
}
