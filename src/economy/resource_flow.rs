//! Resource flow graph — nodes, edges, propagation, starvation (Phase 4 I2 / I4).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::BuildingDefinition;
use crate::economy::supply_chain::IndustrialSupplyChainMembership;
use crate::entities::production::aluminum::AluminumSmelterRuntime;
use crate::entities::types::p_enumz::ResourceType;
use crate::economy::logistics::RouteHandle;
use crate::systems::sim_control::SimControlState;

/// How material moves between facilities (I4 enforces `path_open`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportMode {
    Truck,
    Rail,
    Pipeline,
    Conveyor,
}

/// One produce/consume rate from catalog JSON.
#[derive(Debug, Clone)]
pub struct ResourceRate {
    pub tag: String,
    pub resource: Option<ResourceType>,
    pub rate_per_tick: f32,
}

/// Per-facility flow health (starvation → downstream runtime scaling).
#[derive(Component, Clone, Debug, Default)]
pub struct FacilityFlowState {
    pub starved: bool,
    pub output_scale: f32,
}

/// Per-facility economic node (attached as ECS component at activation).
#[derive(Component, Clone, Debug)]
pub struct ResourceFlowNode {
    pub catalog_id: String,
    pub inventory: HashMap<ResourceType, f32>,
    /// Unmapped designer tags (Bauxite, Alumina, Cement, …).
    pub buffer_by_tag: HashMap<String, f32>,
    pub throughput_limit: f32,
    pub production: Vec<ResourceRate>,
    pub consumption: Vec<ResourceRate>,
}

/// Directed transfer between two operational facilities.
#[derive(Debug, Clone)]
pub struct ResourceFlowEdge {
    pub from: Entity,
    pub to: Entity,
    pub transport_mode: TransportMode,
    pub max_rate: f32,
    pub latency_ticks: f32,
    /// When false, propagation skips this edge (LOG-A-04 nav reachability).
    pub path_open: bool,
    pub route_handle: Option<RouteHandle>,
    /// Prefer moving this buffer tag when set; else first matching typed resource.
    pub buffer_tag: Option<String>,
}

/// Idempotent marker — node registered from catalog.
#[derive(Component, Clone, Copy, Debug)]
pub struct ResourceFlowNodeRegistered;

#[derive(Component, Clone, Copy, Debug)]
pub struct SupplyChainEdgesLinked;

#[derive(Resource, Debug, Default)]
pub struct ResourceFlowRegistry {
    pub edges: Vec<ResourceFlowEdge>,
}

impl ResourceFlowRegistry {
    pub fn add_edge(&mut self, edge: ResourceFlowEdge) {
        self.edges.push(edge);
    }
}

/// Runtime counters for industrial witness / proof JSON.
#[derive(Resource, Debug, Default)]
pub struct ResourceFlowSimWitness {
    pub ticks_propagated: u32,
    pub starvation_events: u32,
    pub overload_events_this_frame: u32,
    /// IND-E03 — cumulative `GridOverloadEvent` count for live proof JSON.
    pub overload_events_total: u64,
}

/// Map designer JSON resource strings to sim `ResourceType` where names align.
#[must_use]
pub fn resource_type_from_tag(tag: &str) -> Option<ResourceType> {
    match tag.trim() {
        "Wood" => Some(ResourceType::Wood),
        "Coal" => Some(ResourceType::Coal),
        "Oil" => Some(ResourceType::Oil),
        "RareEarth" => Some(ResourceType::RareEarth),
        "Metal" => Some(ResourceType::Metal),
        "Steel" => Some(ResourceType::Steel),
        "Concrete" => Some(ResourceType::Concrete),
        "Fertilizer" => Some(ResourceType::Fertilizer),
        "Chemicals" => Some(ResourceType::Chemicals),
        "Electronics" => Some(ResourceType::Electronics),
        "Energy" => Some(ResourceType::Energy),
        "Fuel" => Some(ResourceType::Fuel),
        "Diesel" => Some(ResourceType::Fuel),
        "Ammunition" => Some(ResourceType::Ammunition),
        "WarSupply" => Some(ResourceType::WarSupply),
        "Knowledge" => Some(ResourceType::Knowledge),
        "Labour" => Some(ResourceType::Labour),
        "Food" => Some(ResourceType::Food),
        "Water" => Some(ResourceType::Water),
        "Paper" => Some(ResourceType::Paper),
        "Electricity" => Some(ResourceType::Electricity),
        "Aluminum" => Some(ResourceType::Metal),
        "Alumina" => None,
        "Bauxite" | "Gravel" | "Limestone" | "Cement" => None,
        _ => None,
    }
}

fn rates_from_tags(tags: &[String], default_rate: f32) -> Vec<ResourceRate> {
    tags.iter()
        .map(|tag| ResourceRate {
            tag: tag.clone(),
            resource: resource_type_from_tag(tag),
            rate_per_tick: default_rate,
        })
        .collect()
}

/// Build a flow node from an authoritative building definition.
#[must_use]
pub fn flow_node_from_definition(def: &BuildingDefinition) -> ResourceFlowNode {
    let throughput_limit = (def.power_consumption / 10.0).max(1.0);
    ResourceFlowNode {
        catalog_id: def.id.clone(),
        inventory: HashMap::new(),
        buffer_by_tag: HashMap::new(),
        throughput_limit,
        production: rates_from_tags(&def.produces, 1.0),
        consumption: rates_from_tags(&def.consumes, 1.0),
    }
}

fn chain_catalog_order(chain_id: &str) -> &'static [&'static str] {
    match chain_id {
        "aluminum_primary" => &[
            "aluminum_bauxite_mine",
            "aluminum_alumina_refinery",
            "aluminum_smelter1",
            "aluminum_fabrication_plant",
        ],
        "concrete_portland" | "concrete_geopolymer" => &[
            "concrete_aggregate_mine",
            "concrete_cement_kiln",
            "concrete_mixer_plant",
        ],
        _ => &[],
    }
}

fn infer_edge_buffer_tag(up: &ResourceFlowNode, down: &ResourceFlowNode) -> Option<String> {
    for c in &down.consumption {
        if up
            .production
            .iter()
            .any(|p| p.tag.eq_ignore_ascii_case(&c.tag))
        {
            return Some(c.tag.clone());
        }
    }
    None
}

pub fn register_resource_flow_nodes_system(
    registry_buildings: Res<crate::construction::BuildingDefinitionRegistry>,
    mut commands: Commands,
    q: Query<
        (Entity, &crate::economy::activation::BuildingDefinitionRef),
        (
            With<crate::economy::activation::IndustrialFacilityActivated>,
            Without<ResourceFlowNodeRegistered>,
        ),
    >,
) {
    for (entity, def_ref) in &q {
        let Some(def) = registry_buildings.get(def_ref.catalog_id.as_str()) else {
            continue;
        };
        let node = flow_node_from_definition(def);
        commands.entity(entity).insert((
            node,
            ResourceFlowNodeRegistered,
            FacilityFlowState {
                starved: false,
                output_scale: 1.0,
            },
        ));
    }
}

fn flow_edge_exists(flow: &ResourceFlowRegistry, from: Entity, to: Entity) -> bool {
    flow.edges
        .iter()
        .any(|e| e.from == from && e.to == to)
}

pub fn link_supply_chain_edges_system(
    mut flow: ResMut<ResourceFlowRegistry>,
    mut commands: Commands,
    pending: Query<
        Entity,
        (
            With<ResourceFlowNodeRegistered>,
            With<IndustrialSupplyChainMembership>,
            Without<SupplyChainEdgesLinked>,
        ),
    >,
    nodes: Query<
        (
            Entity,
            &ResourceFlowNode,
            &IndustrialSupplyChainMembership,
        ),
        With<ResourceFlowNodeRegistered>,
    >,
) {
    let pending: Vec<Entity> = pending.iter().collect();
    if pending.is_empty() {
        return;
    }

    let mut by_chain: HashMap<String, HashMap<String, Entity>> = HashMap::new();
    let mut node_snap: HashMap<Entity, ResourceFlowNode> = HashMap::new();

    for (entity, node, membership) in &nodes {
        node_snap.insert(entity, node.clone());
        by_chain
            .entry(membership.chain_id.clone())
            .or_default()
            .insert(node.catalog_id.clone(), entity);
    }

    for (chain_id, catalog_to_entity) in &by_chain {
        let order = chain_catalog_order(chain_id);
        for pair in order.windows(2) {
            let (from_id, to_id) = (pair[0], pair[1]);
            let (Some(&from_e), Some(&to_e)) =
                (catalog_to_entity.get(from_id), catalog_to_entity.get(to_id))
            else {
                continue;
            };
            if flow_edge_exists(&flow, from_e, to_e) {
                continue;
            }
            let buffer_tag = node_snap
                .get(&from_e)
                .and_then(|up| node_snap.get(&to_e).map(|down| infer_edge_buffer_tag(up, down)))
                .flatten();
            flow.add_edge(ResourceFlowEdge {
                from: from_e,
                to: to_e,
                transport_mode: TransportMode::Truck,
                max_rate: 4.0,
                latency_ticks: 1.0,
                path_open: false,
                route_handle: None,
                buffer_tag,
            });
        }
    }

    for entity in pending {
        let linked = flow
            .edges
            .iter()
            .any(|e| e.from == entity || e.to == entity);
        if linked {
            commands.entity(entity).insert(SupplyChainEdgesLinked);
        }
    }
}

fn add_production(node: &mut ResourceFlowNode, scale: f32) {
    let cap = node.throughput_limit * scale;
    let mut produced = 0.0f32;
    for rate in &node.production {
        if produced >= cap {
            break;
        }
        let amount = rate.rate_per_tick.min(cap - produced);
        produced += amount;
        if let Some(rt) = rate.resource {
            *node.inventory.entry(rt).or_insert(0.0) += amount;
        } else {
            *node.buffer_by_tag.entry(rate.tag.clone()).or_insert(0.0) += amount;
        }
    }
}

fn try_consume(node: &mut ResourceFlowNode, scale: f32) -> bool {
    let mut ok = true;
    for rate in &node.consumption {
        let need = rate.rate_per_tick * scale;
        if let Some(rt) = rate.resource {
            let have = node.inventory.get(&rt).copied().unwrap_or(0.0);
            if have >= need {
                node.inventory.insert(rt, have - need);
            } else {
                ok = false;
            }
        } else {
            let have = node.buffer_by_tag.get(&rate.tag).copied().unwrap_or(0.0);
            if have >= need {
                node.buffer_by_tag.insert(rate.tag.clone(), have - need);
            } else {
                ok = false;
            }
        }
    }
    ok
}

#[cfg(test)]
fn transfer_along_edge(
    edge: &ResourceFlowEdge,
    from: &mut ResourceFlowNode,
    to: &mut ResourceFlowNode,
) -> f32 {
    if !edge.path_open {
        return 0.0;
    }
    let mut moved = 0.0f32;
    if let Some(tag) = edge.buffer_tag.as_ref() {
        let available = from.buffer_by_tag.get(tag).copied().unwrap_or(0.0);
        let amount = available.min(edge.max_rate);
        if amount > 0.0 {
            from.buffer_by_tag.insert(tag.clone(), available - amount);
            *to.buffer_by_tag.entry(tag.clone()).or_insert(0.0) += amount;
            moved += amount;
        }
        return moved;
    }
    moved
}

pub fn propagate_resource_flow_system(
    _flow: Res<ResourceFlowRegistry>,
    mut witness: ResMut<ResourceFlowSimWitness>,
    mut nodes: Query<(Entity, &mut ResourceFlowNode, &mut FacilityFlowState)>,
) {
    // Freight moves via [`InTransitLedger`](crate::economy::logistics::InTransitLedger) — no teleport.
    for (_, mut node, mut state) in &mut nodes {
        add_production(&mut node, state.output_scale);
        let fed = try_consume(&mut node, state.output_scale);
        state.starved = !fed;
        if !fed {
            state.output_scale = (state.output_scale * 0.85).max(0.1);
            witness.starvation_events = witness.starvation_events.saturating_add(1);
        } else {
            state.output_scale = 1.0;
        }
    }

    witness.ticks_propagated = witness.ticks_propagated.saturating_add(1);
}

pub fn apply_starvation_to_smelter_system(
    mut q: Query<(&FacilityFlowState, &mut AluminumSmelterRuntime)>,
) {
    for (flow, mut smelter) in &mut q {
        if flow.starved {
            smelter.current_efficiency = (smelter.current_efficiency * 0.55).max(0.05);
        }
    }
}

pub fn collect_grid_overload_witness_system(
    mut reader: MessageReader<crate::entities::production::power::GridOverloadEvent>,
    mut witness: ResMut<ResourceFlowSimWitness>,
) {
    witness.overload_events_this_frame = 0;
    for _ in reader.read() {
        witness.overload_events_this_frame = witness.overload_events_this_frame.saturating_add(1);
        witness.overload_events_total = witness.overload_events_total.saturating_add(1);
    }
}

pub fn transformer_thermal_stress_system(
    mut q: Query<
        (
            &crate::entities::production::power::ElectricalGrid,
            &mut crate::entities::production::power::ThermalComponent,
        ),
        With<crate::entities::production::power::TransformerComponent>,
    >,
) {
    for (grid, mut thermal) in &mut q {
        let ratio = grid.total_load / grid.total_capacity.max(f32::EPSILON);
        if ratio > 1.0 {
            thermal.current_temperature = (thermal.current_temperature + (ratio - 1.0) * 2.5)
                .min(thermal.max_temperature);
        } else {
            thermal.current_temperature = (thermal.current_temperature - 0.05).max(25.0);
        }
    }
}

pub fn economy_sim_running(ctrl: Res<SimControlState>) -> bool {
    ctrl.should_tick()
}

pub struct ResourceFlowPlugin;

impl Plugin for ResourceFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceFlowRegistry>()
            .init_resource::<ResourceFlowSimWitness>()
            .add_message::<crate::entities::production::power::GridOverloadEvent>()
            .add_systems(
                Update,
                (
                    register_resource_flow_nodes_system,
                    link_supply_chain_edges_system,
                )
                    .chain()
                    .after(crate::economy::activation::activate_industrial_facilities_system),
            )
            .add_systems(
                Update,
                (
                    propagate_resource_flow_system,
                    apply_starvation_to_smelter_system,
                    transformer_thermal_stress_system,
                )
                    .chain()
                    .after(crate::economy::logistics::LogisticsSimulationSet::FreightDispatch)
                    .run_if(economy_sim_running),
            )
            .add_systems(
                Update,
                collect_grid_overload_witness_system
                    .after(crate::entities::production::power::grid_topology::emit_grid_overload_signals)
                    .run_if(economy_sim_running),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::economy::activation::{
        activate_industrial_facilities_system, BuildingDefinitionRef,
    };
    use crate::entities::production::aluminum::AluminumSmelterRuntime;
    use crate::strategic::{ConstructionSite, PlannedSite, SiteArchetype, SiteConstructionPhase};
    use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteId};

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

    fn flow_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::economy::logistics::LogisticsThroughputPlugin);
        crate::dev::logistics_throughput_todos::register_logistics_throughput_todo_hooks(&mut app);
        app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
        app.insert_resource(SimControlState::default());
        app.insert_resource(crate::strategic::LogisticsGraph::default());
        app.insert_resource(crate::systems::transport::TransportNavExport::default());
        app.insert_resource(crate::systems::transport::TransportEdgeDirectory::default());
        app.insert_resource(crate::systems::transport::TransportFieldStore::default());
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
        app
    }

    #[test]
    fn maps_concrete_and_electricity_tags() {
        assert_eq!(resource_type_from_tag("Concrete"), Some(ResourceType::Concrete));
        assert_eq!(
            resource_type_from_tag("Electricity"),
            Some(ResourceType::Electricity)
        );
        assert_eq!(resource_type_from_tag("Bauxite"), None);
    }

    #[test]
    fn smelter_node_lists_alumina_consume() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        let def = reg.get("aluminum_smelter1").expect("smelter def");
        let node = flow_node_from_definition(def);
        assert!(node.consumption.iter().any(|r| r.tag == "Alumina"));
    }

    #[test]
    fn aluminum_chain_links_edges() {
        let mut app = flow_test_app();
        spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        spawn_operational(
            &mut app,
            "aluminum_alumina_refinery",
            2,
            BuildSiteTile { x: 1, z: 0 },
        );
        spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 0 });
        app.update();
        assert!(
            app.world().resource::<ResourceFlowRegistry>().edges.len() >= 2,
            "expected mine→refinery→smelter edges"
        );
    }

    #[test]
    fn starved_smelter_drops_efficiency_when_upstream_empty() {
        let mut app = flow_test_app();
        spawn_operational(&mut app, "aluminum_smelter1", 10, BuildSiteTile { x: 0, z: 0 });
        app.update();
        for _ in 0..8 {
            app.update();
        }
        let mut smelter_e = None;
        let mut q = app
            .world_mut()
            .query::<(Entity, &ResourceFlowNode, &AluminumSmelterRuntime)>();
        for (entity, node, smelter) in q.iter(app.world()) {
            if node.catalog_id == "aluminum_smelter1" {
                smelter_e = Some((entity, smelter.current_efficiency));
                break;
            }
        }
        let (entity, eff) = smelter_e.expect("smelter entity");
        assert!(
            app.world().get::<FacilityFlowState>(entity).is_some_and(|f| f.starved),
            "smelter should be starved without upstream alumina"
        );
        assert!(eff < 0.88, "starvation should reduce efficiency from 0.88");
    }

    #[test]
    fn blocked_edge_transfers_nothing() {
        let mut from = ResourceFlowNode {
            catalog_id: "a".into(),
            inventory: HashMap::new(),
            buffer_by_tag: HashMap::from([("Bauxite".into(), 10.0)]),
            throughput_limit: 5.0,
            production: vec![],
            consumption: vec![],
        };
        let mut to = flow_node_from_definition(
            &load_building_definitions_from_dir(default_buildings_dir())
                .get("aluminum_alumina_refinery")
                .expect("def"),
        );
        let edge = ResourceFlowEdge {
            from: Entity::PLACEHOLDER,
            to: Entity::PLACEHOLDER,
            transport_mode: TransportMode::Truck,
            max_rate: 5.0,
            latency_ticks: 0.0,
            path_open: false,
            route_handle: None,
            buffer_tag: Some("Bauxite".into()),
        };
        let moved = transfer_along_edge(&edge, &mut from, &mut to);
        assert_eq!(moved, 0.0);
        assert_eq!(to.buffer_by_tag.get("Bauxite").copied().unwrap_or(0.0), 0.0);
    }
}
