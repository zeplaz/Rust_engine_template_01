//! Precipitation **visual overlay** driven by mean weather from
//! [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate).
//!
//! **ES-5-3:** CPU mesh precip (`PrecipParticle` / `tick_precip_particles`) **retired**.
//! Streaks live in [`crate::render::weather_vfx`] (GPU instanced-quad, domain WeatherPrecip).
//! This plugin keeps the full-view tint overlay only.
//!
//! **ES-5-4 (provisional):** GPU density uses [`WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA`] —
//! designer may refine the table later without blocking ship.

use bevy::prelude::*;

use crate::render::ExtractedCameraMetrics;
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::render::ClimateVisualAggregate;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::engine::states::BaseState;

/// Enable / cap weather visuals (designer can toggle from diagnostics later).
#[derive(Resource, Debug, Clone)]
pub struct WeatherVisualSettings {
    pub enabled: bool,
    pub overlay: bool,
    /// Retained for diagnostics / API; mesh particles are retired (always false at runtime).
    pub particles: bool,
    /// Screen-space aesthetic band when zoomed out — consumed by GPU frontend density.
    pub background_aesthetic: bool,
    pub max_precip_particles: usize,
}

/// **VX-P0-03 / ES-5-4 provisional** — tactical GPU streaks above this zoom_alpha; background below.
pub const WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA: f32 = 0.45;

/// True when precip particles use the tactical (zoomed-in) band.
#[inline]
#[must_use]
pub fn weather_precip_tactical_band(zoom_alpha: f32) -> bool {
    zoom_alpha > WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA
}

/// True when screen-space background rain/snow aesthetic should run (zoomed out, climate active).
#[must_use]
pub fn weather_precip_show_background(
    settings: &WeatherVisualSettings,
    sample: &WeatherPrecipVisualSample,
    precip: f32,
    zoom_alpha: f32,
    strength: f32,
) -> bool {
    let climate_active = sample.chunk_count > 0
        || sample.rain > 0.06
        || sample.snow > 0.04
        || sample.fog > 0.05;
    settings.enabled
        && settings.background_aesthetic
        && climate_active
        && precip > 0.04
        && strength > 0.02
        && !weather_precip_tactical_band(zoom_alpha)
}

/// True when tactical precip streaks should run (zoomed in, climate active).
#[must_use]
pub fn weather_precip_show_tactical(
    settings: &WeatherVisualSettings,
    sample: &WeatherPrecipVisualSample,
    zoom_alpha: f32,
    strength: f32,
) -> bool {
    let climate_active = sample.chunk_count > 0
        || sample.rain > 0.06
        || sample.snow > 0.04
        || sample.fog > 0.05;
    settings.enabled
        && climate_active
        && strength > 0.02
        && weather_precip_tactical_band(zoom_alpha)
}

impl Default for WeatherVisualSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            overlay: true,
            particles: false, // ES-5-3 retired
            background_aesthetic: true,
            max_precip_particles: 0,
        }
    }
}

/// Running mean of chunk weather used by overlay (updated from [`ClimateVisualAggregate`] each frame).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WeatherPrecipVisualSample {
    pub rain: f32,
    pub snow: f32,
    pub fog: f32,
    pub chunk_count: u32,
}

/// Z above tile fallback sprite (z=0) so overlay reads on top of the world in RTT.
const WEATHER_OVERLAY_Z: f32 = 480.0;

#[derive(Component)]
pub struct WeatherVfxCameraChild;

#[derive(Component)]
struct WeatherPrecipOverlay;

#[derive(Resource)]
struct WeatherVfxMaterials {
    overlay: Handle<ColorMaterial>,
}

/// Overlay-only attach — no mesh precip children (ES-5-3).
fn attach_weather_vfx_to_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cameras: Query<Entity, With<crate::gui::MainWorldCamera>>,
    existing: Query<Entity, With<WeatherVfxCameraChild>>,
    settings: Res<WeatherVisualSettings>,
) {
    if !settings.enabled {
        return;
    }
    if !existing.is_empty() {
        return;
    }
    let Ok(cam) = cameras.single() else {
        return;
    };

    let rtt_layers = crate::gui::simulation_map_rtt_render_layers();
    let overlay_mat = materials.add(ColorMaterial::from_color(Color::srgba(0.52, 0.58, 0.78, 0.08)));
    let overlay_mesh = meshes.add(Rectangle::new(4800.0, 4800.0));

    let vfx_root = commands
        .spawn((
            WeatherVfxCameraChild,
            rtt_layers.clone(),
            Name::new("WeatherVfxRoot"),
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                WeatherPrecipOverlay,
                rtt_layers.clone(),
                Mesh2d(overlay_mesh),
                MeshMaterial2d(overlay_mat.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, WEATHER_OVERLAY_Z)),
                Visibility::Visible,
            ));
        })
        .id();

    commands.entity(cam).add_child(vfx_root);
    commands.insert_resource(WeatherVfxMaterials {
        overlay: overlay_mat,
    });
}

fn update_overlay_from_weather(
    settings: Res<WeatherVisualSettings>,
    sample: Res<WeatherPrecipVisualSample>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    vfx_mats: Option<Res<WeatherVfxMaterials>>,
    mut last_alpha: Local<f32>,
) {
    let Some(handles) = vfx_mats else {
        return;
    };
    if !settings.enabled || !settings.overlay {
        *last_alpha = -1.0;
        if let Some(mut m) = materials.get_mut(&handles.overlay) {
            m.color = Color::WHITE.with_alpha(0.0);
        }
        return;
    }
    let rain = sample.rain.clamp(0.0, 1.0);
    let fog = sample.fog.clamp(0.0, 1.0);
    let alpha = (rain * 0.14 + fog * 0.1).min(0.45);
    if (*last_alpha - alpha).abs() < 0.004 && *last_alpha >= 0.0 {
        return;
    }
    *last_alpha = alpha;
    if let Some(mut m) = materials.get_mut(&handles.overlay) {
        m.color = Color::srgba(0.52, 0.58, 0.78, alpha);
    }
}

fn sync_precip_sample_at_camera_focus(
    climate: Res<ClimateVisualAggregate>,
    metrics: Res<ExtractedCameraMetrics>,
    weather: Query<(&Chunk, &ChunkCellMatrix, &ChunkWeather)>,
    mut sample: ResMut<WeatherPrecipVisualSample>,
) {
    let focus = metrics.translation;
    let mut local_rain = 0.0_f32;
    let mut local_snow = 0.0_f32;
    let mut local_fog = 0.0_f32;
    let mut local_n = 0u32;
    for (chunk, matrix, wx) in &weather {
        let center = crate::terrain::generation::chunk_world_center(chunk.coord, matrix.size);
        if focus.distance(center) > matrix.size.x.max(matrix.size.y) as f32 * 2.5 {
            continue;
        }
        local_n += 1;
        local_rain += wx.rain_intensity;
        local_snow += wx.snow_depth;
        local_fog += wx.fog_density;
    }

    let background = 0.35_f32;
    if local_n == 0 || climate.weather_chunk_count == 0 {
        *sample = WeatherPrecipVisualSample {
            rain: climate.mean_rain,
            snow: climate.mean_snow,
            fog: climate.mean_fog_density,
            chunk_count: climate.weather_chunk_count,
        };
        return;
    }
    let local = local_n.max(1) as f32;
    *sample = WeatherPrecipVisualSample {
        rain: climate.mean_rain * background + (local_rain / local) * (1.0 - background),
        snow: climate.mean_snow * background + (local_snow / local) * (1.0 - background),
        fog: climate.mean_fog_density * background + (local_fog / local) * (1.0 - background),
        chunk_count: climate.weather_chunk_count,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{map_scale_for_zoom_alpha, map_zoom_alpha, MAP_ZOOM_CLAMP};

    fn rainy_sample() -> WeatherPrecipVisualSample {
        WeatherPrecipVisualSample {
            rain: 0.5,
            snow: 0.2,
            fog: 0.1,
            chunk_count: 12,
        }
    }

    #[test]
    fn vx_p0_03_background_precip_when_zoomed_out() {
        let settings = WeatherVisualSettings::default();
        let sample = rainy_sample();
        let zoom = map_scale_for_zoom_alpha(0.15, MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
        let zoom_alpha = map_zoom_alpha(zoom);
        assert!(
            zoom_alpha < WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA,
            "expected strategic zoom band, got {zoom_alpha}"
        );
        assert!(!weather_precip_tactical_band(zoom_alpha));
        assert!(weather_precip_show_background(
            &settings, &sample, 0.5, zoom_alpha, 0.2
        ));
        assert!(!weather_precip_show_tactical(
            &settings, &sample, zoom_alpha, 0.2
        ));
    }

    #[test]
    fn vx_p0_03_tactical_precip_when_zoomed_in() {
        let settings = WeatherVisualSettings::default();
        let sample = rainy_sample();
        let zoom = map_scale_for_zoom_alpha(0.75, MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
        let zoom_alpha = map_zoom_alpha(zoom);
        assert!(weather_precip_tactical_band(zoom_alpha));
        assert!(weather_precip_show_tactical(
            &settings, &sample, zoom_alpha, 0.2
        ));
        assert!(!weather_precip_show_background(
            &settings, &sample, 0.5, zoom_alpha, 0.2
        ));
    }

    #[test]
    fn vx_p0_03_background_respects_diagnostics_toggle() {
        let mut settings = WeatherVisualSettings::default();
        settings.background_aesthetic = false;
        let sample = rainy_sample();
        assert!(!weather_precip_show_background(
            &settings, &sample, 0.5, 0.2, 0.2
        ));
    }

    #[test]
    fn es5_3_overlay_only_no_mesh_children_in_attach() {
        let src = include_str!("weather_visual.rs");
        assert!(src.contains("Name::new(\"WeatherVfxRoot\")"));
        let attach_start = src
            .find("fn attach_weather_vfx_to_camera")
            .expect("attach fn");
        let attach_end = src[attach_start..]
            .find("\nfn update_overlay_from_weather")
            .map(|i| attach_start + i)
            .unwrap_or(src.len());
        let attach = &src[attach_start..attach_end];
        assert!(!attach.contains("for i in 0..cap"));
        assert!(attach.contains("WeatherPrecipOverlay"));
    }
}

pub struct WeatherVisualPlugin;

impl Plugin for WeatherVisualPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherVisualSettings::default())
            .init_resource::<WeatherPrecipVisualSample>()
            .add_systems(PostStartup, attach_weather_vfx_to_camera)
            .add_systems(OnEnter(BaseState::Simulation), attach_weather_vfx_to_camera)
            .add_systems(
                Update,
                (
                    sync_precip_sample_at_camera_focus
                        .after(AtmospherePipelineSet::VisualExtract)
                        .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager),
                    update_overlay_from_weather.after(AtmospherePipelineSet::VisualExtract),
                ),
            );
    }
}
