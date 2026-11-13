//! Voxel height + water surface field for PixelWorld3D, derived from the world's own grids.
//!
//! Sources (read only, never written):
//! - [`WorldGenDenseTerrainCache::elevation`] — the carved elevation every sim system reads.
//! - [`WorldWaterGrid`] — the one per-cell water decision ([`WaterClass`]).
//!
//! The field is a presentation projection: ground is the elevation in world units, quantized to
//! [`HEIGHT_STEP`] terraces (the voxel look). Water bodies get a flat surface:
//! - ocean: the world water line;
//! - lake: the lowest dry rim around the connected water body (its outlet level);
//! - river: just under the lowest bank next to the cell, so a river runs in a channel instead of
//!   the sea-level trench the carved elevation would draw in highlands.
//!
//! The bed under a water surface sits a class-dependent depth below it. Picking, terrain meshes,
//! trees, sites and fronts all sample this one field.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{WaterClass, WorldGenDenseTerrainCache, WorldWaterGrid};

/// World units (tiles) per normalized elevation unit in the voxel view.
///
/// Presentation scale: 1.5x the construction conform scale (`TERRAIN_CONFORM_Y_SCALE` = 28 in
/// `construction::terrain_conform`) so relief reads at play zoom. Sim heights are untouched.
pub const PIXEL_WORLD_HEIGHT_SCALE: f32 = 42.0;

/// Terrace step of the voxel ground (world units).
pub const HEIGHT_STEP: f32 = 0.5;

/// Water surface sits this far under the lowest bank / rim.
const SURFACE_UNDER_BANK: f32 = 0.3;

/// No water surface on this cell.
pub const NO_WATER: f32 = f32::NEG_INFINITY;

/// Per-cell voxel ground and water surface, row-major `width x height` (index `y * width + x`,
/// `y` = tile row, +Y north — the same indexing as the dense cache and the terrain raster).
#[derive(Resource, Debug, Default)]
pub struct PixelWorldTerrainField {
    pub width: u32,
    pub height: u32,
    /// Top of the ground column (bed under water), world units.
    pub ground: Vec<f32>,
    /// Water surface, or [`NO_WATER`].
    pub water: Vec<f32>,
    pub class: Vec<WaterClass>,
    pub min_h: f32,
    pub max_h: f32,
    /// Bumps on every rebuild; meshes key on it.
    pub revision: u64,
    source_stamp: Option<u64>,
}

#[inline]
fn quantize(h: f32) -> f32 {
    (h / HEIGHT_STEP).round() * HEIGHT_STEP
}

impl PixelWorldTerrainField {
    #[inline]
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.width > 0 && self.height > 0 && self.ground.len() == (self.width * self.height) as usize
    }

    #[inline]
    #[must_use]
    pub fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height)
            .then(|| y as usize * self.width as usize + x as usize)
    }

    /// Visible surface top of a cell (water surface when it has one). `None` off-world.
    #[inline]
    #[must_use]
    pub fn surface_at_cell(&self, x: i32, y: i32) -> Option<f32> {
        self.index(x, y).map(|i| self.ground[i].max(self.water[i]))
    }

    /// Visible surface top at a tactical XY point (tile units).
    #[inline]
    #[must_use]
    pub fn surface_at(&self, p: Vec2) -> Option<f32> {
        self.surface_at_cell(p.x.floor() as i32, p.y.floor() as i32)
    }

    /// Highest surface of the cells touching a point (a tile corner touches four). Overlays draped
    /// with this never dip into a neighbouring column.
    #[must_use]
    pub fn surface_max_around(&self, p: Vec2) -> f32 {
        let (fx, fy) = (p.x.floor() as i32, p.y.floor() as i32);
        // A point on a cell edge (within 0.05 tile) also touches the cell on the other side.
        let near_lo = |v: f32| v - v.floor() < 0.05;
        let near_hi = |v: f32| v.ceil() - v < 0.05 && v.ceil() > v;
        let xs = [fx - i32::from(near_lo(p.x)), fx + i32::from(near_hi(p.x))];
        let ys = [fy - i32::from(near_lo(p.y)), fy + i32::from(near_hi(p.y))];
        let mut best = f32::NEG_INFINITY;
        for x in xs[0]..=xs[1] {
            for y in ys[0]..=ys[1] {
                if let Some(h) = self.surface_at_cell(x, y) {
                    best = best.max(h);
                }
            }
        }
        if best.is_finite() {
            best
        } else {
            self.min_h
        }
    }

    /// Rebuild from the grids. Returns true when the field changed.
    pub fn rebuild_if_stale(
        &mut self,
        params: &WorldGenParams,
        dense: &WorldGenDenseTerrainCache,
        grid: &WorldWaterGrid,
        dense_changed: bool,
    ) -> bool {
        if !dense.is_live_height_grid() || grid.width != dense.width || grid.height != dense.height {
            return false;
        }
        if !dense_changed && self.source_stamp == Some(grid.stamp) && self.is_ready() {
            return false;
        }
        self.build(params, dense, grid);
        self.source_stamp = Some(grid.stamp);
        self.revision = self.revision.wrapping_add(1);
        true
    }

    fn build(&mut self, params: &WorldGenParams, dense: &WorldGenDenseTerrainCache, grid: &WorldWaterGrid) {
        let (w, h) = (dense.width as usize, dense.height as usize);
        let n = w * h;
        let scale = PIXEL_WORLD_HEIGHT_SCALE;
        let sea = params.biome_tuning.shallow_water_height_max * scale;
        let sea_surface = quantize(sea) - SURFACE_UNDER_BANK;
        self.width = dense.width;
        self.height = dense.height;
        self.class = grid.classes.clone();
        self.ground = dense.elevation.iter().map(|e| quantize(e * scale)).collect();
        self.water = vec![NO_WATER; n];

        let is_open = |i: usize| self.class[i].is_open_water();
        let neighbors4 = |i: usize| -> [Option<usize>; 4] {
            let (x, y) = (i % w, i / w);
            [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ]
        };

        // Standing water: one surface per connected lake / ocean body.
        let mut seen = vec![false; n];
        let mut stack = Vec::new();
        let mut body = Vec::new();
        for start in 0..n {
            if seen[start] || !self.class[start].is_standing() {
                continue;
            }
            body.clear();
            stack.push(start);
            seen[start] = true;
            let mut ocean = false;
            let mut rim = f32::INFINITY;
            while let Some(i) = stack.pop() {
                body.push(i);
                ocean |= self.class[i] == WaterClass::Ocean;
                for j in neighbors4(i).into_iter().flatten() {
                    if self.class[j].is_standing() {
                        if !seen[j] {
                            seen[j] = true;
                            stack.push(j);
                        }
                    } else if !is_open(j) {
                        rim = rim.min(self.ground[j]);
                    }
                }
            }
            let surface = if ocean || !rim.is_finite() {
                sea_surface
            } else {
                (rim - SURFACE_UNDER_BANK).max(sea_surface)
            };
            for &i in &body {
                self.water[i] = surface;
            }
        }

        // Rivers: just under the lowest bank within two cells; wide channels inherit from neighbours.
        let mut pending: Vec<usize> = Vec::new();
        for i in 0..n {
            if self.class[i] != WaterClass::River {
                continue;
            }
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let mut bank = f32::INFINITY;
            for r in 1..=2i32 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ny as usize * w + nx as usize;
                        if !is_open(j) {
                            bank = bank.min(self.ground[j]);
                        }
                    }
                }
                if bank.is_finite() {
                    break;
                }
            }
            if bank.is_finite() {
                self.water[i] = (bank - SURFACE_UNDER_BANK).max(sea_surface);
            } else {
                pending.push(i);
            }
        }
        for _ in 0..8 {
            if pending.is_empty() {
                break;
            }
            let mut still = Vec::new();
            for &i in &pending {
                let known = neighbors4(i)
                    .into_iter()
                    .flatten()
                    .map(|j| self.water[j])
                    .filter(|s| s.is_finite())
                    .fold(f32::INFINITY, f32::min);
                if known.is_finite() {
                    self.water[i] = known;
                } else {
                    still.push(i);
                }
            }
            pending = still;
        }
        for i in pending {
            self.water[i] = sea_surface;
        }

        // Beds sit under the surface; dry ground never drops below the sea surface it borders.
        for i in 0..n {
            let s = self.water[i];
            if s.is_finite() {
                let depth = match self.class[i] {
                    WaterClass::River => 0.75,
                    WaterClass::Lake => 1.5,
                    _ => 2.0,
                };
                self.ground[i] = quantize(s - depth).min(s - 0.25);
            }
        }
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for i in 0..n {
            let top = self.ground[i].max(self.water[i]);
            lo = lo.min(self.ground[i]);
            hi = hi.max(top);
        }
        self.min_h = if lo.is_finite() { lo } else { 0.0 };
        self.max_h = if hi.is_finite() { hi } else { 0.0 };
    }

    /// Camera ray against the voxel surface. Returns the tactical XY of the first column top or
    /// side the ray meets, or the point where it reaches the lowest ground when it misses every
    /// column (callers reject off-world points).
    #[must_use]
    pub fn ray_hit(&self, origin: Vec3, dir: Vec3) -> Option<Vec2> {
        if !self.is_ready() || dir.y >= -1e-4 {
            return None;
        }
        let top = self.max_h + 0.01;
        let bottom = self.min_h - 0.01;
        // Enter where the ray crosses the highest surface (or start at the eye when it is below it).
        let mut t = ((origin.y - top) / -dir.y).max(0.0);
        let t_end = (origin.y - bottom) / -dir.y;
        let horiz = Vec2::new(dir.x, dir.z).length().max(1e-4);
        // Quarter-tile horizontal steps: no column is thinner than a tile.
        let dt = 0.25 / horiz;
        let above = |t: f32| -> bool {
            let p = origin + dir * t;
            let xy = Vec2::new(p.x, -p.z);
            match self.surface_at(xy) {
                Some(s) => p.y > s,
                None => true,
            }
        };
        let mut prev = t;
        let mut steps = 0u32;
        while t <= t_end && steps < 20_000 {
            if !above(t) {
                // Refine between the last point above and this one.
                let (mut a, mut b) = (prev, t);
                for _ in 0..12 {
                    let m = 0.5 * (a + b);
                    if above(m) {
                        a = m;
                    } else {
                        b = m;
                    }
                }
                let p = origin + dir * b;
                return Some(Vec2::new(p.x, -p.z));
            }
            prev = t;
            t += dt;
            steps += 1;
        }
        let p = origin + dir * t_end;
        Some(Vec2::new(p.x, -p.z))
    }
}

/// Keep the field in step with the dense cache + water grid (PixelWorld3D only).
pub fn sync_pixel_world_terrain_field(
    mode: Option<Res<crate::gui::WorldPresentationMode>>,
    params: Res<WorldGenParams>,
    dense: Option<Res<WorldGenDenseTerrainCache>>,
    grid: Option<Res<WorldWaterGrid>>,
    mut field: ResMut<PixelWorldTerrainField>,
) {
    if !mode.is_some_and(|m| m.is_pixel_world()) {
        return;
    }
    let (Some(dense), Some(grid)) = (dense, grid) else {
        return;
    };
    let dense_changed = dense.is_changed() || grid.is_changed();
    field.rebuild_if_stale(params.as_ref(), dense.as_ref(), grid.as_ref(), dense_changed);
}
