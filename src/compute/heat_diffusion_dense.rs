//! **DH-01 / TAC-03** — dense separable heat on a fixed lattice (CPU parity / replay reference).
//!
//! Not the play authority (GPU dense lands in DH-02). Fixed 1-D stencil twice (H then V);
//! missing neighbors use edge clamp — divisor is always constant (true separable, not sparse mean).

use bevy::math::IVec2;
use bevy::prelude::Resource;

use crate::render::extraction::sim_visual_extract::ChunkFireHeat;

/// Horizontal/vertical separable weights: `2 * W_SIDE + W_CENTER == 1`.
pub const DENSE_W_SIDE: f32 = 0.15;
pub const DENSE_W_CENTER: f32 = 0.70;
pub const DENSE_DECAY: f32 = 0.02;
pub const DENSE_SOURCE_BLEND: f32 = 0.35;

/// Play vs parity path selection (**DH-01** gates sparse behind this flag).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeatDiffusionConfig {
    /// When true, play uses sparse Jacobi (legacy). When false (**DH-02 default**),
    /// play uses dense WGSL-parity step via [`super::heat_diffusion_gpu`].
    pub use_sparse_play: bool,
}

impl Default for HeatDiffusionConfig {
    fn default() -> Self {
        Self {
            // DH-02: sparse gated off — dense (WGSL-parity) owns play.
            use_sparse_play: false,
        }
    }
}

/// Dense ROI: chunk lattice origin + size, SoA heat/smoke.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseHeatGrid {
    pub origin: IVec2,
    pub width: u32,
    pub height: u32,
    pub heat: Vec<f32>,
    pub smoke: Vec<f32>,
}

impl DenseHeatGrid {
    #[must_use]
    pub fn new(origin: IVec2, width: u32, height: u32) -> Self {
        let n = (width as usize).saturating_mul(height as usize);
        Self {
            origin,
            width,
            height,
            heat: vec![0.0; n],
            smoke: vec![0.0; n],
        }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.heat.len()
    }

    #[inline]
    #[must_use]
    pub fn idx(&self, lx: u32, ly: u32) -> usize {
        (ly * self.width + lx) as usize
    }

    /// Map world chunk → local index, if inside ROI.
    #[must_use]
    pub fn local_of(&self, chunk: IVec2) -> Option<(u32, u32)> {
        let lx = chunk.x - self.origin.x;
        let ly = chunk.y - self.origin.y;
        if lx < 0 || ly < 0 || lx as u32 >= self.width || ly as u32 >= self.height {
            return None;
        }
        Some((lx as u32, ly as u32))
    }

    /// Inject fire sources (blend into cell). Out-of-ROI rows ignored.
    pub fn inject_sources(&mut self, fire_rows: &[&ChunkFireHeat]) {
        for row in fire_rows {
            let Some((lx, ly)) = self.local_of(row.chunk) else {
                continue;
            };
            let i = self.idx(lx, ly);
            self.heat[i] =
                self.heat[i] * (1.0 - DENSE_SOURCE_BLEND) + row.heat * DENSE_SOURCE_BLEND;
            self.smoke[i] =
                self.smoke[i] * (1.0 - DENSE_SOURCE_BLEND) + row.smoke * DENSE_SOURCE_BLEND;
        }
    }
}

fn sample_clamp(buf: &[f32], w: usize, h: usize, x: i32, y: i32) -> f32 {
    let cx = x.clamp(0, w as i32 - 1) as usize;
    let cy = y.clamp(0, h as i32 - 1) as usize;
    buf[cy * w + cx]
}

/// One separable 1-D pass along X (writes `dst` from `src`).
fn pass_horizontal(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    debug_assert_eq!(src.len(), w * h);
    debug_assert_eq!(dst.len(), w * h);
    for y in 0..h {
        for x in 0..w {
            let l = sample_clamp(src, w, h, x as i32 - 1, y as i32);
            let c = src[y * w + x];
            let r = sample_clamp(src, w, h, x as i32 + 1, y as i32);
            dst[y * w + x] = DENSE_W_SIDE * l + DENSE_W_CENTER * c + DENSE_W_SIDE * r;
        }
    }
}

/// One separable 1-D pass along Y.
fn pass_vertical(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    debug_assert_eq!(src.len(), w * h);
    debug_assert_eq!(dst.len(), w * h);
    for y in 0..h {
        for x in 0..w {
            let d = sample_clamp(src, w, h, x as i32, y as i32 - 1);
            let c = src[y * w + x];
            let u = sample_clamp(src, w, h, x as i32, y as i32 + 1);
            dst[y * w + x] = DENSE_W_SIDE * d + DENSE_W_CENTER * c + DENSE_W_SIDE * u;
        }
    }
}

/// Dense separable step: source → H → V → decay → source (keeps fire cells hot). Deterministic.
pub fn run_dense_heat_step(grid: &mut DenseHeatGrid, fire_rows: &[&ChunkFireHeat]) {
    let w = grid.width as usize;
    let h = grid.height as usize;
    if w == 0 || h == 0 || grid.heat.len() != w * h {
        return;
    }
    grid.inject_sources(fire_rows);

    let mut tmp = vec![0.0f32; w * h];
    pass_horizontal(&grid.heat, &mut tmp, w, h);
    pass_vertical(&tmp, &mut grid.heat, w, h);

    pass_horizontal(&grid.smoke, &mut tmp, w, h);
    pass_vertical(&tmp, &mut grid.smoke, w, h);

    for v in &mut grid.heat {
        *v = (*v * (1.0 - DENSE_DECAY)).clamp(0.0, 1.5);
    }
    for v in &mut grid.smoke {
        *v = (*v * (1.0 - DENSE_DECAY)).clamp(0.0, 1.0);
    }
    // Re-seed after blur/decay so active fires remain sources (parity schedule for DH-02 GPU).
    grid.inject_sources(fire_rows);
}

/// Build a padded ROI covering all fire chunks + `pad` cells of border.
#[must_use]
pub fn dense_roi_from_fire(fire_rows: &[&ChunkFireHeat], pad: i32) -> Option<DenseHeatGrid> {
    if fire_rows.is_empty() {
        return None;
    }
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for row in fire_rows {
        min_x = min_x.min(row.chunk.x);
        min_y = min_y.min(row.chunk.y);
        max_x = max_x.max(row.chunk.x);
        max_y = max_y.max(row.chunk.y);
    }
    let pad = pad.max(0);
    let origin = IVec2::new(min_x - pad, min_y - pad);
    let width = (max_x - min_x + 1 + 2 * pad) as u32;
    let height = (max_y - min_y + 1 + 2 * pad) as u32;
    Some(DenseHeatGrid::new(origin, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tac03_dense_cpu() {
        assert!((2.0 * DENSE_W_SIDE + DENSE_W_CENTER - 1.0).abs() < 1e-6);

        let fire = [ChunkFireHeat {
            chunk: IVec2::new(2, 2),
            heat: 1.0,
            smoke: 0.5,
        }];
        let refs: Vec<&ChunkFireHeat> = fire.iter().collect();
        let mut a = dense_roi_from_fire(&refs, 2).expect("roi");
        let mut b = a.clone();
        run_dense_heat_step(&mut a, &refs);
        run_dense_heat_step(&mut b, &refs);
        assert_eq!(a, b, "deterministic");

        // pad=2, min chunk (2,2) → origin (0,0); source at local (2,2).
        let center = a.local_of(IVec2::new(2, 2)).unwrap();
        let ci = a.idx(center.0, center.1);
        let east = a.idx(center.0 + 1, center.1);
        assert!(a.heat[ci] > 0.2);
        assert!(
            a.heat[east] > 0.0,
            "separable H then V must move heat to neighbor"
        );

        run_dense_heat_step(&mut a, &refs);
        let far = a.idx(0, 0);
        assert!(a.heat[ci] > a.heat[far]);

        // DH-02: sparse gated off by default.
        assert!(!HeatDiffusionConfig::default().use_sparse_play);
    }

    #[test]
    fn dense_weights_are_separable_unity() {
        // Two 1-D passes with these weights ≈ isotropic 5-tap without variable neighbor count.
        let k = [DENSE_W_SIDE, DENSE_W_CENTER, DENSE_W_SIDE];
        let sum: f32 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
}
