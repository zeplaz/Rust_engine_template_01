//! HY-005 — drain construction hydrology dirty events (marks slab dirty only).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::WorldSubstrateRegistry;

use super::event_bus::{HydrologyDirtyReason, HydrologyEventQueue};

/// Drain queued construction events and mark affected substrate chunks dirty.
/// Does **not** mutate `WorldChunkState.hydrology` fields.
pub fn drain_construction_hydro_events(
    queue: &mut HydrologyEventQueue,
    registry: &mut WorldSubstrateRegistry,
) {
    queue.last_drain_count = 0;
    let events: Vec<_> = queue.pending.drain(..).collect();
    for event in events {
        if matches!(
            event.reason,
            HydrologyDirtyReason::ConstructionComplete { .. }
                | HydrologyDirtyReason::DamBreach { .. }
        ) {
            queue.construction_events_drained += 1;
        }
        if registry.chunks.contains(event.key) {
            registry.chunks.dirty.insert(event.key);
            queue.last_drain_count = queue.last_drain_count.saturating_add(1);
            queue.drained_total = queue.drained_total.saturating_add(1);
        }
    }
    queue.clear_tick_dedupe();
}

pub fn hydrology_drain_construction_events_system(
    base: Res<State<BaseState>>,
    mut queue: ResMut<HydrologyEventQueue>,
    mut registry: ResMut<WorldSubstrateRegistry>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    drain_construction_hydro_events(&mut queue, &mut registry);
}
