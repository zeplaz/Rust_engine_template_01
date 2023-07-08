use bevy::prelude::*;

use crate::engine::states::*;

use crate::entities::prelude::*;
use crate::systems::production_consumption::*;
struct PowerSysPlugin{}

impl Plugin for PowerSysPlugin {

    fn build(&self, app: &mut App) {

   app.add_system_set(SystemSet::new()
   .with_system(update_load_system)
   .with_system(manage_electrical_grids_system.in_set(OnUpdate(BaseState::Simulation)))
   .run_if(|_, _, state: Res<State<BaseState>>, sim_state: Res<State<SimulationState>>| {
       *state.current() == BaseState::Simulation && *sim_state.current() == SimulationState::Running
   })
   .before(ProductionPlugin::building_activity_system))
}
}


pub fn manage_electrical_grids_system(
    mut commands: Commands,
    mut grids: Query<&mut ElectricalGrid>,
    transformers: Query<(Entity, &Transform, &TransformerComponent, &ElectricalCapacity)>,
    powerlines: Query<(Entity, &Transform, &PowerLineComponent, &ElectricalCapacity)>,
    buildings: Query<(Entity, &Transform, &Building, Option<&ConsumptionComponent>, Option<&ElectricalLoad>)>,
) {
    // First, clear all grids
   // First, clear all grids
   for (_entity, mut grid) in grids.iter_mut() {
    grid.members.clear();
    grid.total_load = 0.0;
    grid.total_capacity = 0.0;
}


     // Now, for each transformer and powerline, find nearby buildings and add them to the grid
     for (entity, transform, _component, capacity) in transformers.iter().chain(powerlines.iter()) {
        for (building_entity, building_transform, _building, _consumption, load_option) in buildings.iter() {
            if let Some(load) = load_option {
                if transform.is_within_radius(building_transform) {
                    if let Ok((_grid_entity, mut grid)) = grids.get_mut(entity) {
                        grid.members.insert(building_entity);
                        grid.total_load += load.current_load;
                        grid.total_capacity += capacity.capacity;
                    } else {
                        commands.entity(entity).insert(ElectricalGrid {
                            members: [building_entity].iter().cloned().collect(),
                            total_load: load.current_load,
                            total_capacity: capacity.capacity,
                        });
                    }
                }
            }
        }
    }

    // Check if there is any remaining capacity in the grid
    for (entity, _transform, _component, capacity) in transformers.iter().chain(powerlines.iter()) {
        if let Ok((_grid_entity, grid)) = grids.get_mut(entity) {
            if grid.total_capacity < grid.total_load {
                // handle the situation when there is not enough electricity (like browning out or reducing consumption rate)
            }
        }
    }
}

pub fn update_load_system(
    mut commands: Commands,
    mut query: Query<(&mut ElectricalLoad, &ProductionComponent, &ConsumptionComponent)>,
) {
    for (mut load, production, consumption) in query.iter_mut() {
        let production_ratio = production.current_rate / production.max_rate;
        let consumption_ratio = consumption.current_rate / consumption.max_rate;

        // The current load is the base load plus a factor based on production and consumption rates
        load.current_load = load.base_load + (load.base_load * (production_ratio + consumption_ratio) / 2.0);

        // You could also introduce additional factors, such as a penalty for overloading the grid
        if load.current_load > load.base_load * 2.0 {
            load.current_load += load.base_load * 0.1; // 10% penalty for overloading
        }
    }
    
   
}


pub fn update_grid_system(
    mut query: Query<(&mut ElectricalGrid, &TransformerComponent, &PowerLineComponent, &ElectricalLoad, &ElectricalCapacity)>,
) {
    for (mut grid, transformer, power_line, load, capacity) in query.iter_mut() {
        let mut total_load = 0.0;
        let mut total_capacity = transformer.capacity + power_line.capacity + capacity.capacity;
        let max_transfer = transformer.max_transfer;

        for entity in &grid.members {
            if let Ok(load) = query.get::<ElectricalLoad>(*entity) {
                total_load += load.current_load;
            }
        }

        for entity in &grid.connected_grids {
            if let Ok(other_grid) = query.get::<ElectricalGrid>(*entity) {
                total_load += other_grid.total_load.min(max_transfer);
                total_capacity += other_grid.total_capacity.min(max_transfer);
            }
        }

        grid.total_load = total_load;
        grid.total_capacity = total_capacity;
    }
}

//5.. Checking for Overloads

fn check_for_overloads_system(
    mut grid_query: Query<(Entity, &ElectricalGrid)>,
) {
    for (grid_entity, grid) in grid_query.iter() {
        if grid.total_load > grid.total_capacity {
            println!("Grid {} is overloaded!", grid_entity.id());
        }
    }
}

//4. Updating the Total Load and Total Capacity of Grids

fn update_grid_totals_system(
    mut grid_query: Query<(&mut ElectricalGrid, &Children)>,
    load_query: Query<(Entity, &ElectricalLoad)>,
    capacity_query: Query<(Entity, &ElectricalCapacity)>,
) {
    for (mut grid, children) in grid_query.iter_mut() {
        grid.total_load = 0.0;
        grid.total_capacity = 0.0;

        for child in children.iter() {
            if let Ok((_, load)) = load_query.get(*child) {
                grid.total_load += load.current_load;
            }
            if let Ok((_, capacity)) = capacity_query.get(*child) {
                grid.total_capacity += capacity.capacity;
            }
        }
    }
}

//3. Updating Grids When Transformers Are Destroyed


fn update_on_power_infastructure_destroyed(
    mut commands: Commands,
    mut grid_query: Query<&mut ElectricalGrid>,
    removed_transformers: RemovedComponents<TransformerComponent>,
) {
    for removed_transformer in removed_transformers.iter() {
        for mut grid in grid_query.iter_mut() {
            // Remove the destroyed transformer's connected grid from this grid's connected_grids
            grid.connected_grids.remove(removed_transformer.connected_grid);
        }
    }
}

//2. Removing Entities from the Grid


fn remove_from_grid_system(
    mut commands: Commands,
    mut grid_query: Query<&mut ElectricalGrid>,
    mut entity_removed_events: EventReader<Entity>,
) {
    for removed_entity in entity_removed_events.iter() {
        for mut grid in grid_query.iter_mut() {
            grid.members.remove(removed_entity);
        }
    }
}

//1. Adding New Entities to the Grid


fn add_to_grid_system(
    mut commands: Commands,
    mut grid_query: Query<(&mut ElectricalGrid, &Transform)>,
    mut new_entity_query: Query<(Entity, &Transform), Added<ElectricalLoad>>,
) {
    for (new_entity, new_transform) in new_entity_query.iter() {
        for (mut grid, grid_transform) in grid_query.iter_mut() {
            // Check if the new entity is within the grid's radius
            if grid_transform.distance(new_transform) <= grid.radius {
                grid.members.insert(new_entity);
                break;
            }
        }
    }
}