//! ID generation.
//!
//! Ref-books **CONC-04** (gea4v1 4.9.4.6 / 4.9.4.7 / 4.9.5.1 / 4.9.7.4, p305-318) · LEGACY `lock_free`
//! (gea_v1_e4 §4.9, p285).
//!
//! `NEXT_ID` is a pure monotonic counter: `fetch_add` is a single atomic RMW, so every caller gets a
//! unique value under any ordering. No other memory is published through the counter (an `EntityId`
//! is just a number; the data it names is handed over by ECS/channel/lock sync, never by this atomic),
//! so there is no happens-before edge to build and acquire/release would add cost for nothing.
//! Therefore every access here is `Relaxed`. If a future caller ever uses the counter value as a
//! "ready" flag for other memory, that edge belongs in that caller (release on publish, acquire on
//! observe) — do not strengthen this counter. Tripwires: `tests` below.
//!
//! Known bound (not taken): `u32` wraps after 2^32 allocations.
use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Deserialize, Serialize};

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

impl EntityId {
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    
    pub fn from_u32(id: u32) -> Self {
        Self(id)
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

// Reset the ID counter (useful for tests)
pub fn reset_id_counter() {
    NEXT_ID.store(1, Ordering::Relaxed);
}

// Get the current max ID (useful for debugging)
pub fn get_max_id() -> u32 {
    NEXT_ID.load(Ordering::Relaxed)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// CONC-04: concurrent `EntityId::new` yields unique ids and per-thread monotonic order under
    /// `Relaxed`. Never calls `reset_id_counter` (shared static; other tests run in parallel).
    #[test]
    fn relaxed_counter_unique_across_threads_and_monotonic_per_thread() {
        const THREADS: usize = 8;
        const PER: usize = 2_000;
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                std::thread::spawn(|| {
                    let ids: Vec<u32> = (0..PER).map(|_| EntityId::new().as_u32()).collect();
                    assert!(ids.windows(2).all(|w| w[0] < w[1]), "per-thread order must be monotonic");
                    ids
                })
            })
            .collect();
        let mut seen = HashSet::new();
        for h in handles {
            for id in h.join().expect("worker thread") {
                assert!(seen.insert(id), "duplicate id {id}");
            }
        }
        assert_eq!(seen.len(), THREADS * PER);
    }

    /// CONC-04 tripwire: id counters need only `Relaxed`. Strengthening an ordering here needs a
    /// documented happens-before edge (see module docs) and a deliberate edit of this test.
    #[test]
    fn idgen_atomics_stay_relaxed() {
        let src = include_str!("idgen.rs");
        let prod = src.split("#[cfg(test)]").next().expect("prod section");
        let all = prod.matches("Ordering::").count();
        let relaxed = prod.matches("Ordering::Relaxed").count();
        assert!(all >= 3, "expected fetch_add/store/load uses, found {all}");
        assert_eq!(all, relaxed, "idgen must use only Ordering::Relaxed (CONC-04)");
    }
}
