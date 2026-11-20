//! DW-1 -- the ground delta layer. Terrain is the single writer of ground height after world gen.
//!
//! Producers (blast, later demolish / dam / water) push [`GroundDeformOp`]s onto
//! [`GroundDeformQueue`]; [`drain_ground_deform_system`] is the **only** system that applies them.
//!
//! Where the delta lands:
//! * the terrain truth every sim reader already uses -- [`WorldGenDenseTerrainCache::elevation`]
//!   and each chunk's [`ChunkCellMatrix::elevation`] (normalised, `metres / elevation_range_m`) --
//!   so fire / ecology / conform / a freshly seeded water ROI see the deformed ground with no
//!   per-reader change and chunk re-hydration keeps it;
//! * [`GroundDeltaBook`]: the delta itself in metres (the replayable record; the voxel field draws
//!   `terraced DEM + delta` so a few-metre crater is not swallowed by the 8 m height terrace);
//! * [`GroundDeltaJournal`]: what was applied, with a sequence cursor, for readers that follow
//!   incrementally (PixelWorld field ground, water solver bed). Readers never write back.
//!
//! The dense cache is mutated through `bypass_change_detection`: its `is_changed` flag is the
//! "world was (re)generated" trigger for a full field rebuild, and a deformation is announced
//! through the journal instead so a blast never costs a full-world rebuild.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WorldGenDenseTerrainCache};
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

const TILE_M: f32 = TILE_METRES_GAMEPLAY;
const TILE_AREA_M2: f32 = TILE_M * TILE_M;
/// Requests held while the dense terrain is not live yet.
const MAX_PENDING_OPS: usize = 256;
/// Applied cells kept for incremental readers (a large strike is ~150 cells).
const JOURNAL_MAX_CELLS: usize = 1 << 16;
/// Ejecta rim extent relative to the crater radius.
pub const CRATER_RIM_OUTER_FACTOR: f32 = 1.6;

/// One terrain-height change request. Metres everywhere (world XZ metres, not tiles).
#[derive(Clone, Debug, PartialEq)]
pub enum GroundDeformOp {
    /// Dig a paraboloid bowl; the spoil is discarded (use [`Self::Crater`] for a move).
    Bowl { centre_m: Vec2, radius_m: f32, depth_m: f32 },
    /// Raise `volume_m3` of material over the annulus `inner_radius_m..outer_radius_m`
    /// (`inner = 0` makes a heap, `inner > 0` a rim / lip).
    Heap { centre_m: Vec2, inner_radius_m: f32, outer_radius_m: f32, volume_m3: f32 },
    /// Bowl of `depth_m` plus an ejecta rim from `radius_m` to
    /// [`CRATER_RIM_OUTER_FACTOR`] x `radius_m` holding exactly the dug volume.
    Crater { centre_m: Vec2, radius_m: f32, depth_m: f32 },
    /// Move `depth_m` of loose material from one tile to a neighbour (DW-5 wash; volume kept).
    Transfer { from: IVec2, to: IVec2, depth_m: f32 },
}

/// Pending deformation, drained by [`drain_ground_deform_system`] (the sole consumer).
#[derive(Resource, Default, Debug)]
pub struct GroundDeformQueue {
    pending: Vec<GroundDeformOp>,
}

impl GroundDeformQueue {
    pub fn push(&mut self, op: GroundDeformOp) {
        self.pending.push(op);
        if self.pending.len() > MAX_PENDING_OPS {
            self.pending.remove(0);
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Accumulated height delta per tile in metres on top of the authored DEM (row-major `y * w + x`).
#[derive(Resource, Default, Debug)]
pub struct GroundDeltaBook {
    pub width: u32,
    pub height: u32,
    pub delta_m: Vec<f32>,
    /// Bumps on every applied op.
    pub revision: u64,
}

impl GroundDeltaBook {
    #[must_use]
    pub fn delta_at(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return 0.0;
        }
        self.delta_m
            .get(y as usize * self.width as usize + x as usize)
            .copied()
            .unwrap_or(0.0)
    }

    /// Net material moved in or out of the world (m3). ~0 for a conserving crater.
    #[must_use]
    pub fn net_volume_m3(&self) -> f64 {
        self.delta_m.iter().map(|d| f64::from(*d)).sum::<f64>() * f64::from(TILE_AREA_M2)
    }

    fn ensure_size(&mut self, w: u32, h: u32) {
        if self.width != w || self.height != h || self.delta_m.len() != (w as usize) * (h as usize) {
            self.width = w;
            self.height = h;
            self.delta_m = vec![0.0; (w as usize) * (h as usize)];
        }
    }
}

/// One applied change: `delta_m` added to `tile` by journal entry `seq`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundDeltaCell {
    pub seq: u64,
    pub tile: IVec2,
    pub delta_m: f32,
}

/// What the drain applied, oldest first. Readers keep a `seq` cursor and take `seq > cursor`.
#[derive(Resource, Default, Debug)]
pub struct GroundDeltaJournal {
    /// Highest sequence written.
    pub seq: u64,
    /// Bumps when the book was reset (world regenerated) -- readers must resync from the book.
    pub epoch: u64,
    /// Sequences `<= dropped_through` are no longer in `cells`.
    pub dropped_through: u64,
    pub cells: Vec<GroundDeltaCell>,
}

impl GroundDeltaJournal {
    /// Cells newer than `cursor`, or `None` when the journal no longer covers that far back.
    #[must_use]
    pub fn since(&self, cursor: u64) -> Option<impl Iterator<Item = &GroundDeltaCell>> {
        (cursor >= self.dropped_through).then(|| self.cells.iter().filter(move |c| c.seq > cursor))
    }

    fn push_op(&mut self, cells: &[(IVec2, f32)]) {
        self.seq += 1;
        let seq = self.seq;
        self.cells.extend(
            cells
                .iter()
                .map(|&(tile, delta_m)| GroundDeltaCell { seq, tile, delta_m }),
        );
        if self.cells.len() > JOURNAL_MAX_CELLS {
            let drop = self.cells.len() - JOURNAL_MAX_CELLS;
            self.dropped_through = self.cells[drop - 1].seq;
            self.cells.drain(..drop);
        }
    }
}

/// Ordering handle: producers run `.before(Drain)`, readers `.after(Drain)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundDeformSet {
    Drain,
}

/// Crater bowl depth (m) from blast damage and radius: ~16% of the radius for a medium strike
/// (damage 120), shallower for weak hits, deeper for heavy ones, capped at 12 m.
#[must_use]
pub fn crater_depth_m(damage: f32, radius_m: f32) -> f32 {
    (0.16 * radius_m.max(0.0) * (damage / 120.0).clamp(0.4, 1.3)).clamp(0.0, 12.0)
}

#[inline]
fn tile_centre(t: IVec2) -> Vec2 {
    Vec2::new((t.x as f32 + 0.5) * TILE_M, (t.y as f32 + 0.5) * TILE_M)
}

/// Tiles whose centre lies in `r_lo..=r_hi` of `centre`, with their distance. `always_centre`
/// adds the tile under `centre` even when its centre is outside the disk (tiny radii).
fn ring_tiles(centre: Vec2, r_lo: f32, r_hi: f32, dims: UVec2, always_centre: bool) -> Vec<(IVec2, f32)> {
    let c_tile = IVec2::new((centre.x / TILE_M).floor() as i32, (centre.y / TILE_M).floor() as i32);
    let reach = (r_hi / TILE_M).ceil() as i32 + 1;
    let mut out = Vec::new();
    for dz in -reach..=reach {
        for dx in -reach..=reach {
            let t = c_tile + IVec2::new(dx, dz);
            if t.x < 0 || t.y < 0 || t.x as u32 >= dims.x || t.y as u32 >= dims.y {
                continue;
            }
            let d = tile_centre(t).distance(centre);
            if (d >= r_lo && d <= r_hi) || (always_centre && dx == 0 && dz == 0) {
                out.push((t, d));
            }
        }
    }
    out
}

/// Bowl profile: 1 at the centre, 0 at `radius`.
#[inline]
fn bowl_shape(d: f32, radius: f32) -> f32 {
    (1.0 - (d / radius.max(1e-3)).powi(2)).max(0.0)
}

/// Heap / rim profile over `r0..r1`: paraboloid for a disk, one sine lobe for an annulus.
#[inline]
fn heap_shape(d: f32, r0: f32, r1: f32) -> f32 {
    if r0 <= 0.0 {
        bowl_shape(d, r1)
    } else {
        let t = ((d - r0) / (r1 - r0).max(1e-3)).clamp(0.0, 1.0);
        (std::f32::consts::PI * t).sin().max(0.0)
    }
}

fn bowl_cells(centre: Vec2, radius: f32, depth: f32, dims: UVec2) -> Vec<(IVec2, f32)> {
    ring_tiles(centre, 0.0, radius, dims, true)
        .into_iter()
        .map(|(t, d)| (t, -depth * bowl_shape(d, radius).max(0.05)))
        .collect()
}

fn heap_cells(centre: Vec2, r0: f32, r1: f32, volume_m3: f32, dims: UVec2) -> Vec<(IVec2, f32)> {
    let r1 = r1.max(r0 + 0.5);
    let mut tiles = ring_tiles(centre, r0, r1, dims, false);
    if tiles.is_empty() {
        // Thinner than a tile: drop it on the tile at the ring's mid radius.
        let mid = centre + Vec2::X * 0.5 * (r0 + r1);
        tiles = ring_tiles(mid, 0.0, 0.0, dims, true);
        for (_, d) in &mut tiles {
            *d = 0.5 * (r0 + r1);
        }
    }
    let weights: Vec<f32> = tiles
        .iter()
        .map(|&(_, d)| if r0 <= 0.0 { heap_shape(d, 0.0, r1).max(0.05) } else { heap_shape(d, r0, r1).max(0.02) })
        .collect();
    let wsum: f32 = weights.iter().sum();
    if wsum <= 0.0 || volume_m3 == 0.0 {
        return Vec::new();
    }
    let k = volume_m3 / (wsum * TILE_AREA_M2);
    tiles.iter().zip(&weights).map(|(&(t, _), w)| (t, k * w)).collect()
}

/// Per-tile metre deltas of an op on a `dims` grid. Pure; the volume a crater digs is exactly
/// the volume its rim raises (both sums are over the same in-grid cells).
#[must_use]
pub fn op_cells(op: &GroundDeformOp, dims: UVec2) -> Vec<(IVec2, f32)> {
    let inside = |t: IVec2| t.x >= 0 && t.y >= 0 && (t.x as u32) < dims.x && (t.y as u32) < dims.y;
    match *op {
        GroundDeformOp::Transfer { from, to, depth_m } => {
            if !inside(from) || !inside(to) || depth_m <= 0.0 {
                return Vec::new();
            }
            vec![(from, -depth_m), (to, depth_m)]
        }
        GroundDeformOp::Bowl { centre_m, radius_m, depth_m } => {
            bowl_cells(centre_m, radius_m.max(0.5), depth_m.max(0.0), dims)
        }
        GroundDeformOp::Heap { centre_m, inner_radius_m, outer_radius_m, volume_m3 } => {
            heap_cells(centre_m, inner_radius_m.max(0.0), outer_radius_m, volume_m3, dims)
        }
        GroundDeformOp::Crater { centre_m, radius_m, depth_m } => {
            let r = radius_m.max(0.5);
            let mut cells = bowl_cells(centre_m, r, depth_m.max(0.0), dims);
            let dug_m3: f32 = -cells.iter().map(|c| c.1).sum::<f32>() * TILE_AREA_M2;
            let outer = (r * CRATER_RIM_OUTER_FACTOR).max(r + TILE_M);
            let rim = heap_cells(centre_m, r, outer, dug_m3, dims);
            // Merge (a tile can be both bowl edge and rim edge at tiny radii).
            for (t, d) in rim {
                match cells.iter_mut().find(|c| c.0 == t) {
                    Some(c) => c.1 += d,
                    None => cells.push((t, d)),
                }
            }
            cells
        }
    }
}

/// Fold per-tile metre deltas into the terrain truth, the book and the journal.
/// Returns the delta actually applied per tile (normalised elevation is clamped to 0..1).
fn apply_cells(
    cells: &[(IVec2, f32)],
    range_m: f32,
    dense: &mut WorldGenDenseTerrainCache,
    book: &mut GroundDeltaBook,
) -> Vec<(IVec2, f32)> {
    let (w, h) = (dense.width as usize, dense.height as usize);
    let mut applied = Vec::with_capacity(cells.len());
    for &(t, d) in cells {
        if t.x < 0 || t.y < 0 || t.x as usize >= w || t.y as usize >= h || d == 0.0 {
            continue;
        }
        let i = t.y as usize * w + t.x as usize;
        let old = dense.elevation[i];
        let new = (old + d / range_m).clamp(0.0, 1.0);
        let actual_m = (new - old) * range_m;
        if actual_m == 0.0 {
            continue;
        }
        dense.elevation[i] = new;
        book.delta_m[i] += actual_m;
        applied.push((t, actual_m));
    }
    applied
}

/// Sole writer of ground height after world gen. Narrow: queue in, terrain truth + book + journal out.
pub fn drain_ground_deform_system(
    mut queue: ResMut<GroundDeformQueue>,
    mut book: ResMut<GroundDeltaBook>,
    mut journal: ResMut<GroundDeltaJournal>,
    params: Res<WorldGenParams>,
    mut dense: Option<ResMut<WorldGenDenseTerrainCache>>,
    mut chunks: Query<(&Chunk, &mut ChunkCellMatrix)>,
) {
    let Some(dense) = dense.as_mut() else {
        return;
    };
    // A (re)generated world starts with a clean delta (the new DEM has no craters).
    let regenerated = dense.is_added();
    let dense = dense.bypass_change_detection();
    if !dense.is_live_height_grid() {
        return;
    }
    if regenerated || book.width != dense.width || book.height != dense.height {
        book.ensure_size(dense.width, dense.height);
        book.delta_m.iter_mut().for_each(|d| *d = 0.0);
        journal.cells.clear();
        journal.dropped_through = journal.seq;
        journal.epoch += 1;
    }
    if queue.pending.is_empty() {
        return;
    }
    let range_m = params.elevation_range_m.max(1.0);
    let dims = UVec2::new(dense.width, dense.height);
    let ops = std::mem::take(&mut queue.pending);
    let mut touched: Vec<(IVec2, f32)> = Vec::new();
    for op in &ops {
        let cells = op_cells(op, dims);
        let applied = apply_cells(&cells, range_m, dense, &mut book);
        if applied.is_empty() {
            continue;
        }
        book.revision += 1;
        journal.push_op(&applied);
        touched.extend(applied);
    }
    if touched.is_empty() {
        return;
    }
    // Chunk matrices carry their own elevation copy: apply the same metre delta.
    for (chunk, mut matrix) in &mut chunks {
        let origin = chunk.coord * CHUNK_TILES_SIM as i32;
        for &(t, d_m) in &touched {
            let (lx, ly) = (t.x - origin.x, t.y - origin.y);
            if lx < 0 || ly < 0 || lx >= matrix.size.x as i32 || ly >= matrix.size.y as i32 {
                continue;
            }
            let i = matrix.idx(lx as u32, ly as u32);
            matrix.elevation[i] = (matrix.elevation[i] + d_m / range_m).clamp(0.0, 1.0);
        }
    }
}

pub struct GroundDeformPlugin;

impl Plugin for GroundDeformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundDeformQueue>()
            .init_resource::<GroundDeltaBook>()
            .init_resource::<GroundDeltaJournal>()
            .add_systems(Update, drain_ground_deform_system.in_set(GroundDeformSet::Drain));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIMS: UVec2 = UVec2::new(64, 64);

    fn centre_of(t: IVec2) -> Vec2 {
        tile_centre(t)
    }

    fn sum_m3(cells: &[(IVec2, f32)]) -> f32 {
        cells.iter().map(|c| c.1).sum::<f32>() * TILE_AREA_M2
    }

    #[test]
    fn dw1_crater_lowers_centre_raises_rim_and_conserves_volume() {
        for (r, depth) in [(24.0_f32, 1.8_f32), (40.0, 6.4), (64.0, 10.0)] {
            let at = IVec2::new(30, 30);
            let op = GroundDeformOp::Crater { centre_m: centre_of(at), radius_m: r, depth_m: depth };
            let cells = op_cells(&op, DIMS);
            let centre = cells.iter().find(|c| c.0 == at).expect("centre tile").1;
            assert!(centre < -0.5 * depth, "r={r}: centre {centre} must be dug");
            let rim_max = cells.iter().map(|c| c.1).fold(f32::MIN, f32::max);
            assert!(rim_max > 0.0, "r={r}: rim raised");
            let rim_cells = cells.iter().filter(|c| c.1 > 0.0).count();
            assert!(rim_cells >= 4, "r={r}: rim is a ring, got {rim_cells} cells");
            let dug: f32 = -cells.iter().filter(|c| c.1 < 0.0).map(|c| c.1).sum::<f32>() * TILE_AREA_M2;
            assert!(sum_m3(&cells).abs() <= 1e-3 * dug, "r={r}: net {} vs dug {dug}", sum_m3(&cells));
        }
    }

    #[test]
    fn dw1_crater_at_world_edge_still_conserves_in_grid_volume() {
        let op = GroundDeformOp::Crater { centre_m: Vec2::new(8.0, 8.0), radius_m: 40.0, depth_m: 5.0 };
        let cells = op_cells(&op, DIMS);
        assert!(cells.iter().all(|c| c.0.x >= 0 && c.0.y >= 0));
        let dug: f32 = -cells.iter().filter(|c| c.1 < 0.0).map(|c| c.1).sum::<f32>() * TILE_AREA_M2;
        assert!(sum_m3(&cells).abs() <= 1e-3 * dug.max(1.0));
    }

    #[test]
    fn dw1_depth_scales_with_damage_and_radius() {
        assert!(crater_depth_m(120.0, 40.0) > crater_depth_m(55.0, 24.0));
        assert!(crater_depth_m(220.0, 64.0) > crater_depth_m(120.0, 40.0));
        assert!(crater_depth_m(1e6, 1e6) <= 12.0);
    }

    fn drain_app(w: u32, h: u32, elev: f32) -> App {
        let mut app = App::new();
        app.init_resource::<WorldGenParams>();
        app.add_plugins(GroundDeformPlugin);
        let n = (w * h) as usize;
        app.insert_resource(WorldGenDenseTerrainCache {
            width: w,
            height: h,
            elevation: vec![elev; n],
            moisture: vec![0.3; n],
            temperature: vec![0.3; n],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
            region_index: vec![0; n],
        });
        app
    }

    #[test]
    fn dw1_drain_writes_dense_chunks_book_and_journal() {
        let mut app = drain_app(64, 64, 0.5);
        let mut m = ChunkCellMatrix::new(UVec2::new(32, 32));
        m.elevation.iter_mut().for_each(|e| *e = 0.5);
        let ent = app.world_mut().spawn((Chunk { coord: IVec2::new(0, 0) }, m)).id();
        let at = IVec2::new(10, 10);
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Crater {
            centre_m: centre_of(at),
            radius_m: 40.0,
            depth_m: 6.0,
        });
        app.update();
        assert!(app.world().resource::<GroundDeformQueue>().is_empty());
        let range = app.world().resource::<WorldGenParams>().elevation_range_m;
        let dense = app.world().resource::<WorldGenDenseTerrainCache>();
        let i = 10 * 64 + 10;
        assert!(dense.elevation[i] < 0.5);
        let dug_m = (0.5 - dense.elevation[i]) * range;
        assert!(dug_m > 3.0, "dug {dug_m} m");
        let book = app.world().resource::<GroundDeltaBook>();
        assert!((book.delta_at(10, 10) + dug_m).abs() < 1e-2);
        assert!(book.net_volume_m3().abs() < 1.0, "net {}", book.net_volume_m3());
        let matrix = app.world().get::<ChunkCellMatrix>(ent).unwrap();
        assert!((matrix.elevation[10 * 32 + 10] - dense.elevation[i]).abs() < 1e-6);
        let journal = app.world().resource::<GroundDeltaJournal>();
        assert_eq!(journal.seq, 1);
        assert!(journal.since(0).unwrap().any(|c| c.tile == at && c.delta_m < 0.0));
        assert!(journal.since(1).unwrap().next().is_none());
    }
}
