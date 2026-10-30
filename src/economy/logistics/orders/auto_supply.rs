//! Per-faction auto-supply: turns unserved draw demands into standing lines.
//!
//! The advisor ([`suggest_lines_for_unserved`]) only proposes; this system sends its proposals
//! as ordinary [`LogisticsOrderRequest`]s for every faction whose policy is on. AI factions and
//! headless runs default to on; the player's default is off in a windowed game (the orders panel
//! has the toggle and a one-shot "Auto-supply" button) and on without a UI.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::advisor::{best_source_for, suggest_lines_for_unserved};
use super::model::{OrderId, OrderStatus, OwnerId, WaitReason};
use super::sim::LogisticsOrderCommand;
use super::plugin::{LogisticsOrderRequest, LogisticsOrdersSnapshotRes};

/// Which factions get standing lines created for their unserved demands automatically.
#[derive(Resource, Clone, Debug)]
pub struct AutoSupplyPolicy {
    /// Default for [`OwnerId::PLAYER`] when it has no explicit entry (the panel plugin turns
    /// this off in a windowed game).
    pub player_default: bool,
    /// Explicit per-faction choice (panel toggle, scenario scripts).
    pub per_owner: BTreeMap<OwnerId, bool>,
    /// Order-sim ticks between passes.
    pub interval_ticks: u64,
}

impl Default for AutoSupplyPolicy {
    fn default() -> Self {
        Self {
            player_default: true,
            per_owner: BTreeMap::new(),
            interval_ticks: 5,
        }
    }
}

impl AutoSupplyPolicy {
    #[must_use]
    pub fn enabled(&self, owner: OwnerId) -> bool {
        self.per_owner
            .get(&owner)
            .copied()
            .unwrap_or(owner != OwnerId::PLAYER || self.player_default)
    }

    pub fn set(&mut self, owner: OwnerId, on: bool) {
        self.per_owner.insert(owner, on);
    }
}

/// A standing line whose source has been empty this long is re-sourced when another site holds
/// the cargo (the source may have been a plant that sold out, or leftover stock that moved on).
pub const RESOURCE_AFTER_TICKS: u64 = 30;

/// Send the advisor's lines for every enabled faction that has unserved demands, and re-source
/// lines stuck on an empty source.
pub fn auto_supply_unserved_system(
    policy: Res<AutoSupplyPolicy>,
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut last_pass: Local<u64>,
    mut empty_since: Local<std::collections::BTreeMap<OrderId, u64>>,
) {
    let snap = &snapshot.0;
    if snap.tick < *last_pass + policy.interval_ticks.max(1) {
        return;
    }
    *last_pass = snap.tick;
    empty_since.retain(|id, _| snap.orders.iter().any(|o| o.id == *id));
    for o in snap.standing_lines().filter(|o| policy.enabled(o.owner)) {
        if o.status != OrderStatus::Waiting(WaitReason::SourceEmpty) {
            empty_since.remove(&o.id);
            continue;
        }
        let since = *empty_since.entry(o.id).or_insert(snap.tick);
        if snap.tick < since + RESOURCE_AFTER_TICKS {
            continue;
        }
        let Some(demand) = snap.demands.iter().find(|d| d.site == o.sink && d.cargo == o.cargo) else {
            continue;
        };
        if best_source_for(snap, demand).is_some_and(|better| better != o.source) {
            // The replacement line is created by the pass below once this one is gone.
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::Cancel { order: o.id }));
            empty_since.remove(&o.id);
        }
    }
    let mut owners: Vec<OwnerId> = snap.demands.iter().map(|d| d.owner).collect();
    owners.sort();
    owners.dedup();
    for owner in owners.into_iter().filter(|o| policy.enabled(*o)) {
        for cmd in suggest_lines_for_unserved(snap, owner).commands {
            requests.write(LogisticsOrderRequest(cmd));
        }
    }
}
