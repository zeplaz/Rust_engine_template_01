//! Voxel height + water surface field for PixelWorld3D, derived from the world's own grids.
//!
//! Sources (read only, never written):
//! - [`WorldGenDenseTerrainCache::elevation`] — bed every sim system reads.
//! - [`WorldWaterGrid`] — bake hint for look (channel vs standing); not gameplay wet authority.
//! - [`SurfaceWaterField`] — the map-wide live water (WM-5): `water[]` is `ground + depth` read
//!   straight from it, patched in place for the chunks the field reports as moved
//!   ([`PixelWorldTerrainField::follow_surface_water`]); nothing is quantised or rebuilt per change.
//! - [`WorldStaticWaterDepth`] — only the pre-seed bootstrap (before the field exists); class-based
//!   rim fill is the fallback under that.
//!
//! The field is a presentation projection: ground is the elevation in world units, quantized to
//! [`HEIGHT_STEP`] terraces (the voxel look).

use bevy::prelude::*;

use crate::systems::water::{
    water_look_from_depth_speed, SurfaceWaterField, WaterFieldDirtyFeed, WorldStaticWaterDepth,
    CHANNEL_LOOK_SPEED, WET_DEPTH_M,
};
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::ground_deform::GroundDeltaJournal;
use crate::terrain::generation::{WaterClass, WorldGenDenseTerrainCache, WorldWaterGrid};

// Heights: metres from `WorldGenParams::elevation_range_m` (the one vertical contract), drawn at
// `display_units_per_metre()` for ground AND water alike. The old fixed 42-tile scale also got
// multiplied into water depth (`d / 16 m · 42`), drawing a 3.5 m river ~150 m tall.

/// Cells per terrain mesh chunk (`terrain::TERRAIN_CHUNK`).
const CHUNK_CELLS: u32 = super::terrain::TERRAIN_CHUNK;

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
    /// Bumps on every rebuild and whenever water moved enough to matter to meshes draped on the
    /// field (a cell turned wet/dry or its surface drifted by [`HEIGHT_STEP`] since the last bump).
    pub revision: u64,
    /// Bumps on every water patch that changed any `water[]` value (exact data; no threshold).
    pub water_revision: u64,
    /// Display units per real metre used for this build (ground, water, depth lifts).
    pub units_per_metre: f32,
    /// Bumps only when [`Self::ground`] (or the grid size) changes. Ground meshes key on this so
    /// water motion never respawns terrain chunks (PLAN-CLOSE-VOXEL-WATER-v1 V1).
    pub ground_revision: u64,
    /// Bumps only when the whole ground is recomputed (world / grid change). A terrain deformation
    /// patches cells in place (`ground_patches`) and bumps `ground_revision` but not this, so the
    /// ground mesh remeshes only the touched chunks instead of the whole map (DW-1).
    pub ground_full_revision: u64,
    /// Incremental ground patches, newest last: `(patch seq, chunk cx, chunk cy)`, bounded.
    pub ground_patches: Vec<(u64, u32, u32)>,
    pub ground_patch_seq: u64,
    /// Mirror of the terrain ground delta in metres (journal-fed; drawn as `terraced DEM + delta`).
    deform_m: Vec<f32>,
    deform_seq: u64,
    deform_epoch: u64,
    source_stamp: Option<u64>,
    /// Last [`WaterFieldDirtyFeed`] frame folded into `water[]`.
    water_feed_seq: u64,
    /// Coastal (ocean-class) cells of the baked grid: the look hint survives wet/dry flips.
    coast: Vec<bool>,
    /// `water[]` as of the last `revision` bump (drift reference for the threshold above).
    water_drape: Vec<f32>,
}

/// Patch history kept for the mesh builder's cursor.
const GROUND_PATCH_HISTORY: usize = 512;

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
        static_depth: Option<&WorldStaticWaterDepth>,
        surface: Option<&SurfaceWaterField>,
        dense_changed: bool,
    ) -> bool {
        if !dense.is_live_height_grid() || grid.width != dense.width || grid.height != dense.height {
            return false;
        }
        // Live water is not part of the stamp: it arrives per chunk through `follow_surface_water`.
        // Only the first sight of the field (pre-seed bootstrap -> field-sourced) forces a build.
        let live = surface.is_some_and(|f| Self::field_covers(f, dense));
        let stamp = grid
            .stamp
            .wrapping_add(static_depth.map(|s| s.stamp).unwrap_or(0))
            .wrapping_add(u64::from(live) << 60);
        if !dense_changed && self.source_stamp == Some(stamp) && self.is_ready() {
            return false;
        }
        let prev_dims = (self.width, self.height);
        let prev_ground = std::mem::take(&mut self.ground);
        self.build(params, dense, grid, static_depth, surface);
        self.source_stamp = Some(stamp);
        self.revision = self.revision.wrapping_add(1);
        if prev_dims != (self.width, self.height) || prev_ground != self.ground {
            self.ground_revision = self.ground_revision.wrapping_add(1);
            self.ground_full_revision = self.ground_full_revision.wrapping_add(1);
        }
        true
    }

    /// DW-1: follow the terrain ground-delta journal. Cells are patched in place (ground, the
    /// water surface over them, height bounds) and the touched chunks queued for remesh -- no
    /// whole-map rebuild. Returns true when a full rebuild is needed instead (first sight of a
    /// world, grid change, journal overrun).
    pub fn follow_ground_deform(
        &mut self,
        params: &WorldGenParams,
        dense: &WorldGenDenseTerrainCache,
        journal: &GroundDeltaJournal,
    ) -> bool {
        let n = (dense.width as usize) * (dense.height as usize);
        let mut full = false;
        if self.deform_m.len() != n {
            self.deform_m = vec![0.0; n];
            full = true;
        }
        if self.deform_epoch != journal.epoch {
            self.deform_m.fill(0.0);
            self.deform_epoch = journal.epoch;
            self.deform_seq = journal.dropped_through;
            full = true;
        }
        if self.deform_seq == journal.seq {
            return full;
        }
        let Some(cells) = journal.since(self.deform_seq) else {
            warn!("pixel world field: ground journal overran its cursor; full rebuild");
            self.deform_seq = journal.seq;
            return true;
        };
        let (w, h) = (dense.width as i32, dense.height as i32);
        let patch = !full
            && self.is_ready()
            && (self.width, self.height) == (dense.width, dense.height)
            && dense.is_live_height_grid();
        let upm = crate::terrain::world_scale_contract::display_units_per_metre();
        let mut chunks: Vec<(u32, u32)> = Vec::new();
        let mut touched = false;
        for c in cells {
            let (x, y) = (c.tile.x, c.tile.y);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let i = (y * w + x) as usize;
            self.deform_m[i] += c.delta_m;
            if !patch {
                continue;
            }
            let dm = self.deform_m[i];
            let new = if dm == 0.0 {
                quantize(dense.elevation[i] * params.elevation_range_m * upm)
            } else {
                quantize((dense.elevation[i] * params.elevation_range_m - dm) * upm) + dm * upm
            };
            let moved = new - self.ground[i];
            self.ground[i] = new;
            // Live water keeps its depth over a moved bed (mass conserved): the surface rides it.
            if self.water[i].is_finite() {
                self.water[i] += moved;
                self.max_h = self.max_h.max(self.water[i]);
            }
            self.min_h = self.min_h.min(new);
            self.max_h = self.max_h.max(new);
            touched = true;
            // The cell's chunk and any chunk across an edge it borders (side walls read neighbours).
            for (nx, ny) in [(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let ck = (nx as u32 / CHUNK_CELLS, ny as u32 / CHUNK_CELLS);
                if !chunks.contains(&ck) {
                    chunks.push(ck);
                }
            }
        }
        self.deform_seq = journal.seq;
        if !patch {
            return true;
        }
        if touched {
            self.ground_patch_seq = self.ground_patch_seq.wrapping_add(1);
            let seq = self.ground_patch_seq;
            self.ground_patches.extend(chunks.into_iter().map(|(cx, cy)| (seq, cx, cy)));
            if self.ground_patches.len() > GROUND_PATCH_HISTORY {
                let drop = self.ground_patches.len() - GROUND_PATCH_HISTORY;
                self.ground_patches.drain(..drop);
            }
            self.ground_revision = self.ground_revision.wrapping_add(1);
            self.revision = self.revision.wrapping_add(1);
        }
        false
    }

    /// The field is usable as the live water of this grid.
    fn field_covers(f: &SurfaceWaterField, dense: &WorldGenDenseTerrainCache) -> bool {
        f.wired
            && f.depth_m.len() == f.cell_count()
            && f.origin_x == 0
            && f.origin_z == 0
            && f.width == dense.width
            && f.height == dense.height
    }

    /// Water surface + look of cell `(x, y)` (index `i`) straight from the field: `ground + depth`
    /// in display units (the height the GPU water draws), no terracing.
    fn field_cell(&self, f: &SurfaceWaterField, x: u32, y: u32, i: usize, upm: f32) -> (f32, WaterClass) {
        let fi = f.idx(x, y);
        let d = f.depth_m.get(fi).copied().unwrap_or(0.0);
        if d < WET_DEPTH_M {
            return (NO_WATER, WaterClass::Dry);
        }
        let qx = f.qx_m3s.get(fi).copied().unwrap_or(0.0).abs();
        let qy = f.qy_m3s.get(fi).copied().unwrap_or(0.0).abs();
        let speed = (qx + qy) / (d * f.dx_m.max(1e-3)).max(1e-3);
        let coast = self.coast.get(i).copied().unwrap_or(false);
        (self.ground[i] + d * upm, water_look_from_depth_speed(d, speed, coast))
    }

    /// WM-5: fold the field's moved chunks into `water[]` / `class[]` in place (the way
    /// [`Self::follow_ground_deform`] patches the ground). Cost is the touched chunks, not the map.
    /// A consumer that skipped a feed frame re-reads every chunk. Returns true when any `water[]`
    /// value changed.
    pub fn follow_surface_water(&mut self, surface: &SurfaceWaterField, feed: &WaterFieldDirtyFeed) -> bool {
        if !self.is_ready()
            || self.water.len() != self.ground.len()
            || self.water_drape.len() != self.water.len()
            || self.coast.len() != self.water.len()
            || !(surface.wired
                && surface.origin_x == 0
                && surface.origin_z == 0
                && surface.width == self.width
                && surface.height == self.height
                && surface.depth_m.len() == surface.cell_count())
        {
            return false;
        }
        let chunks = feed.chunks_since(self.water_feed_seq);
        self.water_feed_seq = feed.seq;
        let cs = CHUNK_TILES_SIM as i32;
        let (w, h) = (self.width as i32, self.height as i32);
        let rects: Vec<(i32, i32, i32, i32)> = match chunks {
            Some(list) => list
                .iter()
                .map(|c| (c.x * cs, c.y * cs, ((c.x + 1) * cs).min(w), ((c.y + 1) * cs).min(h)))
                .collect(),
            None => vec![(0, 0, w, h)],
        };
        let upm = crate::terrain::world_scale_contract::display_units_per_metre();
        let (mut changed, mut drifted) = (false, false);
        for (x0, y0, x1, y1) in rects {
            for y in y0.max(0)..y1 {
                for x in x0.max(0)..x1 {
                    let i = y as usize * self.width as usize + x as usize;
                    let (water, class) = self.field_cell(surface, x as u32, y as u32, i, upm);
                    let old = self.water[i];
                    if old.to_bits() != water.to_bits() {
                        changed = true;
                        self.water[i] = water;
                        if water.is_finite() {
                            self.max_h = self.max_h.max(water);
                        }
                    }
                    self.class[i] = class;
                    let r = self.water_drape[i];
                    let moved = r.is_finite() != water.is_finite()
                        || (water.is_finite() && (water - r).abs() >= HEIGHT_STEP);
                    if moved {
                        self.water_drape[i] = water;
                        drifted = true;
                    }
                }
            }
        }
        if changed {
            self.water_revision = self.water_revision.wrapping_add(1);
        }
        if drifted {
            self.revision = self.revision.wrapping_add(1);
        }
        changed
    }

    fn build(
        &mut self,
        params: &WorldGenParams,
        dense: &WorldGenDenseTerrainCache,
        grid: &WorldWaterGrid,
        static_depth: Option<&WorldStaticWaterDepth>,
        surface: Option<&SurfaceWaterField>,
    ) {
        let (w, h) = (dense.width as usize, dense.height as usize);
        let n = w * h;
        let upm = crate::terrain::world_scale_contract::display_units_per_metre();
        self.units_per_metre = upm;
        // Normalized elevation → metres (world contract) → display units.
        let scale = params.elevation_range_m * upm;
        let sea = params.biome_tuning.shallow_water_height_max * scale;
        let sea_surface = quantize(sea) - SURFACE_UNDER_BANK;
        self.width = dense.width;
        self.height = dense.height;
        self.class = grid.classes.clone();
        // Terraced DEM plus the (unterraced) deformation delta: a few-metre crater must not be
        // swallowed by the 8 m height terrace. No delta -> identical to `quantize(e * scale)`.
        let range_m = params.elevation_range_m;
        self.ground = dense
            .elevation
            .iter()
            .enumerate()
            .map(|(i, e)| match self.deform_m.get(i).copied().filter(|d| *d != 0.0) {
                Some(dm) => quantize((e * range_m - dm) * upm) + dm * upm,
                None => quantize(e * scale),
            })
            .collect();
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

        // PLAN-WATER-DEPTH-TRUTH-v1 W3: when static depth exists, surface = ground + depth (one fluid).
        // Look class is derived from depth (standing vs channel bake hint), not a second water.
        let live_field = surface.filter(|f| Self::field_covers(f, dense));
        if let Some(st) = static_depth.filter(|_| live_field.is_none()) {
            if st.width == dense.width
                && st.height == dense.height
                && st.depth_m.len() >= n
            {
                for i in 0..n {
                    let d = st.depth_m[i];
                    if d < WET_DEPTH_M {
                        self.water[i] = NO_WATER;
                        self.class[i] = WaterClass::Dry;
                        continue;
                    }
                    let lift = d * upm; // metres → display units (once)
                    self.water[i] = quantize(self.ground[i] + lift.max(SURFACE_UNDER_BANK));
                    let coast = matches!(self.class[i], WaterClass::Ocean);
                    // Static bootstrap has no flux — channel look stays River when bake said River.
                    let speed = if self.class[i] == WaterClass::River {
                        CHANNEL_LOOK_SPEED + 0.01
                    } else {
                        0.0
                    };
                    self.class[i] = water_look_from_depth_speed(d, speed, coast);
                }
            }
        }

        // Class-bootstrap only: dig bed under surface. Depth-truth keeps DEM as bed.
        if static_depth.is_none() {
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
        }

        // WM-5: the map-wide field is the live truth; static depth was only its seed.
        self.coast = grid.classes.iter().map(|c| *c == WaterClass::Ocean).collect();
        if let Some(sf) = live_field {
            for i in 0..n {
                let (x, y) = ((i % w) as u32, (i / w) as u32);
                let (water, class) = self.field_cell(sf, x, y, i, upm);
                self.water[i] = water;
                self.class[i] = class;
            }
        }
        self.water_drape = self.water.clone();

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
    static_depth: Option<Res<WorldStaticWaterDepth>>,
    surface: Option<Res<SurfaceWaterField>>,
    feed: Option<Res<WaterFieldDirtyFeed>>,
    journal: Option<Res<GroundDeltaJournal>>,
    mut field: ResMut<PixelWorldTerrainField>,
) {
    if !mode.is_some_and(|m| m.is_pixel_world()) {
        return;
    }
    let (Some(dense), Some(grid)) = (dense, grid) else {
        return;
    };
    // Terrain deformation arrives through the journal (the dense cache is mutated without its
    // change flag, which is reserved for "world regenerated").
    let deform_full = journal
        .as_deref()
        .is_some_and(|j| field.follow_ground_deform(params.as_ref(), dense.as_ref(), j));
    let dense_changed = dense.is_changed()
        || grid.is_changed()
        || deform_full
        || static_depth.as_ref().is_some_and(|s| s.is_changed());
    // Live water never triggers a rebuild (PLAN-CLOSE-VOXEL-WATER-v1 V0): a rebuild reads the
    // field in full once, and from then on the moved chunks are patched in place (WM-5).
    let rebuilt = field.rebuild_if_stale(
        params.as_ref(),
        dense.as_ref(),
        grid.as_ref(),
        static_depth.as_deref(),
        surface.as_deref(),
        dense_changed,
    );
    if let (Some(surface), Some(feed)) = (surface.as_deref(), feed.as_deref()) {
        if rebuilt {
            field.water_feed_seq = feed.seq;
        } else {
            field.follow_surface_water(surface, feed);
        }
    }
}

#[cfg(test)]
mod ground_revision_tests {
    use super::*;

    fn inputs(n_side: u32, elev: f32) -> (WorldGenDenseTerrainCache, WorldWaterGrid) {
        let n = (n_side * n_side) as usize;
        let dense = WorldGenDenseTerrainCache {
            width: n_side,
            height: n_side,
            elevation: vec![elev; n],
            moisture: vec![0.3; n],
            temperature: vec![0.3; n],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
            region_index: vec![0; n],
        };
        let grid = WorldWaterGrid { width: n_side, height: n_side, classes: vec![WaterClass::Dry; n], stamp: 1 };
        (dense, grid)
    }

    /// CLOSE-V1: a water-driven rebuild (forced, same ground) bumps `revision` but not
    /// `ground_revision`, so terrain chunks never respawn on water motion.
    #[test]
    fn close_v1_water_rebuild_keeps_ground_revision() {
        let params = WorldGenParams::default();
        let (dense, grid) = inputs(8, 0.5);
        let mut f = PixelWorldTerrainField::default();
        assert!(f.rebuild_if_stale(&params, &dense, &grid, None, None, true));
        let (rev, ground_rev) = (f.revision, f.ground_revision);
        assert!(f.rebuild_if_stale(&params, &dense, &grid, None, None, true));
        assert_ne!(f.revision, rev);
        assert_eq!(f.ground_revision, ground_rev);

        let (dense2, _) = inputs(8, 0.7);
        assert!(f.rebuild_if_stale(&params, &dense2, &grid, None, None, true));
        assert_ne!(f.ground_revision, ground_rev);
    }

    /// CLOSE-V1: an unchanged world does not rebuild at all.
    #[test]
    fn close_v1_stable_inputs_do_not_rebuild() {
        let params = WorldGenParams::default();
        let (dense, grid) = inputs(8, 0.5);
        let mut f = PixelWorldTerrainField::default();
        f.rebuild_if_stale(&params, &dense, &grid, None, None, true);
        assert!(!f.rebuild_if_stale(&params, &dense, &grid, None, None, false));
    }
}

#[cfg(test)]
mod surface_water_follow_tests {
    use super::*;
    use crate::terrain::generation::WaterClass;

    const N: u32 = 96; // 3 x 3 sim chunks

    fn app_with_field() -> App {
        let n = (N * N) as usize;
        let mut sf = SurfaceWaterField::default();
        sf.allocate(0, 0, N, N, vec![0.0; n], 0.0);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<WorldGenParams>()
            .insert_resource(crate::gui::WorldPresentationMode {
                kind: crate::gui::WorldPresentationKind::PixelWorld3D,
            })
            .insert_resource(WorldGenDenseTerrainCache {
                width: N,
                height: N,
                elevation: vec![0.5; n],
                moisture: vec![0.3; n],
                temperature: vec![0.3; n],
                family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
                region_index: vec![0; n],
            })
            .insert_resource(WorldWaterGrid { width: N, height: N, classes: vec![WaterClass::Dry; n], stamp: 1 })
            .insert_resource(sf)
            .insert_resource(WaterFieldDirtyFeed::default())
            .init_resource::<PixelWorldTerrainField>()
            .add_systems(Update, sync_pixel_world_terrain_field);
        app
    }

    /// WM-5: a depth change in one chunk reaches `water[]` on the next frame, touches only that
    /// chunk (no rebuild: a sentinel in another chunk survives) and keeps `surface_at` equal to
    /// what the GPU draws (`ground + depth`).
    #[test]
    fn wm5_field_depth_change_patches_only_its_chunk_next_frame() {
        let mut app = app_with_field();
        app.update(); // first sight: full build from the field
        let (x, y) = (40u32, 5u32); // chunk (1, 0)
        let i = (y * N + x) as usize;
        let far = ((70 * N) + 70) as usize; // chunk (2, 2)
        {
            let mut f = app.world_mut().resource_mut::<PixelWorldTerrainField>();
            assert!(f.is_ready() && !f.water[i].is_finite());
            f.water[far] = 123.0; // sentinel: a rebuild or a wider patch would overwrite it
        }
        let (full_rev, ground_rev) = {
            let f = app.world().resource::<PixelWorldTerrainField>();
            (f.ground_full_revision, f.ground_revision)
        };

        {
            let mut sf = app.world_mut().resource_mut::<SurfaceWaterField>();
            let fi = sf.idx(x, y);
            sf.depth_m[fi] = 0.4; // shallow and slow: the old 0.5 m stage stamp never saw this
        }
        {
            let mut feed = app.world_mut().resource_mut::<WaterFieldDirtyFeed>();
            feed.seq += 1;
            feed.chunks = vec![IVec2::new(1, 0)];
        }
        app.update();

        let f = app.world().resource::<PixelWorldTerrainField>();
        let upm = crate::terrain::world_scale_contract::display_units_per_metre();
        assert!(f.water[i].is_finite(), "the shallow water shows");
        assert!((f.water[i] - (f.ground[i] + 0.4 * upm)).abs() < 1e-4, "surface = ground + depth");
        assert_eq!(f.surface_at(Vec2::new(x as f32 + 0.5, y as f32 + 0.5)), Some(f.water[i].max(f.ground[i])));
        assert_eq!(f.water[far], 123.0, "other chunks were not touched");
        assert_eq!(f.ground_full_revision, full_rev, "no full rebuild");
        assert_eq!(f.ground_revision, ground_rev, "ground meshes stay");
        assert_ne!(f.class[i], WaterClass::Dry);
    }
}
