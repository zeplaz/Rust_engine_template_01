//! FIRE-ENTITY-1 — order-sim vehicle heat samples (not presentation trucks).

use bevy::prelude::*;

use crate::systems::fire::entity_exposure::{
    build_heat_index, sample_heat_at_tile, xz_to_world_tile, FireEntityExposureStats,
};
use crate::systems::fire::ChunkFireOverlay;
use crate::systems::weather::ChunkWeather;
use crate::terrain::fire::TRUCK_FIRE_SUSCEPTIBILITY;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::model::{SiteId, Vehicle, VehicleId, VehiclePhase};
use super::plugin::{LogisticsOrderSimRes, LogisticsSiteRegistry};

/// World xz (m) of a vehicle that is mid-leg on a haul (interpolated along its trip); `None` when
/// it is parked or its sites have no position.
pub(super) fn travel_xz(v: &Vehicle, registry: &LogisticsSiteRegistry) -> Option<Vec2> {
    let (from, to, frac) = travel_leg_sample(v)?;
    let a = registry.travel.positions.get(&from)?;
    let b = registry.travel.positions.get(&to)?;
    Some(Vec2::new(a[0] + (b[0] - a[0]) * frac, a[1] + (b[1] - a[1]) * frac))
}

fn travel_leg_sample(v: &Vehicle) -> Option<(SiteId, SiteId, f32)> {
    match (&v.phase, &v.assignment) {
        (VehiclePhase::ToPickup(l), Some(a)) => Some((v.at, a.source, l.fraction())),
        (VehiclePhase::Hauling(l), Some(a)) => Some((a.source, a.sink, l.fraction())),
        _ => None,
    }
}

/// Sample heat at mid-leg vehicles and apply delay → cargo → fleet ladder inside the sim.
pub fn apply_order_sim_vehicle_fire_system(
    mut sim: ResMut<LogisticsOrderSimRes>,
    registry: Res<LogisticsSiteRegistry>,
    mut stats: Option<ResMut<FireEntityExposureStats>>,
    chunks: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkFireOverlay,
        Option<&ChunkWeather>,
    )>,
) {
    let index = build_heat_index(chunks.iter());
    if index.is_empty() {
        return;
    }
    let mut hits: Vec<(VehicleId, f32)> = Vec::new();
    for (vid, v) in &sim.sim.vehicles {
        let Some(xz) = travel_xz(v, &registry) else {
            continue;
        };
        let tile = xz_to_world_tile(xz);
        let (heat, _) = sample_heat_at_tile(&index, tile);
        if heat >= 0.06 {
            hits.push((*vid, heat));
        }
    }
    if hits.is_empty() {
        return;
    }
    let before_d = sim.sim.fire_delayed_trips;
    let before_l = sim.sim.vehicles_lost_to_fire;
    let before_c = sim.sim.cargo_lost_to_fire;
    sim.sim
        .apply_vehicle_fire_hits(&hits, TRUCK_FIRE_SUSCEPTIBILITY);
    if let Some(ref mut s) = stats {
        s.vehicles_delayed = s
            .vehicles_delayed
            .saturating_add(sim.sim.fire_delayed_trips.saturating_sub(before_d));
        s.vehicles_lost_to_fire = s
            .vehicles_lost_to_fire
            .saturating_add(sim.sim.vehicles_lost_to_fire.saturating_sub(before_l));
        s.cargo_lost_to_fire = s
            .cargo_lost_to_fire
            .saturating_add(sim.sim.cargo_lost_to_fire.saturating_sub(before_c));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::model::{
        Assignment, CargoClassMask, CargoKind, Leg, OrderId, OwnerId, VehiclePhase,
    };
    use crate::economy::logistics::orders::sim::{LogisticsOrderSim, SimConfig};

    #[test]
    fn heat_delays_then_destroys_vehicle() {
        let mut sim = LogisticsOrderSim::new(SimConfig::default());
        let owner = OwnerId::PLAYER;
        let home = SiteId(1);
        let sink = SiteId(2);
        sim.sites.insert(home, "a".into());
        sim.sites.insert(sink, "b".into());
        let vid = match sim.apply(crate::economy::logistics::orders::sim::LogisticsOrderCommand::AddVehicle {
            owner,
            home,
            class_mask: CargoClassMask::FREIGHT,
            capacity_units: 10,
            speed_mps: 20.0,
        }) {
            crate::economy::logistics::orders::sim::CommandOutcome::CreatedVehicle(id) => id,
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
        for _ in 0..8 {
            sim.apply_vehicle_fire_hits(&[(vid, 0.9)], 0.55);
        }
        assert!(sim.fire_delayed_trips > 0);
        assert!(
            sim.vehicles_lost_to_fire > 0 || !sim.vehicles.contains_key(&vid),
            "vehicle should be delayed and eventually lost under sustained heat"
        );
    }
}
