//! Precipitation **visual groundwork**: cheap full-view tint + GPU-mesh “particles” driven by mean weather
//! from [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate) (synced in atmosphere visual extract).
//!
//! Not physically accurate—sets up ECS structure, hooks, and tunables for later art/VFX swaps.
//! Overlay + flakes live under the primary [`Camera2d`] so they track the view.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::gui::{camera_translation, camera_zoom, MapCameraDesired, ViewId, ViewManager, MAP_ZOOM_CLAMP};
use crate::render::{resolved_particle_half_extents, ResolvedViewports, ViewportPipelineSet};
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::render::ClimateVisualAggregate;
use crate::render::{trace_particle_routing, DebugRenderTraceConfig};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

/// Enable / cap weather visuals (designer can toggle from diagnostics later).
#[derive(Resource, Debug, Clone)]
pub struct WeatherVisualSettings {
    pub enabled: bool,
    pub overlay: bool,
    pub particles: bool,
    /// Screen-space rain/snow streaks when zoomed out — “digital AE” background (separate from tactical precip).
    pub background_aesthetic: bool,
    pub max_precip_particles: usize,
}

impl Default for WeatherVisualSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            overlay: true,
            particles: true,
            background_aesthetic: true,
            max_precip_particles: 192,
        }
    }
}

/// Running mean of chunk weather used by overlay + particle density (updated from [`ClimateVisualAggregate`] each frame).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WeatherPrecipVisualSample {
    pub rain: f32,
    pub snow: f32,
    pub fog: f32,
    pub chunk_count: u32,
}

#[derive(Component)]
pub struct WeatherVfxCameraChild;

#[derive(Component)]
struct WeatherPrecipOverlay;

#[derive(Component)]
struct PrecipParticle {
    kind: PrecipKind,
    speed: f32,
    wobble: f32,
    /// Local X half-span (set from window once).
    half_width: f32,
    half_height: f32,
}

#[derive(Clone, Copy)]
enum PrecipKind {
    Rain,
    Snow,
}

#[derive(Resource)]
struct WeatherVfxMaterials {
    overlay: Handle<ColorMaterial>,
}

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

    let overlay_mat = materials.add(ColorMaterial::from_color(Color::srgba(0.52, 0.58, 0.78, 0.08)));
    let rain_mat = materials.add(ColorMaterial::from_color(Color::srgba(0.72, 0.76, 0.95, 0.82)));
    let snow_mat = materials.add(ColorMaterial::from_color(Color::WHITE.with_alpha(0.78)));

    let overlay_mesh = meshes.add(Rectangle::new(4800.0, 4800.0));
    let rain_mesh = meshes.add(Rectangle::new(1.8, 10.0));
    let snow_mesh = meshes.add(Rectangle::new(3.2, 3.2));

    let cap = settings.max_precip_particles;
    let mut rng = thread_rng();

    let vfx_root = commands
        .spawn((
            WeatherVfxCameraChild,
            Name::new("WeatherVfxRoot"),
            Transform::default(),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            parent.spawn((
                WeatherPrecipOverlay,
                Mesh2d(overlay_mesh),
                MeshMaterial2d(overlay_mat.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -400.0)),
                Visibility::Visible,
            ));
            for i in 0..cap {
                let snow_bias = rng.gen::<f32>();
                let (kind, mesh, mat, speed) = if snow_bias < 0.35 {
                    (
                        PrecipKind::Snow,
                        snow_mesh.clone(),
                        snow_mat.clone(),
                        rng.gen_range(22.0..48.0),
                    )
                } else {
                    (
                        PrecipKind::Rain,
                        rain_mesh.clone(),
                        rain_mat.clone(),
                        rng.gen_range(320.0..520.0),
                    )
                };
                let hw = 960.0_f32;
                let hh = 540.0_f32;
                let x = rng.gen_range(-hw..hw);
                let y = rng.gen_range(-hh..hh);
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(mat),
                    Transform::from_translation(Vec3::new(x, y, -200.0 + i as f32 * 0.01)),
                    Visibility::Hidden,
                    PrecipParticle {
                        kind,
                        speed,
                        wobble: rng.gen_range(0.0..TAU),
                        half_width: hw,
                        half_height: hh,
                    },
                ));
            }
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
        if let Some(m) = materials.get_mut(&handles.overlay) {
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
    if let Some(m) = materials.get_mut(&handles.overlay) {
        m.color = Color::srgba(0.52, 0.58, 0.78, alpha);
    }
}

fn sync_precip_sample_at_camera_focus(
    climate: Res<ClimateVisualAggregate>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesired>,
    weather: Query<(&Chunk, &ChunkWeather)>,
    mut sample: ResMut<WeatherPrecipVisualSample>,
) {
    let focus = camera_translation(&view_manager, ViewId::WorldMain)
        .unwrap_or_else(|| desired.translation.truncate());
    let mut local_rain = 0.0_f32;
    let mut local_snow = 0.0_f32;
    let mut local_fog = 0.0_f32;
    let mut local_n = 0u32;
    for (chunk, wx) in &weather {
        let center = Vec2::new(chunk.coord.x as f32 + 0.5, chunk.coord.y as f32 + 0.5);
        if focus.distance(center) > 2.5 {
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

fn tick_precip_particles(
    cfg: Res<DebugRenderTraceConfig>,
    time: Res<Time>,
    settings: Res<WeatherVisualSettings>,
    sample: Res<WeatherPrecipVisualSample>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesired>,
    resolved: Res<ResolvedViewports>,
    mut q: Query<(&mut Transform, &mut Visibility, &mut PrecipParticle)>,
    mut last_trace: Local<u64>,
) {
    // Keep precip visibly animating when sim dt stalls but wall clock advances (menu pause, hitch).
    let dt = time.delta_secs().max(1.0 / 120.0);
    let rain = sample.rain.clamp(0.0, 1.0);
    let snow = sample.snow.clamp(0.0, 1.0);
    let precip = (rain * 0.85 + snow * 0.65).clamp(0.0, 1.0);

    let (hw, hh) = resolved_particle_half_extents(&resolved);
    if cfg.particle_routing_trace && *last_trace != resolved.revision {
        *last_trace = resolved.revision;
        bevy::log::info!(
            target: "proc_A_dine01::render::particle",
            "precip half_extents=({hw:.1},{hh:.1}) revision={}",
            resolved.revision,
        );
    }

    let zoom = camera_zoom(&view_manager, ViewId::WorldMain)
        .unwrap_or(desired.scale.x)
        .clamp(MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
    let zoom_t = ((MAP_ZOOM_CLAMP.1 - zoom) / (MAP_ZOOM_CLAMP.1 - MAP_ZOOM_CLAMP.0)).clamp(0.0, 1.0);
    let focus_strength = 0.2 + 0.8 * zoom_t;
    let background_strength = 0.15 + 0.35 * (1.0 - zoom_t);
    let strength = (precip * focus_strength + precip * background_strength * 0.35).clamp(0.0, 1.0);
    let climate_active = sample.chunk_count > 0
        || sample.rain > 0.06
        || sample.snow > 0.04
        || sample.fog > 0.05;
    let tactical_precip = zoom_t > 0.45;

    for (mut xf, mut vis, mut p) in &mut q {
        p.half_width = hw;
        p.half_height = hh;

        let show_tactical = settings.enabled
            && settings.particles
            && climate_active
            && strength > 0.02
            && tactical_precip;
        let show_background = settings.enabled
            && settings.particles
            && settings.background_aesthetic
            && climate_active
            && precip > 0.04
            && !tactical_precip;
        if !show_tactical && !show_background {
            *vis = Visibility::Hidden;
            continue;
        }

        let kind_factor = match p.kind {
            PrecipKind::Rain => rain,
            PrecipKind::Snow => snow.max(rain * 0.25),
        };
        if kind_factor < 0.03 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Visible;

        let motion_strength = if show_background {
            (precip * background_strength * 0.72).clamp(0.1, 0.5)
        } else {
            strength
        };

        p.wobble += dt * 4.5;
        let side = match p.kind {
            PrecipKind::Rain => 0.0,
            PrecipKind::Snow => (p.wobble.sin() * 40.0 + p.wobble.cos() * 18.0) * dt,
        };

        let fall = p.speed * dt * motion_strength;
        xf.translation.x += side;
        xf.translation.y -= fall;

        let margin = 40.0_f32;
        if xf.translation.y < -p.half_height - margin {
            let mut rng = thread_rng();
            xf.translation.y = p.half_height + margin;
            xf.translation.x = rng.gen_range(-p.half_width..p.half_width);
        }
        if xf.translation.x.abs() > p.half_width + margin {
            xf.translation.x = xf.translation.x.signum() * (p.half_width + margin);
        }
    }
    if cfg.particle_routing_trace {
        *last_trace = last_trace.wrapping_add(1);
        if *last_trace % 30 == 0 {
            let visible = q
                .iter()
                .filter(|(_, vis, _)| **vis == Visibility::Visible)
                .count();
            trace_particle_routing(
                &cfg,
                &format!(
                    "weather_precip_particles coordinate_space=screen_hybrid active_count={visible} enabled={} particles={} zoom_t={:.2} strength={:.2}",
                    settings.enabled,
                    settings.particles,
                    zoom_t,
                    strength,
                ),
            );
        }
    }
}

pub struct WeatherVisualPlugin;

impl Plugin for WeatherVisualPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherVisualSettings>()
            .init_resource::<WeatherPrecipVisualSample>()
            .add_systems(PostStartup, attach_weather_vfx_to_camera)
            .add_systems(
                Update,
                (
                    sync_precip_sample_at_camera_focus
                        .after(AtmospherePipelineSet::VisualExtract)
                        .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager),
                    update_overlay_from_weather.after(AtmospherePipelineSet::VisualExtract),
                ),
            )
            .add_systems(
                PostUpdate,
                tick_precip_particles.after(ViewportPipelineSet::Resolve),
            );
    }
}
