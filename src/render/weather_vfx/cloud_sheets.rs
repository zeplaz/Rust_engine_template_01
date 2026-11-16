//! W3 / W6 — soft wind-advected cloud puffs (procedural; not volumetric — LGT-04 presentation).
//!
//! Many small radial-falloff sprites drift with local wind and wrap in a
//! camera-local field. Cover/wind come from [`CameraLocalWeatherSample`]
//! (AtmosphereFrame-derived). No sin bounce, no giant solid rectangles; no sim writes.
//!
//! ## RND-04 — Cloud alpha layering (sort, not OIT)
//!
//! NEW: **RND-04** · gea4v2 **11.3.12** / **11.3.12.1**.
//! LEGACY: `geometry_visual_effects` (gea_v2_e4 p141).
//!
//! Cloud sheets use **premultiplied-style alpha** on ColorMaterial sprites with
//! strictly increasing [`CLOUD_SHEET_LAYER_Z`] (far → near within the deck) and
//! depth-fade alpha so lower layers stay softer. Cross-pass smoke/fire ordering
//! lives in fire `ParticleClass::transparent_draw_order` + Core2d overlay chain
//! (RND-03 owns `core2d_overlay_order.rs`). Debt not taken: true volumetric OIT.
//!
//! ## RND-14 — Card / impostor clouds (intentional)
//!
//! NEW: **RND-14** · gea4v2 **11.6.4** *Card-Based Effects*.
//! LEGACY: `geometry_visual_effects` (gea_v2_e4 p141).
//!
//! Each puff is a **camera-local card** (`Rectangle` mesh + soft puff texture) — a
//! billboard/impostor stack that fakes volume via layered alpha, not ray marching.
//! [`CLOUD_SHEET_LAYER_Z`] strata stay RND-04-owned; this slice only locks the card path.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::gui::MainWorldCamera;
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::systems::atmosphere::pipeline::AtmospherePipelineSet;
use crate::systems::weather::CameraLocalWeatherSample;
use crate::systems::weather::WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;

const PUFF_COUNT: usize = 36;
const PUFF_TEX_SIZE: u32 = 64;
/// Camera-local wrap half-extents (world units under the weather VFX root).
const FIELD_HALF_W: f32 = 240.0;
const FIELD_HALF_H: f32 = 150.0;
/// RND-04: three cloud deck strata — Z must stay strictly increasing (back→front).
pub(crate) const CLOUD_SHEET_LAYER_Z: [f32; 3] = [400.0, 455.0, 510.0];
const LAYER_Z: [f32; 3] = CLOUD_SHEET_LAYER_Z;
/// Visual meters of drift per (m/s) of wind per second — tuned for readable sky motion.
const WIND_VISUAL_SCALE: f32 = 2.8;

#[derive(Component)]
pub struct WeatherCloudSheet {
    pub index: u8,
    pub layer: u8,
    pub offset: Vec2,
    pub base_scale: Vec2,
    pub phase: f32,
    pub speed_mul: f32,
}

#[derive(Resource, Default)]
struct WeatherCloudSheetMaterials {
    handles: Vec<Handle<ColorMaterial>>,
}

/// Cover alpha for the whole cloud deck (0 = clear). Shared by the top-down sprites and the
/// PixelWorld3D layer so both views read the same weather.
#[must_use]
pub(crate) fn cloud_cover_alpha(cloud: f32, tactical: bool) -> f32 {
    if cloud < 0.06 {
        0.0
    } else if tactical {
        (cloud * 0.18).min(0.22)
    } else {
        (cloud * 0.32).min(0.38)
    }
}

/// Per-puff alpha: deck cover × layer depth fade × slow flicker (view time only).
#[must_use]
pub(crate) fn cloud_puff_alpha(cover_alpha: f32, layer: u8, speed_mul: f32, phase: f32, t: f32) -> f32 {
    let depth_fade = 0.55 + 0.45 * (1.0 - f32::from(layer) / 2.0);
    let flicker = 0.82
        + 0.18
            * ((t * 0.22 + phase).sin() * 0.5 + 0.5)
                .mul_add(0.6, (t * 0.11 + phase * 1.7).cos() * 0.2 + 0.5);
    (cover_alpha * depth_fade * flicker * speed_mul.clamp(0.5, 1.1)).min(0.42)
}

pub(crate) fn soft_puff_image() -> Image {
    let w = PUFF_TEX_SIZE;
    let h = PUFF_TEX_SIZE;
    let mut data = vec![0u8; (w * h * 4) as usize];
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let r_max = cx.min(cy);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            // Slightly elliptical falloff so a circle mesh reads softer / less disc-like.
            let r = ((dx * dx) * 0.85 + (dy * dy) * 1.15).sqrt() / r_max;
            // Smoothstep out; denser core, wispy edge.
            let edge = (1.0 - r).clamp(0.0, 1.0);
            let a = (edge * edge * (3.0 - 2.0 * edge)).powf(1.35);
            let i = ((y * w + x) * 4) as usize;
            data[i] = 235;
            data[i + 1] = 240;
            data[i + 2] = 248;
            data[i + 3] = (a * 255.0) as u8;
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    image
}

pub(crate) fn hash01(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9).wrapping_add(salt);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    (x as f32) * (1.0 / 4294967295.0)
}

fn wrap_axis(v: f32, half: f32) -> f32 {
    let span = half * 2.0;
    let mut x = (v + half) % span;
    if x < 0.0 {
        x += span;
    }
    x - half
}

fn attach_cloud_sheets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    cameras: Query<Entity, With<MainWorldCamera>>,
    existing: Query<Entity, With<WeatherCloudSheet>>,
    roots: Query<Entity, With<crate::systems::weather::WeatherVfxCameraChild>>,
) {
    if !existing.is_empty() {
        return;
    }
    let Ok(cam) = cameras.single() else {
        return;
    };
    let rtt = crate::gui::simulation_map_rtt_render_layers();
    let puff_tex = images.add(soft_puff_image());
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let mut handles = Vec::with_capacity(PUFF_COUNT);
    let parent = if let Ok(root) = roots.single() {
        root
    } else {
        let root = commands
            .spawn((
                crate::systems::weather::WeatherVfxCameraChild,
                rtt.clone(),
                Name::new("WeatherCloudSheetRoot"),
                Transform::default(),
                Visibility::Visible,
            ))
            .id();
        commands.entity(cam).add_child(root);
        root
    };

    for i in 0..PUFF_COUNT {
        let layer = (i % 3) as u8;
        let phase = hash01(i as u32, 1) * std::f32::consts::TAU;
        let speed_mul = 0.55 + hash01(i as u32, 2) * 0.9;
        // Small puffs; a few slightly larger "cores" per layer.
        let major = 28.0 + hash01(i as u32, 3) * 38.0 + if i % 7 == 0 { 18.0 } else { 0.0 };
        let aspect = 0.55 + hash01(i as u32, 4) * 0.7;
        let base_scale = Vec2::new(major, major * aspect);
        let offset = Vec2::new(
            (hash01(i as u32, 5) * 2.0 - 1.0) * FIELD_HALF_W,
            (hash01(i as u32, 6) * 2.0 - 1.0) * FIELD_HALF_H,
        );
        let mat = materials.add(ColorMaterial {
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            texture: Some(puff_tex.clone()),
            ..default()
        });
        handles.push(mat.clone());
        let child = commands
            .spawn((
                WeatherCloudSheet {
                    index: i as u8,
                    layer,
                    offset,
                    base_scale,
                    phase,
                    speed_mul,
                },
                rtt.clone(),
                Mesh2d(mesh.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(offset.x, offset.y, LAYER_Z[layer as usize]))
                    .with_scale(Vec3::new(base_scale.x, base_scale.y, 1.0)),
                Visibility::Visible,
                Name::new(format!("WeatherCloudPuff{i}")),
            ))
            .id();
        commands.entity(parent).add_child(child);
    }
    commands.insert_resource(WeatherCloudSheetMaterials { handles });
}

fn update_cloud_sheets(
    local: Res<CameraLocalWeatherSample>,
    cam: Res<ExtractedCameraMetrics>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sheet_mats: Option<Res<WeatherCloudSheetMaterials>>,
    mut sheets: Query<(&mut WeatherCloudSheet, &mut Transform, &mut Visibility)>,
) {
    let Some(mats) = sheet_mats else {
        return;
    };
    let cloud = if local.ready {
        local.sample.cloud.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let wind = if local.ready {
        local.sample.wind_mps
    } else {
        Vec2::ZERO
    };
    let gust = if local.ready {
        local.sample.gust_mps.max(wind.length())
    } else {
        0.0
    };
    let tactical = cam.zoom_alpha > WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA;
    let cover_alpha = cloud_cover_alpha(cloud, tactical);
    if cover_alpha < 0.01 {
        for (_, _, mut vis) in &mut sheets {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let dt = time.delta_secs().clamp(0.0, 0.05);
    let t = time.elapsed_secs();
    let wind_speed = wind.length().max(0.35);
    let wind_dir = if wind.length_squared() > 0.01 {
        wind.normalize()
    } else {
        Vec2::new(1.0, 0.15).normalize()
    };
    let perp = Vec2::new(-wind_dir.y, wind_dir.x);
    // Gust pulses speed forward — never reverse the stream.
    let gust_boost = 1.0 + ((gust / wind_speed.max(1.0)) - 1.0).clamp(0.0, 1.2) * 0.45;
    let field_w = if tactical {
        FIELD_HALF_W * 0.72
    } else {
        FIELD_HALF_W
    };
    let field_h = if tactical {
        FIELD_HALF_H * 0.72
    } else {
        FIELD_HALF_H
    };
    let size_mul = if tactical { 0.62 } else { 1.0 };

    for (mut puff, mut tf, mut vis) in &mut sheets {
        let layer_f = puff.layer as f32;
        // Higher layers drift faster (parallax / jet feel).
        let layer_speed = 0.65 + layer_f * 0.35;
        let advect = wind_dir
            * (wind_speed * WIND_VISUAL_SCALE * layer_speed * puff.speed_mul * gust_boost);
        // Light cross-flow shear — small amplitude, not a bounce.
        let shear = perp
            * ((t * (0.07 + puff.phase * 0.01) + puff.phase).sin()
                * (1.2 + layer_f * 0.4)
                * puff.speed_mul);
        puff.offset += (advect + shear) * dt;
        puff.offset.x = wrap_axis(puff.offset.x, field_w);
        puff.offset.y = wrap_axis(puff.offset.y, field_h);

        // Soft morph: breathe scale along the wind axis, stay cloud-like.
        let breathe = 1.0
            + 0.06 * (t * (0.15 + hash01(puff.index as u32, 9) * 0.1) + puff.phase).sin()
            + 0.04 * cloud;
        let stretch = 1.0 + 0.08 * (wind_speed / 12.0).clamp(0.0, 1.0);
        let sx = puff.base_scale.x * size_mul * breathe * stretch;
        let sy = puff.base_scale.y * size_mul * breathe * (2.0 - stretch);
        tf.translation.x = puff.offset.x;
        tf.translation.y = puff.offset.y;
        tf.translation.z = LAYER_Z[puff.layer as usize];
        tf.scale = Vec3::new(sx, sy, 1.0);
        // Face slightly into the wind so ellipses read as streaming.
        tf.rotation = Quat::from_rotation_z(wind_dir.y.atan2(wind_dir.x) * 0.35);

        let alpha = cloud_puff_alpha(cover_alpha, puff.layer, puff.speed_mul, puff.phase, t);
        *vis = if alpha < 0.02 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if let Some(handle) = mats.handles.get(puff.index as usize) {
            if let Some(mut mat) = materials.get_mut(handle) {
                mat.color = Color::srgba(0.90, 0.93, 0.97, alpha);
            }
        }
    }
}

pub fn register_cloud_sheets(app: &mut App) {
    app.add_systems(
        Update,
        (
            attach_cloud_sheets.after(AtmospherePipelineSet::VisualExtract),
            update_cloud_sheets
                .after(crate::systems::weather::publish_camera_local_weather_sample)
                .after(ExtractedCameraMetricsSet::Sync),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rnd04_cloud_layer_z_is_strictly_increasing() {
        assert!(
            CLOUD_SHEET_LAYER_Z[0] < CLOUD_SHEET_LAYER_Z[1]
                && CLOUD_SHEET_LAYER_Z[1] < CLOUD_SHEET_LAYER_Z[2],
            "cloud sheet Z must be back→front sorted (RND-04 alpha layering): {:?}",
            CLOUD_SHEET_LAYER_Z
        );
        assert_eq!(LAYER_Z, CLOUD_SHEET_LAYER_Z);
    }

    #[test]
    fn rnd04_cloud_puff_alpha_fades_with_layer_depth() {
        let cover = cloud_cover_alpha(0.8, false);
        assert!(cover > 0.0);
        let a0 = cloud_puff_alpha(cover, 0, 1.0, 0.0, 0.0);
        let a2 = cloud_puff_alpha(cover, 2, 1.0, 0.0, 0.0);
        assert!(
            a0 > a2,
            "lower cloud layers keep higher alpha (depth fade): a0={a0} a2={a2}"
        );
        assert!(a0 <= 0.42 && a2 <= 0.42);
    }
}
