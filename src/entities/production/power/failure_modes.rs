//! Failure and derate hooks **scoped by capability**, not by exhaustively matching every `PowerPlantType`.
//! Each system queries a marker (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`) so new fuel
//! types only need archetype + markers, not new `match` arms everywhere.

use bevy::prelude::*;

use crate::entities::production::power::capabilities::{ContainmentBuilding, SteamCycle, VariableRenewable};
use crate::entities::production::power::components::PowerPlant;
use crate::entities::production::power::power_states::PowerPlantType;
use crate::systems::weather::GlobalRenewableWeatherFactors;

/// Placeholder: steam leak, condenser vacuum, feedwater chemistry — adjust `efficiency` or `status` later.
pub fn steam_system_placeholder(_query: Query<&PowerPlant, With<SteamCycle>>) {}

/// Placeholder: scram, containment pressure, decay heat — **only** `ContainmentBuilding` entities.
pub fn nuclear_containment_placeholder(_query: Query<&PowerPlant, With<ContainmentBuilding>>) {}

/// Cloud / wind coupling via world-averaged chunk weather (see `GlobalRenewableWeatherFactors`).
pub fn variable_renewable_placeholder(
    factors: Res<GlobalRenewableWeatherFactors>,
    mut query: Query<&mut PowerPlant, With<VariableRenewable>>,
) {
    for mut plant in &mut query {
        let m = match plant.plant_type {
            PowerPlantType::Wind => factors.wind_capacity_factor,
            PowerPlantType::Solar => factors.solar_capacity_factor,
            _ => 1.0,
        };
        plant.current_output *= m.clamp(0.05, 1.2);
    }
}
