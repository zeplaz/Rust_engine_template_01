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

/// Publish flood block + readiness books from [`SurfaceWaterField`]. Dirties logistics tiles on change.
pub fn publish_flood_consequences_system(
    field: Option<Res<SurfaceWaterField>>,
    mut blocks: ResMut<FloodRouteBlockBook>,
    mut readiness: ResMut<FloodSiteReadinessBook>,
    mut dirty: Option<ResMut<LogisticsRouteDirtyTiles>>,
    directory: Option<Res<TransportEdgeDirectory>>,
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
        return;
    };

    let mut next_blocked = HashSet::new();
    let mut next_depth: HashMap<(i32, i32), f32> = HashMap::new();
    let dx = field.dx_m.max(1.0);
    for lz in 0..field.height {
        for lx in 0..field.width {
            let i = field.idx(lx, lz);
            let depth = field.depth_m.get(i).copied().unwrap_or(0.0);
            if depth < FLOOD_ROUTE_DEPTH_M * 0.25 {
                continue;
            }
            let wx = field.origin_x + lx as i32;
            let wz = field.origin_z + lz as i32;
            let qx = field.qx_m3s.get(i).copied().unwrap_or(0.0);
            let qy = field.qy_m3s.get(i).copied().unwrap_or(0.0);
            let speed = cell_speed_mps(qx, qy, depth, dx);
            if depth > 0.05 {
                next_depth.insert((wx, wz), depth);
            }
            if tile_flood_impassable(depth, speed) {
                next_blocked.insert(format!("t{wx}_{wz}"));
            }
        }
    }

    if let Some(ref mut dirty) = dirty {
        for key in blocks.blocked.difference(&next_blocked) {
            dirty.keys.insert(key.clone());
        }
        for key in next_blocked.difference(&blocks.blocked) {
            dirty.keys.insert(key.clone());
        }
        // Also mark edge endpoints that sit on flooded tiles (directory may use keys).
        if let Some(dir) = directory.as_ref() {
            for meta in dir.by_edge.values() {
                if next_blocked.contains(&meta.head_key) || next_blocked.contains(&meta.tail_key) {
                    dirty.keys.insert(meta.head_key.clone());
                    dirty.keys.insert(meta.tail_key.clone());
                }
            }
        }
    }

    if next_blocked != blocks.blocked {
        blocks.blocked = next_blocked;
        blocks.revision = blocks.revision.wrapping_add(1);
    }
    if next_depth != readiness.depth_by_tile {
        readiness.depth_by_tile = next_depth;
        readiness.revision = readiness.revision.wrapping_add(1);
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
}
