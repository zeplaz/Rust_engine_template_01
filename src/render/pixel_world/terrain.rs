//! Voxel **ground** meshes for PixelWorld3D (water is GPU-only — see `water_gpu`).
//!
//! One opaque ground mesh per [`TERRAIN_CHUNK`] block of cells from [`PixelWorldTerrainField`].
//! Column tops and walls are **greedy run-merged** (GPU-ARRAY land path). Textured with the
//! same world bake / raster Image the top-down view uses (bake samples Texture2DArray materials).
//! Water surfaces are **not** emitted here
//! ([`plan_final_gpu_presentation_v1.md`](../../../dev/plan_final_gpu_presentation_v1.md)).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::field::PixelWorldTerrainField;
use super::PIXEL_WORLD_RENDER_LAYER;
use crate::render::TileWorldFallbackState;

/// Cells per terrain chunk edge.
pub const TERRAIN_CHUNK: u32 = 64;

/// Wall shade (vertex colour) so column sides read against tops before lighting.
const WALL_SHADE: f32 = 0.82;

#[derive(Component)]
pub struct PixelWorldTerrainChunk;

/// Chunk grid index — DESTROY-T2b priority remesh despawns by coord.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelWorldTerrainChunkCoord {
    pub cx: u32,
    pub cy: u32,
}

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
    /// Append another buffer (indices re-based).
    pub fn append(&mut self, other: MeshBuf) {
        let base = self.positions.len() as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.uvs.extend(other.uvs);
        self.colors.extend(other.colors);
        self.indices.extend(other.indices.into_iter().map(|i| i + base));
    }

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
    let wtr = MeshBuf::default();
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

    // PLAN-FINAL-GPU-PRESENTATION-v1: water is GPU-only (water_gpu / water_field shaders).
    // Terrain chunks emit ground only — empty water MeshBuf.
    let _ = floor;
    (g, wtr)
}

#[derive(Default)]
pub(crate) struct TerrainLayerState {
    built_revision: Option<u64>,
    /// DW-1: last `field.ground_patch_seq` whose chunks were queued.
    patch_seen: u64,
    raster: Handle<Image>,
    pending: Vec<(u32, u32)>,
}

/// Chunks rebuilt per frame after a field change.
/// 320² DamFlood = 5×5 chunks — keep this ≥ that so a revision never leaves a half-map hole.
const CHUNKS_PER_FRAME: usize = 48;

/// Keep terrain chunk entities in step with the field and the raster image.
/// DESTROY-T2: ground uses [`PixelWorldGroundMaterial`] (bake albedo + destruction mask).
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_pixel_world_terrain(
    mut commands: Commands,
    mode: Option<Res<crate::gui::WorldPresentationMode>>,
    field: Res<super::field::PixelWorldTerrainField>,
    raster: Option<Res<TileWorldFallbackState>>,
    bake_spike: Option<Res<crate::render::TerrainGpuBakeSpikeState>>,
    mask_gpu: Res<super::destruction_mask::DestructionMaskGpu>,
    mut mask_book: ResMut<super::destruction_mask::DestructionMaskBook>,
    mut image_events: MessageReader<AssetEvent<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<super::ground_material::PixelWorldGroundMaterial>>,
    mut ground_handle: ResMut<super::ground_material::PixelWorldGroundMaterialHandle>,
    chunks: Query<(Entity, &PixelWorldTerrainChunkCoord), With<PixelWorldTerrainChunk>>,
    mut state: Local<TerrainLayerState>,
) {
    // Same display Image as flat tactical (bake when ready).
    let raster_image = bake_spike
        .as_ref()
        .filter(|s| crate::render::spike_bake_ready_for_consumers(s))
        .map(|s| s.world_bake_image.clone())
        .filter(|h| *h != Handle::default())
        .or_else(|| raster.as_deref().map(|r| r.image.clone()))
        .unwrap_or_default();
    let raster_touched = image_events
        .read()
        .any(|e| matches!(e, AssetEvent::Modified { id } if *id == raster_image.id()));
    if !mode.is_some_and(|m| m.is_pixel_world()) || !field.is_ready() {
        return;
    }

    let skirt = field.min_h - 3.0;
    let w = mask_book.width.max(field.width).max(1) as f32;
    let h = mask_book.height.max(field.height).max(1) as f32;
    if ground_handle.0 == Handle::default() {
        ground_handle.0 = materials.add(super::ground_material::PixelWorldGroundMaterial {
            base: super::ground_material::ground_base_material(
                (raster_image != Handle::default()).then(|| raster_image.clone()),
            ),
            extension: super::ground_material::PixelWorldGroundExt {
                uniform: super::ground_material::GroundMaskUniform {
                    extent_skirt: Vec4::new(w, h, skirt, 0.0),
                },
                destruction_mask: mask_gpu.image.clone(),
                // Placeholder until the scorch field exists (mask R <= 2/255 stays under the
                // glow threshold); `sync_ground_scorch_texture` swaps in the real field.
                scorch: mask_gpu.image.clone(),
            },
        });
    }
    if let Some(mut mat) = materials.get_mut(&ground_handle.0) {
        mat.extension.uniform.extent_skirt = Vec4::new(w, h, skirt, 0.0);
        if state.raster != raster_image || raster_touched {
            state.raster = raster_image.clone();
            if raster_image != Handle::default() {
                mat.base.base_color_texture = Some(raster_image.clone());
            }
        }
        if mask_gpu.image != Handle::default() {
            mat.extension.destruction_mask = mask_gpu.image.clone();
        }
    }
    let ground_mat = ground_handle.0.clone();

    let nx = field.width.div_ceil(TERRAIN_CHUNK);
    let ny = field.height.div_ceil(TERRAIN_CHUNK);
    // Ground only: water motion must never respawn terrain (PLAN-CLOSE-VOXEL-WATER-v1 V1).
    // Full-ground changes respawn everything; a terrain deformation patches cells and names only
    // the touched chunks (`ground_patches`), so a blast never costs a whole-map remesh (DW-1).
    if state.built_revision != Some(field.ground_full_revision) {
        state.built_revision = Some(field.ground_full_revision);
        state.patch_seen = field.ground_patch_seq;
        // Out-of-grid chunks go now; in-grid ones stay visible until their rebuild swaps in (V2).
        for (e, coord) in &chunks {
            if coord.cx >= nx || coord.cy >= ny {
                commands.entity(e).despawn();
            }
        }
        state.pending.clear();
        for cy in 0..ny {
            for cx in 0..nx {
                state.pending.push((cx, cy));
            }
        }
    }
    if state.patch_seen != field.ground_patch_seq {
        let seen = state.patch_seen;
        for &(_, cx, cy) in field.ground_patches.iter().filter(|p| p.0 > seen) {
            state.pending.retain(|p| *p != (cx, cy));
            state.pending.push((cx, cy));
        }
        state.patch_seen = field.ground_patch_seq;
    }
    // DESTROY-T2b: destroyed-mask chunks to the front of pending (stack: push = next pop).
    if !mask_book.remesh_priority.is_empty() {
        let priority = std::mem::take(&mut mask_book.remesh_priority);
        for &(cx, cy) in &priority {
            state.pending.retain(|p| *p != (cx, cy));
            state.pending.push((cx, cy));
        }
    }
    let layer = RenderLayers::layer(PIXEL_WORLD_RENDER_LAYER);
    for _ in 0..CHUNKS_PER_FRAME {
        let Some((cx, cy)) = state.pending.pop() else {
            break;
        };
        let (ground, _water) = build_terrain_chunk(field.as_ref(), cx, cy);
        // Swap-on-ready: the old chunk leaves in the same command flush the new one arrives.
        for (e, coord) in &chunks {
            if coord.cx == cx && coord.cy == cy {
                commands.entity(e).despawn();
            }
        }
        if !ground.is_empty() {
            commands.spawn((
                PixelWorldTerrainChunk,
                PixelWorldTerrainChunkCoord { cx, cy },
                Mesh3d(meshes.add(ground.into_mesh())),
                MeshMaterial3d(ground_mat.clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
                layer.clone(),
                Pickable::IGNORE,
            ));
        }
    }
}
