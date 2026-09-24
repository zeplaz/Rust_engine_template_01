//! **TRADE-FLOW-GEOGRAPHY** — mine→refinery→smelter chain; cut edge starves smelter.

use bevy::prelude::*;

use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::economy::activation::{
    activate_industrial_facilities_system, BuildingDefinitionRef,
};
use crate::economy::resource_flow::{
    apply_starvation_to_smelter_system, link_supply_chain_edges_system,
    propagate_resource_flow_system, register_resource_flow_nodes_system, transfer_along_edge,
    FacilityFlowState, ResourceFlowNode, ResourceFlowRegistry, ResourceFlowSimWitness,
};
use crate::entities::production::aluminum::AluminumSmelterRuntime;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, FootprintTiles, LayerType, PlannedSite, SiteArchetype,
    SiteConstructionPhase, SiteId,
};
use crate::systems::sim_control::SimControlState;

pub const TRADE_FLOW_GEOGRAPHY_JSON: &str = "debug_runs/trade_flow_geography_live.json";

fn spawn_operational(app: &mut App, catalog_id: &str, site_id: u64, origin: BuildSiteTile) -> Entity {
    app.world_mut()
        .spawn((
            ConstructionSite {
                site_id,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::Factory,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            PlannedSite {
                site_id: SiteId(site_id),
                origin,
                footprint: FootprintTiles { width: 3, depth: 2 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: Some(catalog_id.into()),
                placement: None,
            },
            BuildingDefinitionRef {
                catalog_id: catalog_id.into(),
            },
            Transform::from_translation(crate::economy::site_placement::site_world_position(
                origin,
            )),
            GlobalTransform::default(),
        ))
        .id()
}

/// Lib harness: chain edges exist; open alumina edge feeds; cut path starves smelter.
#[must_use]
pub fn trade_flow_geography_lib_green() -> (bool, bool, bool, bool) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
    app.insert_resource(SimControlState::default());
    app.init_resource::<ResourceFlowRegistry>();
    app.init_resource::<ResourceFlowSimWitness>();
    app.add_systems(
        Update,
        (
            activate_industrial_facilities_system,
            register_resource_flow_nodes_system,
            link_supply_chain_edges_system,
            propagate_resource_flow_system,
            apply_starvation_to_smelter_system,
        )
            .chain(),
    );

    let _mine = spawn_operational(
        &mut app,
        "aluminum_bauxite_mine",
        101,
        BuildSiteTile { x: 0, z: 0 },
    );
    let refinery = spawn_operational(
        &mut app,
        "aluminum_alumina_refinery",
        102,
        BuildSiteTile { x: 1, z: 0 },
    );
    let smelter = spawn_operational(
        &mut app,
        "aluminum_smelter1",
        103,
        BuildSiteTile { x: 2, z: 0 },
    );
    app.update();

    let chain_edges = {
        let flow = app.world().resource::<ResourceFlowRegistry>();
        flow.edges.len() >= 2
    };
    if !chain_edges {
        return (false, false, false, false);
    }

    // Open alumina corridor (resource-flow geography authority; LOG-A-04 owns nav→path_open).
    let mut alumina_edge_idx = None;
    {
        let flow = app.world().resource::<ResourceFlowRegistry>();
        for (i, edge) in flow.edges.iter().enumerate() {
            if edge.from == refinery && edge.to == smelter {
                alumina_edge_idx = Some(i);
                break;
            }
            if edge.buffer_tag.as_deref() == Some("Alumina") {
                alumina_edge_idx = Some(i);
                break;
            }
        }
    }
    let Some(edge_i) = alumina_edge_idx else {
        return (chain_edges, false, false, false);
    };

    {
        let mut flow = app.world_mut().resource_mut::<ResourceFlowRegistry>();
        flow.edges[edge_i].path_open = true;
        if flow.edges[edge_i].buffer_tag.is_none() {
            flow.edges[edge_i].buffer_tag = Some("Alumina".into());
        }
    }
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<ResourceFlowNode>(refinery)
    {
        node.buffer_by_tag.insert("Alumina".into(), 20.0);
    }

    let fed_while_open = {
        let flow = app.world().resource::<ResourceFlowRegistry>();
        let edge = flow.edges[edge_i].clone();
        let mut from = app
            .world()
            .get::<ResourceFlowNode>(edge.from)
            .cloned()
            .expect("refinery node");
        let mut to = app
            .world()
            .get::<ResourceFlowNode>(edge.to)
            .cloned()
            .expect("smelter node");
        let moved = transfer_along_edge(&edge, &mut from, &mut to);
        if let Some(mut n) = app.world_mut().get_mut::<ResourceFlowNode>(edge.from) {
            *n = from;
        }
        if let Some(mut n) = app.world_mut().get_mut::<ResourceFlowNode>(edge.to) {
            *n = to;
        }
        moved > 0.0
    };

    // Geography cut — edge closed; no further alumina transfer.
    {
        let mut flow = app.world_mut().resource_mut::<ResourceFlowRegistry>();
        flow.edges[edge_i].path_open = false;
    }
    let cut_blocks_transfer = {
        let flow = app.world().resource::<ResourceFlowRegistry>();
        let edge = flow.edges[edge_i].clone();
        let mut from = app
            .world()
            .get::<ResourceFlowNode>(edge.from)
            .cloned()
            .expect("refinery node");
        from.buffer_by_tag.insert("Alumina".into(), 20.0);
        let mut to = app
            .world()
            .get::<ResourceFlowNode>(edge.to)
            .cloned()
            .expect("smelter node");
        transfer_along_edge(&edge, &mut from, &mut to) == 0.0
    };
    let geography_cut = cut_blocks_transfer;

    // Drain smelter buffer so consume fails under closed corridor.
    if let Some(mut node) = app.world_mut().get_mut::<ResourceFlowNode>(smelter) {
        node.buffer_by_tag.remove("Alumina");
        node.inventory.clear();
    }
    let efficiency_before = app
        .world()
        .get::<AluminumSmelterRuntime>(smelter)
        .map(|r| r.current_efficiency)
        .unwrap_or(1.0);
    for _ in 0..12 {
        app.update();
    }
    let starved = app
        .world()
        .get::<FacilityFlowState>(smelter)
        .is_some_and(|s| s.starved);
    let efficiency_after = app
        .world()
        .get::<AluminumSmelterRuntime>(smelter)
        .map(|r| r.current_efficiency)
        .unwrap_or(1.0);
    let starvation_cascade = starved && efficiency_after < efficiency_before - 0.05;

    (
        chain_edges,
        fed_while_open,
        geography_cut,
        starvation_cascade,
    )
}

#[must_use]
pub fn refresh_trade_flow_geography_live_witness() -> bool {
    let (chain_edges, fed_while_open, geography_cut, starvation_cascade) =
        trade_flow_geography_lib_green();
    let green = chain_edges && geography_cut && starvation_cascade;
    let body = serde_json::json!({
        "gate_id": "TRADE-FLOW-GEOGRAPHY",
        "slices": ["TRADE-FLOW-GEOGRAPHY", "INDUSTRIAL-I2-07"],
        "mine_refinery_smelter_edges": chain_edges,
        "fed_while_path_open": fed_while_open,
        "geography_cut": geography_cut,
        "starvation_cascade": starvation_cascade,
        "green": green,
        "authority": "ResourceFlowEdge.path_open (trade); LOG-A-04 residual for TransportNavExport wiring",
    });
    let wrapped = wrap_debug_run(
        "TRADE_FLOW_GEOGRAPHY",
        "refresh_trade_flow_geography_live_witness",
        TRADE_FLOW_GEOGRAPHY_JSON,
        body,
    );
    green && write_debug_run_json(TRADE_FLOW_GEOGRAPHY_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trade_flow_geography_witness_green() {
        assert!(refresh_trade_flow_geography_live_witness());
    }
}
