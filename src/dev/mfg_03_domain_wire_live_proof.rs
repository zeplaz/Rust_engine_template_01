//! **MFG-03-DOMAIN-WIRE** — ManufacturingNode spawn-on-activate + core tick lib witness.

use bevy::prelude::*;

use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::economy::activation::{
    activate_industrial_facilities_system, BuildingDefinitionRef, IndustrialFacilityActivated,
};
use crate::entities::production::core::{
    tick_manufacturing_nodes, ManufacturingBlueprintRegistry, ManufacturingNode,
};
use crate::strategic::{ConstructionSite, SiteArchetype, SiteConstructionPhase};
use crate::systems::sim_control::SimControlState;

pub const MFG_03_DOMAIN_WIRE_JSON: &str = "debug_runs/mfg_03_domain_wire_live.json";

/// Activate an operational smelter, assert `ManufacturingNode`, then tick throughput.
#[must_use]
pub fn mfg_03_domain_wire_lib_green() -> (bool, bool, bool) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
    app.init_resource::<SimControlState>();
    app.init_resource::<ManufacturingBlueprintRegistry>();
    app.add_systems(
        Update,
        (
            activate_industrial_facilities_system,
            tick_manufacturing_nodes,
        )
            .chain(),
    );

    let e = app
        .world_mut()
        .spawn((
            ConstructionSite {
                site_id: 43,
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
    let Some(node) = app.world().get::<ManufacturingNode>(e) else {
        return (activated, false, false);
    };
    let node_on_activate = node.blueprint_id == "mfg_aluminum_cast_v1";
    let throughput_ticked = node.current_throughput > 0.0;
    (activated, node_on_activate, throughput_ticked)
}

#[must_use]
pub fn refresh_mfg_03_domain_wire_live_witness() -> bool {
    let (activated, manufacturing_node_on_activate, manufacturing_core_tick) =
        mfg_03_domain_wire_lib_green();
    let green = activated && manufacturing_node_on_activate && manufacturing_core_tick;
    let body = serde_json::json!({
        "gate_id": "MFG-03-DOMAIN-WIRE",
        "slices": ["MFG-03-DOMAIN-WIRE", "INDUSTRIAL-MFG-01"],
        "activated": activated,
        "manufacturing_node_on_activate": manufacturing_node_on_activate,
        "manufacturing_core_tick": manufacturing_core_tick,
        "catalog_id": "aluminum_smelter1",
        "blueprint_id": "mfg_aluminum_cast_v1",
        "green": green,
    });
    let wrapped = wrap_debug_run(
        "MFG_03_DOMAIN_WIRE",
        "refresh_mfg_03_domain_wire_live_witness",
        MFG_03_DOMAIN_WIRE_JSON,
        body,
    );
    green && write_debug_run_json(MFG_03_DOMAIN_WIRE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mfg_03_domain_wire_witness_green() {
        assert!(refresh_mfg_03_domain_wire_live_witness());
    }
}
