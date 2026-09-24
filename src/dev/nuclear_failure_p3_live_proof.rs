//! **COD-NUCLEAR-P3-MELTDOWN** / **COD-NUCLEAR-WSS-FALLOUT-001** —
//! MeltdownEvent + containment breach → slab `contamination.radiation`
//! (`debug_runs/nuclear_meltdown_live.json`).

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::entities::components::Operational;
use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
use crate::entities::production::power::components::{PowerPlant, ThermalComponent};
use crate::entities::production::power::failure_modes::{
    apply_containment_breach_radiation_hook_system, attach_nuclear_containment_runtime,
    nuclear_decay_heat_cooling_system, nuclear_loop_scram_system, nuclear_meltdown_breach_system,
    substrate_contamination_radiation_max, sync_nuclear_offsite_from_utility,
    ContainmentBreachEvent, MeltdownEvent, NuclearContainmentRuntime, NuclearCoolingDegradedEvent,
    NuclearDieselState, NuclearScramEvent, CONTAINMENT_BREACH_RADIATION_SEED,
};
use crate::entities::production::power::plant_definition::{
    CapabilityFlags, NuclearFailureProfile, PlantDefinition,
};
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::production::power::power_states::PowerPlantType;
use crate::entities::types::{EmergencyType, OperationalStatus};
use crate::infrastructure::UtilityConnection;
use crate::substrate::WorldSubstrateRegistry;

pub const NUCLEAR_MELTDOWN_JSON: &str = "debug_runs/nuclear_meltdown_live.json";

fn p3_profile() -> NuclearFailureProfile {
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
        meltdown_threshold_c: 200.0,
        decay_heat_rise_c_per_s: 40.0,
        decay_heat_time_compress: 1.0,
        decay_heat_cool_c_per_s: 0.05,
        breach_delay_s: 0.4,
    }
}

#[must_use]
pub fn nuclear_p3_meltdown_lib_green() -> bool {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
        .init_resource::<PlantDefinitionRegistry>()
        .init_resource::<WorldSubstrateRegistry>()
        .add_message::<NuclearScramEvent>()
        .add_message::<NuclearCoolingDegradedEvent>()
        .add_message::<MeltdownEvent>()
        .add_message::<ContainmentBreachEvent>()
        .add_systems(
            Update,
            (
                attach_power_plant_capabilities,
                attach_nuclear_containment_runtime,
                sync_nuclear_offsite_from_utility,
                nuclear_loop_scram_system,
                nuclear_decay_heat_cooling_system,
                nuclear_meltdown_breach_system,
                apply_containment_breach_radiation_hook_system,
            )
                .chain(),
        );

    {
        let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
        defs.insert_for_test(PlantDefinition {
            id: "pwr_witness_p3".into(),
            display_name: "Witness PWR P3".into(),
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
            nuclear_failure_profile: Some(p3_profile()),
        });
    }

    let nuclear = app
        .world_mut()
        .spawn((
            PowerPlant {
                definition_id: "pwr_witness_p3".into(),
                plant_type: PowerPlantType::Nuclear,
                max_output: 1100.0,
                current_output: 900.0,
                status: OperationalStatus::Operational,
                efficiency: 1.0,
            },
            UtilityConnection::power(1, 0.1, false),
            Transform::from_xyz(4.0, 0.0, 4.0),
            Operational {
                maintenance_level: 1.0,
                malfunctions: Default::default(),
                emergencies: Default::default(),
                operational_status: OperationalStatus::Operational,
            },
        ))
        .id();

    for _ in 0..60 {
        app.update();
    }

    let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
    let thermal = app.world().get::<ThermalComponent>(nuclear).unwrap();
    let op = app.world().get::<Operational>(nuclear).unwrap();
    let rad_max = substrate_contamination_radiation_max(
        app.world().resource::<WorldSubstrateRegistry>(),
    );

    runtime.scrammed
        && runtime.diesel == NuclearDieselState::Failed
        && runtime.meltdown_threshold_reached
        && runtime.meltdown_active
        && runtime.meltdown_event_emitted
        && runtime.containment_breached
        && runtime.breach_event_emitted
        && thermal.current_temperature >= 200.0 - 1.0
        && op.emergencies.contains(&EmergencyType::ReactorBreach)
        && !runtime.offsite_connected
        && rad_max >= CONTAINMENT_BREACH_RADIATION_SEED - f32::EPSILON
}

#[must_use]
pub fn refresh_nuclear_meltdown_live_witness() -> bool {
    let green = nuclear_p3_meltdown_lib_green();
    let body = serde_json::json!({
        "gate_id": "COD-NUCLEAR-P3-MELTDOWN",
        "slices": [
            "COD-NUCLEAR-MELTDOWN-001",
            "COD-NUCLEAR-EVENTS-001",
            "COD-NUCLEAR-WSS-FALLOUT-001"
        ],
        "meltdown_lib_green": green,
        "meltdown_event_emitted": green,
        "containment_breach_fallout_hook": green,
        "contamination_radiation_injected": green,
        "contamination_radiation_seed": CONTAINMENT_BREACH_RADIATION_SEED,
        "meltdown_enabled_gate": true,
        "green": green,
    });
    let wrapped = wrap_debug_run(
        "COD-NUCLEAR-P3-MELTDOWN",
        "refresh_nuclear_meltdown_live_witness",
        NUCLEAR_MELTDOWN_JSON,
        body,
    );
    green && write_debug_run_json(NUCLEAR_MELTDOWN_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nuclear_p3_meltdown_witness_green() {
        assert!(refresh_nuclear_meltdown_live_witness());
    }
}
