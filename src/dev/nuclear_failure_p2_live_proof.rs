//! **COD-NUCLEAR-P2-COOLING** — decay heat → `ThermalComponent` after diesel fail
//! (`debug_runs/nuclear_failure_p2_live.json`).

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
use crate::entities::production::power::components::{PowerPlant, ThermalComponent};
use crate::entities::production::power::failure_modes::{
    attach_nuclear_containment_runtime, nuclear_decay_heat_cooling_system, nuclear_loop_scram_system,
    sync_nuclear_offsite_from_utility, NuclearContainmentRuntime, NuclearCoolingDegradedEvent,
    NuclearDieselState, NuclearScramEvent,
};
use crate::entities::production::power::plant_definition::{
    CapabilityFlags, NuclearFailureProfile, PlantDefinition,
};
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::production::power::power_states::PowerPlantType;
use crate::entities::types::OperationalStatus;
use crate::infrastructure::UtilityConnection;

pub const NUCLEAR_FAILURE_P2_JSON: &str = "debug_runs/nuclear_failure_p2_live.json";

fn p2_profile() -> NuclearFailureProfile {
    NuclearFailureProfile {
        requires_offsite_power_when: vec![OperationalStatus::Operational],
        offsite_power_mw: 12.0,
        diesel_backup_mw: 10.0,
        diesel_fuel_hours: 0.001,
        passive_cooling: false,
        scram_on_loop: true,
        meltdown_enabled: true,
        scram_delay_s: 0.2,
        diesel_start_delay_s: 0.1,
        diesel_time_compress: 3600.0,
        core_stable_temp_c: 80.0,
        meltdown_threshold_c: 400.0,
        decay_heat_rise_c_per_s: 20.0,
        decay_heat_time_compress: 1.0,
        decay_heat_cool_c_per_s: 0.05,
        breach_delay_s: 45.0,
    }
}

#[must_use]
pub fn nuclear_p2_cooling_lib_green() -> bool {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
        .init_resource::<PlantDefinitionRegistry>()
        .add_message::<NuclearScramEvent>()
        .add_message::<NuclearCoolingDegradedEvent>()
        .add_systems(
            Update,
            (
                attach_power_plant_capabilities,
                attach_nuclear_containment_runtime,
                sync_nuclear_offsite_from_utility,
                nuclear_loop_scram_system,
                nuclear_decay_heat_cooling_system,
            )
                .chain(),
        );

    {
        let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
        defs.insert_for_test(PlantDefinition {
            id: "pwr_witness_p2".into(),
            display_name: "Witness PWR P2".into(),
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
            nuclear_failure_profile: Some(p2_profile()),
        });
    }

    let nuclear = app
        .world_mut()
        .spawn((
            PowerPlant {
                definition_id: "pwr_witness_p2".into(),
                plant_type: PowerPlantType::Nuclear,
                max_output: 1100.0,
                current_output: 900.0,
                status: OperationalStatus::Operational,
                efficiency: 1.0,
            },
            UtilityConnection::power(1, 0.1, false),
        ))
        .id();

    for _ in 0..40 {
        app.update();
    }

    let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
    let thermal = app.world().get::<ThermalComponent>(nuclear).unwrap();

    runtime.scrammed
        && runtime.diesel == NuclearDieselState::Failed
        && runtime.cooling_degraded
        && runtime.cooling_alert_emitted
        && thermal.current_temperature > 80.0
        && !runtime.offsite_connected
}

#[must_use]
pub fn refresh_nuclear_failure_p2_live_witness() -> bool {
    let green = nuclear_p2_cooling_lib_green();
    let body = serde_json::json!({
        "gate_id": "COD-NUCLEAR-P2-COOLING",
        "slices": ["COD-NUCLEAR-COOLING-001"],
        "cooling_lib_green": green,
        "heat_rise_after_diesel_empty": green,
        "meltdown_deferred_p3": true,
        "green": green,
    });
    let wrapped = wrap_debug_run(
        "COD-NUCLEAR-P2-COOLING",
        "refresh_nuclear_failure_p2_live_witness",
        NUCLEAR_FAILURE_P2_JSON,
        body,
    );
    green && write_debug_run_json(NUCLEAR_FAILURE_P2_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nuclear_p2_cooling_witness_green() {
        assert!(refresh_nuclear_failure_p2_live_witness());
    }
}
