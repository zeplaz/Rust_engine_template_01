//! Bounded fire extract scan-set (`scan_set = residency ∪ active ∪ warm-rim`).
//!
//! Phase 6 dirty-queue hook is deferred — empty [`FireExtractDirtyQueue`] for MVP.

use bevy::math::IVec2;
use rustc_hash::FxHashSet;

use crate::io::streaming::ChunkResidencyTable;
use crate::render::fire_chunk_runtime::{ChunkCoord, FireChunkRuntime, FireSimulationSnapshot};
use crate::render::sim_visual_extract::FIRE_VISUAL_ACTIVE_HEAT_EPS;
use crate::render::FireExtractDirtyQueue;

const MOORE_NEIGHBORS: [IVec2; 8] = [
    IVec2::new(-1, -1),
    IVec2::new(0, -1),
    IVec2::new(1, -1),
    IVec2::new(-1, 0),
    IVec2::new(1, 0),
    IVec2::new(-1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 1),
];

/// Expand `set` by one Moore neighborhood (8-connected rim).
pub fn expand_moore_rim_one(set: &mut FxHashSet<ChunkCoord>) {
    let seeds: Vec<ChunkCoord> = set.iter().copied().collect();
    for c in seeds {
        for d in MOORE_NEIGHBORS {
            set.insert(c + d);
        }
    }
}

/// Build bounded extract scan set. Returns empty set when `full_reconcile` (caller uses legacy path).
#[must_use]
pub fn build_fire_extract_scan_set(
    residency: Option<&ChunkResidencyTable>,
    runtime: &FireChunkRuntime,
    prev_snapshot: &FireSimulationSnapshot,
    dirty_queue: &FireExtractDirtyQueue,
    full_reconcile: bool,
) -> FxHashSet<ChunkCoord> {
    if full_reconcile {
        return FxHashSet::default();
    }
    let mut set = FxHashSet::default();
    if let Some(table) = residency {
        for coord in table.entries.keys() {
            set.insert(*coord);
        }
    }
    for (coord, c) in &runtime.chunks {
        if c.active || c.visual_active || c.dirty {
            set.insert(*coord);
        }
    }
    for h in &prev_snapshot.chunk_heat {
        if h.heat > FIRE_VISUAL_ACTIVE_HEAT_EPS {
            set.insert(h.chunk);
        }
    }
    for coord in &dirty_queue.coords {
        set.insert(*coord);
    }
    expand_moore_rim_one(&mut set);
    set
}

/// Domain for neighbor glow / rim decay on bounded ticks.
#[must_use]
pub fn fire_extract_glow_domain(scan_set: &FxHashSet<ChunkCoord>) -> FxHashSet<ChunkCoord> {
    let mut domain = scan_set.clone();
    expand_moore_rim_one(&mut domain);
    domain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::streaming::{ChunkResidencyEntry, ChunkResidencyRole, ChunkResidencyTable};

    #[test]
    fn scan_set_includes_residency_and_active_runtime() {
        let mut residency = ChunkResidencyTable::default();
        residency
            .entries
            .insert(
                ChunkCoord::new(0, 0),
                ChunkResidencyEntry {
                    coord: ChunkCoord::new(0, 0),
                    role: ChunkResidencyRole::Core,
                    orb_priority: 0,
                },
            );
        residency.entries.insert(
            ChunkCoord::new(1, 0),
            ChunkResidencyEntry {
                coord: ChunkCoord::new(1, 0),
                role: ChunkResidencyRole::Core,
                orb_priority: 0,
            },
        );
        let mut runtime = FireChunkRuntime::default();
        runtime.chunks.insert(
            ChunkCoord::new(5, 5),
            crate::render::fire_chunk_runtime::FireChunk {
                coord: ChunkCoord::new(5, 5),
                active: true,
                visual_active: true,
                ..Default::default()
            },
        );
        let sim = FireSimulationSnapshot::default();
        let dirty = FireExtractDirtyQueue::default();
        let set = build_fire_extract_scan_set(Some(&residency), &runtime, &sim, &dirty, false);
        assert!(set.contains(&ChunkCoord::new(0, 0)));
        assert!(set.contains(&ChunkCoord::new(5, 5)));
        assert!(set.len() >= 4, "Moore rim expands scan set");
    }

    #[test]
    fn full_reconcile_returns_empty_scan_set_sentinel() {
        let set = build_fire_extract_scan_set(None, &FireChunkRuntime::default(), &FireSimulationSnapshot::default(), &FireExtractDirtyQueue::default(), true);
        assert!(set.is_empty());
    }
}
