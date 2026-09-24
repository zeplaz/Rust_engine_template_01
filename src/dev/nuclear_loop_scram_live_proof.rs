//! **COD-NUCLEAR-P1** — LOOP/SCRAM/DIESEL lib witness (`debug_runs/nuclear_loop_scram_live.json`).

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
use crate::entities::production::power::components::PowerPlant;
use crate::entities::production::power::failure_modes::{
    attach_nuclear_containment_runtime, nuclear_loop_scram_system, sync_nuclear_offsite_from_utility,
    NuclearContainmentRuntime, NuclearScramEvent,
};
use crate::entities::production::power::plant_definition::{
    CapabilityFlags, NuclearFailureProfile, PlantDefinition,
};
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::production::power::power_states::PowerPlantType;
use crate::entities::types::OperationalStatus;
use crate::infrastructure::UtilityConnection;

pub const NUCLEAR_LOOP_SCRAM_JSON: &str = "debug_runs/nuclear_loop_scram_live.json";

fn p1_profile() -> NuclearFailureProfile {
    NuclearFailureProfile {
        requires_offsite_power_when: vec![OperationalStatus::Operational],
        offsite_power_mw: 12.0,
        diesel_backup_mw: 10.0,
        diesel_fuel_hours: 72.0,
        passive_cooling: false,
        scram_on_loop: true,
        meltdown_enabled: true,
        scram_delay_s: 0.5,
        diesel_start_delay_s: 0.25,
        diesel_time_compress: 60.0,
        core_stable_temp_c: 80.0,
        meltdown_threshold_c: 1200.0,
        decay_heat_rise_c_per_s: 0.2,
        decay_heat_time_compress: 60.0,
        decay_heat_cool_c_per_s: 0.05,
        breach_delay_s: 45.0,
    }
}

#[must_use]
pub fn nuclear_loop_scram_p1_lib_green() -> bool {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
        .init_resource::<PlantDefinitionRegistry>()
        .add_message::<NuclearScramEvent>()
        .add_systems(
            Update,
            (
                attach_power_plant_capabilities,
                attach_nuclear_containment_runtime,
                sync_nuclear_offsite_from_utility,
                nuclear_loop_scram_system,
            )
                .chain(),
        );

    {
        let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
        defs.insert_for_test(PlantDefinition {
            id: "pwr_witness_loop".into(),
            display_name: "Witness PWR".into(),
            plant_type: PowerPlantType::Nuclear,
            narrative: Default::default(),
            output_model: Default::default(),
            operational: Default::default(),
            instance_template: Default::default(),
            capabilities: CapabilityFlags {
                is_steam_cycle: true,
                is_nuclear_containment: true,
                is_variable_renewable: false,
            },
            emissions: Default::default(),
            economics: Default::default(),
            research: Default::default(),
            grid_interface: Default::default(),
            nuclear_failure_profile: Some(p1_profile()),
        });
    }

    let nuclear = app
        .world_mut()
        .spawn((
            PowerPlant {
                definition_id: "pwr_witness_loop".into(),
                plant_type: PowerPlantType::Nuclear,
                max_output: 1100.0,
                current_output: 900.0,
                status: OperationalStatus::Operational,
                efficiency: 1.0,
            },
            UtilityConnection::power(1, 0.1, false),
        ))
        .id();

    let coal = app
        .world_mut()
        .spawn(PowerPlant {
            definition_id: String::new(),
            plant_type: PowerPlantType::Coal,
            max_output: 400.0,
            current_output: 350.0,
            status: OperationalStatus::Operational,
            efficiency: 1.0,
        })
        .id();

    for _ in 0..8 {
        app.update();
    }

    let plant = app.world().get::<PowerPlant>(nuclear).unwrap();
    let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
    let coal_ok = app.world().get::<PowerPlant>(coal).unwrap().status == OperationalStatus::Operational
        && app.world().get::<NuclearContainmentRuntime>(coal).is_none();

    plant.status == OperationalStatus::ExternalShutdown
        && runtime.scrammed
        && !runtime.offsite_connected
        && coal_ok
}

#[must_use]
pub fn refresh_nuclear_loop_scram_live_witness() -> bool {
    let green = nuclear_loop_scram_p1_lib_green();
    let body = serde_json::json!({
        "gate_id": "COD-NUCLEAR-P1",
        "slices": [
            "COD-NUCLEAR-GRID-LINK-001",
            "COD-NUCLEAR-LOOP-SCRAM-001",
            "COD-NUCLEAR-DIESEL-001",
            "COD-NUCLEAR-EVENTS-001"
        ],
        "loop_scram_lib_green": green,
        "coal_no_meltdown_path": true,
        "meltdown_deferred_p2": true,
        "green": green,
    });
    let wrapped = wrap_debug_run(
        "COD-NUCLEAR-P1",
        "refresh_nuclear_loop_scram_live_witness",
        NUCLEAR_LOOP_SCRAM_JSON,
        body,
    );
    green && write_debug_run_json(NUCLEAR_LOOP_SCRAM_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nuclear_p1_loop_scram_witness_green() {
        assert!(refresh_nuclear_loop_scram_live_witness());
    }
}
