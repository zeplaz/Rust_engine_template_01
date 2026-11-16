//! Versioned bind-group cache keyed by [`BindGroupId`]; entries invalidate when bound buffer versions change.
//!
//! # RND-11 — resource views / bindless (gea4v2 11.4.9.2–3, p126-128)
//!
//! LEGACY keywords: `rendering_pipeline` (gea_v2_e4 §11.4 p98), `pipeline_app_layer` (gea_v2_e4 §11.5 p133).
//!
//! The book's *resource view* is a cheap descriptor over a long-lived resource; the *bindless* idea is
//! that the set of descriptors is built **once** and addressed by a stable handle, not rebuilt per draw.
//! Here the wgpu analogue is [`BindGroup`]:
//!
//! - **Sole view/bind cache.** [`GPUBindGroupRegistry`] is the only place a pass keeps a `BindGroup`
//!   that depends on a registered buffer. Passes ask it via [`GPUBindGroupRegistry::get_or_build`].
//! - **Stable key.** A [`BindGroupId`] is a numeric constant (no string / pointer identity); the
//!   `ALL_BIND_GROUP_IDS` table pins the set and a compile-time assert rejects duplicates.
//! - **No per-frame recreate.** A cached entry is reused while it records exactly the requested
//!   `(BufferId, version)` dependency list and every live version still matches. [`GPUBufferRegistry`]
//!   only bumps `version` on (re)allocation, so in-place per-frame uploads never rebuild a group.
//!   The builder closure runs only on a miss, and [`BindGroupCacheStats`] counts builds vs reuses.
//! - **Same key, different dependencies ⇒ rebuild.** Reusing a key for a different buffer set can
//!   never serve a stale group (the recorded dependency list is compared, not just the versions).
//!
//! Single writer: the render-world `prepare_*_bind_groups` system that owns a given id. Debt not
//! taken: older `is_valid` + `insert` call sites remain valid but are migrated to `get_or_build`
//! per pass as those files are next touched.

use std::collections::HashMap;

use bevy::prelude::Resource;
use bevy::render::render_resource::{BindGroup, BufferBinding};

use crate::render::core::gpu_buffer_registry::{BufferId, GPUBufferRegistry};

/// Stable numeric identity for a cached bind group (no string keys on the hot path).
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BindGroupId(pub u64);

/// Weather/fire field compute pass: packed fire instance storage (`weather_fire_field.wgsl` group 1).
pub const WEATHER_FIRE_FIELD_FIRE_BIND_GROUP: BindGroupId = BindGroupId(1);
/// World fire particle instancing compute pass (`fire_particle.wgsl` group 1).
pub const WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP: BindGroupId = BindGroupId(2);
/// Expanded instanced-quad vertices (`fire_particle.wgsl` group 2).
pub const WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP: BindGroupId = BindGroupId(3);
/// Spark sim state for expand pass (`fire_particle.wgsl` group 3).
pub const WORLD_FIRE_PARTICLE_SPARK_BIND_GROUP: BindGroupId = BindGroupId(4);
/// Spark compute: instance storage (`fire_spark_compute.wgsl` group 1).
pub const FIRE_SPARK_INSTANCES_BIND_GROUP: BindGroupId = BindGroupId(5);
/// Spark compute: sim state read/write (`fire_spark_compute.wgsl` group 2).
pub const FIRE_SPARK_STATE_BIND_GROUP: BindGroupId = BindGroupId(6);
/// Spark compute: attractor storage (`fire_spark_compute.wgsl` group 3).
pub const FIRE_SPARK_ATTRACTORS_BIND_GROUP: BindGroupId = BindGroupId(7);
/// Water particle instancing compute pass (`water_particle.wgsl` group 1).
pub const WORLD_WATER_PARTICLE_DRAW_BIND_GROUP: BindGroupId = BindGroupId(8);
/// Expanded water particle vertices (`water_particle.wgsl` group 2).
pub const WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP: BindGroupId = BindGroupId(9);

/// Every registered bind-group key. Add new ids here; the const assert below rejects duplicates.
pub const ALL_BIND_GROUP_IDS: [BindGroupId; 9] = [
    WEATHER_FIRE_FIELD_FIRE_BIND_GROUP,
    WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP,
    WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP,
    WORLD_FIRE_PARTICLE_SPARK_BIND_GROUP,
    FIRE_SPARK_INSTANCES_BIND_GROUP,
    FIRE_SPARK_STATE_BIND_GROUP,
    FIRE_SPARK_ATTRACTORS_BIND_GROUP,
    WORLD_WATER_PARTICLE_DRAW_BIND_GROUP,
    WORLD_WATER_PARTICLE_EXPANDED_BIND_GROUP,
];

const _: () = {
    let mut i = 0;
    while i < ALL_BIND_GROUP_IDS.len() {
        let mut j = i + 1;
        while j < ALL_BIND_GROUP_IDS.len() {
            assert!(
                ALL_BIND_GROUP_IDS[i].0 != ALL_BIND_GROUP_IDS[j].0,
                "duplicate BindGroupId in ALL_BIND_GROUP_IDS"
            );
            j += 1;
        }
        i += 1;
    }
};

/// Buffer dependency recorded when a bind group was built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindGroupBufferBinding {
    pub buffer_id: BufferId,
    pub buffer_version: u64,
}

/// One cached view: the group, the buffer versions it was built against, and its build generation.
#[derive(Debug, Clone)]
pub struct GPUBindGroupEntry<G = BindGroup> {
    pub bind_group: G,
    pub bindings: Vec<BindGroupBufferBinding>,
    /// Monotonic build generation (bumps once per build; stable across reuse).
    pub version: u64,
}

/// Result of [`GPUBindGroupRegistry::get_or_build`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindGroupEnsure {
    /// Cached group still matches the requested dependencies — builder not called.
    Reused,
    /// Builder ran; entry replaced/created.
    Built,
    /// A requested buffer is not registered; any stale entry was dropped, builder not called.
    MissingBuffer,
}

/// Cache counters (hits vs builds) for the registry lifetime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BindGroupCacheStats {
    pub builds: u64,
    pub reuses: u64,
}

/// Group-type-agnostic cache core (so the key/version contract is testable without a GPU device).
struct BindGroupCache<G> {
    entries: HashMap<BindGroupId, GPUBindGroupEntry<G>>,
    next_version: u64,
    stats: BindGroupCacheStats,
}

impl<G> Default for BindGroupCache<G> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            next_version: 0,
            stats: BindGroupCacheStats::default(),
        }
    }
}

impl<G> BindGroupCache<G> {
    /// Entry exists and records exactly `buffers` (same order) at their live versions.
    fn is_current(
        &self,
        id: BindGroupId,
        buffers: &[BufferId],
        live_version: &impl Fn(BufferId) -> Option<u64>,
    ) -> bool {
        let Some(entry) = self.entries.get(&id) else {
            return false;
        };
        entry.bindings.len() == buffers.len()
            && entry
                .bindings
                .iter()
                .zip(buffers)
                .all(|(binding, &buffer)| {
                    binding.buffer_id == buffer && live_version(buffer) == Some(binding.buffer_version)
                })
    }

    /// Every recorded binding still matches its live buffer version.
    fn is_valid(&self, id: BindGroupId, live_version: &impl Fn(BufferId) -> Option<u64>) -> bool {
        self.entries.get(&id).is_some_and(|entry| {
            entry
                .bindings
                .iter()
                .all(|b| live_version(b.buffer_id) == Some(b.buffer_version))
        })
    }

    fn insert(&mut self, id: BindGroupId, bind_group: G, bindings: Vec<BindGroupBufferBinding>) {
        self.next_version = self.next_version.wrapping_add(1);
        self.stats.builds += 1;
        self.entries.insert(
            id,
            GPUBindGroupEntry {
                bind_group,
                bindings,
                version: self.next_version,
            },
        );
    }

    fn get_or_build(
        &mut self,
        id: BindGroupId,
        buffers: &[BufferId],
        live_version: &impl Fn(BufferId) -> Option<u64>,
        build: impl FnOnce() -> G,
    ) -> BindGroupEnsure {
        if self.is_current(id, buffers, live_version) {
            self.stats.reuses += 1;
            return BindGroupEnsure::Reused;
        }
        // Miss path only: the dependency list is allocated here, never on a hit.
        let mut bindings = Vec::with_capacity(buffers.len());
        for &buffer_id in buffers {
            let Some(buffer_version) = live_version(buffer_id) else {
                self.entries.remove(&id);
                return BindGroupEnsure::MissingBuffer;
            };
            bindings.push(BindGroupBufferBinding {
                buffer_id,
                buffer_version,
            });
        }
        let bind_group = build();
        self.insert(id, bind_group, bindings);
        BindGroupEnsure::Built
    }
}

#[derive(Resource, Default)]
pub struct GPUBindGroupRegistry {
    cache: BindGroupCache<BindGroup>,
}

impl GPUBindGroupRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.entries.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: BindGroupId) -> Option<&GPUBindGroupEntry> {
        self.cache.entries.get(&id)
    }

    /// Hit/build counters since startup (builds include legacy [`Self::insert`] calls).
    #[must_use]
    pub fn stats(&self) -> BindGroupCacheStats {
        self.cache.stats
    }

    /// Returns `true` when cached bindings still match the live buffer versions.
    #[must_use]
    pub fn is_valid(&self, id: BindGroupId, registry: &GPUBufferRegistry) -> bool {
        self.cache
            .is_valid(id, &|buffer| registry.get(buffer).map(|b| b.version))
    }

    pub fn invalidate(&mut self, id: BindGroupId) {
        self.cache.entries.remove(&id);
    }

    /// Unconditional (re)insert. Prefer [`Self::get_or_build`], which skips the build on a hit.
    pub fn insert(
        &mut self,
        id: BindGroupId,
        bind_group: BindGroup,
        bindings: Vec<BindGroupBufferBinding>,
    ) {
        self.cache.insert(id, bind_group, bindings);
    }

    /// Single funnel for view/bind reuse: runs `build` only when `id` has no entry, its dependency
    /// list differs from `buffers`, or any buffer version moved. Otherwise the cached group (same
    /// identity, same [`GPUBindGroupEntry::version`]) is kept.
    pub fn get_or_build(
        &mut self,
        id: BindGroupId,
        registry: &GPUBufferRegistry,
        buffers: &[BufferId],
        build: impl FnOnce() -> BindGroup,
    ) -> BindGroupEnsure {
        self.cache
            .get_or_build(id, buffers, &|buffer| registry.get(buffer).map(|b| b.version), build)
    }
}

#[must_use]
pub fn buffer_binding_for(registry: &GPUBufferRegistry, id: BufferId) -> Option<BufferBinding<'_>> {
    let entry = registry.get(id)?;
    Some(BufferBinding {
        buffer: &entry.buffer,
        offset: 0,
        size: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::HashSet;

    const A: BufferId = BufferId(100);
    const B: BufferId = BufferId(101);

    fn live(pairs: &[(BufferId, u64)]) -> impl Fn(BufferId) -> Option<u64> + '_ {
        move |id| pairs.iter().find(|(b, _)| *b == id).map(|(_, v)| *v)
    }

    #[test]
    fn bind_group_id_ordering_is_stable() {
        assert!(BindGroupId(1) < BindGroupId(2));
        assert_eq!(WEATHER_FIRE_FIELD_FIRE_BIND_GROUP, BindGroupId(1));
        assert_eq!(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP, BindGroupId(2));
        assert_eq!(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, BindGroupId(3));
    }

    #[test]
    fn rnd11_id_table_is_unique_and_dense() {
        let set: HashSet<_> = ALL_BIND_GROUP_IDS.iter().collect();
        assert_eq!(set.len(), ALL_BIND_GROUP_IDS.len());
        for (i, id) in ALL_BIND_GROUP_IDS.iter().enumerate() {
            assert_eq!(id.0, i as u64 + 1, "ids stay dense/stable (wire-visible in docs)");
        }
    }

    /// Tripwire: same key + same deps ⇒ same cached identity and exactly one build.
    #[test]
    fn rnd11_same_key_reuses_cached_group_identity() {
        let mut cache = BindGroupCache::<u32>::default();
        let versions = [(A, 7)];
        let live = live(&versions);
        let builds = Cell::new(0u32);
        let key = WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP;

        let first = cache.get_or_build(key, &[A], &live, || {
            builds.set(builds.get() + 1);
            0xBEEF
        });
        assert_eq!(first, BindGroupEnsure::Built);
        let generation = cache.entries[&key].version;

        for _ in 0..100 {
            let r = cache.get_or_build(key, &[A], &live, || {
                builds.set(builds.get() + 1);
                0xDEAD
            });
            assert_eq!(r, BindGroupEnsure::Reused);
        }
        assert_eq!(builds.get(), 1, "builder must not run on hits");
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.entries[&key].bind_group, 0xBEEF, "identity preserved");
        assert_eq!(cache.entries[&key].version, generation);
        assert_eq!(cache.stats, BindGroupCacheStats { builds: 1, reuses: 100 });
    }

    #[test]
    fn rnd11_buffer_version_bump_rebuilds_once() {
        let mut cache = BindGroupCache::<u32>::default();
        let key = FIRE_SPARK_STATE_BIND_GROUP;
        let v1 = [(A, 1)];
        cache.get_or_build(key, &[A], &live(&v1), || 1);
        let gen1 = cache.entries[&key].version;

        let v2 = [(A, 2)];
        assert!(!cache.is_valid(key, &live(&v2)));
        assert_eq!(cache.get_or_build(key, &[A], &live(&v2), || 2), BindGroupEnsure::Built);
        assert_eq!(cache.get_or_build(key, &[A], &live(&v2), || 3), BindGroupEnsure::Reused);
        assert_eq!(cache.entries[&key].bind_group, 2);
        assert!(cache.entries[&key].version > gen1);
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn rnd11_same_key_different_dependencies_rebuilds() {
        let mut cache = BindGroupCache::<u32>::default();
        let key = FIRE_SPARK_INSTANCES_BIND_GROUP;
        let vs = [(A, 1), (B, 1)];
        let live = live(&vs);
        cache.get_or_build(key, &[A], &live, || 1);
        // Per-buffer versions all match, but the dependency set changed: must not serve stale.
        assert_eq!(cache.get_or_build(key, &[A, B], &live, || 2), BindGroupEnsure::Built);
        assert_eq!(cache.get_or_build(key, &[B], &live, || 3), BindGroupEnsure::Built);
        assert_eq!(cache.entries[&key].bind_group, 3);
        assert_eq!(cache.entries[&key].bindings.len(), 1);
    }

    #[test]
    fn rnd11_missing_buffer_drops_entry_without_building() {
        let mut cache = BindGroupCache::<u32>::default();
        let key = WORLD_WATER_PARTICLE_DRAW_BIND_GROUP;
        let vs = [(A, 1)];
        cache.get_or_build(key, &[A], &live(&vs), || 1);
        let gone: [(BufferId, u64); 0] = [];
        let ran = Cell::new(false);
        let r = cache.get_or_build(key, &[A], &live(&gone), || {
            ran.set(true);
            9
        });
        assert_eq!(r, BindGroupEnsure::MissingBuffer);
        assert!(!ran.get());
        assert!(cache.entries.is_empty(), "stale view must not survive its buffer");
    }

    #[test]
    fn rnd11_distinct_keys_are_isolated() {
        let mut cache = BindGroupCache::<u32>::default();
        let vs = [(A, 1), (B, 1)];
        let live = live(&vs);
        cache.get_or_build(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP, &[A], &live, || 1);
        cache.get_or_build(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, &[B], &live, || 2);
        assert_eq!(cache.entries.len(), 2);
        cache.entries.remove(&WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP);
        assert_eq!(
            cache.get_or_build(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, &[B], &live, || 9),
            BindGroupEnsure::Reused
        );
    }
}
