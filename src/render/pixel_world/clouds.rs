//! PixelWorld3D cloud deck — product weather in the voxel view.
//!
//! The top-down cloud puffs ([`crate::render::weather_vfx::cloud_sheets`]) are `Mesh2d` sprites
//! parented to the 2D `MainWorldCamera` on the map RTT layers. The voxel camera only draws
//! [`super::PIXEL_WORLD_RENDER_LAYER`], so with cloud cover set the sprites were live but never
//! seen. This layer draws the same deck (same cover / alpha functions, same wind source,
//! [`CameraLocalWeatherSample`]) as soft horizontal sheets between the camera and the ground.
//!
//! Read only: cover and wind come from [`CameraLocalWeatherSample`] (AtmosphereFrame-derived,
//! W6/X4); nothing here touches `massing_columns`, `FootprintMatrix`, fire, or volumetrics
//! (LGT-04 skies/clouds presentation). Puff placement is a pure function of index + wind
//! integration (no RNG), field-relative so it follows the camera.

use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::prelude::*;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::terrain::MeshBuf;
use super::PIXEL_WORLD_RENDER_LAYER;
use crate::gui::WorldPresentationMode;
use crate::render::extraction::extracted_camera_metrics::ExtractedCameraMetrics;
use crate::render::weather_vfx::cloud_sheets::{cloud_cover_alpha, cloud_puff_alpha, hash01, soft_puff_image};
use crate::systems::weather::{CameraLocalWeatherSample, WeatherVisualSettings, WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA};

const PUFF_COUNT: usize = 36;
/// Reference field of the top-down sprites (puff sizes are authored against it).
const REF_HALF_W: f32 = 240.0;
const REF_HALF_H: f32 = 150.0;
/// Deck heights as a fraction of the camera's height above the focus (below the eye).
const LAYER_HEIGHT: [f32; 3] = [0.35, 0.50, 0.65];
/// Field fractions per second per (m/s) of wind.
const WIND_FIELD_RATE: f32 = 0.01;

#[derive(Component)]
pub struct PixelWorldCloudLayer;

pub(crate) struct CloudLayerState {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
    /// Puff centres in field units (`-1..1` on each axis), advected by wind.
    offsets: Vec<Vec2>,
    mesh_upload_key: u64,
}

impl Default for CloudLayerState {
    fn default() -> Self {
        let offsets = (0..PUFF_COUNT as u32)
            .map(|i| Vec2::new(hash01(i, 5) * 2.0 - 1.0, hash01(i, 6) * 2.0 - 1.0))
            .collect();
        Self {
            material: None,
            mesh: None,
            entity: None,
            offsets,
            mesh_upload_key: 0,
        }
    }
}

#[inline]
fn hash_layer_key(mut h: u64, v: u64) -> u64 {
    h ^= v.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h.rotate_left(17)
}

fn cloud_mesh_upload_key(
    local: &CameraLocalWeatherSample,
    cam: &PixelWorldCameraFrame,
    input: &CloudDeckInput,
    t_quant: u32,
) -> u64 {
    let mut h = local.frame_revision;
    h = hash_layer_key(h, input.cloud.to_bits() as u64);
    h = hash_layer_key(h, input.wind_mps.x.to_bits() as u64);
    h = hash_layer_key(h, input.wind_mps.y.to_bits() as u64);
    h = hash_layer_key(h, u64::from(input.tactical as u8));
    h = hash_layer_key(h, u64::from(t_quant));
    h = hash_layer_key(h, cam.ground_min.x.to_bits() as u64);
    h = hash_layer_key(h, cam.ground_max.x.to_bits() as u64);
    h
}

/// Inputs for one frame of the deck (all read from existing resources).
pub(crate) struct CloudDeckInput {
    pub cloud: f32,
    pub wind_mps: Vec2,
    pub gust_mps: f32,
    pub tactical: bool,
    pub dt: f32,
    pub t: f32,
}

/// Advect puff offsets for one simulation step (wind + shear).
pub(crate) fn advect_cloud_puff_offsets(offsets: &mut [Vec2], input: &CloudDeckInput) {
    let wind_speed = input.wind_mps.length().max(0.35);
    let wind_dir = if input.wind_mps.length_squared() > 0.01 {
        input.wind_mps.normalize()
    } else {
        Vec2::new(1.0, 0.15).normalize()
    };
    let perp = Vec2::new(-wind_dir.y, wind_dir.x);
    let gust_boost =
        1.0 + ((input.gust_mps.max(wind_speed) / wind_speed.max(1.0)) - 1.0).clamp(0.0, 1.2) * 0.45;
    for (i, off) in offsets.iter_mut().enumerate() {
        let iu = i as u32;
        let layer = (i % 3) as u8;
        let layer_f = f32::from(layer);
        let phase = hash01(iu, 1) * std::f32::consts::TAU;
        let speed_mul = 0.55 + hash01(iu, 2) * 0.9;
        let layer_speed = 0.65 + layer_f * 0.35;
        let advect = wind_dir * (wind_speed * WIND_FIELD_RATE * layer_speed * speed_mul * gust_boost);
        let shear = perp * ((input.t * (0.07 + phase * 0.01) + phase).sin() * 0.004 * (1.0 + layer_f * 0.3) * speed_mul);
        *off += (advect + shear) * input.dt;
        off.x = (off.x + 1.0).rem_euclid(2.0) - 1.0;
        off.y = (off.y + 1.0).rem_euclid(2.0) - 1.0;
    }
}

/// Build the sheet mesh from current offsets. Empty when the sky is clear.
pub(crate) fn build_cloud_mesh_from_offsets(
    offsets: &[Vec2],
    input: &CloudDeckInput,
    cam: &PixelWorldCameraFrame,
) -> MeshBuf {
    let mut buf = MeshBuf::default();
    let cover = cloud_cover_alpha(input.cloud.clamp(0.0, 1.0), input.tactical);
    if cover < 0.01 {
        return buf;
    }
    let centre = (cam.ground_min + cam.ground_max) * 0.5;
    // Margin: the decks sit under the eye, so they project a little toward the camera.
    let half = ((cam.ground_max - cam.ground_min) * 0.5 * 1.3).max(Vec2::splat(8.0));
    let k = Vec2::new(half.x / REF_HALF_W, half.y / REF_HALF_H);
    let height = (cam.eye.y - cam.focus_height).max(6.0);
    let wind_speed = input.wind_mps.length().max(0.35);
    let wind_dir = if input.wind_mps.length_squared() > 0.01 { input.wind_mps.normalize() } else { Vec2::new(1.0, 0.15).normalize() };
    let yaw = wind_dir.y.atan2(wind_dir.x) * 0.35;
    let (sy, cy) = yaw.sin_cos();
    let stretch = 1.0 + 0.08 * (wind_speed / 12.0).clamp(0.0, 1.0);
    for (i, off) in offsets.iter().enumerate() {
        let iu = i as u32;
        let layer = (i % 3) as u8;
        let phase = hash01(iu, 1) * std::f32::consts::TAU;
        let speed_mul = 0.55 + hash01(iu, 2) * 0.9;
        let major = 28.0 + hash01(iu, 3) * 38.0 + if i % 7 == 0 { 18.0 } else { 0.0 };
        let aspect = 0.55 + hash01(iu, 4) * 0.7;

        let alpha = cloud_puff_alpha(cover, layer, speed_mul, phase, input.t);
        if alpha < 0.02 {
            continue;
        }
        let breathe = 1.0 + 0.06 * (input.t * (0.15 + hash01(iu, 9) * 0.1) + phase).sin() + 0.04 * input.cloud;
        let size = Vec2::new(major * breathe * stretch, major * aspect * breathe * (2.0 - stretch)) * k * 0.5;
        let c = centre + *off * half;
        let ex = Vec2::new(cy, sy) * size.x;
        let ey = Vec2::new(-sy, cy) * size.y;
        let y = cam.focus_height + height * LAYER_HEIGHT[layer as usize];
        let at = |p: Vec2| Vec3::new(p.x, y, -p.y);
        let tint = [0.90, 0.93, 0.97, alpha];
        buf.quad(
            [at(c - ex - ey), at(c + ex - ey), at(c + ex + ey), at(c - ex + ey)],
            Vec3::Y,
            [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)],
            tint,
        );
    }
    buf
}

/// Advect the puffs and build the sheet mesh. Empty when the sky is clear.
pub(crate) fn build_cloud_mesh(offsets: &mut [Vec2], input: &CloudDeckInput, cam: &PixelWorldCameraFrame) -> MeshBuf {
    advect_cloud_puff_offsets(offsets, input);
    build_cloud_mesh_from_offsets(offsets, input, cam)
}

/// Draw the cloud deck in the voxel view; hidden otherwise (top-down keeps its sprites).
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_clouds(
    mut commands: Commands,
    time: Res<Time>,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    local: Res<CameraLocalWeatherSample>,
    metrics: Res<ExtractedCameraMetrics>,
    settings: Option<Res<WeatherVisualSettings>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut state: Local<CloudLayerState>,
) {
    let enabled = settings.as_deref().is_none_or(|s| s.enabled);
    let on = enabled && local.ready && mode.as_deref().is_some_and(|m| m.is_pixel_world()) && cam.active && field.is_ready();
    let hide = |commands: &mut Commands, e: Option<Entity>| {
        if let Some(e) = e {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
    };
    if !on {
        hide(&mut commands, state.entity);
        return;
    }
    let t_quant = (time.elapsed_secs() * 20.0) as u32;
    let input = CloudDeckInput {
        cloud: local.sample.cloud,
        wind_mps: local.sample.wind_mps,
        gust_mps: local.sample.gust_mps,
        tactical: metrics.zoom_alpha > WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA,
        dt: time.delta_secs().clamp(0.0, 0.05),
        t: t_quant as f32 / 20.0,
    };
    advect_cloud_puff_offsets(&mut state.offsets, &input);
    let upload_key = cloud_mesh_upload_key(&local, &cam, &input, t_quant);
    if upload_key == state.mesh_upload_key && state.mesh.is_some() {
        if let Some(e) = state.entity {
            commands.entity(e).try_insert(Visibility::Inherited);
        }
        return;
    }
    let buf = build_cloud_mesh_from_offsets(&state.offsets, &input, &cam);
    let drawable = !buf.is_empty() && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    if drawable {
        state.mesh_upload_key = upload_key;
    } else {
        state.mesh_upload_key = 0;
    }
    if !drawable {
        hide(&mut commands, state.entity);
        return;
    }
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(images.add(soft_puff_image())),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                fog_enabled: false,
                ..default()
            })
        })
        .clone();
    match state.entity {
        Some(e) => {
            commands.entity(e).try_insert((Mesh3d(mesh), Visibility::Inherited));
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((
                        PixelWorldCloudLayer,
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                        // Rebuilt around the camera every frame; the stored bounds would go stale.
                        NoFrustumCulling,
                        RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER),
                        Pickable::IGNORE,
                    ))
                    .id(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> PixelWorldCameraFrame {
        PixelWorldCameraFrame {
            active: true,
            eye: Vec3::new(50.0, 40.0, 70.0),
            focus: Vec2::new(50.0, 50.0),
            focus_height: 2.0,
            ground_min: Vec2::new(30.0, 30.0),
            ground_max: Vec2::new(80.0, 90.0),
            ..Default::default()
        }
    }

    fn input(cloud: f32) -> CloudDeckInput {
        CloudDeckInput { cloud, wind_mps: Vec2::new(6.0, 1.0), gust_mps: 8.0, tactical: false, dt: 0.016, t: 3.0 }
    }

    fn offsets() -> Vec<Vec2> {
        CloudLayerState::default().offsets
    }

    #[test]
    fn clear_sky_draws_no_clouds() {
        assert!(build_cloud_mesh(&mut offsets(), &input(0.02), &cam()).is_empty());
    }

    #[test]
    fn overcast_draws_sheets_under_the_eye_over_the_footprint() {
        let c = cam();
        let buf = build_cloud_mesh(&mut offsets(), &input(0.9), &c);
        assert!(!buf.is_empty());
        assert_eq!(buf.positions.len() % 4, 0);
        assert!(buf.positions.iter().all(|p| p[1] > c.focus_height && p[1] < c.eye.y));
        assert!(buf.colors.iter().all(|col| col[3] >= 0.02 && col[3] <= 0.42));
    }

    #[test]
    fn wind_advects_and_wraps_in_field_units() {
        let mut o = offsets();
        let before = o.clone();
        let mut inp = input(0.9);
        inp.dt = 0.05;
        for _ in 0..400 {
            build_cloud_mesh(&mut o, &inp, &cam());
        }
        assert_ne!(before, o);
        assert!(o.iter().all(|p| p.x.abs() <= 1.0 && p.y.abs() <= 1.0));
    }
}
