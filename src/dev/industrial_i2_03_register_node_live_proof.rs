//! **INDUSTRIAL-I2-03** — ResourceFlowNode register-on-activate lib witness.

use bevy::prelude::*;

use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::economy::activation::{
    activate_industrial_facilities_system, BuildingDefinitionRef, IndustrialFacilityActivated,
};
use crate::economy::resource_flow::{
    register_resource_flow_nodes_system, ResourceFlowNode, ResourceFlowNodeRegistered,
};
use crate::strategic::{ConstructionSite, SiteArchetype, SiteConstructionPhase};

pub const INDUSTRIAL_I2_03_JSON: &str = "debug_runs/industrial_i2_03_register_node_live.json";

/// Activate an operational smelter and assert catalog rates land on `ResourceFlowNode`.
#[must_use]
pub fn industrial_i2_03_register_node_lib_green() -> (bool, bool, bool) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
    app.add_systems(
        Update,
        (
            activate_industrial_facilities_system,
            register_resource_flow_nodes_system,
        )
            .chain(),
    );

    let e = app
        .world_mut()
        .spawn((
            ConstructionSite {
                site_id: 42,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::Factory,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            BuildingDefinitionRef {
                catalog_id: "aluminum_smelter1".into(),
            },
        ))
        .id();

    app.update();

    let activated = app.world().get::<IndustrialFacilityActivated>(e).is_some();
    let registered = app.world().get::<ResourceFlowNodeRegistered>(e).is_some();
    let Some(node) = app.world().get::<ResourceFlowNode>(e) else {
        return (activated, registered, false);
    };
    let alumina_consume = node.consumption.iter().any(|r| r.tag == "Alumina");
    let aluminum_produce = node.production.iter().any(|r| r.tag == "Aluminum");
    (
        activated && registered,
        alumina_consume,
        aluminum_produce,
    )
}

#[must_use]
pub fn refresh_industrial_i2_03_register_node_live_witness() -> bool {
    let (registered_on_activate, alumina_consume, aluminum_produce) =
        industrial_i2_03_register_node_lib_green();
    let green = registered_on_activate && alumina_consume && aluminum_produce;
    let body = serde_json::json!({
        "gate_id": "INDUSTRIAL-I2-03",
        "slices": ["INDUSTRIAL-I2-03"],
        "register_node_on_activate": registered_on_activate,
        "alumina_consume": alumina_consume,
        "aluminum_produce": aluminum_produce,
        "catalog_id": "aluminum_smelter1",
        "green": green,
    });
    let wrapped = wrap_debug_run(
        "INDUSTRIAL_I2_03",
        "refresh_industrial_i2_03_register_node_live_witness",
        INDUSTRIAL_I2_03_JSON,
        body,
    );
    green && write_debug_run_json(INDUSTRIAL_I2_03_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn industrial_i2_03_register_node_witness_green() {
        assert!(refresh_industrial_i2_03_register_node_live_witness());
    }
}
