//! Per-faction auto-supply: turns unserved draw demands into standing lines.
//!
//! The advisor ([`suggest_lines_for_unserved`]) only proposes; this system sends its proposals
//! as ordinary [`LogisticsOrderRequest`]s for every faction whose policy is on. AI factions and
//! headless runs default to on; the player's default is off in a windowed game (the orders panel
//! has the toggle and a one-shot "Auto-supply" button) and on without a UI.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::advisor::suggest_lines_for_unserved;
use super::model::OwnerId;
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

/// Send the advisor's lines for every enabled faction that has unserved demands.
pub fn auto_supply_unserved_system(
    policy: Res<AutoSupplyPolicy>,
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut last_pass: Local<u64>,
) {
    let snap = &snapshot.0;
    if snap.tick < *last_pass + policy.interval_ticks.max(1) {
        return;
    }
    *last_pass = snap.tick;
    let mut owners: Vec<OwnerId> = snap.demands.iter().map(|d| d.owner).collect();
    owners.sort();
    owners.dedup();
    for owner in owners.into_iter().filter(|o| policy.enabled(*o)) {
        for cmd in suggest_lines_for_unserved(snap, owner).commands {
            requests.write(LogisticsOrderRequest(cmd));
        }
    }
}
