//! D4a / spine P3 — drone cargo demand book + refill from shared front stock pool.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::capability_toe::CapabilityToeRegistry;
use super::drone_missions::{DronePackage, DroneRole};
use super::formation::Formation;
use crate::economy::logistics::orders::LogisticsOrderRequest;

/// Aggregated UAS BOM demand (dump). Sole writer: [`super::toe_demand::publish_toe_cargo_demands_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct DroneCargoDemandBook {
    pub by_side_tag: BTreeMap<(u8, String), u32>,
    pub total_units: u32,
}

/// Aligned P3 airframe tag (all roles). Legacy role-specific tags remain in cargo registry as aliases.
#[must_use]
pub fn airframe_tag(_role: DroneRole) -> &'static str {
    "drone_airframe"
}

/// Typed sink: top up [`DronePackage`] toward TOE from a shared pool (**CS-6** / CQ-40).
///
/// Package count writer besides sortie [`DronePackage::take`] (via mission step).
/// Scheduled only via [`super::toe_refill::refill_toe_from_front_system`].
pub fn refill_drone_packages_sink(
    toe: &CapabilityToeRegistry,
    pool: &mut super::front_stock_pool::FrontStockPool,
    packages: &mut Query<(Entity, &Formation, &mut DronePackage)>,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) {
    super::toe_refill::refill_packages_sink(
        |role| {
            let row = toe.drone(role);
            (row.toe_count, row.lines)
        },
        pool,
        packages,
        requests,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::balance::UnitKind;
    use bevy::math::IVec2;

    #[test]
    fn airframe_tag_aligned() {
        assert_eq!(airframe_tag(DroneRole::FpvStrike), "drone_airframe");
        assert_eq!(airframe_tag(DroneRole::Interceptor), "drone_airframe");
        let p = DronePackage::with_count(DroneRole::FpvStrike, 1);
        let _ = Formation::new(0, UnitKind::Infantry, IVec2::ZERO);
        assert_eq!(p.count(DroneRole::FpvStrike), 1);
    }
}
