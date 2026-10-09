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
/// Drainage, support, concealment, flood basins, and erosion are filled from elevation and landform.
/// Erosion uses slope and drainage. Curvature does not write it.
#[derive(Component, Debug, Clone)]
pub struct ChunkDerivedMetrics {
    pub size: UVec2,
    /// Max absolute elevation delta to cardinal neighbors (0..1 scale, same as elevation).
    /// Border cells use stitched neighbor chunk edges when available.
    pub slope_grade: Vec<f32>,
    /// Mean-neighbor elevation minus this cell. Positive is a hollow, negative is a ridge.
    pub profile_curvature: Vec<f32>,
    /// Unit-area flow accumulated toward the steepest downhill neighbor.
    pub drainage: Vec<f32>,
    /// Flat cells hold buildings. Steep cells do not.
    pub support_capacity: Vec<f32>,
    /// Hollow, covered cells hide more than ridges.
    pub concealment: Vec<f32>,
    /// Shear from slope and accumulated drainage. Curvature is not an input.
    pub erosion_risk: Vec<f32>,
    /// `1` on cells at the chunk minimum elevation, otherwise `0`.
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
        let mut derived = Self {
            size,
            slope_grade: compute_slope_grade(matrix),
            profile_curvature: compute_profile_curvature(matrix),
            drainage: zero_f32_layer(size),
            support_capacity: zero_f32_layer(size),
            concealment: zero_f32_layer(size),
            erosion_risk: zero_f32_layer(size),
            flood_basin_id: zero_u32_layer(size),
        };
        fill_landform_layers(&mut derived, matrix);
        derived
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

/// Cardinal slope and profile curvature using neighbor chunk edges.
///
/// Curvature is the mean of the four cardinal elevations minus the cell: positive hollow, negative ridge.
pub fn stitch_chunk_landform(
    coord: IVec2,
    matrix: &ChunkCellMatrix,
    world: &HashMap<IVec2, (UVec2, Vec<f32>)>,
) -> (Vec<f32>, Vec<f32>) {
    let w = matrix.size.x;
    let h = matrix.size.y;
    if w == 0 || h == 0 {
        return (Vec::new(), Vec::new());
    }
    let w_us = w as usize;
    let h_us = h as usize;
    let expected = w_us * h_us;
    if matrix.elevation.len() != expected {
        let z = vec![0.0f32; expected];
        return (z.clone(), z);
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

    let mut slope = vec![0.0f32; expected];
    let mut curve = vec![0.0f32; expected];
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
            slope[i] = max_d;
            curve[i] = (((z_w + z_e + z_n + z_s) * 0.25) - z).clamp(-1.0, 1.0);
        }
    }
    (slope, curve)
}

/// Cardinal max |Δz| using neighbor chunk **south row** for north-of-us, **north row** for south, etc.
pub fn stitch_chunk_slope_grades(
    coord: IVec2,
    matrix: &ChunkCellMatrix,
    world: &HashMap<IVec2, (UVec2, Vec<f32>)>,
) -> Vec<f32> {
    stitch_chunk_landform(coord, matrix, world).0
}

/// Stitched profile curvature. Positive hollow, negative ridge.
pub fn stitch_chunk_profile_curvature(
    coord: IVec2,
    matrix: &ChunkCellMatrix,
    world: &HashMap<IVec2, (UVec2, Vec<f32>)>,
) -> Vec<f32> {
    stitch_chunk_landform(coord, matrix, world).1
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

/// Discrete laplacian of elevation. Missing neighbors copy the cell, so borders stay quiet until stitch.
pub fn compute_profile_curvature(matrix: &ChunkCellMatrix) -> Vec<f32> {
    let w = matrix.size.x;
    let h = matrix.size.y;
    let n = (w * h) as usize;
    let mut out = vec![0.0f32; n];
    if w == 0 || h == 0 || matrix.elevation.len() != n {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let i = matrix.idx(x, y);
            let z = matrix.elevation[i];
            let mut acc = 0.0f32;
            for (nx, ny) in [
                (x as i32 + 1, y as i32),
                (x as i32 - 1, y as i32),
                (x as i32, y as i32 + 1),
                (x as i32, y as i32 - 1),
            ] {
                acc += if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                    matrix.elevation[matrix.idx(nx as u32, ny as u32)]
                } else {
                    z
                };
            }
            out[i] = ((acc * 0.25) - z).clamp(-1.0, 1.0);
        }
    }
    out
}

/// Mean slope and mean profile curvature over the chunk. Flat ground is `(0, 0)`.
pub fn landform_means(matrix: &ChunkCellMatrix) -> (f32, f32) {
    let slope = compute_slope_grade(matrix);
    let curve = compute_profile_curvature(matrix);
    let n = slope.len();
    if n == 0 || curve.len() != n {
        return (0.0, 0.0);
    }
    let inv = 1.0 / n as f32;
    (
        slope.iter().sum::<f32>() * inv,
        curve.iter().sum::<f32>() * inv,
    )
}

/// Per-cell cover. Steep and ridge cells stay thinner. Hollow cells stay denser.
#[must_use]
pub fn cell_canopy_scale(slope: f32, curvature: f32) -> f32 {
    let steep = (slope / 0.2).clamp(0.0, 1.0);
    let hollow = (curvature / 0.04).clamp(-1.0, 1.0);
    (1.0 - steep * 0.5 + hollow * 0.35).clamp(0.35, 1.35)
}

/// Same landform cover, then moisture raises it and heat lowers it.
#[must_use]
pub fn cell_canopy_scale_climate(slope: f32, curvature: f32, moisture: f32, temperature: f32) -> f32 {
    let base = cell_canopy_scale(slope, curvature);
    let wet = (moisture.clamp(0.0, 1.0) - 0.5) * 0.24;
    let hot = (temperature.clamp(0.0, 1.0) - 0.5) * 0.16;
    (base + wet - hot).clamp(0.35, 1.35)
}

#[must_use]
pub fn cell_canopy_scales(matrix: &ChunkCellMatrix) -> Vec<f32> {
    let slope = compute_slope_grade(matrix);
    let curve = compute_profile_curvature(matrix);
    slope
        .iter()
        .zip(curve.iter())
        .map(|(s, c)| cell_canopy_scale(*s, *c))
        .collect()
}

/// Flat ground supports a site. A slope of 0.2 or steeper supports none.
#[must_use]
pub fn support_from_slope(slope: f32) -> f32 {
    (1.0 - (slope / 0.2).clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

/// Concealment follows canopy: hollows hold more cover than ridges.
#[must_use]
pub fn concealment_from_landform(slope: f32, curvature: f32) -> f32 {
    ((cell_canopy_scale(slope, curvature) - 0.35) / 1.0).clamp(0.0, 1.0)
}

/// Each cell starts with one unit of area and gives it to the steepest lower cardinal neighbor.
#[must_use]
pub fn drainage_accumulation(elev: &[f32], w: u32, h: u32) -> Vec<f32> {
    let n = (w as usize).saturating_mul(h as usize);
    if w == 0 || h == 0 || elev.len() != n {
        return vec![0.0; n];
    }
    let mut flow = vec![1.0f32; n];
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        elev[b]
            .partial_cmp(&elev[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    for i in order {
        let x = (i % w as usize) as i32;
        let y = (i / w as usize) as i32;
        let z = elev[i];
        let mut best: Option<(usize, f32)> = None;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let j = ny as usize * w as usize + nx as usize;
            let drop = z - elev[j];
            if drop > 0.0 && best.is_none_or(|(_, best_drop)| drop > best_drop) {
                best = Some((j, drop));
            }
        }
        if let Some((j, _)) = best {
            let give = flow[i];
            flow[j] += give;
        }
    }
    flow
}

/// Local accumulation, plus one unit of inflow where a neighbor chunk is higher across the shared edge.
#[must_use]
pub fn drainage_with_neighbor_inflow(
    coord: IVec2,
    matrix: &ChunkCellMatrix,
    world: &HashMap<IVec2, (UVec2, Vec<f32>)>,
) -> Vec<f32> {
    let w = matrix.size.x;
    let h = matrix.size.y;
    let mut flow = drainage_accumulation(&matrix.elevation, w, h);
    if w == 0 || h == 0 || flow.len() != (w as usize) * (h as usize) {
        return flow;
    }
    let expected = (w as usize) * (h as usize);
    let neighbor = |dir: IVec2| -> Option<&Vec<f32>> {
        world.get(&(coord + dir)).and_then(|(sz, elev)| {
            if sz.x != w || sz.y != h || elev.len() != expected {
                None
            } else {
                Some(elev)
            }
        })
    };
    let sample = |elev: &[f32], x: u32, y: u32| elev[(y * w + x) as usize];
    if let Some(elev) = neighbor(IVec2::new(-1, 0)) {
        for y in 0..h {
            if sample(elev, w - 1, y) > sample(&matrix.elevation, 0, y) {
                flow[matrix.idx(0, y)] += 1.0;
            }
        }
    }
    if let Some(elev) = neighbor(IVec2::new(1, 0)) {
        for y in 0..h {
            if sample(elev, 0, y) > sample(&matrix.elevation, w - 1, y) {
                flow[matrix.idx(w - 1, y)] += 1.0;
            }
        }
    }
    if let Some(elev) = neighbor(IVec2::new(0, -1)) {
        for x in 0..w {
            if sample(elev, x, h - 1) > sample(&matrix.elevation, x, 0) {
                flow[matrix.idx(x, 0)] += 1.0;
            }
        }
    }
    if let Some(elev) = neighbor(IVec2::new(0, 1)) {
        for x in 0..w {
            if sample(elev, x, 0) > sample(&matrix.elevation, x, h - 1) {
                flow[matrix.idx(x, h - 1)] += 1.0;
            }
        }
    }
    flow
}

/// Erosion is slope times drainage. Curvature is not an input.
#[must_use]
pub fn erosion_from_slope_drainage(slope: f32, drainage: f32) -> f32 {
    let steep = (slope / 0.2).clamp(0.0, 1.0);
    let flow = (drainage.max(0.0) / 4.0).clamp(0.0, 1.0);
    (steep * flow).clamp(0.0, 1.0)
}
#[must_use]
pub fn flood_basin_ids(elev: &[f32]) -> Vec<u32> {
    let Some(min_z) = elev.iter().copied().reduce(f32::min) else {
        return Vec::new();
    };
    elev.iter()
        .map(|z| if (*z - min_z).abs() <= 1e-4 { 1 } else { 0 })
        .collect()
}

/// Writes drainage, support, concealment, flood basins, and erosion. Erosion ignores curvature.
pub fn fill_landform_layers(derived: &mut ChunkDerivedMetrics, matrix: &ChunkCellMatrix) {
    let n = derived.slope_grade.len();
    if derived.profile_curvature.len() != n || matrix.elevation.len() != n {
        return;
    }
    derived.drainage = drainage_accumulation(&matrix.elevation, matrix.size.x, matrix.size.y);
    derived.support_capacity = derived
        .slope_grade
        .iter()
        .copied()
        .map(support_from_slope)
        .collect();
    derived.concealment = derived
        .slope_grade
        .iter()
        .zip(derived.profile_curvature.iter())
        .map(|(slope, curve)| concealment_from_landform(*slope, *curve))
        .collect();
    derived.flood_basin_id = flood_basin_ids(&matrix.elevation);
    derived.erosion_risk = derived
        .slope_grade
        .iter()
        .zip(derived.drainage.iter())
        .map(|(slope, drain)| erosion_from_slope_drainage(*slope, *drain))
        .collect();
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
        assert!(d.profile_curvature.iter().all(|&c| c.abs() < 1e-6));
    }

    #[test]
    fn bowl_is_positive_curvature_ridge_is_negative() {
        let size = UVec2::new(5, 5);
        let mut bowl = ChunkCellMatrix::new(size);
        for v in &mut bowl.elevation {
            *v = 0.6;
        }
        let i = bowl.idx(2, 2);
        bowl.elevation[i] = 0.2;
        let hollow = ChunkDerivedMetrics::from_chunk_matrix(&bowl);
        assert!(
            hollow.profile_curvature[i] > 0.05,
            "hollow should be positive, got {}",
            hollow.profile_curvature[i]
        );

        let mut ridge = ChunkCellMatrix::new(size);
        for v in &mut ridge.elevation {
            *v = 0.2;
        }
        ridge.elevation[i] = 0.8;
        let crest = ChunkDerivedMetrics::from_chunk_matrix(&ridge);
        assert!(
            crest.profile_curvature[i] < -0.05,
            "ridge should be negative, got {}",
            crest.profile_curvature[i]
        );
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
        let curve = stitch_chunk_profile_curvature(IVec2::new(1, 0), &right, &world);
        assert!(
            curve[i] < -0.1,
            "high side of a cliff should read as a ridge, got {}",
            curve[i]
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

    #[test]
    fn same_chunk_hollow_holds_more_cover_than_ridge() {
        let size = UVec2::new(7, 7);
        let mut m = ChunkCellMatrix::new(size);
        for v in &mut m.elevation {
            *v = 0.4;
        }
        let hollow = m.idx(1, 1);
        let ridge = m.idx(5, 5);
        m.elevation[hollow] = 0.05;
        m.elevation[ridge] = 0.95;
        let cover = cell_canopy_scales(&m);
        assert!(
            cover[hollow] > cover[ridge],
            "hollow {} ridge {}",
            cover[hollow],
            cover[ridge]
        );
    }

    #[test]
    fn valley_drains_and_hides_more_than_a_peak_and_erosion_stays_zero() {
        let size = UVec2::new(5, 5);
        let mut m = ChunkCellMatrix::new(size);
        for v in &mut m.elevation {
            *v = 0.55;
        }
        let pit = m.idx(2, 2);
        let peak = m.idx(0, 0);
        m.elevation[pit] = 0.05;
        m.elevation[peak] = 0.95;
        let d = ChunkDerivedMetrics::from_chunk_matrix(&m);
        let flat = m.idx(4, 4);
        assert!(
            d.drainage[pit] > d.drainage[peak],
            "pit {} peak {}",
            d.drainage[pit],
            d.drainage[peak]
        );
        assert!(d.support_capacity[flat] > d.support_capacity[peak]);
        assert!(d.concealment[pit] > d.concealment[peak]);
        assert_eq!(d.flood_basin_id[pit], 1);
        assert_eq!(d.flood_basin_id[peak], 0);
        assert!(
            d.erosion_risk[peak] > d.erosion_risk[flat],
            "steep {} flat {}",
            d.erosion_risk[peak],
            d.erosion_risk[flat]
        );
        assert!(
            erosion_from_slope_drainage(0.4, 8.0) > erosion_from_slope_drainage(0.0, 1.0)
        );
    }

    #[test]
    fn wet_canopy_exceeds_dry_canopy_at_the_same_slope() {
        let wet = cell_canopy_scale_climate(0.0, 0.0, 1.0, 0.2);
        let dry = cell_canopy_scale_climate(0.0, 0.0, 0.0, 0.2);
        assert!(wet > dry, "wet {wet} dry {dry}");
    }

    #[test]
    fn neighbor_chunk_adds_drainage_on_the_shared_edge() {
        let size = UVec2::new(4, 4);
        let mut right = ChunkCellMatrix::new(size);
        for v in &mut right.elevation {
            *v = 0.2;
        }
        let mut left_elev = vec![0.2f32; 16];
        for y in 0..4u32 {
            left_elev[(y * 4 + 3) as usize] = 0.9;
        }
        let alone = drainage_with_neighbor_inflow(IVec2::new(1, 0), &right, &HashMap::new());
        let mut world = HashMap::new();
        world.insert(IVec2::ZERO, (size, left_elev));
        let stitched = drainage_with_neighbor_inflow(IVec2::new(1, 0), &right, &world);
        let edge = right.idx(0, 2);
        assert!(
            stitched[edge] > alone[edge],
            "stitched {} alone {}",
            stitched[edge],
            alone[edge]
        );
    }

    #[test]
    fn fold_drainage_does_not_flow_uphill() {
        let elev = vec![0.1f32, 0.5, 0.9];
        let flow = drainage_accumulation(&elev, 3, 1);
        assert!((flow[2] - 1.0).abs() < 1e-5, "peak took water {}", flow[2]);
        assert!(flow[1] > flow[2], "mid {} peak {}", flow[1], flow[2]);
        assert!(flow[0] > flow[1], "toe {} mid {}", flow[0], flow[1]);
    }

    #[test]
    fn fold_basin_id_stays_zero_off_the_floor() {
        let elev = vec![0.0f32, 5.0e-5, 1.0e-3];
        let ids = flood_basin_ids(&elev);
        assert_eq!(ids[0], 1);
        assert_eq!(ids[1], 1);
        assert_eq!(ids[2], 0);
    }

    #[test]
    fn fold_support_is_one_on_flat_zero_on_a_face() {
        assert!((support_from_slope(0.0) - 1.0).abs() < 1e-6);
        assert!((support_from_slope(-0.2) - 1.0).abs() < 1e-6);
        assert!((support_from_slope(0.2) - 0.0).abs() < 1e-6);
        assert!((support_from_slope(1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn fold_concealment_clamps_to_unit() {
        let hollow = concealment_from_landform(0.0, 10.0);
        let face = concealment_from_landform(10.0, -10.0);
        assert!((hollow - 1.0).abs() < 1e-5);
        assert!((face - 0.0).abs() < 1e-5);
    }
}
