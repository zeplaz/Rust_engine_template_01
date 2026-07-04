//! MIG-A5 — Bevy 0.19 [`RemoteAllocator`] spine for Wave C stream apply batches.
//!
//! Pre-allocates entity ids before main-thread chunk apply so async hydrate can later
//! `spawn_at` without blocking on the world's allocator. Unused ids are freed after apply.

use bevy::ecs::entity::RemoteAllocator;
use bevy::prelude::*;

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

/// Startup — clone world's [`EntityAllocator`] into a [`RemoteAllocator`].
pub fn init_streaming_entity_reserve_spine(world: &mut World) {
    if !mig_a5_stream_entity_reserve_enabled() {
        return;
    }
    let remote = world.entity_allocator().build_remote_allocator();
    world.resource_mut::<StreamingEntityReserveSpine>().remote_allocator = Some(remote);
}

/// Exclusive — reserve one entity id per pending chunk body before ECS apply.
pub fn mig_a5_reserve_before_stream_apply(world: &mut World) {
    if !mig_a5_stream_entity_reserve_enabled() {
        return;
    }
    let batch = world.resource::<PendingStreamApplyQueue>().ready_bodies.len();
    if batch == 0 {
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
    let mut reserved = Vec::with_capacity(batch);
    for _ in 0..batch {
        reserved.push(remote.alloc());
    }
    {
        let mut spine = world.resource_mut::<StreamingEntityReserveSpine>();
        spine.last_batch_preallocated = batch as u32;
        spine.total_preallocated = spine.total_preallocated.saturating_add(batch as u64);
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

    #[test]
    fn mig_a5_spine_defaults_empty() {
        let spine = StreamingEntityReserveSpine::default();
        assert!(spine.remote_allocator.is_none());
        assert_eq!(spine.last_batch_preallocated, 0);
    }
}
