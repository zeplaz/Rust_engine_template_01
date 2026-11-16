//! **CS-6** — one refill dispatcher + typed sinks (PLAN-COMBAT-SUSTAINMENT / anti-clone).
//!
//! Schedule: rebuild pool once → [`refill_toe_from_front_system`] (drone · missile · personnel).
//! CQ-40: package top-up shares one [`TopUpPackage`] kernel; sinks stay typed adapters.

use bevy::prelude::*;

use super::capability_toe::{CapabilityToeRegistry, ToeLine};
use super::drone_missions::{DronePackage, DroneRole};
use super::drone_supply;
use super::formation::Formation;
use super::front_stock_pool::{FrontStockPool, FrontStockPoolFrame};
use super::missile_supply::{self, MissilePackage};
use super::capability_toe::MissileClass;
use super::personnel_supply::{self, PersonnelRestockBook};
use crate::economy::logistics::orders::LogisticsOrderRequest;
use crate::systems::sim_control::SimStepClock;

/// CQ-40 — kinds that can be topped up toward a TOE count.
pub trait TopUpKind: Copy + 'static {
    fn all() -> &'static [Self];
    fn index(self) -> usize;
}

impl TopUpKind for DroneRole {
    fn all() -> &'static [Self] {
        &DroneRole::ALL
    }
    fn index(self) -> usize {
        DroneRole::index(self)
    }
}

impl TopUpKind for MissileClass {
    fn all() -> &'static [Self] {
        &MissileClass::ALL
    }
    fn index(self) -> usize {
        MissileClass::index(self)
    }
}

/// CQ-40 — package buffer with count / bump for a kind.
pub trait TopUpPackage {
    type Kind: TopUpKind;
    fn count(&self, kind: Self::Kind) -> u32;
    fn bump(&mut self, kind: Self::Kind);
}

impl TopUpPackage for DronePackage {
    type Kind = DroneRole;
    fn count(&self, kind: DroneRole) -> u32 {
        DronePackage::count(self, kind)
    }
    fn bump(&mut self, kind: DroneRole) {
        let i = kind.index();
        self.counts[i] = self.counts[i].saturating_add(1);
    }
}

impl TopUpPackage for MissilePackage {
    type Kind = MissileClass;
    fn count(&self, kind: MissileClass) -> u32 {
        MissilePackage::count(self, kind)
    }
    fn bump(&mut self, kind: MissileClass) {
        let i = kind.index();
        self.counts[i] = self.counts[i].saturating_add(1);
    }
}

/// Shared TOE top-up kernel (CQ-40) — one control flow for drone/missile packages.
pub fn refill_packages_sink<P, F>(
    toe_row: F,
    pool: &mut FrontStockPool,
    packages: &mut Query<(Entity, &Formation, &mut P)>,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) where
    P: TopUpPackage + Component<Mutability = bevy::ecs::component::Mutable>,
    F: Fn(P::Kind) -> (u32, Vec<ToeLine>),
{
    let mut order: Vec<(Entity, u8)> = packages
        .iter()
        .map(|(e, f, _)| (e, f.faction_slot))
        .collect();
    order.sort_by_key(|(e, slot)| (*slot, e.to_bits()));

    for (entity, slot) in order {
        let Ok((_, _, mut pkg)) = packages.get_mut(entity) else {
            continue;
        };
        for &kind in P::Kind::all() {
            let (toe_count, lines) = toe_row(kind);
            while pkg.count(kind) < toe_count {
                if !pool.try_consume_bom_unit(slot, &lines, requests) {
                    break;
                }
                pkg.bump(kind);
            }
        }
    }
}

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
