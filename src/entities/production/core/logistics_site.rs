//! Physical logistics sites: roll up inventories for a hub + member entities
//! (warehouse, siding, trucks) without a global resource pool.

use bevy::prelude::*;

use super::ResourceStorage;

/// Marks the **hub** entity for a logistics site (yard, plant complex, port).
/// HUD roll-up includes this entity's [`ResourceStorage`] (if any) plus every
/// [`LogisticsSiteMember`] pointing here that also has storage.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LogisticsSiteRoot;

/// Membership: clicking this entity focuses the **hub** for combined on-site HUD.
#[derive(Component, Debug, Clone, Copy)]
pub struct LogisticsSiteMember {
    pub hub: Entity,
}

#[inline]
pub fn resolve_logistics_focus_entity(
    clicked: Entity,
    member_of: Option<&LogisticsSiteMember>,
    is_hub: bool,
) -> Entity {
    if is_hub {
        return clicked;
    }
    if let Some(m) = member_of {
        return m.hub;
    }
    clicked
}

/// [`ResourceStorage`] entities included when the resolved focus is `hub_or_single`.
pub fn storage_entities_for_focus(
    hub_or_single: Entity,
    is_hub: bool,
    storage_entities: &Query<Entity, With<ResourceStorage>>,
    member_q: &Query<(Entity, &LogisticsSiteMember)>,
) -> Vec<Entity> {
    if !is_hub {
        return vec![hub_or_single];
    }
    let mut out: Vec<Entity> = storage_entities
        .iter()
        .filter(|&e| e == hub_or_single)
        .collect();
    for (e, m) in member_q.iter() {
        if m.hub == hub_or_single && e != hub_or_single && storage_entities.contains(e) {
            out.push(e);
        }
    }
    out
}
