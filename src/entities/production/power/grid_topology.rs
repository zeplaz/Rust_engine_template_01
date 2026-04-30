//! Rebuild **electrical grid membership** from transformer / line hosts and nearby buildings.
//! Ported from the old `systems/production/power_systems.rs` `manage_electrical_grids_system` idea,
//! updated for Bevy 0.18 and `entities/production/power` components.

use bevy::prelude::*;

use crate::entities::production::power::components::{
    ElectricalComponent, ElectricalGrid, PowerLineComponent, TransformerComponent,
};
use crate::entities::structure::components::Building;

/// World-space radius for associating a load bus with a transformer or line segment (square metres, XZ plane).
#[derive(Resource, Clone, Copy, Debug)]
pub struct GridConnectionRadiusSq(pub f32);

impl Default for GridConnectionRadiusSq {
    fn default() -> Self {
        Self(48.0 * 48.0)
    }
}

/// Fired when a host bus `ElectricalGrid` exceeds capacity (legacy `check_for_overloads_system` parity).
#[derive(Message, Debug, Clone)]
pub struct GridOverloadEvent {
    pub grid_entity: Entity,
    pub total_load: f32,
    pub total_capacity: f32,
}

pub fn rebuild_electrical_grid_topology(
    radius: Res<GridConnectionRadiusSq>,
    mut hosts: Query<
        (&Transform, &mut ElectricalGrid),
        Or<(With<TransformerComponent>, With<PowerLineComponent>)>,
    >,
    buildings: Query<(Entity, &Transform), (With<Building>, With<ElectricalComponent>)>,
) {
    let r2 = radius.0;

    for (host_tf, mut grid) in &mut hosts {
        grid.members.clear();
        let host_pos = host_tf.translation;
        for (b_entity, b_tf) in buildings.iter() {
            if host_pos.distance_squared(b_tf.translation) <= r2 {
                grid.members.insert(b_entity);
            }
        }
    }
}

pub fn recalculate_grid_totals_from_members(
    mut grids: Query<(Entity, &mut ElectricalGrid, &ElectricalComponent)>,
    loads: Query<&ElectricalComponent>,
) {
    for (_host, mut grid, host_ec) in &mut grids {
        let members: Vec<Entity> = grid.members.iter().copied().collect();
        grid.total_load = 0.0;
        grid.total_capacity = host_ec.capacity;
        for member in members {
            if let Ok(ec) = loads.get(member) {
                grid.total_load += ec.current_load;
                grid.total_capacity += ec.capacity;
            }
        }
    }
}

pub fn purge_removed_power_components_from_grids(
    mut grids: Query<&mut ElectricalGrid>,
    mut removed: RemovedComponents<ElectricalComponent>,
) {
    for entity in removed.read() {
        for mut grid in &mut grids {
            grid.members.remove(&entity);
            grid.connected_grids.remove(&entity);
        }
    }
}

pub fn purge_stale_grid_references(
    mut grids: Query<&mut ElectricalGrid>,
    live_grid_host: Query<Entity, Or<(With<TransformerComponent>, With<PowerLineComponent>)>>,
    valid_member: Query<Entity, (With<Building>, With<ElectricalComponent>)>,
) {
    for mut grid in &mut grids {
        grid
            .members
            .retain(|&e| valid_member.contains(e));
        grid
            .connected_grids
            .retain(|&e| live_grid_host.contains(e));
    }
}

pub fn emit_grid_overload_signals(
    grids: Query<
        (Entity, &ElectricalGrid),
        Or<(With<TransformerComponent>, With<PowerLineComponent>)>,
    >,
    mut writer: MessageWriter<GridOverloadEvent>,
) {
    for (entity, grid) in &grids {
        if grid.total_load > grid.total_capacity && grid.total_capacity > f32::EPSILON {
            writer.write(GridOverloadEvent {
                grid_entity: entity,
                total_load: grid.total_load,
                total_capacity: grid.total_capacity,
            });
        }
    }
}
