//! Ownership-event **documentation** and future hooks.
//! Canonical event type: `crate::events::ownership_events::OwnershipChangeEvent`.
//!
//! Visual reactivity should map `EntityId` → Bevy `Entity` via a registry resource before mutating `Sprite` / materials.

use bevy::prelude::*;

use crate::events::ownership_events::OwnershipChangeEvent;

/// Placeholder for ownership-driven presentation; extend when render entities are keyed by `EntityId`.
pub fn ownership_change_visual_hook(mut _events: MessageReader<OwnershipChangeEvent>) {
    for _event in _events.read() {
        // Wire to faction color updates / labels once ECS linkage exists.
    }
}
