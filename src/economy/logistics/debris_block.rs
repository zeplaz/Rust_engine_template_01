//! DW-1 reader: ground rubble / crater -> logistics route block book (transport domain publishes only).
//!
//! Terrain owns ground height ([`GroundDeltaBook`] / [`GroundDeltaJournal`]); this module only reads
//! them. A tile buried under more than [`DEBRIS_ROUTE_RUBBLE_M`] of rubble, or sunk deeper than
//! [`DEBRIS_ROUTE_CRATER_M`], is impassable for road / rail edges until the delta shrinks again
//! (a clearing op lowers the rubble -> the tile leaves the book). Same consumer pattern as the flood
//! book: routes read [`DebrisRouteBlockBook::blocks_key`] and dirty tiles invalidate cached paths.

use std::collections::HashSet;

use bevy::prelude::*;

use super::types::LogisticsRouteDirtyTiles;
use crate::terrain::ground_deform::{GroundDeltaBook, GroundDeltaJournal};

/// Rubble / heap height (m) at which a road or rail tile is blocked.
pub const DEBRIS_ROUTE_RUBBLE_M: f32 = 1.0;
/// Crater depth (m) at which a road or rail tile is blocked.
pub const DEBRIS_ROUTE_CRATER_M: f32 = 1.5;

/// Tile keys (`t{x}_{z}`) impassable because the ground was heaped or dug out.
#[derive(Resource, Clone, Debug, Default)]
pub struct DebrisRouteBlockBook {
    pub blocked: HashSet<String>,
    pub revision: u64,
}

impl DebrisRouteBlockBook {
    #[must_use]
    pub fn blocks_key(&self, key: &str) -> bool {
        self.blocked.contains(key)
    }

    #[must_use]
    pub fn blocks_tile(&self, x: i32, z: i32) -> bool {
        self.blocked.contains(&format!("t{x}_{z}"))
    }
}

/// Whether a net ground delta (m) closes a road / rail tile.
#[must_use]
pub fn tile_debris_impassable(delta_m: f32) -> bool {
    delta_m >= DEBRIS_ROUTE_RUBBLE_M || delta_m <= -DEBRIS_ROUTE_CRATER_M
}

fn tile_key(x: i32, z: i32) -> String {
    format!("t{x}_{z}")
}

/// Follow the ground journal and keep [`DebrisRouteBlockBook`] in step. Dirties changed tiles so
/// cached routes through them are rebuilt. Narrow: ground books in, block book + dirty keys out.
pub fn publish_debris_route_blocks_system(
    book: Option<Res<GroundDeltaBook>>,
    journal: Option<Res<GroundDeltaJournal>>,
    mut blocks: ResMut<DebrisRouteBlockBook>,
    mut dirty: Option<ResMut<LogisticsRouteDirtyTiles>>,
    mut cursor: Local<(u64, u64)>,
) {
    let (Some(book), Some(journal)) = (book, journal) else {
        return;
    };
    let (seq, epoch) = *cursor;
    if seq == journal.seq && epoch == journal.epoch {
        return;
    }
    let mut changed: Vec<String> = Vec::new();
    let incremental = epoch == journal.epoch && journal.since(seq).is_some();
    if incremental {
        let tiles: HashSet<IVec2> = journal.since(seq).into_iter().flatten().map(|c| c.tile).collect();
        for t in tiles {
            let key = tile_key(t.x, t.y);
            if tile_debris_impassable(book.delta_at(t.x, t.y)) {
                if blocks.blocked.insert(key.clone()) {
                    changed.push(key);
                }
            } else if blocks.blocked.remove(&key) {
                changed.push(key);
            }
        }
    } else {
        // New world or journal overrun: rebuild from the book (the full record).
        let mut next = HashSet::new();
        let w = book.width as usize;
        for (i, d) in book.delta_m.iter().enumerate() {
            if tile_debris_impassable(*d) {
                next.insert(tile_key((i % w.max(1)) as i32, (i / w.max(1)) as i32));
            }
        }
        changed.extend(blocks.blocked.symmetric_difference(&next).cloned());
        blocks.blocked = next;
    }
    *cursor = (journal.seq, journal.epoch);
    if changed.is_empty() {
        return;
    }
    blocks.revision = blocks.revision.wrapping_add(1);
    if let Some(dirty) = dirty.as_mut() {
        dirty.keys.extend(changed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::ground_deform::{GroundDeformOp, GroundDeformPlugin, GroundDeformQueue};
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::WorldGenDenseTerrainCache;

    #[test]
    fn debris_thresholds() {
        assert!(!tile_debris_impassable(0.9));
        assert!(tile_debris_impassable(1.0));
        assert!(!tile_debris_impassable(-1.4));
        assert!(tile_debris_impassable(-1.5));
        assert!(!tile_debris_impassable(0.0));
    }

    fn app_with_ground(w: u32, h: u32) -> App {
        let mut app = App::new();
        app.init_resource::<WorldGenParams>();
        app.add_plugins(GroundDeformPlugin);
        let n = (w * h) as usize;
        app.insert_resource(WorldGenDenseTerrainCache {
            width: w,
            height: h,
            elevation: vec![0.5; n],
            moisture: vec![0.3; n],
            temperature: vec![0.3; n],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
            region_index: vec![0; n],
        });
        app.init_resource::<DebrisRouteBlockBook>()
            .init_resource::<LogisticsRouteDirtyTiles>()
            .add_systems(
                Update,
                publish_debris_route_blocks_system.after(crate::terrain::ground_deform::GroundDeformSet::Drain),
            );
        app
    }

    #[test]
    fn deep_crater_blocks_centre_tile_and_dirties_it() {
        let mut app = app_with_ground(64, 64);
        let at = IVec2::new(30, 30);
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Bowl {
            centre_m: Vec2::new((at.x as f32 + 0.5) * 16.0, (at.y as f32 + 0.5) * 16.0),
            radius_m: 40.0,
            depth_m: 6.0,
        });
        app.update();
        let blocks = app.world().resource::<DebrisRouteBlockBook>();
        assert!(blocks.blocks_tile(30, 30), "centre of a 6 m bowl is blocked");
        assert!(!blocks.blocks_tile(5, 5), "far tile stays open");
        assert!(app.world().resource::<LogisticsRouteDirtyTiles>().keys.contains("t30_30"));
    }

    #[test]
    fn clearing_rubble_reopens_tile() {
        let mut app = app_with_ground(64, 64);
        let c = Vec2::new(20.5 * 16.0, 20.5 * 16.0);
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Heap {
            centre_m: c,
            inner_radius_m: 0.0,
            outer_radius_m: 12.0,
            volume_m3: 16.0 * 16.0 * 3.0,
        });
        app.update();
        assert!(app.world().resource::<DebrisRouteBlockBook>().blocks_tile(20, 20));
        // Level the heap off with a bowl (stand-in for a clearing crew's Transfer / dig).
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Bowl {
            centre_m: c,
            radius_m: 12.0,
            depth_m: 3.0,
        });
        app.update();
        assert!(!app.world().resource::<DebrisRouteBlockBook>().blocks_tile(20, 20));
    }
}
