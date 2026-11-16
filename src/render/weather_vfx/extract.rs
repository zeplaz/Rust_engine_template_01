//! Climate / camera-local → [`WeatherPrecipFrame`] frontend extract (EFFECTS-SYSTEM ES-5).
//!
//! **W6 / X4:** density and per-streak rain/snow from [`AtmosphereFrame`] at camera / streak XY
//! when the frame is built; [`ClimateVisualAggregate`] remains the fallback for tests / dry boot.
//! Presentation-only — never writes sim authorities (LGT-04 skies/clouds presentation).

use bevy::math::Vec4;
use bevy::prelude::*;

use crate::gui::ZoomFrame;
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;
use crate::render::particle_domains::ParticleDomainRegistry;
use crate::render::ClimateVisualAggregate;
use crate::systems::atmosphere::frame::AtmosphereFrame;
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::systems::weather::{
    rain_snow01_from_sample, CameraLocalWeatherSample, WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA,
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

    /// PixelWorld3D: scatter over the ground footprint the voxel camera sees (`half_diag` in
    /// tiles). Unlike [`Self::from_zoom_and_view`] there is no 64-tile floor — at play zoom the
    /// footprint is a few tiles wide and the floor would leave the view nearly dry.
    #[must_use]
    pub fn from_ground_footprint(zoom_alpha: f32, half_diag: f32) -> Self {
        let mut band = Self::from_zoom_and_view(zoom_alpha, half_diag);
        band.scatter_radius = half_diag.max(4.0);
        band
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
    atmos: Option<&AtmosphereFrame>,
) {
    let (rain, snow, fog, wind_speed, wind_mps, local_ok, frame_rev) =
        if let Some(loc) = local.filter(|l| l.ready) {
            (
                loc.rain01,
                loc.snow01,
                loc.fog,
                loc.sample.wind_mps.length() / 20.0,
                loc.sample.wind_mps,
                true,
                loc.frame_revision,
            )
        } else {
            (
                climate.mean_rain,
                climate.mean_snow,
                climate.mean_fog_density,
                climate.mean_wind_speed,
                climate.mean_wind_mps,
                false,
                atmos.map(|a| a.revision()).unwrap_or(0),
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
    frame.frame_revision = frame_rev;

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
        atmos.filter(|a| a.is_weather_ready()),
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
    atmos: Option<&AtmosphereFrame>,
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
        let (streak_rain, streak_snow) = if let Some(a) = atmos {
            match a.weather_at_pos_m(Vec2::new(x, y) * TILE_METRES_GAMEPLAY) {
                Some(s) => rain_snow01_from_sample(&s),
                None => (rain, snow),
            }
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
    atmos: Option<Res<AtmosphereFrame>>,
    mut frame: ResMut<WeatherPrecipFrame>,
    mut domains: ResMut<ParticleDomainRegistry>,
    mut diag: Option<ResMut<crate::systems::atmosphere::AtmosphereDiagnostics>>,
    pixel_cam: Option<Res<crate::render::pixel_world::PixelWorldCameraFrame>>,
) {
    let zoom_alpha = zoom_frame
        .map(|z| z.zoom_alpha)
        .unwrap_or(cam.zoom_alpha);
    // Approximate visible half-diagonal from view pixels / zoom (px-per-tile).
    let zoom = cam.zoom_level.max(0.08);
    let half_w = cam.view_pixels.x.max(1.0) * 0.5 / zoom;
    let half_h = cam.view_pixels.y.max(1.0) * 0.5 / zoom;
    let half_diag = (half_w * half_w + half_h * half_h).sqrt();
    // PixelWorld3D active: the ortho view rect is not what the tilted perspective camera sees.
    // Scatter streaks over the voxel ground footprint instead (same frame, same writer).
    if let Some(px) = pixel_cam.as_deref().filter(|p| p.active) {
        let centre = (px.ground_min + px.ground_max) * 0.5;
        let half_ext = (px.ground_max - px.ground_min) * 0.5;
        let voxel_cam = ExtractedCameraMetrics {
            translation: centre,
            ..*cam
        };
        apply_weather_precip_frame_from_climate_at(
            &climate,
            &mut frame,
            &mut domains,
            false,
            Some(&voxel_cam),
            WeatherPrecipZoomBand::from_ground_footprint(zoom_alpha, half_ext.max_element()),
            local.as_deref(),
            atmos.as_deref(),
        );
    } else {
        apply_weather_precip_frame_from_climate_at(
            &climate,
            &mut frame,
            &mut domains,
            false,
            Some(&cam),
            WeatherPrecipZoomBand::from_zoom_and_view(zoom_alpha, half_diag),
            local.as_deref(),
            atmos.as_deref(),
        );
    }
    if let Some(d) = diag.as_mut() {
        d.presentation_frame_revision = frame.frame_revision;
    }
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

    #[test]
    fn precip_extract_does_not_read_weather_field() {
        let src = include_str!("extract.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap();
        assert!(!prod.contains("Res<WeatherField>"));
        assert!(!prod.contains("field.sample("));
        assert!(prod.contains("AtmosphereFrame"));
    }
}
