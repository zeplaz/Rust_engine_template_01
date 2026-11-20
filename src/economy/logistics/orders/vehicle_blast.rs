//! Blast losses for order-sim vehicles (haul trucks mid-leg).
//!
//! Every detonation in `BlastCraterBook.recent` (seq cursor, hit or miss) destroys the vehicles
//! whose interpolated trip position lies where the blast is still lethal to a truck. Same
//! bookkeeping as a fire loss (cargo in flight, reservations) through
//! [`LogisticsOrderSim::apply_vehicle_blast_losses`]; the logistics sim stays the sole writer of
//! its fleet. Parked vehicles are not modelled (no position of their own).

use bevy::prelude::*;

use super::model::VehicleId;
use super::plugin::{LogisticsOrderSimRes, LogisticsSiteRegistry};
use super::vehicle_fire::travel_xz;
use crate::construction::{blast_radius_falloff, BlastCraterBook};

/// Effective blast damage (after falloff) that destroys a loaded or empty truck (a building needs 50).
pub const VEHICLE_BLAST_KILL_DAMAGE: f32 = 25.0;

/// Vehicles from `positions` destroyed by one detonation.
#[must_use]
pub fn blast_victims(
    positions: &[(VehicleId, Vec2)],
    centre_m: Vec2,
    radius_m: f32,
    damage: f32,
) -> Vec<VehicleId> {
    positions
        .iter()
        .filter(|(_, at)| damage * blast_radius_falloff(at.distance(centre_m), radius_m) >= VEHICLE_BLAST_KILL_DAMAGE)
        .map(|(v, _)| *v)
        .collect()
}

/// Narrow: reads the crater book and the vehicle positions, writes the logistics sim's fleet only.
pub fn apply_order_sim_vehicle_blast_system(
    mut sim: ResMut<LogisticsOrderSimRes>,
    registry: Res<LogisticsSiteRegistry>,
    craters: Option<Res<BlastCraterBook>>,
    mut cursor: Local<u64>,
) {
    let Some(craters) = craters else {
        return;
    };
    let from = *cursor;
    for &(seq, centre_m, radius_m, damage) in craters.recent.iter().filter(|(s, ..)| *s > from) {
        *cursor = (*cursor).max(seq);
        let positions: Vec<(VehicleId, Vec2)> = sim
            .sim
            .vehicles
            .iter()
            .filter_map(|(vid, v)| travel_xz(v, &registry).map(|xz| (*vid, xz)))
            .collect();
        let victims = blast_victims(&positions, centre_m, radius_m, damage);
        if !victims.is_empty() {
            sim.sim.apply_vehicle_blast_losses(&victims);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::model::{
        Assignment, CargoClassMask, CargoKind, Leg, OrderId, OwnerId, SiteId, VehiclePhase,
    };
    use crate::economy::logistics::orders::sim::{CommandOutcome, LogisticsOrderCommand, LogisticsOrderSim, SimConfig};

    #[test]
    fn only_trucks_inside_the_lethal_core_are_victims() {
        let near = (VehicleId(1), Vec2::new(105.0, 100.0));
        let edge = (VehicleId(2), Vec2::new(190.0, 100.0));
        let far = (VehicleId(3), Vec2::new(900.0, 100.0));
        let v = blast_victims(&[near, edge, far], Vec2::new(100.0, 100.0), 100.0, 80.0);
        assert_eq!(v, vec![VehicleId(1)]);
    }

    #[test]
    fn a_blast_destroys_a_hauling_truck_and_its_load() {
        let mut sim = LogisticsOrderSim::new(SimConfig::default());
        let (home, sink) = (SiteId(1), SiteId(2));
        sim.sites.insert(home, "a".into());
        sim.sites.insert(sink, "b".into());
        let vid = match sim.apply(LogisticsOrderCommand::AddVehicle {
            owner: OwnerId::PLAYER,
            home,
            class_mask: CargoClassMask::FREIGHT,
            capacity_units: 10,
            speed_mps: 20.0,
        }) {
            CommandOutcome::CreatedVehicle(id) => id,
            other => panic!("{other:?}"),
        };
        {
            let v = sim.vehicles.get_mut(&vid).unwrap();
            v.phase = VehiclePhase::Hauling(Leg::new(20));
            v.assignment = Some(Assignment {
                order: OrderId(1),
                source: home,
                sink,
                cargo: CargoKind::new("steel"),
                load: 5,
            });
        }
        sim.apply_vehicle_blast_losses(&[vid]);
        assert!(!sim.vehicles.contains_key(&vid));
        assert_eq!(sim.vehicles_lost_to_blast, 1);
        assert_eq!(sim.cargo_lost_to_blast, 5);
        assert_eq!(sim.vehicles_lost_to_fire, 0, "a blast loss is not counted as a fire loss");
        // A vehicle that is already gone is not counted twice.
        sim.apply_vehicle_blast_losses(&[vid]);
        assert_eq!(sim.vehicles_lost_to_blast, 1);
    }
}
