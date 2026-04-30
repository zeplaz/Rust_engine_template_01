use bevy::prelude::*;

use crate::systems::sim_control::SimControlState;
use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
use crate::entities::production::power::components::{ElectricalComponent, PowerPlant};
use crate::entities::production::power::failure_modes::{
    nuclear_containment_placeholder, steam_system_placeholder, variable_renewable_placeholder,
};
use crate::entities::production::power::grid_topology::{
    emit_grid_overload_signals, purge_removed_power_components_from_grids, purge_stale_grid_references,
    rebuild_electrical_grid_topology, recalculate_grid_totals_from_members, GridConnectionRadiusSq,
    GridOverloadEvent,
};
use crate::entities::production::power::plant_profile::PlantArchetype;
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::types::OperationalStatus;

pub struct PowerRuntimePlugin;

impl Plugin for PowerRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridConnectionRadiusSq>()
            .init_resource::<PlantDefinitionRegistry>()
            .add_message::<GridOverloadEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    rebuild_electrical_grid_topology,
                    purge_removed_power_components_from_grids,
                    purge_stale_grid_references,
                    recalculate_grid_totals_from_members,
                    emit_grid_overload_signals,
                    update_electrical_load_system,
                    update_power_output_system,
                    steam_system_placeholder,
                    nuclear_containment_placeholder,
                    variable_renewable_placeholder,
                )
                    .chain()
                    .run_if(power_simulation_running),
            );
    }
}

/// Runs when the sim loop should advance (`SimControlState`), matching designer pause/step semantics.
/// When `BaseState` / `SimulationState` are initialized on the app, add them here for stricter editor parity;
/// see `power_legacy_functional_parity_v1.md` §1.
fn power_simulation_running(ctrl: Res<SimControlState>) -> bool {
    ctrl.should_tick()
}

fn update_electrical_load_system(mut query: Query<&mut ElectricalComponent>) {
    for mut electrical in &mut query {
        electrical.current_load = electrical
            .current_load
            .clamp(0.0, electrical.max_transfer.max(f32::EPSILON));
    }
}

fn update_power_output_system(
    defs: Res<PlantDefinitionRegistry>,
    mut query: Query<(&mut PowerPlant, Option<&ElectricalComponent>)>,
) {
    for (mut plant, electrical) in &mut query {
        let load_ratio = electrical
            .map(|e| {
                if e.max_transfer <= f32::EPSILON {
                    0.0
                } else {
                    (e.current_load / e.max_transfer).clamp(0.0, 1.0)
                }
            })
            .unwrap_or(0.0);

        let arch = PlantArchetype::for_type(plant.plant_type);
        let type_modifier = arch.efficiency_factor;
        let def_eff = if plant.definition_id.is_empty() {
            type_modifier
        } else if let Some(d) = defs.get(plant.definition_id.as_str()) {
            d.base_efficiency_factor() * d.status_efficiency_mult(plant.status)
        } else {
            type_modifier
        };
        let status_cap = if plant.definition_id.is_empty() {
            if plant.status == OperationalStatus::Operational {
                1.0
            } else {
                0.0
            }
        } else if let Some(d) = defs.get(plant.definition_id.as_str()) {
            d.status_output_fraction(plant.status)
        } else if plant.status == OperationalStatus::Operational {
            1.0
        } else {
            0.0
        };

        let effective = (1.0 - load_ratio * 0.5)
            * def_eff
            * plant.efficiency.clamp(0.0, 1.0)
            * status_cap;
        plant.current_output = (plant.max_output * effective).clamp(0.0, plant.max_output);
    }
}
