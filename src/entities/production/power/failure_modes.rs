//! Failure and derate hooks **scoped by capability**, not by exhaustively matching every `PowerPlantType`.
//! Each system queries a marker (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`) so new fuel
//! types only need archetype + markers, not new `match` arms everywhere.

use bevy::prelude::*;

use crate::entities::production::power::capabilities::{ContainmentBuilding, SteamCycle, VariableRenewable};
use crate::entities::production::power::components::PowerPlant;

/// Placeholder: steam leak, condenser vacuum, feedwater chemistry — adjust `efficiency` or `status` later.
pub fn steam_system_placeholder(_query: Query<&PowerPlant, With<SteamCycle>>) {}

/// Placeholder: scram, containment pressure, decay heat — **only** `ContainmentBuilding` entities.
pub fn nuclear_containment_placeholder(_query: Query<&PowerPlant, With<ContainmentBuilding>>) {}

/// Placeholder: cloud cover / wind resource coupling — **only** variable renewable markers.
pub fn variable_renewable_placeholder(_query: Query<&PowerPlant, With<VariableRenewable>>) {}
