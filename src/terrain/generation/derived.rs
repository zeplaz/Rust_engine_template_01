//! Chunk-local derived terrain metrics — [`derived_metric_pipeline_v1.md`](../../../prompts/designer_questions/terrain_world/ontology/derived_metric_pipeline_v1.md).
//!
//! **Chunk borders are storage partitions, not simulation truth:** interior cells use only
//! in-chunk neighbors; **edge** cells sample the aligned **row/column** from adjacent chunks
//! when present (same `size` only in v1).

use std::collections::HashMap;

use bevy::prelude::{Component, IVec2, UVec2};

use super::cell_matrix::ChunkCellMatrix;

/// Per-cell scalars derived from [`ChunkCellMatrix`] + optional neighbor chunks.
///
/// **Stub layers:** `drainage`, `support_capacity`, `concealment`, `erosion_risk`, `flood_basin_id` are
/// zero-filled until their passes exist; only [`ChunkDerivedMetrics::slope_grade`] is computed today.
#[derive(Component, Debug, Clone)]
pub struct ChunkDerivedMetrics {
    pub size: UVec2,
    /// Max absolute elevation delta to cardinal neighbors (0..1 scale, same as elevation).
    /// Border cells use stitched neighbor chunk edges when available.
    pub slope_grade: Vec<f32>,
    /// Hydrology accumulation (stub: zeros).
    pub drainage: Vec<f32>,
    /// Construction interpretation input (stub: zeros).
    pub support_capacity: Vec<f32>,
    /// Recon / concealment heuristic (stub: zeros).
    pub concealment: Vec<f32>,
    /// Erosion / substrate stress (stub: zeros).
    pub erosion_risk: Vec<f32>,
    /// Basin ids, 0 = unset (stub: zeros).
    pub flood_basin_id: Vec<u32>,
}

#[inline]
fn cell_count_u32(size: UVec2) -> usize {
    (size.x as usize).saturating_mul(size.y as usize)
}

fn zero_f32_layer(size: UVec2) -> Vec<f32> {
    vec![0.0f32; cell_count_u32(size)]
}

fn zero_u32_layer(size: UVec2) -> Vec<u32> {
    vec![0u32; cell_count_u32(size)]
}

impl ChunkDerivedMetrics {
    /// Local-only (no neighbor chunks). Prefer [`stitch_chunk_slope_grades`] in multi-chunk worlds.
    pub fn from_chunk_matrix(matrix: &ChunkCellMatrix) -> Self {
        let size = matrix.size;
        Self {
            size,
            slope_grade: compute_slope_grade(matrix),
            drainage: zero_f32_layer(size),
            support_capacity: zero_f32_layer(size),
            concealment: zero_f32_layer(size),
            erosion_risk: zero_f32_layer(size),
            flood_basin_id: zero_u32_layer(size),
        }
    }

    /// After [`Self::slope_grade`] is reassigned (e.g. stitching), resize stub layers to match.
    pub fn sync_stub_layers_to_slope_len(&mut self) {
        let n = self.slope_grade.len();
        self.drainage.resize(n, 0.0);
        self.support_capacity.resize(n, 0.0);
        self.concealment.resize(n, 0.0);
        self.erosion_risk.resize(n, 0.0);
        self.flood_basin_id.resize(n, 0);
    }
}

/// Build `coord` → `(size, elevation clone)` for all loaded chunks, then assign stitched slopes.
pub fn stitch_all_chunk_slope_grades(
    snapshots: &HashMap<IVec2, (UVec2, Vec<f32>)>,
    coord: IVec2,
    matrix: &ChunkCellMatrix,
) -> Vec<f32> {
    stitch_chunk_slope_grades(coord, matrix, snapshots)
}

#[inline]
fn elevat(w: u32, elev: &[f32], x: u32, y: u32) -> f32 {
    elev[(y * w + x) as usize]
}

fn row_as_vec(elev: &[f32], w: u32, y: u32) -> Option<Vec<f32>> {
    let yu = y as usize;
    let wu = w as usize;
    let start = yu * wu;
    let end = start + wu;
    if end > elev.len() {
        return None;
    }
    Some(elev[start..end].to_vec())
}

fn col_as_vec(elev: &[f32], w: u32, h: u32, x: u32) -> Option<Vec<f32>> {
    let xu = x as usize;
    let wu = w as usize;
    if xu >= wu {
        return None;
    }
    let mut v = Vec::with_capacity(h as usize);
    for y in 0..h {
        v.push(elev[(y * w + x) as usize]);
    }
    Some(v)
}

/// Cardinal max |Δz| using neighbor chunk **south row** for north-of-us, **north row** for south, etc.
pub fn stitch_chunk_slope_grades(
    coord: IVec2,
    matrix: &ChunkCellMatrix,
    world: &HashMap<IVec2, (UVec2, Vec<f32>)>,
) -> Vec<f32> {
    let w = matrix.size.x;
    let h = matrix.size.y;
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let w_us = w as usize;
    let h_us = h as usize;
    let expected = w_us * h_us;
    if matrix.elevation.len() != expected {
        return vec![0.0f32; expected];
    }

    let north_row = world.get(&(coord + IVec2::new(0, -1))).and_then(|(sz, e)| {
        if sz.x != w || sz.y != h || e.len() != expected {
            return None;
        }
        row_as_vec(e, w, h - 1)
    });
    let south_row = world.get(&(coord + IVec2::new(0, 1))).and_then(|(sz, e)| {
        if sz.x != w || sz.y != h || e.len() != expected {
            return None;
        }
        row_as_vec(e, w, 0)
    });
    let west_col = world.get(&(coord + IVec2::new(-1, 0))).and_then(|(sz, e)| {
        if sz.x != w || sz.y != h || e.len() != expected {
            return None;
        }
        col_as_vec(e, w, h, w - 1)
    });
    let east_col = world.get(&(coord + IVec2::new(1, 0))).and_then(|(sz, e)| {
        if sz.x != w || sz.y != h || e.len() != expected {
            return None;
        }
        col_as_vec(e, w, h, 0)
    });

    let nr = north_row.as_deref();
    let sr = south_row.as_deref();
    let wc = west_col.as_deref();
    let ec = east_col.as_deref();

    let mut out = vec![0.0f32; expected];
    for y in 0..h {
        for x in 0..w {
            let i = matrix.idx(x, y);
            let z = matrix.elevation[i];

            let z_w = if x > 0 {
                elevat(w, &matrix.elevation, x - 1, y)
            } else if let Some(col) = wc {
                col.get(y as usize).copied().unwrap_or(z)
            } else {
                z
            };

            let z_e = if x + 1 < w {
                elevat(w, &matrix.elevation, x + 1, y)
            } else if let Some(col) = ec {
                col.get(y as usize).copied().unwrap_or(z)
            } else {
                z
            };

            let z_n = if y > 0 {
                elevat(w, &matrix.elevation, x, y - 1)
            } else if let Some(row) = nr {
                row.get(x as usize).copied().unwrap_or(z)
            } else {
                z
            };

            let z_s = if y + 1 < h {
                elevat(w, &matrix.elevation, x, y + 1)
            } else if let Some(row) = sr {
                row.get(x as usize).copied().unwrap_or(z)
            } else {
                z
            };

            let mut max_d = (z - z_w).abs();
            max_d = max_d.max((z - z_e).abs());
            max_d = max_d.max((z - z_n).abs());
            max_d = max_d.max((z - z_s).abs());
            out[i] = max_d;
        }
    }
    out
}

/// Cardinal-neighbor max \|Δelevation\| within chunk only (legacy / single-chunk).
pub fn compute_slope_grade(matrix: &ChunkCellMatrix) -> Vec<f32> {
    let w = matrix.size.x;
    let h = matrix.size.y;
    let n = (w * h) as usize;
    let mut out = vec![0.0f32; n];
    if w == 0 || h == 0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let i = matrix.idx(x, y);
            let z = matrix.elevation[i];
            let mut max_d = 0.0f32;
            for (nx, ny) in [
                (x as i32 + 1, y as i32),
                (x as i32 - 1, y as i32),
                (x as i32, y as i32 + 1),
                (x as i32, y as i32 - 1),
            ] {
                let nz = if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                    matrix.elevation[matrix.idx(nx as u32, ny as u32)]
                } else {
                    z
                };
                max_d = max_d.max((z - nz).abs());
            }
            out[i] = max_d;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::ChunkCellMatrix;

    #[test]
    fn flat_chunk_zero_slope() {
        let size = UVec2::new(8, 8);
        let mut m = ChunkCellMatrix::new(size);
        for v in &mut m.elevation {
            *v = 0.42;
        }
        let d = ChunkDerivedMetrics::from_chunk_matrix(&m);
        assert!(d.slope_grade.iter().all(|&s| s < 1e-6));
    }

    #[test]
    fn ramp_nonzero_slope_interior() {
        let size = UVec2::new(8, 8);
        let mut m = ChunkCellMatrix::new(size);
        for y in 0..size.y {
            for x in 0..size.x {
                let i = m.idx(x, y);
                m.elevation[i] = x as f32 / (size.x.saturating_sub(1).max(1) as f32);
            }
        }
        let d = ChunkDerivedMetrics::from_chunk_matrix(&m);
        let ix = 4;
        let iy = 4;
        let i = m.idx(ix, iy);
        assert!(
            d.slope_grade[i] > 0.01,
            "interior ramp should show positive slope_grade, got {}",
            d.slope_grade[i]
        );
    }

    #[test]
    fn stitch_two_chunks_vertical_edge() {
        let size = UVec2::new(4, 4);
        let mut left = ChunkCellMatrix::new(size);
        for v in &mut left.elevation {
            *v = 0.0;
        }
        let mut right = ChunkCellMatrix::new(size);
        for v in &mut right.elevation {
            *v = 1.0;
        }
        let mut world = HashMap::new();
        world.insert(IVec2::ZERO, (size, left.elevation.clone()));
        world.insert(IVec2::new(1, 0), (size, right.elevation.clone()));
        let slopes = stitch_chunk_slope_grades(IVec2::new(1, 0), &right, &world);
        let i = right.idx(0, 2);
        assert!(
            slopes[i] > 0.9,
            "shared vertical boundary should see full step, got {}",
            slopes[i]
        );
    }

    #[test]
    fn stub_layers_len_matches_slope_after_sync() {
        let size = UVec2::new(3, 2);
        let mut m = ChunkCellMatrix::new(size);
        m.elevation[0] = 1.0;
        let mut d = ChunkDerivedMetrics::from_chunk_matrix(&m);
        let area = (size.x * size.y) as usize;
        assert_eq!(d.slope_grade.len(), area);
        // Simulate stitched slope reassignment with a different length
        d.slope_grade = vec![0.0; 12];
        assert_ne!(d.drainage.len(), d.slope_grade.len());
        d.sync_stub_layers_to_slope_len();
        assert_eq!(d.drainage.len(), 12);
        assert_eq!(d.support_capacity.len(), 12);
        assert_eq!(d.concealment.len(), 12);
        assert_eq!(d.erosion_risk.len(), 12);
        assert_eq!(d.flood_basin_id.len(), 12);
    }
}
