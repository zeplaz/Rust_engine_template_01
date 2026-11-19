//! PixelWorld3D water — GPU field Material (PLAN-FINAL-GPU-PRESENTATION-v1).
//!
//! **GPU-D:** static XZ grid (rebuild only on ROI size/origin change). Vertex shader
//! displaces Y from DF-3 field depth + companion bed texture.
//! [`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md) Upgrade D.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use bevy::shader::ShaderRef;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::PIXEL_WORLD_RENDER_LAYER;
use crate::gui::WorldPresentationMode;
use crate::systems::water::{SurfaceWaterGpuField, SurfaceWaterGpuFieldMeta, DEPTH_SCALE_M};

pub const WATER_FIELD_3D_WGSL: &str = "shaders/water/water_field_3d.wesl";

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct WaterField3dUniform {
    pub time_secs: f32,
    pub zoom_alpha: f32,
    /// depth01 → world Y: DEPTH_SCALE_M / TILE_METRES * PIXEL_WORLD_HEIGHT_SCALE
    pub height_scale: f32,
    pub lift: f32,
    pub origin_extent: Vec4,
    /// xy = bed_min_h, bed_range (world Y); zw unused
    pub bed_range: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterFieldMaterial3d {
    #[uniform(0)]
    pub uniform: WaterField3dUniform,
    #[texture(1)]
    #[sampler(2)]
    pub field: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub bed: Handle<Image>,
    pub alpha_mode: AlphaMode,
}

impl Material for WaterFieldMaterial3d {
    fn vertex_shader() -> ShaderRef {
        WATER_FIELD_3D_WGSL.into()
    }

    fn fragment_shader() -> ShaderRef {
        WATER_FIELD_3D_WGSL.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Component)]
pub struct PixelWorldGpuWater;

#[derive(Default)]
pub(crate) struct GpuWaterState {
    entity: Option<Entity>,
    material: Option<Handle<WaterFieldMaterial3d>>,
    mesh: Option<Handle<Mesh>>,
    bed_image: Option<Handle<Image>>,
    /// Geometry key only (not depth revision).
    geom_key: Option<u64>,
    bed_field_rev: Option<u64>,
    /// `(bed_min, bed_range)` the bed texture was encoded with — the uniform must match the
    /// texture, not the live field range (which moves with water tops). CLOSE-W1.
    bed_encode: (f32, f32),
}

/// Near / default grid density (texels per mesh step). Coarser when zoomed out (RND-16).
const GRID_STEP_NEAR: u32 = 2;
const GRID_STEP_MID: u32 = 4;
const GRID_STEP_FAR: u32 = 8;
const LIFT: f32 = 0.04;

/// RND-16 — distance LOD for the static water grid (dense near, coarse far).
/// Uses tactical-style `px_per_tile` already on [`PixelWorldCameraFrame`].
#[must_use]
pub(crate) fn water_grid_step_for_zoom(px_per_tile: f32) -> u32 {
    if px_per_tile >= 12.0 {
        GRID_STEP_NEAR
    } else if px_per_tile >= 5.0 {
        GRID_STEP_MID
    } else {
        GRID_STEP_FAR
    }
}

pub struct PixelWorldWaterGpuPlugin;

impl Plugin for PixelWorldWaterGpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterFieldMaterial3d>::default());
    }
}

/// Display units for depth01 = 1 (`DEPTH_SCALE_M` metres), same units-per-metre as the ground.
fn height_scale(field: &PixelWorldTerrainField) -> f32 {
    DEPTH_SCALE_M * field.units_per_metre
}

fn make_bed_image(w: u32, h: u32) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("pixel_world_water_bed"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(vec![0u8; 4 * width as usize * height as usize]);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

fn upload_bed_texture(
    images: &mut Assets<Image>,
    handle: &Handle<Image>,
    gpu: &SurfaceWaterGpuField,
    field: &PixelWorldTerrainField,
) -> (f32, f32) {
    let Some(mut img) = images.get_mut(handle) else {
        return (field.min_h, (field.max_h - field.min_h).max(0.001));
    };
    let Some(data) = img.data.as_mut() else {
        return (field.min_h, (field.max_h - field.min_h).max(0.001));
    };
    let tw = gpu.width.max(1);
    let th = gpu.height.max(1);
    let step = gpu.step.max(1);
    let ox = gpu.origin_x;
    let oz = gpu.origin_z;
    let min_h = field.min_h;
    let range = (field.max_h - field.min_h).max(0.001);
    for tz in 0..th {
        for tx in 0..tw {
            let wx = ox + (tx * step) as i32;
            let wz = oz + (tz * step) as i32;
            let bed = field
                .index(wx, wz)
                .map(|i| field.ground[i])
                .unwrap_or(min_h);
            let bed01 = ((bed - min_h) / range).clamp(0.0, 1.0);
            let u = (bed01 * 255.0).round() as u8;
            let o = ((tz * tw + tx) * 4) as usize;
            if o + 3 < data.len() {
                data[o] = u;
                data[o + 1] = u;
                data[o + 2] = u;
                data[o + 3] = 255;
            }
        }
    }
    img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    if let Some(d) = img.data.as_ref() {
        img.data = Some(d.clone());
    }
    (min_h, range)
}

/// Static flat grid in local XZ; Y displaced in VS. Covers full ROI (FS discards dry).
fn build_static_water_grid(gpu: &SurfaceWaterGpuField, grid_step: u32) -> Mesh {
    let tw = gpu.width.max(1);
    let th = gpu.height.max(1);
    let step_tex = gpu.step.max(1);
    let grid_step = grid_step.max(1);
    let ox = gpu.origin_x;
    let oz = gpu.origin_z;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let corner = |tx: u32, tz: u32| -> ([f32; 3], [f32; 2]) {
        let wx = ox + (tx * step_tex) as i32;
        let wz = oz + (tz * step_tex) as i32;
        let u = (tx as f32 + 0.5) / tw as f32;
        let v = (tz as f32 + 0.5) / th as f32;
        ([wx as f32, 0.0, -(wz as f32)], [u, v])
    };

    let mut gy = 0_u32;
    while gy < th {
        let mut gx = 0_u32;
        while gx < tw {
            let gx1 = (gx + grid_step).min(tw.saturating_sub(1).max(gx));
            let gy1 = (gy + grid_step).min(th.saturating_sub(1).max(gy));
            let (p0, u0) = corner(gx, gy);
            let (p1, u1) = corner(gx1, gy);
            let (p2, u2) = corner(gx1, gy1);
            let (p3, u3) = corner(gx, gy1);
            let base = positions.len() as u32;
            for (p, uv) in [(p0, u0), (p1, u1), (p2, u2), (p3, u3)] {
                positions.push(p);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push(uv);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            if gx1 <= gx {
                break;
            }
            gx = gx1;
        }
        let gy1 = (gy + grid_step).min(th.saturating_sub(1).max(gy));
        if gy1 <= gy {
            break;
        }
        gy = gy1;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_gpu_water(
    mut commands: Commands,
    time: Res<Time>,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    gpu: Option<Res<SurfaceWaterGpuField>>,
    meta: Option<Res<SurfaceWaterGpuFieldMeta>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterFieldMaterial3d>>,
    mut state: Local<GpuWaterState>,
) {
    let on = mode.as_deref().is_some_and(|m| m.is_pixel_world())
        && cam.active
        && field.is_ready();
    let Some(gpu) = gpu.filter(|g| g.wired && g.wet_texels > 0 && g.image != Handle::default())
    else {
        hide(&mut commands, &mut state);
        state.geom_key = None;
        return;
    };
    if !on {
        hide(&mut commands, &mut state);
        return;
    }

    let origin_extent = Vec4::new(
        gpu.origin_x as f32,
        gpu.origin_z as f32,
        (gpu.width.max(1) * gpu.step.max(1)) as f32,
        (gpu.height.max(1) * gpu.step.max(1)) as f32,
    );
    let zoom_alpha = (cam.px_per_tile / 16.0).clamp(0.25, 1.0);
    let time_secs = time.elapsed_secs();
    let h_scale = height_scale(field.as_ref());

    // Bed texture — refresh when terrain field revision changes.
    let bed_handle = state
        .bed_image
        .get_or_insert_with(|| images.add(make_bed_image(gpu.width.max(1), gpu.height.max(1))))
        .clone();
    let need_bed_resize = images
        .get(&bed_handle)
        .is_none_or(|img| img.width() != gpu.width.max(1) || img.height() != gpu.height.max(1));
    if need_bed_resize {
        let h = images.add(make_bed_image(gpu.width.max(1), gpu.height.max(1)));
        state.bed_image = Some(h.clone());
        state.bed_field_rev = None;
    }
    let bed_handle = state.bed_image.clone().unwrap_or(bed_handle);
    let (bed_min, bed_range) = if state.bed_field_rev != Some(field.ground_revision) || need_bed_resize {
        let r = upload_bed_texture(images.as_mut(), &bed_handle, gpu.as_ref(), field.as_ref());
        state.bed_field_rev = Some(field.ground_revision);
        state.bed_encode = r;
        r
    } else {
        state.bed_encode
    };

    if let Some(h) = state.material.clone() {
        if let Some(mut m) = materials.get_mut(&h) {
            m.uniform.time_secs = time_secs;
            m.uniform.zoom_alpha = zoom_alpha;
            m.uniform.height_scale = h_scale;
            m.uniform.lift = LIFT;
            m.uniform.origin_extent = origin_extent;
            m.uniform.bed_range = Vec4::new(bed_min, bed_range, 0.0, 0.0);
            m.field = gpu.image.clone();
            m.bed = bed_handle.clone();
        }
    }

    // Geometry key: ROI + RND-16 grid LOD band — depth/flow changes do NOT rebuild mesh (GPU-D.1).
    let grid_step = water_grid_step_for_zoom(cam.px_per_tile);
    let geom_key = ((gpu.width as u64) << 40)
        ^ ((gpu.height as u64) << 20)
        ^ ((gpu.step as u64) << 10)
        ^ ((grid_step as u64) << 4)
        ^ (gpu.origin_x as u64).wrapping_mul(0x9E37)
        ^ (gpu.origin_z as u64).wrapping_mul(0x85EB);
    let _meta_rev = meta.map(|m| m.revision).unwrap_or(gpu.revision);

    if state.geom_key != Some(geom_key) {
        state.geom_key = Some(geom_key);
        let mesh = build_static_water_grid(gpu.as_ref(), grid_step);
        let mesh_handle = if let Some(h) = state.mesh.clone() {
            if let Some(mut m) = meshes.get_mut(&h) {
                *m = mesh;
            }
            h
        } else {
            let h = meshes.add(mesh);
            state.mesh = Some(h.clone());
            h
        };

        let mat = state
            .material
            .get_or_insert_with(|| {
                materials.add(WaterFieldMaterial3d {
                    uniform: WaterField3dUniform {
                        time_secs,
                        zoom_alpha,
                        height_scale: h_scale,
                        lift: LIFT,
                        origin_extent,
                        bed_range: Vec4::new(bed_min, bed_range, 0.0, 0.0),
                    },
                    field: gpu.image.clone(),
                    bed: bed_handle.clone(),
                    alpha_mode: AlphaMode::Blend,
                })
            })
            .clone();
        if let Some(mut m) = materials.get_mut(&mat) {
            m.uniform.time_secs = time_secs;
            m.uniform.zoom_alpha = zoom_alpha;
            m.uniform.height_scale = h_scale;
            m.uniform.lift = LIFT;
            m.uniform.origin_extent = origin_extent;
            m.uniform.bed_range = Vec4::new(bed_min, bed_range, 0.0, 0.0);
            m.field = gpu.image.clone();
            m.bed = bed_handle.clone();
        }

        let layer = RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER);
        match state.entity {
            Some(e) => {
                commands.entity(e).try_insert((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(mat),
                    Visibility::Inherited,
                ));
            }
            None => {
                state.entity = Some(
                    commands
                        .spawn((
                            PixelWorldGpuWater,
                            // Flat y=0 grid displaced in the vertex shader: its mesh Aabb sits
                            // far below the drawn surface, so CPU frustum culling dropped it.
                            NoFrustumCulling,
                            Mesh3d(mesh_handle),
                            MeshMaterial3d(mat),
                            Transform::IDENTITY,
                            Visibility::Inherited,
                            layer,
                            Pickable::IGNORE,
                        ))
                        .id(),
                );
            }
        }
    } else if let Some(e) = state.entity {
        commands.entity(e).try_insert(Visibility::Inherited);
    }
}

fn hide(commands: &mut Commands, state: &mut GpuWaterState) {
    if let Some(e) = state.entity {
        commands.entity(e).try_insert(Visibility::Hidden);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One water look: flat and voxel water shaders carry the identical colour/alpha block.
    #[test]
    fn water_look_ramps_match() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let block = |p: &str| -> String {
            let src = std::fs::read_to_string(root.join(p)).expect(p).replace("
", "
");
            let start = src.find("const WATER_DEEP").expect("ramp start");
            let end = src.find("const WATER_ALPHA_MAX").expect("ramp end");
            src[start..end].to_string()
        };
        assert_eq!(
            block("assets/shaders/water/water_field.wgsl"),
            block("assets/shaders/water/water_field_3d.wesl")
        );
    }

    #[test]
    fn water_field_3d_wesl_has_vertex_displace() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(root.join("assets/shaders/water/water_field_3d.wesl"))
            .expect("water_field_3d.wesl");
        assert!(src.contains("@vertex"));
        assert!(src.contains("fn fragment"));
        assert!(src.contains("field_tex"));
        assert!(src.contains("bed_tex"));
        assert!(src.contains("depth01 * material.height_scale"));
    }

    #[test]
    fn rnd16_water_grid_step_coarsens_when_zoomed_out() {
        assert_eq!(water_grid_step_for_zoom(16.0), GRID_STEP_NEAR);
        assert_eq!(water_grid_step_for_zoom(8.0), GRID_STEP_MID);
        assert_eq!(water_grid_step_for_zoom(2.0), GRID_STEP_FAR);
        assert!(water_grid_step_for_zoom(2.0) > water_grid_step_for_zoom(16.0));
    }
}
