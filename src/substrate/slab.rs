//! `ChunkKey` + generic `ChunkSlab<T>` paging storage (WSS-PLAN-002).
//!
//! # Property-centric paging (ECS-02)
//!
//! Book: gea4v2 17.2.2 / 17.2.2.3–4 *Property-Centric Architectures* / properties vs
//! components / pros & cons (printed p561-564; NEW TOC p578-582).
//! LEGACY keyword: `runtime_object_model` (gea_v2_e4 §17.2 p563; prefer over gea_e3).
//! Plan: PLAN-REF-BOOKS-UPGRADE-v3 slice V4-ECS02.
//!
//! [`ChunkSlab<T>`] is a **property/page table**, not an object-centric bag: each
//! domain owns a homogeneous payload `T` keyed by [`ChunkKey`] (SoA-style column of
//! like-typed pages). Book trade-offs we accept here:
//! * **Pros** — iterate one domain's pages without pulling unrelated entity fields;
//!   only resident/dirty keys pay working-set cost (memory-efficient sparse store).
//! * **Cons** — reconstructing "everything about one logical object" needs a gather
//!   across slabs/layers; debugger watch of a single game object is not the primary
//!   access path (pair with field-layer epochs in `world_field_layers`).
//!
//! **Do not regress** MEM-05 ([`SLAB_GROW_QUANTUM`]) or MEM-10 (page table ≠ residency,
//! [`ChunkKey`] handles). ECS-02 documents layout trade-offs only.
//!
//! # Virtual memory pages (MEM-10)
//!
//! Book: gea4v1 3.5.2 / 3.5.2.3–4 *Virtual Memory* / page faults / TLB (p186-189).
//! LEGACY keyword: `memory_architectures` (gea_v1_e4 §3.5 p183; prefer over gea_e3).
//! Plan: PLAN-REF-BOOKS-UPGRADE-v3 slice V4-MEM10.
//!
//! Engine mirror of OS paging (handles, not raw pointers):
//! * **Page table** — [`ChunkSlab::chunks`] maps virtual [`ChunkKey`] → page payload `T`.
//! * **Residency** — [`ChunkSlab::resident`] is the working set; a key may sit in the
//!   page table without being resident (and must not be treated as hot without
//!   [`ChunkSlab::is_resident`]).
//! * **Fault-in** — [`ChunkSlab::insert`] brings a page into the table (explicit fault);
//!   [`ChunkSlab::get`] of a missing key returns `None` (miss). Consumers hold
//!   [`ChunkKey`] copies; they never stash raw pointers into slab storage.
//! * **TLB** — HashMap lookup is the translation cache: resolve `ChunkKey` → `&T` /
//!   `&mut T` through the table each use; do not cache interior pointers across
//!   growth / eviction.
//!
//! # Chunky growth (MEM-05)
//!
//! Book: gea4v1 6.3.5 *Dynamic Arrays and Chunky Allocation* (p436) — a growable
//! container extends its backing store in fixed-size chunks, never per push.
//! LEGACY keyword: `memory_management` (gea_v1_e4 p412). Plan: PLAN-REF-BOOKS-UPGRADE-v3
//! slice V4-MEM05. **Do not regress** [`SLAB_GROW_QUANTUM`] / [`chunk_multiple_capacity`].
//!
//! The slab's index tables (`chunks` / `resident` / `dirty`) are hash tables whose
//! rehash cost is the realloc analogue. [`ChunkSlab::insert`] therefore tops the
//! table up by [`SLAB_GROW_QUANTUM`] slots the moment it is full, and
//! [`ChunkSlab::reserve_chunks`] lets a streaming window pre-size once for a whole
//! batch. Both round the requested slot count up to a multiple of the quantum via
//! [`chunk_multiple_capacity`], so the number of table growths over `N` inserts is
//! bounded by `N / SLAB_GROW_QUANTUM + 1`, independent of insert granularity.
//!
//! Capacity bookkeeping only: no second authority, no change to chunk contents,
//! dirty/resident semantics, or iteration contract.

use std::collections::{HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
}

impl ChunkKey {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<IVec2> for ChunkKey {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<ChunkKey> for IVec2 {
    fn from(k: ChunkKey) -> Self {
        IVec2::new(k.x, k.y)
    }
}

/// Growth quantum (slots) for [`ChunkSlab`] index tables — MEM-05 chunky allocation.
pub const SLAB_GROW_QUANTUM: usize = 64;

/// Smallest multiple of [`SLAB_GROW_QUANTUM`] that holds `needed` slots (`0` → `0`).
#[must_use]
pub const fn chunk_multiple_capacity(needed: usize) -> usize {
    needed.div_ceil(SLAB_GROW_QUANTUM) * SLAB_GROW_QUANTUM
}

/// Generic paged storage — one pattern for all persistent domains.
#[derive(Resource, Debug)]
pub struct ChunkSlab<T> {
    pub chunks: HashMap<ChunkKey, T>,
    pub resident: HashSet<ChunkKey>,
    pub dirty: HashSet<ChunkKey>,
}

impl<T> Default for ChunkSlab<T> {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            resident: HashSet::new(),
            dirty: HashSet::new(),
        }
    }
}

impl<T> ChunkSlab<T> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    #[must_use]
    pub fn contains(&self, key: ChunkKey) -> bool {
        self.chunks.contains_key(&key)
    }

    #[must_use]
    pub fn get(&self, key: ChunkKey) -> Option<&T> {
        self.chunks.get(&key)
    }

    pub fn get_mut(&mut self, key: ChunkKey) -> Option<&mut T> {
        self.dirty.insert(key);
        self.chunks.get_mut(&key)
    }

    pub fn insert(&mut self, key: ChunkKey, value: T) {
        // Grow in quanta, never per push: only a *new* key on a full table triggers growth.
        // (`HashMap::insert` reserves before probing, so an overwrite on a full table
        // would regrow; the occupied-entry path avoids that.)
        if let Some(slot) = self.chunks.get_mut(&key) {
            *slot = value;
        } else {
            if self.chunks.len() == self.chunks.capacity() {
                self.reserve_chunks(SLAB_GROW_QUANTUM);
            }
            self.chunks.insert(key, value);
        }
        self.dirty.insert(key);
    }

    /// Pre-size all index tables for `additional` more chunks, rounding the target
    /// slot count up to a multiple of [`SLAB_GROW_QUANTUM`]. No-op when headroom exists.
    pub fn reserve_chunks(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }
        let target = chunk_multiple_capacity(self.chunks.len().saturating_add(additional));
        let extra = target - self.chunks.len();
        if self.chunks.capacity() - self.chunks.len() < extra {
            self.chunks.reserve(extra);
        }
        let target_set = |len: usize| chunk_multiple_capacity(len.saturating_add(additional));
        let resident_extra = target_set(self.resident.len()) - self.resident.len();
        if self.resident.capacity() - self.resident.len() < resident_extra {
            self.resident.reserve(resident_extra);
        }
        let dirty_extra = target_set(self.dirty.len()) - self.dirty.len();
        if self.dirty.capacity() - self.dirty.len() < dirty_extra {
            self.dirty.reserve(dirty_extra);
        }
    }

    /// Slot capacity of the chunk table (observability for tripwires).
    #[must_use]
    pub fn chunk_capacity(&self) -> usize {
        self.chunks.capacity()
    }

    pub fn mark_dirty(&mut self, key: ChunkKey) {
        if self.chunks.contains_key(&key) {
            self.dirty.insert(key);
        }
    }

    pub fn clear_dirty(&mut self, key: ChunkKey) {
        self.dirty.remove(&key);
    }

    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn set_resident(&mut self, key: ChunkKey, resident: bool) {
        if resident {
            self.resident.insert(key);
        } else {
            self.resident.remove(&key);
        }
    }

    pub fn clear_resident(&mut self, key: ChunkKey) {
        self.resident.remove(&key);
    }

    #[must_use]
    pub fn is_resident(&self, key: ChunkKey) -> bool {
        self.resident.contains(&key)
    }

    #[must_use]
    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(i: usize) -> ChunkKey {
        ChunkKey::new((i % 97) as i32, (i / 97) as i32)
    }

    #[test]
    fn mem05_chunk_multiple_capacity_rounds_up_to_quantum() {
        assert_eq!(chunk_multiple_capacity(0), 0);
        assert_eq!(chunk_multiple_capacity(1), SLAB_GROW_QUANTUM);
        assert_eq!(chunk_multiple_capacity(SLAB_GROW_QUANTUM), SLAB_GROW_QUANTUM);
        assert_eq!(
            chunk_multiple_capacity(SLAB_GROW_QUANTUM + 1),
            2 * SLAB_GROW_QUANTUM
        );
        for n in 0..(4 * SLAB_GROW_QUANTUM) {
            let c = chunk_multiple_capacity(n);
            assert_eq!(c % SLAB_GROW_QUANTUM, 0, "n={n}");
            assert!(c >= n && c - n < SLAB_GROW_QUANTUM, "n={n}");
        }
    }

    #[test]
    fn mem05_reserve_chunks_gives_at_least_quantum_multiple_headroom() {
        let mut slab: ChunkSlab<u8> = ChunkSlab::default();
        slab.reserve_chunks(1);
        assert!(slab.chunk_capacity() >= SLAB_GROW_QUANTUM);
        slab.reserve_chunks(3 * SLAB_GROW_QUANTUM + 1);
        assert!(slab.chunk_capacity() >= 4 * SLAB_GROW_QUANTUM);
        let cap = slab.chunk_capacity();
        slab.reserve_chunks(0);
        slab.reserve_chunks(1);
        assert_eq!(slab.chunk_capacity(), cap, "headroom present -> no regrow");
    }

    #[test]
    fn mem05_per_push_inserts_grow_table_in_chunks_not_per_push() {
        const N: usize = 4096;
        let mut slab: ChunkSlab<u32> = ChunkSlab::default();
        let mut growths = 0usize;
        let mut last_cap = slab.chunk_capacity();
        for i in 0..N {
            slab.insert(key(i), i as u32);
            let cap = slab.chunk_capacity();
            if cap != last_cap {
                // Each growth must leave at least one quantum of headroom.
                assert!(
                    cap - slab.len() >= SLAB_GROW_QUANTUM - 1,
                    "growth at len={} left headroom {}",
                    slab.len(),
                    cap - slab.len()
                );
                growths += 1;
                last_cap = cap;
            }
        }
        assert_eq!(slab.len(), N);
        assert!(
            growths <= N / SLAB_GROW_QUANTUM + 1,
            "growths={growths} must be bounded by N/quantum"
        );
    }

    #[test]
    fn mem05_overwrite_of_existing_key_on_full_table_does_not_grow() {
        let mut slab: ChunkSlab<u8> = ChunkSlab::default();
        for i in 0..SLAB_GROW_QUANTUM {
            slab.insert(key(i), 0);
        }
        // Fill to the current capacity exactly, then overwrite an existing key.
        let mut i = SLAB_GROW_QUANTUM;
        while slab.len() < slab.chunk_capacity() {
            slab.insert(key(i), 0);
            i += 1;
        }
        let cap = slab.chunk_capacity();
        slab.insert(key(0), 9);
        assert_eq!(slab.chunk_capacity(), cap);
        assert_eq!(slab.get(key(0)), Some(&9));
    }

    #[test]
    fn mem05_growth_preserves_dirty_and_resident_semantics() {
        let mut slab: ChunkSlab<u8> = ChunkSlab::default();
        for i in 0..200 {
            slab.insert(key(i), 1);
            slab.set_resident(key(i), true);
        }
        assert_eq!(slab.dirty_count(), 200);
        assert_eq!(slab.resident_count(), 200);
        slab.clear_dirty(key(3));
        assert_eq!(slab.dirty_count(), 199);
    }

    /// MEM-10 tripwire: virtual page id is a plain `Copy` handle — not a raw pointer into the slab.
    #[test]
    fn mem10_chunk_key_is_copy_handle_not_pointer() {
        assert_eq!(std::mem::size_of::<ChunkKey>(), 8);
        let k = ChunkKey::new(3, -2);
        let k2 = k;
        assert_eq!(k, k2);
        assert_eq!(k.x, 3);
        assert_eq!(k.y, -2);
    }

    /// MEM-10 tripwire: page table (`chunks`) and residency set are distinct; fault-in is
    /// `insert` (table present) + optional `set_resident`; miss returns `None`.
    #[test]
    fn mem10_page_table_residency_and_fault_in() {
        let mut slab: ChunkSlab<u32> = ChunkSlab::default();
        let k = key(0);
        assert!(slab.get(k).is_none(), "miss before fault-in");
        assert!(!slab.is_resident(k));

        slab.insert(k, 42); // fault-in → page table row
        assert_eq!(slab.get(k), Some(&42));
        assert!(slab.contains(k));
        assert!(
            !slab.is_resident(k),
            "table presence ≠ residency (working set is separate)"
        );

        slab.set_resident(k, true);
        assert!(slab.is_resident(k));
        assert_eq!(slab.resident_count(), 1);

        slab.clear_resident(k);
        assert!(!slab.is_resident(k));
        assert!(
            slab.contains(k),
            "clearing residency must not drop the page-table row"
        );
        assert_eq!(slab.get(k), Some(&42), "resolve via table after residency flip");
    }

    /// MEM-10 tripwire: MEM-05 grow quantum stays pinned (no regress of chunky-alloc contract).
    #[test]
    fn mem10_does_not_regress_slab_grow_quantum() {
        assert_eq!(SLAB_GROW_QUANTUM, 64);
        assert_eq!(chunk_multiple_capacity(1), 64);
        assert_eq!(chunk_multiple_capacity(64), 64);
        assert_eq!(chunk_multiple_capacity(65), 128);
    }

    /// ECS-02 tripwire: slab is a homogeneous property/page column keyed by [`ChunkKey`],
    /// not an object-centric bag — like-typed pages only (gea4v2 17.2.2 SoA trade-off).
    #[test]
    fn ecs02_slab_is_homogeneous_property_pages_not_object_bags() {
        let mut fire: ChunkSlab<u16> = ChunkSlab::default();
        let mut heat: ChunkSlab<f32> = ChunkSlab::default();
        let k = key(0);
        fire.insert(k, 7);
        heat.insert(k, 0.25);
        // Same virtual key, independent property tables (gather is explicit across slabs).
        assert_eq!(fire.get(k), Some(&7));
        assert_eq!(heat.get(k), Some(&0.25));
        assert_eq!(std::mem::size_of_val(fire.get(k).unwrap()), 2);
        assert_eq!(std::mem::size_of_val(heat.get(k).unwrap()), 4);
        // Sparse: missing property for another key does not allocate a ghost object.
        assert!(fire.get(key(1)).is_none());
        assert!(heat.get(key(1)).is_none());
    }

    /// ECS-02 tripwire: documenting layout must not regress MEM-05 / MEM-10 contracts.
    #[test]
    fn ecs02_does_not_regress_mem05_mem10_slab_contracts() {
        assert_eq!(SLAB_GROW_QUANTUM, 64);
        assert_eq!(chunk_multiple_capacity(65), 128);
        assert_eq!(std::mem::size_of::<ChunkKey>(), 8);

        let mut slab: ChunkSlab<u32> = ChunkSlab::default();
        let k = key(0);
        slab.insert(k, 1);
        assert!(slab.contains(k));
        assert!(
            !slab.is_resident(k),
            "MEM-10: page-table presence ≠ residency"
        );
        slab.set_resident(k, true);
        assert!(slab.is_resident(k));
        let before = slab.chunk_capacity();
        for i in 1..SLAB_GROW_QUANTUM {
            slab.insert(key(i), i as u32);
        }
        assert!(
            slab.chunk_capacity() >= before,
            "MEM-05: growth only in quanta / reserve path"
        );
        // HashMap's reported capacity need not be a multiple of SLAB_GROW_QUANTUM;
        // the MEM-05 contract is on our reserve/round helpers, not hashbrown's table size.
        let mut reserved: ChunkSlab<u32> = ChunkSlab::default();
        reserved.reserve_chunks(1);
        assert!(reserved.chunk_capacity() >= SLAB_GROW_QUANTUM);
        assert_eq!(chunk_multiple_capacity(1) % SLAB_GROW_QUANTUM, 0);
        assert_eq!(chunk_multiple_capacity(65), 128);
    }
}
