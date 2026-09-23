//! Climate → [`WeatherPrecipFrame`] frontend extract (EFFECTS-SYSTEM ES-5).
//!
//! Preserves single climate scan ([`ClimateVisualAggregate`]).
//! ES-5-3: CPU mesh retired — domain always eligible when climate feeds.
//! ES-5-4 provisional: density/spread from zoom_alpha (tactical vs background).

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
use crate::systems::weather::WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;

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

/// Pure apply — used by scheduled system and unit/live proofs.
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
) {
    frame.mean_rain = climate.mean_rain;
    frame.mean_snow = climate.mean_snow;
    frame.mean_fog = climate.mean_fog_density;
    frame.mean_wind = climate.mean_wind_speed;
    frame.weather_chunk_count = climate.weather_chunk_count;
    frame.climate_fed = true;
    frame.gpu_authority = false;
    frame.zoom_alpha = zoom.zoom_alpha;
    frame.tactical_band = zoom.tactical;

    domains.set_weather_precip_active(true);
    let origin = camera.map(|c| c.translation).unwrap_or(bevy::math::Vec2::ZERO);
    frame.streaks = build_precip_streak_instances_at(climate, origin, zoom);
    frame.instance_count = frame.streaks.len();
    frame.camera_centered = camera.is_some();
}

#[must_use]
pub fn build_precip_streak_instances(climate: &ClimateVisualAggregate) -> Vec<GpuInstancedQuadInstance> {
    build_precip_streak_instances_at(
        climate,
        bevy::math::Vec2::ZERO,
        WeatherPrecipZoomBand::from_zoom_alpha(0.5),
    )
}

#[must_use]
pub fn build_precip_streak_instances_at(
    climate: &ClimateVisualAggregate,
    origin: bevy::math::Vec2,
    zoom: WeatherPrecipZoomBand,
) -> Vec<GpuInstancedQuadInstance> {
    let precip = (climate.mean_rain * 0.85 + climate.mean_snow * 0.65).clamp(0.0, 1.0);
    if precip < PRECIP_STREAK_THRESHOLD || climate.weather_chunk_count == 0 {
        return Vec::new();
    }
    let raw = ((precip * WEATHER_PRECIP_STREAK_CAP as f32 * zoom.density_scale) as usize)
        .clamp(1, WEATHER_PRECIP_STREAK_CAP);
    let n = raw;
    let wind = climate.mean_wind_speed;
    let half = (0.35 + precip * 0.45) * zoom.half_scale;
    let radius = zoom.scatter_radius;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let fi = i as f32;
        let x = origin.x + (fi * 17.13 + wind * 3.1).sin() * radius;
        let y = origin.y + (fi * 9.71 + wind * 1.7).cos() * radius;
        out.push(GpuInstancedQuadInstance {
            position_xyz_custom0: Vec4::new(x, y, PRECIP_Z, climate.mean_rain),
            custom1_custom2_custom3_custom4: Vec4::new(
                climate.mean_snow,
                PRECIP_CLASS_ID,
                half,
                climate.mean_fog_density,
            ),
        });
    }
    out
}

pub(crate) fn fill_weather_precip_frame_from_climate(
    climate: Res<ClimateVisualAggregate>,
    cam: Res<ExtractedCameraMetrics>,
    zoom_frame: Option<Res<ZoomFrame>>,
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
    apply_weather_precip_frame_from_climate_at(
        &climate,
        &mut frame,
        &mut domains,
        false,
        Some(&cam),
        WeatherPrecipZoomBand::from_zoom_and_view(zoom_alpha, half_diag),
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
            .after(ExtractedCameraMetricsSet::Sync),
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
            &climate,
            bevy::math::Vec2::ZERO,
            WeatherPrecipZoomBand::from_zoom_alpha(0.8),
        );
        let bg = build_precip_streak_instances_at(
            &climate,
            bevy::math::Vec2::ZERO,
            WeatherPrecipZoomBand::from_zoom_alpha(0.2),
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
        );
        assert!(frame.camera_centered);
        assert!(frame.tactical_band);
        let first = frame.streaks[0].position_xyz_custom0;
        assert!((first.x - 100.0).abs() < 50.0);
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
