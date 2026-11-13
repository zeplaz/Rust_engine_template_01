//! Voxel terrain + water meshes for PixelWorld3D.
//!
//! One opaque ground mesh and one translucent water mesh per [`TERRAIN_CHUNK`] block of cells,
//! built from [`PixelWorldTerrainField`]. Column tops and walls are run-merged along a row (equal
//! heights share one quad). The ground is textured with the terrain raster the top-down view draws
//! ([`TileWorldFallbackState::image`], one texel per cell, nearest sampling), so terrain family,
//! roads, fire tint and site stamps read the same in both views — no second terrain colouring.
//! Rebuilt only when the field revision changes; the material is re-touched when the raster image
//! changes so the GPU bind group follows the repaint.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::field::PixelWorldTerrainField;
use super::PIXEL_WORLD_RENDER_LAYER;
use crate::render::TileWorldFallbackState;
use crate::terrain::generation::WaterClass;

/// Cells per terrain chunk edge.
pub const TERRAIN_CHUNK: u32 = 64;

/// Wall shade (vertex colour) so column sides read against tops before lighting.
const WALL_SHADE: f32 = 0.82;

#[derive(Component)]
pub struct PixelWorldTerrainChunk;

/// Vertex buffers for one mesh (positions / normals / uvs / colours).
#[derive(Default)]
pub(crate) struct MeshBuf {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshBuf {
    /// Quad from four corners in perimeter order; winding is fixed so the face points along `n`.
    pub fn quad(&mut self, p: [Vec3; 4], n: Vec3, uv: [Vec2; 4], c: [f32; 4]) {
        let base = self.positions.len() as u32;
        let face = (p[1] - p[0]).cross(p[2] - p[0]);
        let order: [usize; 4] = if face.dot(n) >= 0.0 { [0, 1, 2, 3] } else { [0, 3, 2, 1] };
        for k in order {
            self.positions.push(p[k].to_array());
            self.normals.push(n.to_array());
            self.uvs.push(uv[k].to_array());
            self.colors.push(c);
        }
        self.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Axis-aligned box without a bottom face, `lo`..`hi` corners, top tinted `top`, sides `side`.
    pub fn open_box(&mut self, lo: Vec3, hi: Vec3, top: [f32; 4], side: [f32; 4]) {
        let uv = [Vec2::ZERO; 4];
        let (x0, y0, z0, x1, y1, z1) = (lo.x, lo.y, lo.z, hi.x, hi.y, hi.z);
        self.quad(
            [Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1)],
            Vec3::Y,
            uv,
            top,
        );
        self.quad(
            [Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1)],
            Vec3::Z,
            uv,
            side,
        );
        self.quad(
            [Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0)],
            Vec3::NEG_Z,
            uv,
            side,
        );
        self.quad(
            [Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0)],
            Vec3::X,
            uv,
            side,
        );
        self.quad(
            [Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0)],
            Vec3::NEG_X,
            uv,
            side,
        );
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Tactical cell `(x, y)` corner → 3D (`+Y` up, tile row runs along `-Z`).
#[inline]
fn p3(x: f32, y: f32, h: f32) -> Vec3 {
    Vec3::new(x, h, -y)
}

/// Texture coordinate of a tactical point on the world raster (row 0 = texture top).
#[inline]
fn uv(x: f32, y: f32, w: f32, h: f32) -> Vec2 {
    Vec2::new(x / w, y / h)
}

/// Ground under the world edge: walls close down to this.
fn skirt_floor(field: &PixelWorldTerrainField) -> f32 {
    field.min_h - 3.0
}

/// Ground + water meshes for one chunk of cells.
pub(crate) fn build_terrain_chunk(field: &PixelWorldTerrainField, cx: u32, cy: u32) -> (MeshBuf, MeshBuf) {
    let (w, h) = (field.width as i32, field.height as i32);
    let (fw, fh) = (field.width as f32, field.height as f32);
    let x_lo = (cx * TERRAIN_CHUNK) as i32;
    let y_lo = (cy * TERRAIN_CHUNK) as i32;
    let x_hi = (x_lo + TERRAIN_CHUNK as i32).min(w);
    let y_hi = (y_lo + TERRAIN_CHUNK as i32).min(h);
    let floor = skirt_floor(field);
    let ground_at = |x: i32, y: i32| -> f32 {
        field.index(x, y).map_or(floor, |i| field.ground[i])
    };
    let mut g = MeshBuf::default();
    let mut wtr = MeshBuf::default();
    let white = [1.0, 1.0, 1.0, 1.0];
    let wall = [WALL_SHADE, WALL_SHADE, WALL_SHADE, 1.0];

    // Ground tops, run-merged along x.
    for y in y_lo..y_hi {
        let mut x = x_lo;
        while x < x_hi {
            let top = ground_at(x, y);
            let mut x_end = x + 1;
            while x_end < x_hi && ground_at(x_end, y) == top {
                x_end += 1;
            }
            let (x0, x1, y0, y1) = (x as f32, x_end as f32, y as f32, (y + 1) as f32);
            g.quad(
                [p3(x0, y0, top), p3(x1, y0, top), p3(x1, y1, top), p3(x0, y1, top)],
                Vec3::Y,
                [uv(x0, y0, fw, fh), uv(x1, y0, fw, fh), uv(x1, y1, fw, fh), uv(x0, y1, fw, fh)],
                white,
            );
            x = x_end;
        }
    }

    // Walls facing south (-row, +Z) and north (+row, -Z), run-merged along x.
    for (dy, n) in [(-1i32, Vec3::Z), (1i32, Vec3::NEG_Z)] {
        for y in y_lo..y_hi {
            let edge_y = if dy < 0 { y as f32 } else { (y + 1) as f32 };
            let v_mid = (y as f32 + 0.5) / fh;
            let mut x = x_lo;
            while x < x_hi {
                let top = ground_at(x, y);
                let below = ground_at(x, y + dy);
                if below >= top {
                    x += 1;
                    continue;
                }
                let mut x_end = x + 1;
                while x_end < x_hi && ground_at(x_end, y) == top && ground_at(x_end, y + dy) == below {
                    x_end += 1;
                }
                let (x0, x1) = (x as f32, x_end as f32);
                let (u0, u1) = (x0 / fw, x1 / fw);
                g.quad(
                    [p3(x0, edge_y, below), p3(x1, edge_y, below), p3(x1, edge_y, top), p3(x0, edge_y, top)],
                    n,
                    [Vec2::new(u0, v_mid), Vec2::new(u1, v_mid), Vec2::new(u1, v_mid), Vec2::new(u0, v_mid)],
                    wall,
                );
                x = x_end;
            }
        }
    }
    // Walls facing west (-X) and east (+X), run-merged along y.
    for (dx, n) in [(-1i32, Vec3::NEG_X), (1i32, Vec3::X)] {
        for x in x_lo..x_hi {
            let edge_x = if dx < 0 { x as f32 } else { (x + 1) as f32 };
            let u_mid = (x as f32 + 0.5) / fw;
            let mut y = y_lo;
            while y < y_hi {
                let top = ground_at(x, y);
                let below = ground_at(x + dx, y);
                if below >= top {
                    y += 1;
                    continue;
                }
                let mut y_end = y + 1;
                while y_end < y_hi && ground_at(x, y_end) == top && ground_at(x + dx, y_end) == below {
                    y_end += 1;
                }
                let (y0, y1) = (y as f32, y_end as f32);
                let (v0, v1) = (y0 / fh, y1 / fh);
                g.quad(
                    [p3(edge_x, y0, below), p3(edge_x, y1, below), p3(edge_x, y1, top), p3(edge_x, y0, top)],
                    n,
                    [Vec2::new(u_mid, v0), Vec2::new(u_mid, v1), Vec2::new(u_mid, v1), Vec2::new(u_mid, v0)],
                    wall,
                );
                y = y_end;
            }
        }
    }

    // Water: flat surfaces per cell run, plus falls where a river steps down to lower water.
    let water_at = |x: i32, y: i32| -> Option<(f32, WaterClass)> {
        let i = field.index(x, y)?;
        let s = field.water[i];
        s.is_finite().then(|| (s, field.class[i]))
    };
    for y in y_lo..y_hi {
        let mut x = x_lo;
        while x < x_hi {
            let Some((s, class)) = water_at(x, y) else {
                x += 1;
                continue;
            };
            let mut x_end = x + 1;
            while x_end < x_hi && water_at(x_end, y).is_some_and(|(s2, c2)| s2 == s && c2 == class) {
                x_end += 1;
            }
            let depth = (s - ground_at(x, y)).max(0.0);
            let c = water_color(class, depth);
            let (x0, x1, y0, y1) = (x as f32, x_end as f32, y as f32, (y + 1) as f32);
            wtr.quad(
                [p3(x0, y0, s), p3(x1, y0, s), p3(x1, y1, s), p3(x0, y1, s)],
                Vec3::Y,
                [Vec2::ZERO; 4],
                c,
            );
            x = x_end;
        }
    }
    for y in y_lo..y_hi {
        for x in x_lo..x_hi {
            let Some((s, _)) = water_at(x, y) else {
                continue;
            };
            for (dx, dy, n) in [(1, 0, Vec3::X), (-1, 0, Vec3::NEG_X), (0, 1, Vec3::NEG_Z), (0, -1, Vec3::Z)] {
                let lower = match water_at(x + dx, y + dy) {
                    Some((s2, _)) if s2 < s - 0.05 => s2,
                    // World edge: close the sea down to the skirt.
                    None if field.index(x + dx, y + dy).is_none() => floor,
                    _ => continue,
                };
                let (ex0, ey0, ex1, ey1) = match (dx, dy) {
                    (1, 0) => ((x + 1) as f32, y as f32, (x + 1) as f32, (y + 1) as f32),
                    (-1, 0) => (x as f32, y as f32, x as f32, (y + 1) as f32),
                    (0, 1) => (x as f32, (y + 1) as f32, (x + 1) as f32, (y + 1) as f32),
                    _ => (x as f32, y as f32, (x + 1) as f32, y as f32),
                };
                let foam = [0.80, 0.90, 0.95, 0.85];
                wtr.quad(
                    [p3(ex0, ey0, lower), p3(ex1, ey1, lower), p3(ex1, ey1, s), p3(ex0, ey0, s)],
                    n,
                    [Vec2::ZERO; 4],
                    foam,
                );
            }
        }
    }
    (g, wtr)
}

/// Water tint: deeper and standing water darker, rivers lighter and greener.
fn water_color(class: WaterClass, depth: f32) -> [f32; 4] {
    let d = (depth / 2.0).clamp(0.0, 1.0);
    let (shallow, deep) = match class {
        WaterClass::River => ([0.22, 0.52, 0.66], [0.14, 0.40, 0.58]),
        WaterClass::Lake => ([0.18, 0.46, 0.66], [0.08, 0.28, 0.52]),
        _ => ([0.14, 0.40, 0.64], [0.04, 0.18, 0.42]),
    };
    let mix = |a: f32, b: f32| a + (b - a) * d;
    [mix(shallow[0], deep[0]), mix(shallow[1], deep[1]), mix(shallow[2], deep[2]), 0.86]
}

#[derive(Default)]
pub(crate) struct TerrainLayerState {
    built_revision: Option<u64>,
    raster: Handle<Image>,
    ground_material: Option<Handle<StandardMaterial>>,
    water_material: Option<Handle<StandardMaterial>>,
    pending: Vec<(u32, u32)>,
}

/// Chunks rebuilt per frame after a field change (spreads the first build over a few frames).
const CHUNKS_PER_FRAME: usize = 12;

/// Keep terrain chunk entities in step with the field and the raster image.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_terrain(
    mut commands: Commands,
    mode: Option<Res<crate::gui::WorldPresentationMode>>,
    field: Res<super::field::PixelWorldTerrainField>,
    raster: Option<Res<TileWorldFallbackState>>,
    mut image_events: MessageReader<AssetEvent<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks: Query<Entity, With<PixelWorldTerrainChunk>>,
    mut state: Local<TerrainLayerState>,
) {
    let raster_image = raster.as_deref().map(|r| r.image.clone()).unwrap_or_default();
    let raster_touched = image_events
        .read()
        .any(|e| matches!(e, AssetEvent::Modified { id } if *id == raster_image.id()));
    if !mode.is_some_and(|m| m.is_pixel_world()) || !field.is_ready() {
        return;
    }
    let ground_mat = state
        .ground_material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.95,
                reflectance: 0.08,
                ..default()
            })
        })
        .clone();
    let water_mat = state
        .water_material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.12,
                reflectance: 0.6,
                ..default()
            })
        })
        .clone();
    // Re-point / re-touch the ground material when the raster handle or its pixels change, so the
    // render world rebuilds the bind group against the repainted texture.
    if state.raster != raster_image || raster_touched {
        state.raster = raster_image.clone();
        if let Some(mut m) = materials.get_mut(&ground_mat) {
            m.base_color_texture = (raster_image != Handle::default()).then(|| raster_image.clone());
        }
    }

    if state.built_revision != Some(field.revision) {
        state.built_revision = Some(field.revision);
        for e in &chunks {
            commands.entity(e).despawn();
        }
        state.pending.clear();
        let nx = field.width.div_ceil(TERRAIN_CHUNK);
        let ny = field.height.div_ceil(TERRAIN_CHUNK);
        for cy in 0..ny {
            for cx in 0..nx {
                state.pending.push((cx, cy));
            }
        }
    }
    let layer = RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER);
    for _ in 0..CHUNKS_PER_FRAME {
        let Some((cx, cy)) = state.pending.pop() else {
            break;
        };
        let (ground, water) = build_terrain_chunk(field.as_ref(), cx, cy);
        if !ground.is_empty() {
            commands.spawn((
                PixelWorldTerrainChunk,
                Mesh3d(meshes.add(ground.into_mesh())),
                MeshMaterial3d(ground_mat.clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
                layer.clone(),
                Pickable::IGNORE,
            ));
        }
        if !water.is_empty() {
            commands.spawn((
                PixelWorldTerrainChunk,
                Mesh3d(meshes.add(water.into_mesh())),
                MeshMaterial3d(water_mat.clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
                layer.clone(),
                Pickable::IGNORE,
            ));
        }
    }
}
