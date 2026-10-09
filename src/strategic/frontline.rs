//! Emergent **frontline** metrics from control-field competition (not a placed object).
//!
//! Two layers, one writer ([`derive_frontline_from_control_system`]):
//! - **Chunk layer** (every frame, legacy contract): `contested_chunks`, `stability`, `flux` — slots 0/1.
//! - **Cell layer** (PLAN-FRONT-COMBAT-v1 FC-1, when [`FrontBalance`] is present): per hostile pair
//!   front cells + oriented boundary polylines from [`crate::combat::derive_fronts`].

use bevy::prelude::*;

use crate::combat::{derive_fronts, ControlGrid, FrontBalance, FrontSegment, Hostility};
use crate::strategic::settlement::{FactionPairStance, FactionStanceBook};
use crate::strategic::ChunkStrategicOverlay;
use crate::systems::sim_control::SimTick;

/// Derived tactical contour: contested chunks where two faction slots both hold meaningful control.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontlineState {
    pub stability: f32,
    pub flux: f32,
    pub contested_chunks: Vec<IVec2>,
    /// Cell-level fronts per hostile pair (empty until the combat balance resource exists).
    pub segments: Vec<FrontSegment>,
    /// Bumped whenever `segments` is recomputed.
    pub revision: u64,
    /// Sim tick of the last cell-level derivation.
    pub derived_at_tick: u64,
}

impl FrontlineState {
    /// All front tiles across segments (a tile facing two enemies appears twice).
    pub fn front_tiles(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.segments.iter().flat_map(|s| s.cells.iter().map(|c| c.tile))
    }

    #[must_use]
    pub fn total_length_m(&self) -> f32 {
        self.segments.iter().map(|s| s.length_m).sum()
    }
}

/// Hostility from the stance book; with no `Enemy` pair anywhere, slots 0/1 stay at war (legacy front).
#[must_use]
pub fn hostility_from_stances(stances: Option<&FactionStanceBook>) -> Hostility {
    let Some(book) = stances else {
        return Hostility::legacy_pair();
    };
    let h = Hostility::from_fn(|a, b| book.stance(a, b) == FactionPairStance::Enemy);
    if h.is_empty() {
        Hostility::legacy_pair()
    } else {
        h
    }
}

/// Gather overlays into a sparse control grid (cells below `floor` dropped).
#[must_use]
pub fn control_grid_from_overlays<'a>(overlays: impl Iterator<Item = &'a ChunkStrategicOverlay>, floor: f32) -> ControlGrid {
    let mut grid = ControlGrid::default();
    for ov in overlays {
        let origin = IVec2::new(
            ov.chunk_coord.x.saturating_mul(ov.size.x as i32),
            ov.chunk_coord.y.saturating_mul(ov.size.y as i32),
        );
        grid.absorb_slab(origin, ov.size.x, &ov.faction_control, floor);
    }
    grid
}

/// Marks chunks where `faction_control[slot0]` and `faction_control[slot1]` are both significant and close;
/// on the balance cadence also derives cell-level fronts for every hostile pair.
pub fn derive_frontline_from_control_system(
    overlays: Query<&ChunkStrategicOverlay>,
    mut front: ResMut<FrontlineState>,
    balance: Option<Res<FrontBalance>>,
    stances: Option<Res<FactionStanceBook>>,
    tick: Option<Res<SimTick>>,
    mut last_derive: Local<Option<u64>>,
) {
    let mut contested = Vec::new();
    const TIE_LOW: f32 = 0.25;
    const TIE_BAND: f32 = 0.3;

    for ov in overlays.iter() {
        let mut chunk_contested = false;
        let n = ov.len_cells();
        for ci in 0..n {
            let c0 = ov.faction_control[ci][0];
            let c1 = ov.faction_control[ci][1];
            if c0 > TIE_LOW && c1 > TIE_LOW && (c0 - c1).abs() < TIE_BAND {
                chunk_contested = true;
                break;
            }
        }
        if chunk_contested {
            contested.push(ov.chunk_coord);
        }
    }

    contested.sort_by_key(|c| (c.y, c.x));
    contested.dedup();
    let k = contested.len() as f32;
    front.contested_chunks = contested;
    front.flux = (k * 0.04).min(1.0);
    front.stability = (1.0 - k * 0.02).max(0.0);

    let Some(balance) = balance else {
        return;
    };
    let now = tick.map_or(0, |t| t.0);
    let due = match *last_derive {
        None => true,
        Some(prev) => now < prev || now - prev >= balance.front.derive_every_ticks,
    };
    if !due {
        return;
    }
    *last_derive = Some(now);
    let grid = control_grid_from_overlays(overlays.iter(), balance.front.own_min);
    let hostility = hostility_from_stances(stances.as_deref());
    let derived = derive_fronts(
        &grid,
        &hostility,
        balance.front.own_min,
        balance.front.contest_band,
        balance.tile_size_m,
    );
    if derived.segments != front.segments {
        front.segments = derived.segments;
        front.revision = front.revision.wrapping_add(1);
    }
    front.derived_at_tick = now;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_overlay(ov: ChunkStrategicOverlay, with_balance: bool) -> App {
        let mut app = App::new();
        app.init_resource::<FrontlineState>()
            .add_systems(Update, derive_frontline_from_control_system);
        if with_balance {
            app.insert_resource(FrontBalance::default());
        }
        app.world_mut().spawn(ov);
        app
    }

    fn split_overlay() -> ChunkStrategicOverlay {
        let mut ov = ChunkStrategicOverlay::new(IVec2::new(1, 0), UVec2::new(4, 2));
        for ci in 0..8 {
            let x = ci % 4;
            ov.faction_control[ci][usize::from(x >= 2)] = 0.8;
        }
        ov
    }

    #[test]
    fn legacy_chunk_layer_unchanged_without_balance() {
        let mut app = app_with_overlay(split_overlay(), false);
        app.update();
        let f = app.world().resource::<FrontlineState>();
        assert!(f.contested_chunks.is_empty());
        assert!(f.segments.is_empty());
        assert_eq!(f.revision, 0);
    }

    #[test]
    fn cell_layer_derives_world_tile_front() {
        let mut app = app_with_overlay(split_overlay(), true);
        app.update();
        let f = app.world().resource::<FrontlineState>();
        assert_eq!(f.segments.len(), 1);
        assert_eq!(f.revision, 1);
        // Chunk (1,0) of width 4 ⇒ world x 4..8; boundary at x = 6.
        let p = &f.segments[0].polylines[0];
        assert_eq!(p.points, vec![IVec2::new(6, 0), IVec2::new(6, 2)]);
        assert!((f.total_length_m() - 2.0 * 25.0).abs() < 1e-4);
        // Same input → revision stays.
        app.update();
        assert_eq!(app.world().resource::<FrontlineState>().revision, 1);
    }

    #[test]
    fn stance_book_drives_hostility() {
        let mut book = FactionStanceBook::default();
        assert_eq!(hostility_from_stances(Some(&book)), Hostility::legacy_pair());
        book.set_pair(2, 5, FactionPairStance::Enemy);
        let h = hostility_from_stances(Some(&book));
        assert!(h.hostile(5, 2));
        assert!(!h.hostile(0, 1));
    }
}
