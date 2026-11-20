//! DF-9 — flood → logistics block book + site readiness requests (water publishes only).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::economy::logistics::types::LogisticsRouteDirtyTiles;
use crate::systems::transport::TransportEdgeDirectory;

use super::surface_water::SurfaceWaterField;

/// Depth (m) at which roads/rail become impassable.
pub const FLOOD_ROUTE_DEPTH_M: f32 = 0.5;
/// Depth-averaged speed proxy (m/s) that also blocks routes.
pub const FLOOD_ROUTE_SPEED_MPS: f32 = 2.0;

/// Water-domain publish: tile keys (`t{x}_{z}`) impassable due to flood.
#[derive(Resource, Clone, Debug, Default)]
pub struct FloodRouteBlockBook {
    pub blocked: HashSet<String>,
    pub revision: u64,
}

impl FloodRouteBlockBook {
    #[must_use]
    pub fn blocks_key(&self, key: &str) -> bool {
        self.blocked.contains(key)
    }

    #[must_use]
    pub fn blocks_tile(&self, x: i32, z: i32) -> bool {
        self.blocked.contains(&format!("t{x}_{z}"))
    }
}

/// Water-domain publish: max flood depth under world tiles (site owner applies readiness).
#[derive(Resource, Clone, Debug, Default)]
pub struct FloodSiteReadinessBook {
    pub depth_by_tile: HashMap<(i32, i32), f32>,
    pub revision: u64,
}

impl FloodSiteReadinessBook {
    #[must_use]
    pub fn max_depth_on_tiles(&self, tiles: &[(i32, i32)]) -> f32 {
        tiles
            .iter()
            .filter_map(|t| self.depth_by_tile.get(t).copied())
            .fold(0.0_f32, f32::max)
    }
}

#[must_use]
pub fn tile_flood_impassable(depth_m: f32, speed_mps: f32) -> bool {
    depth_m >= FLOOD_ROUTE_DEPTH_M || speed_mps >= FLOOD_ROUTE_SPEED_MPS
}

/// Depth-averaged speed proxy from cell flux and depth (m/s).
#[must_use]
pub fn cell_speed_mps(qx_m3s: f32, qy_m3s: f32, depth_m: f32, dx_m: f32) -> f32 {
    let h = depth_m.max(1e-3);
    let q = qx_m3s.abs().max(qy_m3s.abs());
    // q is m³/s across a face ≈ (speed · depth · dx); invert with tile width.
    (q / (h * dx_m.max(1.0))).clamp(0.0, 50.0)
}

/// What the publisher already put in the books, per cell, so a step only touches the cells of the
/// tiles the solver moved water in. System-local state: the books stay plain published data.
#[derive(Default)]
pub struct FloodPublishTrack {
    shape: (i32, i32, u32, u32),
    /// Depth published into [`FloodSiteReadinessBook`] per cell (0 = no entry).
    depth: Vec<f32>,
    /// Cell currently in [`FloodRouteBlockBook`].
    blocked: Vec<bool>,
    /// Published cells per activity tile; tiles with any stay candidates until they drain.
    live_cells: Vec<u32>,
    live_tiles: Vec<u32>,
    /// Scratch: tile already queued this run.
    queued: Vec<bool>,
    candidates: Vec<u32>,
}

impl FloodPublishTrack {
    fn reset_for(&mut self, field: &SurfaceWaterField) {
        let n = field.cell_count();
        let nt = field.water_tile_count();
        self.shape = (field.origin_x, field.origin_z, field.width, field.height);
        self.depth = vec![0.0; n];
        self.blocked = vec![false; n];
        self.live_cells = vec![0; nt];
        self.live_tiles.clear();
        self.queued = vec![false; nt];
        self.candidates.clear();
    }

    fn matches(&self, field: &SurfaceWaterField) -> bool {
        self.shape == (field.origin_x, field.origin_z, field.width, field.height)
            && self.depth.len() == field.cell_count()
            && self.live_cells.len() == field.water_tile_count()
    }

    fn queue(&mut self, t: u32) {
        if let Some(q) = self.queued.get_mut(t as usize) {
            if !*q {
                *q = true;
                self.candidates.push(t);
            }
        }
    }
}

fn tile_key(wx: i32, wz: i32) -> String {
    format!("t{wx}_{wz}")
}

/// Publish flood block + readiness books from [`SurfaceWaterField`]. Incremental: only the tiles the
/// last solver step touched (plus tiles that still hold published cells, so a drained tile
/// clears) are rescanned; String keys are built only when a cell flips. Dirties logistics tiles
/// on change.
pub fn publish_flood_consequences_system(
    field: Option<Res<SurfaceWaterField>>,
    mut blocks: ResMut<FloodRouteBlockBook>,
    mut readiness: ResMut<FloodSiteReadinessBook>,
    mut dirty: Option<ResMut<LogisticsRouteDirtyTiles>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    mut track: Local<FloodPublishTrack>,
) {
    let Some(field) = field.filter(|f| f.wired) else {
        if !blocks.blocked.is_empty() || !readiness.depth_by_tile.is_empty() {
            if let Some(ref mut dirty) = dirty {
                for key in blocks.blocked.drain() {
                    dirty.keys.insert(key);
                }
            } else {
                blocks.blocked.clear();
            }
            readiness.depth_by_tile.clear();
            blocks.revision = blocks.revision.wrapping_add(1);
            readiness.revision = readiness.revision.wrapping_add(1);
        }
        *track = FloodPublishTrack::default();
        return;
    };

    // Candidate tiles: everything on the first run / a new field shape, else the tiles the solver
    // stepped last plus the tiles still holding published cells.
    if !track.matches(&field) {
        if !blocks.blocked.is_empty() || !readiness.depth_by_tile.is_empty() {
            if let Some(ref mut dirty) = dirty {
                dirty.keys.extend(blocks.blocked.iter().cloned());
            }
            blocks.blocked.clear();
            readiness.depth_by_tile.clear();
            blocks.revision = blocks.revision.wrapping_add(1);
            readiness.revision = readiness.revision.wrapping_add(1);
        }
        track.reset_for(&field);
        for t in 0..field.water_tile_count() as u32 {
            track.queue(t);
        }
    } else {
        for &t in field.stepped_water_tiles() {
            track.queue(t);
        }
        for k in 0..track.live_tiles.len() {
            let t = track.live_tiles[k];
            track.queue(t);
        }
    }

    let dx = field.dx_m.max(1.0);
    let w = field.width as usize;
    let mut flipped: Vec<(i32, i32)> = Vec::new();
    let mut depth_changed = false;
    let candidates = std::mem::take(&mut track.candidates);
    for &t in &candidates {
        let t = t as usize;
        track.queued[t] = false;
        let (x0, z0, x1, z1) = field.water_tile_rect(t);
        for lz in z0..z1 {
            for lx in x0..x1 {
                let i = lz * w + lx;
                let d = field.depth_m.get(i).copied().unwrap_or(0.0);
                let (mut want_depth, mut want_block) = (0.0_f32, false);
                if d >= FLOOD_ROUTE_DEPTH_M * 0.25 {
                    if d > 0.05 {
                        want_depth = d;
                    }
                    let qx = field.qx_m3s.get(i).copied().unwrap_or(0.0);
                    let qy = field.qy_m3s.get(i).copied().unwrap_or(0.0);
                    want_block = tile_flood_impassable(d, cell_speed_mps(qx, qy, d, dx));
                }
                let (wx, wz) = (field.origin_x + lx as i32, field.origin_z + lz as i32);
                if want_depth != track.depth[i] {
                    let had = track.depth[i] > 0.0;
                    track.depth[i] = want_depth;
                    if want_depth > 0.0 {
                        readiness.depth_by_tile.insert((wx, wz), want_depth);
                        if !had {
                            track.live_cells[t] += 1;
                        }
                    } else {
                        readiness.depth_by_tile.remove(&(wx, wz));
                        track.live_cells[t] -= 1;
                    }
                    depth_changed = true;
                }
                if want_block != track.blocked[i] {
                    track.blocked[i] = want_block;
                    if want_block {
                        blocks.blocked.insert(tile_key(wx, wz));
                    } else {
                        blocks.blocked.remove(&tile_key(wx, wz));
                    }
                    flipped.push((wx, wz));
                }
            }
        }
    }
    track.candidates = candidates;
    track.candidates.clear();
    // A tile stays a candidate while it still publishes depth (a drained tile clears next step).
    track.live_tiles.clear();
    for t in 0..track.live_cells.len() {
        if track.live_cells[t] > 0 {
            track.live_tiles.push(t as u32);
        }
    }

    if depth_changed {
        readiness.revision = readiness.revision.wrapping_add(1);
    }
    if flipped.is_empty() {
        return;
    }
    blocks.revision = blocks.revision.wrapping_add(1);
    if let Some(ref mut dirty) = dirty {
        let changed: HashSet<String> = flipped.iter().map(|(x, z)| tile_key(*x, *z)).collect();
        dirty.keys.extend(changed.iter().cloned());
        // Also the far endpoints of edges that touch a changed tile (directory keys).
        if let Some(dir) = directory.as_ref() {
            for meta in dir.by_edge.values() {
                if changed.contains(&meta.head_key) || changed.contains(&meta.tail_key) {
                    dirty.keys.insert(meta.head_key.clone());
                    dirty.keys.insert(meta.tail_key.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flooded_road_blocks_route_thresholds() {
        assert!(!tile_flood_impassable(0.2, 0.5));
        assert!(tile_flood_impassable(0.5, 0.0));
        assert!(tile_flood_impassable(0.1, 2.0));
        let mut book = FloodRouteBlockBook::default();
        book.blocked.insert("t3_7".into());
        assert!(book.blocks_tile(3, 7));
        assert!(!book.blocks_tile(4, 7));
    }

    fn flood_app(field: SurfaceWaterField) -> App {
        use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(1),
            TransportEdgeMeta {
                profile: "default_road".into(),
                head_key: "t4_3".into(),
                tail_key: "t5_3".into(),
                ..Default::default()
            },
        );
        let mut app = App::new();
        app.insert_resource(field)
            .insert_resource(dir)
            .init_resource::<FloodRouteBlockBook>()
            .init_resource::<FloodSiteReadinessBook>()
            .init_resource::<LogisticsRouteDirtyTiles>()
            .add_systems(Update, publish_flood_consequences_system);
        app
    }

    fn road_blocked(app: &App) -> bool {
        use crate::systems::transport::TransportEdgeId;
        crate::economy::logistics::routes::flood_blocks_route_edge(
            app.world().get_resource::<FloodRouteBlockBook>(),
            app.world().get_resource::<TransportEdgeDirectory>(),
            TransportEdgeId(1),
        )
    }

    /// A static river is in the one field now: where it is deep it blocks the road that fords it,
    /// and when it dries the block lifts. Only changed tiles are dirtied (incremental).
    #[test]
    fn static_river_cell_in_field_blocks_the_road_until_dry() {
        let stamp = super::super::WorldStaticWaterDepth {
            width: 32,
            height: 32,
            depth_m: (0..32 * 32).map(|i| if i % 32 == 5 { 2.5 } else { 0.0 }).collect(),
            stamp: 0,
        };
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 32, 32, vec![0.0; 32 * 32], 0.0);
        assert!(field.seed_from_static_depth(&stamp) > 0);
        let mut app = flood_app(field);
        app.update();
        assert!(road_blocked(&app), "road onto the river cell t5_3 is blocked");
        assert!(app.world().resource::<FloodRouteBlockBook>().blocks_tile(5, 3));
        assert!(!app.world().resource::<FloodRouteBlockBook>().blocks_tile(4, 3), "dry bank open");
        assert!(app.world().resource::<LogisticsRouteDirtyTiles>().keys.contains("t5_3"));

        // Nothing moved: no new dirtying, same blocks.
        app.world_mut().resource_mut::<LogisticsRouteDirtyTiles>().keys.clear();
        let rev = app.world().resource::<FloodRouteBlockBook>().revision;
        app.update();
        assert!(app.world().resource::<LogisticsRouteDirtyTiles>().keys.is_empty());
        assert_eq!(app.world().resource::<FloodRouteBlockBook>().revision, rev);

        // The river dries: the cell leaves the book even though no solver step listed its tile.
        app.world_mut().resource_mut::<SurfaceWaterField>().depth_m.fill(0.0);
        app.update();
        assert!(!road_blocked(&app));
        assert!(app.world().resource::<FloodRouteBlockBook>().blocked.is_empty());
        assert!(app.world().resource::<FloodSiteReadinessBook>().depth_by_tile.is_empty());
    }

    /// Depth under the block threshold is readiness data only; fast shallow water still blocks.
    #[test]
    fn shallow_slow_water_does_not_block_but_is_published_as_depth() {
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 16, 16, vec![0.0; 256], 0.0);
        let i = field.idx(3, 3);
        field.depth_m[i] = 0.2;
        let mut app = flood_app(field);
        app.update();
        assert!(app.world().resource::<FloodRouteBlockBook>().blocked.is_empty());
        assert_eq!(app.world().resource::<FloodSiteReadinessBook>().depth_by_tile.get(&(3, 3)), Some(&0.2));
    }
}
