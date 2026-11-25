//! Presentation entities for logistics trucks — read [`LogisticsOrderSimRes`], never write sim.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::economy::logistics::orders::{
    LogisticsOrderSimRes, LogisticsSiteRegistry, SiteId, VehicleId, VehiclePhase,
};
use crate::entities::components::Waypoints;
use crate::presentation::visual_asm::{VisualActionPhase, VisualActionState};
use crate::systems::navigation::{point_along_polyline, truck_leg_key, RoadVehicleKinematics, TruckMotionLeg};

/// Sideways distance (metres) of a moving truck from its route centre line: right-hand lane.
const TRUCK_LANE_OFFSET_M: f32 = 2.8;

/// Marks a presentation proxy for one order-sim vehicle.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TruckPresentation {
    pub vehicle_id: VehicleId,
    /// Wagons behind the engine for a train (RD-7); 0 for a truck.
    pub wagons: u8,
}

#[derive(Resource, Default)]
pub struct TruckPresentationIndex {
    pub by_vehicle: HashMap<VehicleId, Entity>,
}

fn site_xz(registry: &LogisticsSiteRegistry, site: SiteId) -> Option<Vec2> {
    if let Some(p) = registry.travel.positions.get(&site) {
        return Some(Vec2::new(p[0], p[1]));
    }
    registry
        .meta
        .get(&site)
        .map(|m| Vec2::new(m.world_xz[0], m.world_xz[1]))
}

fn phase_to_visual(phase: &VehiclePhase) -> VisualActionPhase {
    match phase {
        VehiclePhase::Idle => VisualActionPhase::Idle,
        VehiclePhase::Loading(_) | VehiclePhase::Unloading(_) => VisualActionPhase::Work,
        VehiclePhase::ToPickup(_) | VehiclePhase::Hauling(_) => VisualActionPhase::Move,
    }
}

fn truck_world_xz(
    v: &crate::economy::logistics::orders::Vehicle,
    registry: &LogisticsSiteRegistry,
) -> Option<Vec2> {
    let at = site_xz(registry, v.at)?;
    match &v.phase {
        VehiclePhase::Idle | VehiclePhase::Loading(_) | VehiclePhase::Unloading(_) => Some(at),
        VehiclePhase::ToPickup(leg) => {
            let Some(a) = v.assignment.as_ref() else {
                return Some(at);
            };
            let dest = site_xz(registry, a.source)?;
            Some(at.lerp(dest, leg.fraction()))
        }
        VehiclePhase::Hauling(leg) => {
            let Some(a) = v.assignment.as_ref() else {
                return Some(at);
            };
            let dest = site_xz(registry, a.sink)?;
            Some(at.lerp(dest, leg.fraction()))
        }
    }
}

/// Sync presentation proxies from order-sim (sole writer of truck entities).
pub fn sync_truck_presentation(
    mut commands: Commands,
    sim: Option<Res<LogisticsOrderSimRes>>,
    registry: Option<Res<LogisticsSiteRegistry>>,
    mut index: ResMut<TruckPresentationIndex>,
    mut q: Query<(&TruckPresentation, &mut Transform, &mut VisualActionState, &Waypoints, &TruckMotionLeg)>,
) {
    let Some(sim) = sim else {
        return;
    };
    let Some(registry) = registry else {
        return;
    };

    let live: std::collections::HashSet<VehicleId> = sim.sim.vehicles.keys().copied().collect();

    // Despawn stale
    let stale: Vec<VehicleId> = index
        .by_vehicle
        .keys()
        .copied()
        .filter(|id| !live.contains(id))
        .collect();
    for id in stale {
        if let Some(e) = index.by_vehicle.remove(&id) {
            commands.entity(e).despawn();
        }
    }

    for (id, v) in &sim.sim.vehicles {
        let Some(xz) = truck_world_xz(v, &registry) else {
            continue;
        };
        let phase = phase_to_visual(&v.phase);
        let nav_driven = matches!(
            v.phase,
            VehiclePhase::ToPickup(_) | VehiclePhase::Hauling(_)
        );
        if let Some(&e) = index.by_vehicle.get(id) {
            if let Ok((_, mut tf, mut vis, wp, leg)) = q.get_mut(e) {
                // Sole writer of the proxy transform. A driving truck sits `fraction` of the way
                // along the planned route (built road, bridge deck included, else cross-country),
                // so it arrives exactly when the order sim says it does; until the route is
                // planned it falls back to the straight line.
                let driven = nav_driven
                    .then(|| match &v.phase {
                        VehiclePhase::ToPickup(l) | VehiclePhase::Hauling(l) => Some(l.fraction()),
                        _ => None,
                    })
                    .flatten()
                    .filter(|_| leg.leg_key == truck_leg_key(v) && wp.points.len() >= 2)
                    .and_then(|f| point_along_polyline(&wp.points, f));
                let (pos, heading) = driven.unwrap_or((xz, Vec2::ZERO));
                // RD-9: a moving truck keeps to the right-hand lane of its road (trains stay on
                // the track centre); 2D and 3D both read this transform.
                let lane = if v.mode.is_rail() { Vec2::ZERO } else { Vec2::new(heading.y, -heading.x) * TRUCK_LANE_OFFSET_M };
                let pos = pos + lane;
                tf.translation.x = pos.x;
                tf.translation.y = pos.y;
                if heading != Vec2::ZERO {
                    tf.rotation = Quat::from_rotation_z(heading.y.atan2(heading.x));
                }
                vis.facing = facing_from_motion(v, &registry);
                vis.set_phase(phase);
            }
        } else {
            let facing = facing_from_motion(v, &registry);
            let color = match (v.mode, phase) {
                // RD-7 trains: dark red engine-and-wagons bar.
                (crate::economy::logistics::orders::model::VehicleMode::Rail { .. }, _) => Color::srgb(0.6, 0.18, 0.15),
                (_, VisualActionPhase::Move) => Color::srgb(0.85, 0.65, 0.15),
                (_, VisualActionPhase::Work) => Color::srgb(0.55, 0.75, 0.25),
                _ => Color::srgb(0.45, 0.5, 0.55),
            };
            let (size, label) = match v.mode {
                crate::economy::logistics::orders::model::VehicleMode::Rail { wagons } => {
                    (Vec2::new(6.0 + 3.0 * f32::from(wagons), 3.5), format!("train:{}", id.0))
                }
                crate::economy::logistics::orders::model::VehicleMode::Road => (Vec2::new(6.0, 3.0), format!("truck:{}", id.0)),
            };
            let e = commands
                .spawn((
                    TruckPresentation {
                        vehicle_id: *id,
                        wagons: match v.mode {
                            crate::economy::logistics::orders::model::VehicleMode::Rail { wagons } => wagons,
                            crate::economy::logistics::orders::model::VehicleMode::Road => 0,
                        },
                    },
                    Transform::from_xyz(xz.x, xz.y, 2.0),
                    Sprite {
                        color,
                        custom_size: Some(size),
                        ..default()
                    },
                    VisualActionState {
                        phase,
                        phase_age_secs: 0.0,
                        facing,
                    },
                    Waypoints::default(),
                    RoadVehicleKinematics::default(),
                    TruckMotionLeg {
                        vehicle_id: *id,
                        leg_key: 0,
                        route_version: 0,
                    },
                    Name::new(label),
                ))
                .id();
            index.by_vehicle.insert(*id, e);
        }
    }
}

fn facing_from_motion(
    v: &crate::economy::logistics::orders::Vehicle,
    registry: &LogisticsSiteRegistry,
) -> u8 {
    let Some(at) = site_xz(registry, v.at) else {
        return 0;
    };
    let dest = match &v.phase {
        VehiclePhase::ToPickup(_) => v.assignment.as_ref().and_then(|a| site_xz(registry, a.source)),
        VehiclePhase::Hauling(_) => v.assignment.as_ref().and_then(|a| site_xz(registry, a.sink)),
        _ => None,
    };
    let Some(dest) = dest else {
        return 0;
    };
    let d = dest - at;
    if d.length_squared() < 1e-4 {
        return 0;
    }
    // 8-way facing from atan2 (iso bake grid).
    let ang = d.y.atan2(d.x);
    let sector = ((ang + std::f32::consts::PI) / (std::f32::consts::TAU / 8.0)).floor() as i32;
    sector.rem_euclid(8) as u8
}
