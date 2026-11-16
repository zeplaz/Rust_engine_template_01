//! # PHYS-11 — Temporal coherency, broad / mid / narrow phase (V4-PHYS11)
//!
//! Dual-cite: NEW **PHYS-11** (gea4v2 **14.3.6** *Performance Optimizations*
//! p379-380) · LEGACY keywords **`collision_detection`** (gea_v2_e4 §14.3 p359) /
//! **`physics_collision`** (gea_v2_e4 p352). Prefer gea_v2_e4 over gea_e3.
//!
//! **Reuse last tick's pairs** (matrix note): GEA 14.3.6 stresses temporal
//! coherency — contacts persist across frames, so the pipeline **warm-starts**
//! from the previous narrow set before a full broad rebuild. Phases:
//!
//! | Phase | Role |
//! |:---|:---|
//! | Broad | Spatial cell hash → candidate [`CollisionPairKey`]s (deterministic sort) |
//! | Mid | [`CollisionFilter::allows`] + solid-contact gate (PHYS-04 table) |
//! | Narrow | Cell-level survivors written into the coherent cache |
//! | Warm | Re-validate prior narrow pairs first; count hits as coherency savings |
//!
//! **Sole writer:** [`step_coherent_pair_pipeline`] mutates [`CoherentPairCache`].
//! Callers supply body samples; this module does not invent a rigid-body world.
//!
//! **Voxels in scope:** voxel / massing surfaces resolve the same PHYS-04
//! [`CollisionFilter`] via `MaterialId` — share this pair pipeline; do **not**
//! add a voxel-only broad-phase writer.
//!
//! Debt not taken: physics middleware · SAT/GJK narrow (PHYS-01) · swept CCD
//! (PHYS-02 / ballistics) · dual-writing PHYS-04 material collision table.
//!
//! Accept: behavioral `phys11_*` / `coherent_pair_*` (no GPU, no source-scan).

use bevy::prelude::*;
use std::collections::HashMap;

use crate::terrain::material::{
    CollisionFilter, CollisionLayer, DEFAULT_LAYER_COLLIDES_WITH,
};

/// Registers [`CoherentPairCache`] — sole owner of the coherent pair surface.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoherentPairCache>();
    }
}

/// Pipeline stage marker (broad → mid → narrow).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollisionPhase {
    Broad,
    Mid,
    Narrow,
}

/// Canonical phase order (PHYS-11).
pub const COLLISION_PHASE_ORDER: [CollisionPhase; 3] = [
    CollisionPhase::Broad,
    CollisionPhase::Mid,
    CollisionPhase::Narrow,
];

/// Stable body handle for pair keys (sim / probe id — not Bevy Entity bits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollisionBodyId(pub u32);

/// Ordered undirected pair (a < b) for deterministic cache keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollisionPairKey {
    pub a: CollisionBodyId,
    pub b: CollisionBodyId,
}

impl CollisionPairKey {
    #[inline]
    #[must_use]
    pub fn ordered(a: CollisionBodyId, b: CollisionBodyId) -> Option<Self> {
        if a == b {
            return None;
        }
        if a < b {
            Some(Self { a, b })
        } else {
            Some(Self { a: b, b: a })
        }
    }
}

/// Broad-phase sample: spatial cell + PHYS-04 filter.
#[derive(Clone, Copy, Debug)]
pub struct CollisionBodySample {
    pub id: CollisionBodyId,
    /// Broad-phase spatial hash cell (chunk / tile space).
    pub cell: IVec2,
    pub filter: CollisionFilter,
    /// False → mid-phase rejects solid contacts (fluid / pass-through).
    pub solid_contact: bool,
}

/// Sole coherent pair cache for this domain (PHYS-11).
#[derive(Resource, Clone, Debug, Default)]
pub struct CoherentPairCache {
    pub tick: u64,
    /// Previous tick's narrow survivors (warm input).
    pub warm_pairs: Vec<CollisionPairKey>,
    pub broad: Vec<CollisionPairKey>,
    pub mid: Vec<CollisionPairKey>,
    pub narrow: Vec<CollisionPairKey>,
    /// How many warm pairs re-validated into mid this tick.
    pub warm_hits: u32,
    pub broad_generated: u32,
}

/// Broad phase: same-cell body pairs, sorted deterministically.
#[must_use]
pub fn broad_phase_cell_pairs(bodies: &[CollisionBodySample]) -> Vec<CollisionPairKey> {
    let mut pairs = Vec::new();
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            if bodies[i].cell != bodies[j].cell {
                continue;
            }
            if let Some(key) = CollisionPairKey::ordered(bodies[i].id, bodies[j].id) {
                pairs.push(key);
            }
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

/// Mid phase: PHYS-04 filter symmetry + both sides solid.
#[must_use]
pub fn mid_phase_filter(
    pairs: &[CollisionPairKey],
    bodies: &[CollisionBodySample],
) -> Vec<CollisionPairKey> {
    let by_id: HashMap<CollisionBodyId, &CollisionBodySample> =
        bodies.iter().map(|b| (b.id, b)).collect();
    let mut out = Vec::new();
    for &key in pairs {
        let Some(a) = by_id.get(&key.a) else {
            continue;
        };
        let Some(b) = by_id.get(&key.b) else {
            continue;
        };
        if !a.solid_contact || !b.solid_contact {
            continue;
        }
        if !a.filter.allows(b.filter) {
            continue;
        }
        out.push(key);
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Narrow phase (cell-level): mid survivors are contacts for this tick.
#[inline]
#[must_use]
pub fn narrow_phase_accept(mid: &[CollisionPairKey]) -> Vec<CollisionPairKey> {
    let mut narrow = mid.to_vec();
    narrow.sort_unstable();
    narrow.dedup();
    narrow
}

/// Re-validate warm pairs that still exist and pass mid gates.
#[must_use]
pub fn warm_revalidate(
    warm: &[CollisionPairKey],
    bodies: &[CollisionBodySample],
) -> Vec<CollisionPairKey> {
    mid_phase_filter(warm, bodies)
}

/// Sole writer: warm → broad → mid → narrow; store narrow for next tick.
pub fn step_coherent_pair_pipeline(
    cache: &mut CoherentPairCache,
    bodies: &[CollisionBodySample],
    tick: u64,
) {
    let warm_ok = warm_revalidate(&cache.warm_pairs, bodies);
    let warm_hits = warm_ok.len() as u32;

    let broad = broad_phase_cell_pairs(bodies);
    let broad_generated = broad.len() as u32;

    let mut mid_seed = warm_ok;
    mid_seed.extend(broad.iter().copied());
    mid_seed.sort_unstable();
    mid_seed.dedup();
    let mid = mid_phase_filter(&mid_seed, bodies);
    let narrow = narrow_phase_accept(&mid);

    cache.tick = tick;
    cache.warm_hits = warm_hits;
    cache.broad_generated = broad_generated;
    cache.broad = broad;
    cache.mid = mid;
    cache.narrow = narrow;
    cache.warm_pairs = cache.narrow.clone();
}

/// Helper: solid terrain-style filter for tests / probes.
#[inline]
#[must_use]
pub fn solid_terrain_filter() -> CollisionFilter {
    CollisionFilter {
        layer: CollisionLayer::Terrain,
        collides_with: DEFAULT_LAYER_COLLIDES_WITH[CollisionLayer::Terrain as usize],
    }
}

/// Helper: fluid pass-through filter (mid must reject).
#[inline]
#[must_use]
pub fn fluid_pass_filter() -> CollisionFilter {
    CollisionFilter {
        layer: CollisionLayer::Fluid,
        collides_with: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(id: u32, cell: IVec2, filter: CollisionFilter, solid: bool) -> CollisionBodySample {
        CollisionBodySample {
            id: CollisionBodyId(id),
            cell,
            filter,
            solid_contact: solid,
        }
    }

    #[test]
    fn phys11_broad_phase_same_cell_only() {
        let f = solid_terrain_filter();
        let bodies = [
            body(1, IVec2::new(0, 0), f, true),
            body(2, IVec2::new(0, 0), f, true),
            body(3, IVec2::new(1, 0), f, true),
        ];
        let pairs = broad_phase_cell_pairs(&bodies);
        assert_eq!(pairs.len(), 1);
        assert_eq!(
            pairs[0],
            CollisionPairKey::ordered(CollisionBodyId(1), CollisionBodyId(2)).unwrap()
        );
    }

    #[test]
    fn phys11_mid_phase_respects_phys04_filter() {
        let solid = solid_terrain_filter();
        let fluid = fluid_pass_filter();
        let bodies = [
            body(1, IVec2::ZERO, solid, true),
            body(2, IVec2::ZERO, solid, true),
            body(3, IVec2::ZERO, fluid, false),
        ];
        let broad = broad_phase_cell_pairs(&bodies);
        let mid = mid_phase_filter(&broad, &bodies);
        assert!(mid.iter().any(|p| p.a.0 == 1 && p.b.0 == 2));
        assert!(!mid.iter().any(|p| p.a.0 == 1 && p.b.0 == 3 || p.a.0 == 2 && p.b.0 == 3));
    }

    #[test]
    fn phys11_warm_reuse_counts_coherent_hits() {
        let f = solid_terrain_filter();
        let bodies = [
            body(10, IVec2::new(2, 2), f, true),
            body(11, IVec2::new(2, 2), f, true),
        ];
        let mut cache = CoherentPairCache::default();
        step_coherent_pair_pipeline(&mut cache, &bodies, 1);
        assert_eq!(cache.narrow.len(), 1);
        assert_eq!(cache.warm_hits, 0);
        step_coherent_pair_pipeline(&mut cache, &bodies, 2);
        assert!(cache.warm_hits >= 1, "second tick must warm-hit prior pair");
        assert_eq!(cache.narrow.len(), 1);
        assert_eq!(cache.tick, 2);
    }

    #[test]
    fn phys11_plugin_registers_coherent_cache() {
        let mut app = App::new();
        app.add_plugins(CollisionPlugin);
        assert!(app.world().get_resource::<CoherentPairCache>().is_some());
    }

    #[test]
    fn coherent_pair_key_ordered_deterministic() {
        let k1 = CollisionPairKey::ordered(CollisionBodyId(5), CollisionBodyId(2)).unwrap();
        let k2 = CollisionPairKey::ordered(CollisionBodyId(2), CollisionBodyId(5)).unwrap();
        assert_eq!(k1, k2);
        assert!(k1.a < k1.b);
        assert!(CollisionPairKey::ordered(CollisionBodyId(1), CollisionBodyId(1)).is_none());
    }
}
