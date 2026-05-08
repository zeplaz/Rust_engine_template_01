//! Precipitation **visual groundwork**: cheap full-view tint + GPU-mesh “particles” driven by mean [`ChunkWeather`].
//!
//! Not physically accurate—sets up ECS structure, hooks, and tunables for later art/VFX swaps.
//! Overlay + flakes live under the primary [`Camera2d`] so they track the view.

use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use super::chunk_weather::ChunkWeather;

/// Enable / cap weather visuals (designer can toggle from diagnostics later).
#[derive(Resource, Debug, Clone)]
pub struct WeatherVisualSettings {
    pub enabled: bool,
    pub overlay: bool,
    pub particles: bool,
    pub max_precip_particles: usize,
}

impl Default for WeatherVisualSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            overlay: true,
            particles: true,
            max_precip_particles: 192,
        }
    }
}

/// Running mean of chunk weather used by overlay + particle density (updated each frame).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WeatherPrecipVisualSample {
    pub rain: f32,
    pub snow: f32,
    pub fog: f32,
    pub chunk_count: u32,
}

#[derive(Component)]
pub(crate) struct WeatherVfxCameraChild;

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

fn sample_chunk_weather_for_visuals(
    query: Query<&ChunkWeather>,
    mut out: ResMut<WeatherPrecipVisualSample>,
) {
    let mut n = 0u32;
    let mut rain = 0f32;
    let mut snow = 0f32;
    let mut fog = 0f32;
    for w in &query {
        n += 1;
        rain += w.rain_intensity;
        snow += w.snow_depth;
        fog += w.fog_density;
    }
    if n == 0 {
        *out = WeatherPrecipVisualSample::default();
        return;
    }
    let nf = n as f32;
    *out = WeatherPrecipVisualSample {
        rain: rain / nf,
        snow: snow / nf,
        fog: fog / nf,
        chunk_count: n,
    };
}

fn attach_weather_vfx_to_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cameras: Query<Entity, With<Camera2d>>,
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
) {
    let Some(handles) = vfx_mats else {
        return;
    };
    if !settings.enabled || !settings.overlay {
        if let Some(m) = materials.get_mut(&handles.overlay) {
            m.color = Color::WHITE.with_alpha(0.0);
        }
        return;
    }
    let rain = sample.rain.clamp(0.0, 1.0);
    let fog = sample.fog.clamp(0.0, 1.0);
    let alpha = (rain * 0.14 + fog * 0.1).min(0.45);
    if let Some(m) = materials.get_mut(&handles.overlay) {
        m.color = Color::srgba(0.52, 0.58, 0.78, alpha);
    }
}

fn tick_precip_particles(
    time: Res<Time>,
    settings: Res<WeatherVisualSettings>,
    sample: Res<WeatherPrecipVisualSample>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut q: Query<(&mut Transform, &mut Visibility, &mut PrecipParticle)>,
) {
    let dt = time.delta_secs();
    let rain = sample.rain.clamp(0.0, 1.0);
    let snow = sample.snow.clamp(0.0, 1.0);
    let precip = (rain * 0.85 + snow * 0.65).clamp(0.0, 1.0);

    let (mut hw, mut hh) = (960.0_f32, 540.0_f32);
    if let Ok(win) = windows.single() {
        hw = (win.width() * 0.5).max(200.0);
        hh = (win.height() * 0.5).max(200.0);
    }

    for (mut xf, mut vis, mut p) in &mut q {
        p.half_width = hw;
        p.half_height = hh;

        let show = settings.enabled && settings.particles && sample.chunk_count > 0 && precip > 0.02;
        if !show {
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

        p.wobble += dt * 4.5;
        let side = match p.kind {
            PrecipKind::Rain => 0.0,
            PrecipKind::Snow => (p.wobble.sin() * 40.0 + p.wobble.cos() * 18.0) * dt,
        };

        let fall = p.speed * dt * precip * 0.85;
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
                    sample_chunk_weather_for_visuals,
                    update_overlay_from_weather.after(sample_chunk_weather_for_visuals),
                    tick_precip_particles.after(sample_chunk_weather_for_visuals),
                ),
            );
    }
}
