//! Fleets grow with the economy: a faction's depot builds a vehicle when that faction's fleet of a
//! cargo class cannot keep up (plan LO-5 / spine P4, vehicle production feeds the pool).
//!
//! Reads only the snapshot; writes only [`LogisticsOrderCommand::BuildVehicle`] requests, which the
//! order sim applies atomically (materials debited from the depot's stock, or rejected). Runs for
//! factions whose logistics are automated ([`AutoSupplyPolicy::enabled`]); a player who runs
//! logistics by hand adds vehicles from the orders panel.
//!
//! **P4:** class of waiting orders resolves through [`CargoKindRegistry`] when present (Steel /
//! Machinery / munitions / personnel), so growth tracks registry classes — not a second vehicle ECS.
//! Logistics **formations** that amplify haul capacity remain DEFER (see plan charter).

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::auto_supply::AutoSupplyPolicy;
use super::cargo_registry::CargoKindRegistry;
use super::model::{CargoClassMask, CargoKind, OrderStatus, OwnerId, WaitReason};
use super::plugin::{LogisticsOrderRequest, LogisticsOrdersSnapshotRes};
use super::sim::LogisticsOrderCommand;
use super::snapshot::{LogisticsOrdersSnapshot, SiteRole};
use super::model::SiteId;

/// One buildable vehicle: what it carries and what it costs (tonnes from the depot's stock).
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleRecipe {
    pub class_mask: CargoClassMask,
    pub capacity_units: u32,
    pub speed_mps: f32,
    pub cost: Vec<(&'static str, u32)>,
}

/// When and what depots build.
#[derive(Resource, Clone, Debug)]
pub struct FleetGrowthPolicy {
    /// Order-sim ticks (sim seconds) between two builds of one faction: the build time.
    pub interval_ticks: u64,
    /// Build when at least this share of the faction's vehicles of a class is busy...
    pub busy_fraction: f32,
    /// ...and at least this many of its orders wait for a vehicle of that class.
    pub min_waiting_orders: u32,
    /// No faction grows past this many vehicles.
    pub max_fleet: u32,
    pub recipes: Vec<VehicleRecipe>,
}

impl Default for FleetGrowthPolicy {
    fn default() -> Self {
        Self {
            interval_ticks: 120,
            busy_fraction: 0.85,
            min_waiting_orders: 2,
            max_fleet: 64,
            recipes: vec![
                // 10 t box truck: ~6 t of steel body/chassis, 2 t of drivetrain.
                VehicleRecipe {
                    class_mask: CargoClassMask::FREIGHT,
                    capacity_units: 10,
                    speed_mps: 12.0,
                    cost: vec![("Steel", 6), ("Machinery", 2)],
                },
                // 20 m3 tanker.
                VehicleRecipe {
                    class_mask: CargoClassMask::TANKER,
                    capacity_units: 20,
                    speed_mps: 11.0,
                    cost: vec![("Steel", 8), ("Machinery", 2)],
                },
                // Troop carrier (personnel class) — same Steel/Machinery spine.
                VehicleRecipe {
                    class_mask: CargoClassMask::TROOP,
                    capacity_units: 12,
                    speed_mps: 15.0,
                    cost: vec![("Steel", 5), ("Machinery", 2)],
                },
            ],
        }
    }
}

/// True when every recipe cost tag is present in the cargo registry (P4 gate).
#[must_use]
pub fn recipe_costs_in_registry(policy: &FleetGrowthPolicy, reg: &CargoKindRegistry) -> bool {
    policy
        .recipes
        .iter()
        .all(|r| r.cost.iter().all(|(tag, _)| reg.contains(tag)))
}

fn cargo_class_bit(reg: Option<&CargoKindRegistry>, tag: &str) -> u8 {
    let class = reg
        .map(|r| r.class_of(&CargoKind::new(tag)))
        .unwrap_or_else(|| CargoKind::new(tag).class());
    CargoClassMask::of(class).0
}

/// The recipe to build for `owner`, if one of its classes is saturated: the class with the most
/// orders waiting for a vehicle, among classes whose fleet is at least `busy_fraction` busy.
#[must_use]
pub fn saturated_recipe<'a>(
    policy: &'a FleetGrowthPolicy,
    snap: &LogisticsOrdersSnapshot,
    owner: OwnerId,
    registry: Option<&CargoKindRegistry>,
) -> Option<&'a VehicleRecipe> {
    let mut waiting: BTreeMap<u8, u32> = BTreeMap::new();
    for o in snap.orders.iter().filter(|o| o.owner == owner) {
        if o.status == OrderStatus::Waiting(WaitReason::NoVehicles) {
            *waiting
                .entry(cargo_class_bit(registry, o.cargo.as_str()))
                .or_default() += 1;
        }
    }
    policy
        .recipes
        .iter()
        .filter_map(|r| {
            let wait: u32 = waiting
                .iter()
                .filter(|(mask, _)| r.class_mask.0 & **mask != 0)
                .map(|(_, n)| *n)
                .sum();
            if wait < policy.min_waiting_orders.max(1) {
                return None;
            }
            let (total, busy) = snap
                .vehicles
                .iter()
                .filter(|v| v.owner == owner && v.class_mask == r.class_mask)
                .fold((0u32, 0u32), |(t, b), v| (t + 1, b + u32::from(v.order.is_some())));
            let saturated = total == 0 || busy as f32 >= policy.busy_fraction * total as f32;
            saturated.then_some((wait, r))
        })
        .max_by_key(|(wait, _)| *wait)
        .map(|(_, r)| r)
}

/// Build one vehicle per saturated faction per `interval_ticks`, at its depot that holds the
/// materials (lowest id first; a depot short of materials builds nothing).
pub fn grow_depot_fleets_system(
    policy: Res<FleetGrowthPolicy>,
    auto: Res<AutoSupplyPolicy>,
    registry: Res<CargoKindRegistry>,
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut last_build: Local<BTreeMap<OwnerId, u64>>,
) {
    let snap = &snapshot.0;
    let mut owners: Vec<OwnerId> = snap
        .sites
        .iter()
        .filter(|s| s.role == SiteRole::Depot)
        .map(|s| s.owner)
        .collect();
    owners.sort();
    owners.dedup();
    for owner in owners.into_iter().filter(|o| auto.enabled(*o)) {
        if last_build
            .get(&owner)
            .is_some_and(|t| snap.tick < t + policy.interval_ticks)
        {
            continue;
        }
        if snap.fleet(owner).total >= policy.max_fleet {
            continue;
        }
        let Some(recipe) = saturated_recipe(&policy, snap, owner, Some(&registry)) else {
            continue;
        };
        // Refuse builds whose materials are not registry cargo (P4).
        if recipe.cost.iter().any(|(tag, _)| !registry.contains(tag)) {
            continue;
        }
        let depot = snap.sites_of(owner, SiteRole::Depot).find(|d| {
            recipe.cost.iter().all(|(tag, n)| {
                d.stock
                    .iter()
                    .find(|r| r.cargo == *tag)
                    .is_some_and(|r| r.on_hand.saturating_sub(r.reserved_out) >= *n)
            })
        });
        let Some(depot) = depot else { continue };
        requests.write(LogisticsOrderRequest(LogisticsOrderCommand::BuildVehicle {
            owner,
            home: depot.id,
            class_mask: recipe.class_mask,
            capacity_units: recipe.capacity_units,
            speed_mps: recipe.speed_mps,
            cost: recipe
                .cost
                .iter()
                .map(|(tag, n)| (CargoKind::new(*tag), *n))
                .collect(),
        }));
        last_build.insert(owner, snap.tick);
    }
}

/// Does `owner` need another train? Yes when a yard → yard order waits for a vehicle and every
/// train the owner has is busy (or it has none).
#[must_use]
pub fn owner_needs_a_train(snap: &LogisticsOrdersSnapshot, owner: OwnerId) -> Option<SiteId> {
    let yard = |id: SiteId| snap.site(id).is_some_and(|s| s.role == SiteRole::Yard);
    let waiting = snap.orders.iter().find(|o| {
        o.owner == owner
            && yard(o.source)
            && yard(o.sink)
            && matches!(o.status, OrderStatus::Waiting(WaitReason::NoVehicles | WaitReason::NoCompatibleVehicle))
    })?;
    let trains: Vec<_> = snap.vehicles.iter().filter(|v| v.owner == owner && v.rail).collect();
    (trains.is_empty() || trains.iter().all(|v| v.order.is_some())).then_some(waiting.source)
}

/// Automated factions: a yard holding the materials builds a train when trains are short
/// (at most one per faction per `rail.build_interval_s`). The player builds from the yard's card.
pub fn grow_yard_trains_system(
    auto: Res<AutoSupplyPolicy>,
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut last_build: Local<BTreeMap<OwnerId, u64>>,
) {
    let snap = &snapshot.0;
    let rail = &crate::infrastructure::road_tier_table().rail;
    let mut owners: Vec<OwnerId> = snap.sites.iter().filter(|s| s.role == SiteRole::Yard).map(|s| s.owner).collect();
    owners.sort();
    owners.dedup();
    for owner in owners.into_iter().filter(|o| auto.enabled(*o)) {
        if last_build.get(&owner).is_some_and(|t| snap.tick < t + rail.build_interval_s) {
            continue;
        }
        let Some(wanted_at) = owner_needs_a_train(snap, owner) else {
            continue;
        };
        let cost = rail.train_cost();
        let has = |y: &&crate::economy::logistics::orders::snapshot::SiteRow| {
            cost.iter().all(|(tag, n)| y.stock.iter().find(|r| r.cargo == *tag).is_some_and(|r| r.on_hand.saturating_sub(r.reserved_out) >= *n))
        };
        // The yard the train is wanted at, else any yard of the owner with the materials.
        let Some(yard) = snap.site(wanted_at).filter(has).or_else(|| snap.sites_of(owner, SiteRole::Yard).find(has)) else {
            continue;
        };
        requests.write(LogisticsOrderRequest(LogisticsOrderCommand::BuildTrain {
            owner,
            home: yard.id,
            wagons: rail.build_wagons,
            wagon_capacity: rail.wagon_capacity,
            speed_mps: rail.speed_mps(),
            cost: cost.iter().map(|(t, n)| (CargoKind::new(*t), *n)).collect(),
        }));
        last_build.insert(owner, snap.tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::cargo_registry::load_cargo_kind_registry;
    use crate::economy::logistics::orders::model::CargoClass;

    #[test]
    fn default_recipes_cover_freight_tanker_troop_and_registry_steel_machinery() {
        let policy = FleetGrowthPolicy::default();
        let (reg, _) = load_cargo_kind_registry();
        assert!(recipe_costs_in_registry(&policy, &reg));
        assert!(policy
            .recipes
            .iter()
            .any(|r| r.class_mask == CargoClassMask::FREIGHT));
        assert!(policy
            .recipes
            .iter()
            .any(|r| r.class_mask == CargoClassMask::TANKER));
        assert!(policy
            .recipes
            .iter()
            .any(|r| r.class_mask == CargoClassMask::TROOP));
        assert_eq!(reg.get("Steel").unwrap().class, CargoClass::Bulk);
        assert_eq!(reg.get("Machinery").unwrap().class, CargoClass::Bulk);
    }
}
