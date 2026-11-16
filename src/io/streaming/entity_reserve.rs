//! MIG-A5 — Bevy 0.19 [`RemoteAllocator`] spine for Wave C stream apply batches.
//!
//! **IO-03** — Memory for object spawning (gea4v2 17.4.4 / 17.4.4.1 offline · 17.4.4.2 dynamic).
//! LEGACY: `memory_management` (gea_v1_e4 p412) · `world_streaming` (gea_v2_e4 p589).
//!
//! Offline: [`StreamingSpineBudget::max_spawn_entities_per_chunk`] sizes the per-body reserve.
//! Dynamic: [`StreamingSpineBudget::max_spawn_entities_per_frame`] caps RemoteAllocator fill
//! so hydrate/reconstruct never plan more spawns than the frame can reserve.
//!
//! Pre-allocates entity ids before main-thread chunk apply so async hydrate can later
//! `spawn_at` without blocking on the world's allocator. Unused ids are freed after apply.
//! Spawn capacity is gated in [`super::budget`] *ahead of hydrate*; this module fills the
//! RemoteAllocator for the already-clamped pending batch.

use bevy::ecs::entity::RemoteAllocator;
use bevy::prelude::*;

use super::budget::{spawn_reserve_for_batch, StreamingSpineBudget};
use super::PendingStreamApplyQueue;

/// Holds a clone of the world's remote allocator + batch reservation stats.
#[derive(Resource, Clone, Default)]
pub struct StreamingEntityReserveSpine {
    pub remote_allocator: Option<RemoteAllocator>,
    pub last_batch_preallocated: u32,
    pub total_preallocated: u64,
}

#[must_use]
pub fn mig_a5_stream_entity_reserve_enabled() -> bool {
    std::env::var("MIG_A5")
        .ok()
        .is_none_or(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

/// Pure gate: how many entity ids the reserve spine should allocate for `ready_bodies`.
#[must_use]
pub fn reserved_entity_count_for_ready_batch(
    ready_body_count: usize,
    budget: &StreamingSpineBudget,
) -> usize {
    spawn_reserve_for_batch(
        ready_body_count,
        budget.max_spawn_entities_per_chunk,
        budget.max_spawn_entities_per_frame,
    )
}

/// Startup — clone world's [`EntityAllocator`] into a [`RemoteAllocator`].
pub fn init_streaming_entity_reserve_spine(world: &mut World) {
    if !mig_a5_stream_entity_reserve_enabled() {
        return;
    }
    let remote = world.entity_allocator().build_remote_allocator();
    world.resource_mut::<StreamingEntityReserveSpine>().remote_allocator = Some(remote);
}

/// Exclusive — reserve budgeted entity ids per pending chunk body before ECS apply.
pub fn mig_a5_reserve_before_stream_apply(world: &mut World) {
    if !mig_a5_stream_entity_reserve_enabled() {
        return;
    }
    let batch = world.resource::<PendingStreamApplyQueue>().ready_bodies.len();
    if batch == 0 {
        return;
    }
    let reserve_n = {
        let budget = world.resource::<StreamingSpineBudget>();
        reserved_entity_count_for_ready_batch(batch, budget)
    };
    if reserve_n == 0 {
        return;
    }
    let remote = world
        .resource::<StreamingEntityReserveSpine>()
        .remote_allocator
        .clone();
    let Some(remote) = remote else {
        return;
    };
    if !world.entity_allocator().has_remote_allocator(&remote) {
        return;
    }
    let mut reserved = Vec::with_capacity(reserve_n);
    for _ in 0..reserve_n {
        reserved.push(remote.alloc());
    }
    {
        let mut spine = world.resource_mut::<StreamingEntityReserveSpine>();
        spine.last_batch_preallocated = reserve_n as u32;
        spine.total_preallocated = spine.total_preallocated.saturating_add(reserve_n as u64);
    }
    world
        .resource_mut::<PendingStreamApplyQueue>()
        .reserved_entities
        .extend(reserved);
}

/// Exclusive — free ids that were not consumed by spawn-at apply (current path updates in place).
pub fn mig_a5_release_after_stream_apply(world: &mut World) {
    if !mig_a5_stream_entity_reserve_enabled() {
        return;
    }
    let entities = std::mem::take(
        &mut world
            .resource_mut::<PendingStreamApplyQueue>()
            .reserved_entities,
    );
    if entities.is_empty() {
        return;
    }
    world.entity_allocator_mut().free_many(&entities);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::streaming::budget::{
        STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT, STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT,
    };

    fn test_budget(per_chunk: usize, per_frame: usize) -> StreamingSpineBudget {
        StreamingSpineBudget {
            max_hydrate_chunks_per_frame: 24,
            max_reconstruct_chunks_per_frame: 4,
            max_pending_chunks: 1024,
            max_spawn_entities_per_chunk: per_chunk,
            max_spawn_entities_per_frame: per_frame,
        }
    }

    #[test]
    fn mig_a5_spine_defaults_empty() {
        let spine = StreamingEntityReserveSpine::default();
        assert!(spine.remote_allocator.is_none());
        assert_eq!(spine.last_batch_preallocated, 0);
    }

    #[test]
    fn reserve_gate_matches_offline_times_chunks() {
        let budget = test_budget(1, STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT);
        assert_eq!(reserved_entity_count_for_ready_batch(5, &budget), 5);
        assert_eq!(
            reserved_entity_count_for_ready_batch(0, &budget),
            0,
            "empty ready batch must not reserve"
        );
    }

    #[test]
    fn reserve_gate_clamps_to_dynamic_frame_budget() {
        let budget = test_budget(2, 6);
        assert_eq!(reserved_entity_count_for_ready_batch(10, &budget), 6);
    }

    #[test]
    fn reserve_gate_closed_when_offline_or_dynamic_zero() {
        assert_eq!(
            reserved_entity_count_for_ready_batch(3, &test_budget(0, 8)),
            0
        );
        assert_eq!(
            reserved_entity_count_for_ready_batch(3, &test_budget(1, 0)),
            0
        );
    }

    #[test]
    fn default_offline_ceiling_is_one_entity_per_chunk() {
        assert_eq!(STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT, 1);
        let budget = test_budget(
            STREAMING_SPAWN_ENTITIES_PER_CHUNK_DEFAULT,
            STREAMING_SPAWN_ENTITIES_PER_FRAME_DEFAULT,
        );
        assert_eq!(reserved_entity_count_for_ready_batch(7, &budget), 7);
    }
}
