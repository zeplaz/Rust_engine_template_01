//! **CS-6** — one refill dispatcher + typed sinks (PLAN-COMBAT-SUSTAINMENT / anti-clone).
//!
//! Schedule: rebuild pool once → [`refill_toe_from_front_system`] (drone · missile · personnel).
//! Sinks stay typed; protocol is not cloned across three Bevy systems.

use bevy::prelude::*;

use super::capability_toe::CapabilityToeRegistry;
use super::drone_missions::DronePackage;
use super::drone_supply;
use super::formation::Formation;
use super::front_stock_pool::FrontStockPoolFrame;
use super::missile_supply::{self, MissilePackage};
use super::personnel_supply::{self, PersonnelRestockBook};
use crate::economy::logistics::orders::LogisticsOrderRequest;
use crate::systems::sim_control::SimStepClock;

/// Single TOE refill pass: drone packages → missile packages → personnel credits.
///
/// Sole schedule writer of discrete TOE restock from [`FrontStockPoolFrame`] this frame
/// (formation continuous supply stays separate).
pub fn refill_toe_from_front_system(
    clock: Res<SimStepClock>,
    toe: Res<CapabilityToeRegistry>,
    mut frame: ResMut<FrontStockPoolFrame>,
    mut drones: Query<(Entity, &Formation, &mut DronePackage)>,
    mut missiles: Query<(Entity, &Formation, &mut MissilePackage)>,
    mut personnel: ResMut<PersonnelRestockBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }
    drone_supply::refill_drone_packages_sink(&toe, &mut frame.pool, &mut drones, &mut requests);
    missile_supply::refill_missile_packages_sink(&toe, &mut frame.pool, &mut missiles, &mut requests);
    personnel_supply::refill_personnel_credits_sink(
        &mut frame.pool,
        &mut personnel,
        &mut requests,
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn cs6() {
        // Dispatcher is the sole scheduled refill; sinks live in typed modules.
        let src = include_str!("mod.rs");
        assert!(
            src.contains("refill_toe_from_front_system"),
            "CS-6: schedule must call refill_toe_from_front_system"
        );
        assert!(
            !src.contains("refill_drone_packages_from_front_system")
                && !src.contains("refill_missile_packages_from_front_system")
                && !src.contains("refill_personnel_credits_from_front_system"),
            "CS-6: three refill_* systems must not remain on the schedule"
        );
    }
}
