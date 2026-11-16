//! PixelWorld3D weather precipitation (rain / snow streaks) — product weather in the voxel view.
//!
//! The top-down view draws precip with a Core2d vertex-pull pass ([`crate::render::weather_vfx`])
//! that only runs on the 2D `TileDebugRenderHost` camera. In the voxel view that camera is
//! inactive and the [`super::PixelWorldCamera`] (Camera3d) only draws [`super::PIXEL_WORLD_RENDER_LAYER`],
//! so without this layer rain was simulated and extracted but never drawn.
//!
//! **Read only.** Rows come from [`WeatherPrecipFrame`] (sole writer
//! `fill_weather_precip_frame_from_climate`, which samples [`AtmosphereFrame`] per streak and
//! scatters over the voxel ground footprint while this view is active — W6/X4). Nothing here
//! touches `massing_columns`, `FootprintMatrix`, fire seeds, or volumetrics (LGT-04); precip is
//! not construction massing and not fog.
//!
//! Fall animation is a pure function of streak index and view time — no sim state, no RNG.

use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::prelude::*;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::terrain::MeshBuf;
use super::{tile_to_world3, PIXEL_WORLD_RENDER_LAYER};
use crate::gui::WorldPresentationMode;
use crate::render::weather_vfx::WeatherPrecipFrame;
use crate::systems::weather::WeatherVisualSettings;

#[derive(Component)]
pub struct PixelWorldPrecipLayer;

#[derive(Default)]
pub(crate) struct PrecipLayerState {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

/// Stable per-streak phase in `[0, 1)` (SplitMix64 finalizer over the index).
#[inline]
fn streak_unit(i: usize, salt: u64) -> f32 {
    let mut x = (i as u64).wrapping_add(salt).wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / 16_777_216.0
}

/// Camera-facing streak quads for the prepared precip rows, falling from `eye height` to the
/// voxel surface. Sizes scale with the camera's height above the focus so they stay readable at
/// every zoom (the view runs 1.5–64 px per tile).
pub(crate) fn build_precip_mesh(frame: &WeatherPrecipFrame, cam: &PixelWorldCameraFrame, field: &PixelWorldTerrainField, time_secs: f32) -> MeshBuf {
    let mut buf = MeshBuf::default();
    let scale = (cam.eye.y - cam.focus_height).max(6.0);
    let lean = (frame.wind_mps / 20.0).clamp_length_max(0.6);
    let axis = Vec3::new(lean.x, -1.0, -lean.y).normalize();
    let top_y = cam.eye.y.max(cam.focus_height + 6.0);
    for (i, s) in frame.streaks.iter().enumerate() {
        let (x, y) = (s.position_xyz_custom0.x, s.position_xyz_custom0.y);
        let (rain, snow, fog) = (s.position_xyz_custom0.w.max(0.0), s.custom1_custom2_custom3_custom4.x.max(0.0), s.custom1_custom2_custom3_custom4.w);
        let amount = rain + snow;
        if amount <= 0.0 {
            continue;
        }
        let Some(floor) = field.surface_at(Vec2::new(x, y)) else {
            continue;
        };
        let snow_w = snow / amount;
        let column = (top_y - floor).max(1.0);
        let speed = scale * (0.9 - 0.72 * snow_w) * (0.8 + 0.4 * streak_unit(i, 7));
        let t = (streak_unit(i, 1) + time_secs * speed / column).fract();
        let p = tile_to_world3(Vec2::new(x, y), top_y - t * column);
        let to_eye = cam.eye - p;
        let right = to_eye.cross(axis).normalize_or_zero();
        if right == Vec3::ZERO {
            continue;
        }
        let half_len = scale * (0.015 - 0.011 * snow_w);
        let half_w = scale * (0.0010 + 0.0040 * snow_w);
        let a = axis * half_len;
        let r = right * half_w;
        let tint = [0.72 + 0.28 * snow_w, 0.82 + 0.18 * snow_w, 1.0, (0.30 + 0.50 * amount.min(1.0)) * (1.0 - 0.3 * fog.clamp(0.0, 1.0))];
        buf.quad([p - a - r, p - a + r, p + a + r, p + a - r], to_eye.normalize_or_zero(), [Vec2::ZERO; 4], tint);
    }
    buf
}

/// Draw the precip rows in the voxel view; hidden otherwise (top-down keeps its Core2d pass).
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_precip(
    mut commands: Commands,
    time: Res<Time>,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    precip: Option<Res<WeatherPrecipFrame>>,
    settings: Option<Res<WeatherVisualSettings>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<PrecipLayerState>,
) {
    let enabled = settings.as_deref().is_none_or(|s| s.enabled);
    let on = enabled && mode.as_deref().is_some_and(|m| m.is_pixel_world()) && cam.active && field.is_ready();
    let frame = precip.as_deref().filter(|f| on && !f.streaks.is_empty());
    let Some(frame) = frame else {
        if let Some(e) = state.entity {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        return;
    };
    let buf = build_precip_mesh(frame, &cam, &field, time.elapsed_secs());
    let drawable = !buf.is_empty() && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    let material = state
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
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
            commands.entity(e).try_insert((Mesh3d(mesh), vis));
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((
                        PixelWorldPrecipLayer,
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        vis,
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
    use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;
    use bevy::math::Vec4;

    fn flat_field() -> PixelWorldTerrainField {
        let mut f = PixelWorldTerrainField::default();
        f.width = 8;
        f.height = 8;
        f.ground = vec![1.0; 64];
        f.water = vec![f32::NEG_INFINITY; 64];
        f.class = vec![Default::default(); 64];
        f
    }

    fn cam() -> PixelWorldCameraFrame {
        PixelWorldCameraFrame {
            active: true,
            eye: Vec3::new(4.0, 20.0, 12.0),
            focus: Vec2::new(4.0, 4.0),
            focus_height: 1.0,
            ..Default::default()
        }
    }

    fn rain_frame(streaks: Vec<GpuInstancedQuadInstance>) -> WeatherPrecipFrame {
        WeatherPrecipFrame { instance_count: streaks.len(), streaks, ..Default::default() }
    }

    fn row(x: f32, y: f32, rain: f32, snow: f32) -> GpuInstancedQuadInstance {
        GpuInstancedQuadInstance {
            position_xyz_custom0: Vec4::new(x, y, 360.0, rain),
            custom1_custom2_custom3_custom4: Vec4::new(snow, 1.0, 1.0, 0.0),
        }
    }

    #[test]
    fn rain_rows_become_quads_above_the_surface() {
        let frame = rain_frame(vec![row(3.5, 3.5, 0.6, 0.0), row(5.5, 2.5, 0.4, 0.0)]);
        let buf = build_precip_mesh(&frame, &cam(), &flat_field(), 1.25);
        assert_eq!(buf.positions.len(), 8, "one quad per streak");
        assert!(buf.positions.iter().all(|p| p[1] <= 20.0 + 1.0));
        assert!(buf.colors.iter().all(|c| c[2] >= 0.99 && c[3] > 0.0));
    }

    #[test]
    fn dry_and_off_world_rows_draw_nothing() {
        let frame = rain_frame(vec![row(3.5, 3.5, 0.0, 0.0), row(-40.0, 3.5, 0.6, 0.0)]);
        assert!(build_precip_mesh(&frame, &cam(), &flat_field(), 0.0).is_empty());
    }

    #[test]
    fn fall_is_a_pure_function_of_time() {
        let frame = rain_frame(vec![row(3.5, 3.5, 0.6, 0.0)]);
        let a = build_precip_mesh(&frame, &cam(), &flat_field(), 2.0);
        let b = build_precip_mesh(&frame, &cam(), &flat_field(), 2.0);
        let c = build_precip_mesh(&frame, &cam(), &flat_field(), 2.3);
        assert_eq!(a.positions, b.positions);
        assert_ne!(a.positions, c.positions);
    }

    #[test]
    fn snow_is_white_and_slower_than_rain() {
        let snow = build_precip_mesh(&rain_frame(vec![row(3.5, 3.5, 0.0, 0.6)]), &cam(), &flat_field(), 0.0);
        assert!(snow.colors[0][0] > 0.99);
        let drop = |s: f32| {
            let f = rain_frame(vec![row(3.5, 3.5, 1.0 - s, s)]);
            let y = |t: f32| {
                let b = build_precip_mesh(&f, &cam(), &flat_field(), t);
                b.positions.iter().map(|p| p[1]).sum::<f32>() / b.positions.len() as f32
            };
            (y(0.0) - y(0.01)).abs()
        };
        assert!(drop(0.0) > drop(1.0));
    }
}
