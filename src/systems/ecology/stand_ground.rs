//! DW-1 reader: craters and rubble heaps destroy the stand on the tiles they cover (ecology domain).
//!
//! Terrain owns ground height ([`GroundDeltaBook`] / [`GroundDeltaJournal`]); this system only reads
//! them and is the sole writer of the "cleared" mark on [`ChunkCellStand`]. A cleared cell is bare,
//! carries no fuel and is skipped by capacity refresh, so it does not regrow while the ground stays
//! deformed (the stand is gone, not just burnt).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::chunk::Chunk;
use crate::terrain::ground_deform::{GroundDeltaBook, GroundDeltaJournal};

use super::cell_stand::ChunkCellStand;

/// Net ground change (m, either sign) beyond which a tile carries no live stand.
pub const STAND_CLEAR_DELTA_M: f32 = 0.25;

/// Whether a net ground delta destroys the stand on that tile.
#[must_use]
pub fn ground_delta_clears_stand(delta_m: f32) -> bool {
    delta_m.abs() >= STAND_CLEAR_DELTA_M
}

/// Follow the ground journal (and chunks whose stand was just spawned) and clear stands on deformed tiles.
pub fn stand_ground_deform_system(
    book: Option<Res<GroundDeltaBook>>,
    journal: Option<Res<GroundDeltaJournal>>,
    mut cursor: Local<(u64, u64)>,
    mut q: Query<(&Chunk, &ChunkCellMatrix, &mut ChunkCellStand)>,
) {
    let (Some(book), Some(journal)) = (book, journal) else {
        return;
    };
    let journal_moved = cursor.0 != journal.seq || cursor.1 != journal.epoch;
    let fresh_tiles: Option<HashSet<IVec2>> = if journal_moved && cursor.1 == journal.epoch {
        journal
            .since(cursor.0)
            .map(|cells| cells.map(|c| c.tile).collect())
    } else {
        None
    };
    *cursor = (journal.seq, journal.epoch);
    for (chunk, matrix, mut stand) in &mut q {
        let (sx, sy) = (matrix.size.x as i32, matrix.size.y as i32);
        if sx <= 0 || sy <= 0 || stand.kind.len() != (sx * sy) as usize {
            continue;
        }
        let origin = IVec2::new(chunk.coord.x * sx, chunk.coord.y * sy);
        let hit = |stand: &mut ChunkCellStand, t: IVec2| {
            let (lx, ly) = (t.x - origin.x, t.y - origin.y);
            if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
                return;
            }
            let i = (ly * sx + lx) as usize;
            if !stand.is_cleared(i) && ground_delta_clears_stand(book.delta_at(t.x, t.y)) {
                stand.clear_cell(i);
            }
        };
        match (&fresh_tiles, stand.is_added()) {
            // Incremental: only tiles the journal touched since the last run.
            (Some(tiles), false) => {
                if !tiles.is_empty() {
                    for &t in tiles {
                        hit(&mut stand, t);
                    }
                }
            }
            // New stand, new world, or journal overrun: reconcile the whole chunk with the book.
            _ => {
                if !journal_moved && !stand.is_added() {
                    continue;
                }
                for ly in 0..sy {
                    for lx in 0..sx {
                        hit(&mut stand, origin + IVec2::new(lx, ly));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::cell_stand::StandKind;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::WorldGenDenseTerrainCache;
    use crate::terrain::ground_deform::{
        GroundDeformOp, GroundDeformPlugin, GroundDeformQueue, GroundDeformSet,
    };

    const TILE: f32 = 16.0;

    fn app_with_chunk() -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<WorldGenParams>();
        app.add_plugins(GroundDeformPlugin);
        let n = 64 * 64;
        app.insert_resource(WorldGenDenseTerrainCache {
            width: 64,
            height: 64,
            elevation: vec![0.5; n],
            moisture: vec![0.3; n],
            temperature: vec![0.3; n],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
            region_index: vec![0; n],
        });
        app.add_systems(Update, stand_ground_deform_system.after(GroundDeformSet::Drain));
        let mut stand = ChunkCellStand::new(32 * 32);
        for i in 0..32 * 32 {
            stand.kind[i] = StandKind::Grass;
            stand.full_cap[i] = 0.5;
            stand.fuel_cap[i] = 0.5;
        }
        let e = app
            .world_mut()
            .spawn((Chunk { coord: IVec2::ZERO }, ChunkCellMatrix::new(UVec2::new(32, 32)), stand))
            .id();
        (app, e)
    }

    fn centre(x: i32, y: i32) -> Vec2 {
        Vec2::new((x as f32 + 0.5) * TILE, (y as f32 + 0.5) * TILE)
    }

    #[test]
    fn crater_clears_stand_on_dug_tiles_only() {
        let (mut app, e) = app_with_chunk();
        app.update();
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Bowl {
            centre_m: centre(10, 10),
            radius_m: 40.0,
            depth_m: 6.0,
        });
        app.update();
        let s = app.world().get::<ChunkCellStand>(e).unwrap();
        let c = 10 * 32 + 10;
        assert!(s.is_cleared(c));
        assert_eq!(s.kind[c], StandKind::Bare);
        assert_eq!(s.fuel_cap[c], 0.0);
        assert_eq!(s.kind[25 * 32 + 25], StandKind::Grass, "far cell keeps its stand");
    }

    #[test]
    fn rubble_heap_clears_stand_but_a_dusting_does_not() {
        let (mut app, e) = app_with_chunk();
        app.update();
        let mut q = app.world_mut().resource_mut::<GroundDeformQueue>();
        q.push(GroundDeformOp::Heap {
            centre_m: centre(5, 5),
            inner_radius_m: 0.0,
            outer_radius_m: 12.0,
            volume_m3: TILE * TILE * 2.0,
        });
        q.push(GroundDeformOp::Heap {
            centre_m: centre(20, 20),
            inner_radius_m: 0.0,
            outer_radius_m: 12.0,
            volume_m3: TILE * TILE * 0.05,
        });
        app.update();
        let s = app.world().get::<ChunkCellStand>(e).unwrap();
        assert!(s.is_cleared(5 * 32 + 5), "2 m rubble buries the stand");
        assert!(!s.is_cleared(20 * 32 + 20), "5 cm of dust is not destruction");
        assert_eq!(s.kind[20 * 32 + 20], StandKind::Grass);
    }

    #[test]
    fn stand_spawned_after_the_blast_is_reconciled_from_the_book() {
        let (mut app, e) = app_with_chunk();
        app.update();
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Bowl {
            centre_m: centre(10, 10),
            radius_m: 40.0,
            depth_m: 6.0,
        });
        app.update();
        app.world_mut().entity_mut(e).despawn();
        let mut stand = ChunkCellStand::new(32 * 32);
        stand.kind.iter_mut().for_each(|k| *k = StandKind::Grass);
        app.world_mut()
            .spawn((Chunk { coord: IVec2::ZERO }, ChunkCellMatrix::new(UVec2::new(32, 32)), stand));
        app.update();
        let mut q = app.world_mut().query::<&ChunkCellStand>();
        let s = q.single(app.world()).unwrap();
        assert!(s.is_cleared(10 * 32 + 10));
    }

    #[test]
    fn delta_threshold_is_symmetric() {
        assert!(ground_delta_clears_stand(-0.3));
        assert!(ground_delta_clears_stand(0.3));
        assert!(!ground_delta_clears_stand(0.1));
    }
}
