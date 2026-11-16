//! **TAC-05** — chokepoint / defensibility / high-ground precompute (aig3 6.2.3).
//!
//! Sole writer: world-gen / [`crate::terrain::generation::passes::p5_agent_overlay`].
//! Fortification + faction AI read only.

use std::collections::BTreeMap;

use bevy::prelude::{IVec2, Resource};

/// Per-tile terrain tactical scores (0..1), gen-authored.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainAnalysisCell {
    /// Narrow approach / saddle-like (high when local relief funnels traffic).
    pub chokepoint: f32,
    /// Defensive value (high ground + cover from slope).
    pub defensibility: f32,
    /// Relative elevation advantage vs local neighborhood.
    pub high_ground: f32,
}

/// World-tile keyed analysis book. Sparse — only cells above a quiet floor are stored.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct TerrainAnalysisBook {
    pub cells: BTreeMap<(i32, i32), TerrainAnalysisCell>,
    pub revision: u64,
}

impl TerrainAnalysisBook {
    #[must_use]
    pub fn get(&self, tile: IVec2) -> Option<TerrainAnalysisCell> {
        self.cells.get(&(tile.x, tile.y)).copied()
    }

    #[must_use]
    pub fn defensibility_at(&self, tile: IVec2) -> f32 {
        self.get(tile).map(|c| c.defensibility).unwrap_or(0.0)
    }

    #[must_use]
    pub fn chokepoint_at(&self, tile: IVec2) -> f32 {
        self.get(tile).map(|c| c.chokepoint).unwrap_or(0.0)
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.revision = self.revision.wrapping_add(1);
    }

    /// Merge a rect of analysis (world origin = `origin` for local `(0,0)`).
    pub fn merge_rect(
        &mut self,
        origin: IVec2,
        width: u32,
        height: u32,
        cells: &[TerrainAnalysisCell],
    ) {
        let n = (width as usize).saturating_mul(height as usize);
        if cells.len() != n || n == 0 {
            return;
        }
        for y in 0..height {
            for x in 0..width {
                let i = (y * width + x) as usize;
                let c = cells[i];
                if c.chokepoint < 0.05 && c.defensibility < 0.05 && c.high_ground < 0.05 {
                    continue;
                }
                let wx = origin.x + x as i32;
                let wy = origin.y + y as i32;
                self.cells.insert((wx, wy), c);
            }
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// Mean defensibility over `tiles` (missing → 0).
    #[must_use]
    pub fn mean_defensibility<'a>(&self, tiles: impl Iterator<Item = &'a IVec2>) -> f32 {
        let (sum, n) = tiles.fold((0.0f32, 0u32), |(s, n), t| {
            (s + self.defensibility_at(*t), n + 1)
        });
        if n == 0 {
            0.0
        } else {
            sum / n as f32
        }
    }
}

/// Precompute analysis for a row-major DEM (`len == width * height`).
#[must_use]
pub fn compute_terrain_analysis_rect(
    width: u32,
    height: u32,
    dem: &[f32],
) -> Vec<TerrainAnalysisCell> {
    let w = width as usize;
    let h = height as usize;
    let n = w.saturating_mul(h);
    if n == 0 || dem.len() != n {
        return Vec::new();
    }

    let mut mean = 0.0f32;
    for &v in dem {
        mean += v;
    }
    mean /= n as f32;

    let mut out = vec![TerrainAnalysisCell::default(); n];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let h0 = dem[i];
            let mut min_n = h0;
            let mut max_n = h0;
            let mut slope_sum = 0.0f32;
            let mut neighbors = 0u32;
            for (dx, dy) in [
                (-1i32, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let hn = dem[ny as usize * w + nx as usize];
                min_n = min_n.min(hn);
                max_n = max_n.max(hn);
                slope_sum += (h0 - hn).abs();
                neighbors += 1;
            }
            let relief = (max_n - min_n).max(0.0);
            let avg_slope = if neighbors == 0 {
                0.0
            } else {
                slope_sum / neighbors as f32
            };
            // High ground: above local mean of neighborhood + global mean bias.
            let high = ((h0 - mean) * 4.0 + (h0 - min_n) * 2.0).clamp(0.0, 1.0);
            // Chokepoint: moderate elevation with high local relief (saddle / gap feel).
            let choke = (relief * 3.0 * (1.0 - (h0 - mean).abs() * 2.0).clamp(0.0, 1.0))
                .clamp(0.0, 1.0);
            // Defensibility: high ground + slope cover, less open flat.
            let def = (high * 0.65 + avg_slope.clamp(0.0, 1.0) * 0.35).clamp(0.0, 1.0);
            out[i] = TerrainAnalysisCell {
                chokepoint: choke,
                defensibility: def,
                high_ground: high,
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tac05() {
        // Peak in the middle → high ground / defensibility there; rim flatter.
        let w = 5u32;
        let h = 5u32;
        let mut dem = vec![0.2f32; 25];
        dem[12] = 0.9; // center
        let a = compute_terrain_analysis_rect(w, h, &dem);
        let b = compute_terrain_analysis_rect(w, h, &dem);
        assert_eq!(a, b, "deterministic");
        assert!(a[12].high_ground > a[0].high_ground);
        assert!(a[12].defensibility > a[0].defensibility);

        let mut book = TerrainAnalysisBook::default();
        book.merge_rect(IVec2::new(10, 20), w, h, &a);
        assert!(book.defensibility_at(IVec2::new(12, 22)) > 0.0);
        assert_eq!(book.defensibility_at(IVec2::new(0, 0)), 0.0);
    }
}
