//! HAUL-AMPLIFY-1 — Formation / doctrine mul on order-sim trip capacity (no second fleet ECS).

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::combat::balance::UnitKind;
use crate::combat::formation::Formation;

use super::model::OwnerId;
use super::plugin::LogisticsOrderSimRes;

/// How much one formation kind amplifies haul payload (1.0 = baseline).
#[must_use]
pub fn unit_kind_haul_capacity_mul(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Engineer => 1.25,
        UnitKind::Mechanized => 1.18,
        UnitKind::Armor => 1.12,
        UnitKind::Infantry => 1.05,
        UnitKind::Artillery | UnitKind::AirDefense | UnitKind::Recon => 1.0,
    }
}

/// Aggregate per-owner mul: max kind bonus among that faction's formations (capped).
#[must_use]
pub fn haul_capacity_mul_by_owner<'a>(
    formations: impl Iterator<Item = &'a Formation>,
) -> BTreeMap<OwnerId, f32> {
    let mut out: BTreeMap<OwnerId, f32> = BTreeMap::new();
    for f in formations {
        let owner = OwnerId(f.faction_slot);
        let mul = unit_kind_haul_capacity_mul(f.kind).clamp(1.0, 1.5);
        let e = out.entry(owner).or_insert(1.0);
        *e = (*e).max(mul);
    }
    out
}

/// Sole writer of [`LogisticsOrderSim::haul_capacity_mul_by_owner`].
pub fn sync_haul_amplify_mul_system(
    formations: Query<&Formation>,
    mut res: ResMut<LogisticsOrderSimRes>,
) {
    res.sim.haul_capacity_mul_by_owner = haul_capacity_mul_by_owner(formations.iter());
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn mechanized_beats_infantry() {
        assert!(
            unit_kind_haul_capacity_mul(UnitKind::Mechanized)
                > unit_kind_haul_capacity_mul(UnitKind::Infantry)
        );
    }

    #[test]
    fn owner_takes_max_kind() {
        let a = Formation::new(0, UnitKind::Infantry, IVec2::ZERO);
        let b = Formation::new(0, UnitKind::Engineer, IVec2::ONE);
        let m = haul_capacity_mul_by_owner([&a, &b].into_iter());
        assert!(
            (m[&OwnerId(0)] - unit_kind_haul_capacity_mul(UnitKind::Engineer)).abs() < 1e-5
        );
    }

    #[test]
    fn effective_capacity_scales_without_extra_vehicles() {
        use crate::economy::logistics::orders::sim::{LogisticsOrderSim, SimConfig};
        let mut sim = LogisticsOrderSim::new(SimConfig::default());
        sim.haul_capacity_mul_by_owner
            .insert(OwnerId::PLAYER, 1.25);
        assert_eq!(sim.effective_capacity_units(OwnerId::PLAYER, 10), 13);
        assert_eq!(sim.effective_capacity_units(OwnerId(1), 10), 10);
        assert_eq!(sim.vehicles.len(), 0, "fleet entity count unchanged");
    }
}
