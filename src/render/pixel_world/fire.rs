//! PixelWorld3D fire VFX — flames, embers and smoke in the voxel view.
//!
//! The top-down view draws fire particles with a Core2d pass (`gpu_fire_particle_raster`) that
//! only runs on the 2D `TileDebugRenderHost` camera. In the voxel view that camera is inactive and
//! the [`super::PixelWorldCamera`] (Camera3d) only draws [`super::PIXEL_WORLD_RENDER_LAYER`], so the
//! fire rows were animated every frame but never drawn — the only sign of fire was the ground
//! tint blinking. Same situation and same fix shape as [`super::rain`].
//!
//! **Read only.** Rows come from [`WorldFireParticleFrame`] (sole writer: the fire VFX pack).
//! Each animated row carries its ground position and height lift
//! ([`particle_ground_and_height_m`]); this layer stands it on the voxel surface and billboards
//! it toward the camera. Flames / embers / sparks draw additive, smoke draws blended.

use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::prelude::*;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::terrain::MeshBuf;
use super::{tile_to_world3, PIXEL_WORLD_RENDER_LAYER};
use crate::gui::WorldPresentationMode;
use crate::render::fire_vfx::particle_anim::particle_ground_and_height_m;
use crate::render::fire_vfx::{GpuParticleInstance, WorldFireParticleFrame};

#[derive(Component)]
pub struct PixelWorldFireLayer;

#[derive(Default)]
struct FirePass {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

#[derive(Default)]
pub(crate) struct FireLayerState {
    glow: FirePass,
    smoke: FirePass,
}

/// Smoke rows (class 0.75) and atmosphere garnish (class 1.0) blend; the rest glow.
#[inline]
fn is_smoke(row: &GpuParticleInstance) -> bool {
    row.ember_class_radius_smoke.y > 0.5
}

/// Billboard quads for the rows of one pass, standing on the voxel surface.
pub(crate) fn build_fire_mesh(
    rows: &[GpuParticleInstance],
    smoke: bool,
    cam: &PixelWorldCameraFrame,
    field: &PixelWorldTerrainField,
) -> MeshBuf {
    let mut buf = MeshBuf::default();
    // Keep tiny embers readable at any zoom (view runs 1.5–64 px per tile).
    let min_half = (cam.eye.y - cam.focus_height).max(6.0) * 0.0025;
    for row in rows.iter().filter(|r| is_smoke(r) == smoke) {
        let alpha = row.ember_class_radius_smoke.x.clamp(0.0, 1.0);
        if alpha <= 0.01 {
            continue;
        }
        let (ground, height_m) = particle_ground_and_height_m(row);
        let Some(floor) = field.surface_at(ground) else {
            continue;
        };
        let p = tile_to_world3(ground, floor + height_m * field.units_per_metre.max(1e-4));
        let to_eye = cam.eye - p;
        let right = to_eye.cross(Vec3::Y).normalize_or_zero();
        if right == Vec3::ZERO {
            continue;
        }
        let up = right.cross(to_eye).normalize_or_zero();
        let half = row.ember_class_radius_smoke.z.abs().max(min_half);
        let (r, u) = (right * half, up * half * if smoke { 1.0 } else { 1.6 });
        let (heat, tint_id) = if smoke {
            crate::render::fire_vfx::pack::decode_smoke_tint(row.world_xyz_heat.w)
        } else {
            (row.world_xyz_heat.w.clamp(0.0, 1.0), 0)
        };
        let tint = if smoke {
            // Collapse dust takes its colour from the destroyed structure's material.
            let [r, g, b] = crate::construction::SiteMaterial::from_tint_id(tint_id)
                .map_or([0.30, 0.29, 0.28], |m| m.dust_rgb());
            [r, g, b, alpha * 0.55]
        } else {
            // Deep red → orange → yellow-white with heat; additive, so alpha scales the glow.
            [1.0, 0.25 + 0.6 * heat, 0.04 + 0.35 * heat * heat, alpha]
        };
        buf.quad([p - r - u, p + r - u, p + r + u, p - r + u], to_eye.normalize_or_zero(), [Vec2::ZERO; 4], tint);
    }
    buf
}

fn sync_pass(
    commands: &mut Commands,
    pass: &mut FirePass,
    buf: MeshBuf,
    alpha_mode: AlphaMode,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let drawable = !buf.is_empty() && crate::render::upload_layer_mesh(meshes, &mut pass.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = pass.mesh.clone() else {
        return;
    };
    let material = pass
        .material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                fog_enabled: false,
                ..default()
            })
        })
        .clone();
    match pass.entity {
        Some(e) => {
            commands.entity(e).try_insert((Mesh3d(mesh), vis));
        }
        None => {
            pass.entity = Some(
                commands
                    .spawn((
                        PixelWorldFireLayer,
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform::IDENTITY,
                        vis,
                        // Billboards rebuilt around the camera every frame; stored bounds go stale.
                        NoFrustumCulling,
                        RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER),
                        Pickable::IGNORE,
                    ))
                    .id(),
            );
        }
    }
}

fn hide(commands: &mut Commands, pass: &FirePass) {
    if let Some(e) = pass.entity {
        commands.entity(e).try_insert(Visibility::Hidden);
    }
}

/// Draw the fire rows in the voxel view; hidden otherwise (top-down keeps its Core2d pass).
pub(crate) fn sync_pixel_world_fire(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    frame: Option<Res<WorldFireParticleFrame>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<FireLayerState>,
) {
    let on = mode.as_deref().is_some_and(|m| m.is_pixel_world()) && cam.active && field.is_ready();
    let Some(frame) = frame.as_deref().filter(|f| on && !f.instances.is_empty()) else {
        hide(&mut commands, &state.glow);
        hide(&mut commands, &state.smoke);
        return;
    };
    let glow = build_fire_mesh(&frame.instances, false, &cam, &field);
    let smoke = build_fire_mesh(&frame.instances, true, &cam, &field);
    let FireLayerState { glow: g, smoke: s } = &mut *state;
    sync_pass(&mut commands, g, glow, AlphaMode::Add, &mut meshes, &mut materials);
    sync_pass(&mut commands, s, smoke, AlphaMode::Blend, &mut meshes, &mut materials);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec4;

    fn flat_field() -> PixelWorldTerrainField {
        let mut f = PixelWorldTerrainField::default();
        f.width = 8;
        f.height = 8;
        f.ground = vec![1.0; 64];
        f.water = vec![f32::NEG_INFINITY; 64];
        f.class = vec![Default::default(); 64];
        f.units_per_metre = 1.0 / 16.0;
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

    fn row(x: f32, y: f32, class: f32, alpha: f32) -> GpuParticleInstance {
        GpuParticleInstance {
            world_xyz_heat: Vec4::new(x, y, 1.0, 0.8),
            ember_class_radius_smoke: Vec4::new(alpha, class, 0.2, 0.0),
        }
    }

    #[test]
    fn flame_and_smoke_rows_split_into_their_passes() {
        let rows = [row(3.5, 3.5, 0.0, 0.9), row(4.5, 3.5, 0.25, 0.7), row(3.5, 4.5, 0.75, 0.6)];
        let glow = build_fire_mesh(&rows, false, &cam(), &flat_field());
        let smoke = build_fire_mesh(&rows, true, &cam(), &flat_field());
        assert_eq!(glow.positions.len(), 8, "one quad per flame/ember row");
        assert_eq!(smoke.positions.len(), 4, "one quad per smoke row");
        assert!(glow.positions.iter().all(|p| p[1] >= 1.0 - 0.5), "stands on the voxel surface");
    }

    #[test]
    fn faded_and_off_world_rows_draw_nothing() {
        let rows = [row(3.5, 3.5, 0.0, 0.0), row(-40.0, 3.5, 0.0, 0.9)];
        assert!(build_fire_mesh(&rows, false, &cam(), &flat_field()).is_empty());
    }
}
