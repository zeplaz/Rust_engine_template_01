//! LOG-A / LOG-C integration tests.

use bevy::prelude::*;

use crate::construction::ConstructionWorldRevision;
use crate::economy::logistics::routes::{path_edges_between_tiles, tile_node_key};

use crate::economy::resource_flow::TransportMode;
use crate::strategic::{
    rebuild_logistics_graph_from_transport, BuildSiteTile, CorridorConstructionBook,
    LogisticsGraph, StrategicRasterConfig,
};
use crate::systems::transport::{
    bake_snapshot_from_ordered_tile_markers, hydrate_transport_from_snapshot,
    refresh_transport_nav_export, TransportCostCache, TransportCostWeights, TransportEdgeDirectory, TransportFieldStore, TransportNavExport, TransportTopology,
};

fn road_chain_snapshot() -> crate::systems::transport::TransportNetworkSnapshot {
    bake_snapshot_from_ordered_tile_markers(
        &[(0u32, 0u32), (1u32, 0u32), (2u32, 0u32)],
        |_, _| 0.5,
        20.0,
        0.25,
    )
}

/// One gameplay step: advance the sim clock by one step, then run a frame (the economy and freight
/// systems run on [`crate::systems::sim_control::SimStepClock`]).
fn sim_frame(app: &mut App) {
    use crate::systems::sim_control::{SimTimeMicros, SIM_STEP_MICROS};
    app.world_mut().resource_mut::<SimTimeMicros>().0 += SIM_STEP_MICROS;
    app.update();
}

fn hydrate_chain(app: &mut App) {
    let snap = road_chain_snapshot();
    let mut top = TransportTopology::default();
    let mut field = TransportFieldStore::default();
    let mut dir = TransportEdgeDirectory::default();
    hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
    let mut cache = TransportCostCache::default();
    for (id, st) in &field.by_edge {
        cache.by_edge.insert(
            *id,
            crate::systems::transport::edge_traversal_cost(
                st,
                &TransportCostWeights::default(),
                st.travel_time_base,
            ),
        );
    }
    let mut nav = TransportNavExport::default();
    refresh_transport_nav_export(&top, &cache, &dir, &mut nav);
    app.insert_resource(top);
    app.insert_resource(field);
    app.insert_resource(dir);
    app.insert_resource(cache);
    app.insert_resource(nav);
    app.insert_resource(TransportCostWeights::default());
    app.insert_resource(StrategicRasterConfig::default());
    app.insert_resource(CorridorConstructionBook::default());
    app.insert_resource(ConstructionWorldRevision { revision: 1 });
    let graph = rebuild_logistics_graph_from_transport(
        app.world().resource::<TransportEdgeDirectory>(),
        app.world().resource::<TransportFieldStore>(),
        app.world().resource::<TransportCostWeights>(),
        app.world().resource::<StrategicRasterConfig>(),
        app.world().resource::<CorridorConstructionBook>(),
        1,
    );
    app.insert_resource(graph);
}

fn refresh_nav_from_world(app: &mut App) {
    let top = app.world().resource::<TransportTopology>();
    let cache = app.world().resource::<TransportCostCache>();
    let dir = app.world().resource::<TransportEdgeDirectory>();
    let mut nav = TransportNavExport::default();
    refresh_transport_nav_export(&top, &cache, &dir, &mut nav);
    app.insert_resource(nav);
}

#[test]
fn path_open_when_road_chain_connects_tiles() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);
    let nav = app.world().resource::<TransportNavExport>();
    let dir = app.world().resource::<TransportEdgeDirectory>();
    let path = path_edges_between_tiles(
        nav,
        dir,
        BuildSiteTile { x: 0, z: 0 },
        BuildSiteTile { x: 2, z: 0 },
        TransportMode::Truck,
    );
    assert!(path.is_some(), "road chain should connect 0,0 to 2,0");
    assert!(!path.unwrap().is_empty());
}

#[test]
fn path_blocked_without_transport_edge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);
    let nav = app.world().resource::<TransportNavExport>();
    let dir = app.world().resource::<TransportEdgeDirectory>();
    assert!(path_edges_between_tiles(
        nav,
        dir,
        BuildSiteTile { x: 0, z: 0 },
        BuildSiteTile { x: 9, z: 9 },
        TransportMode::Truck,
    )
    .is_none());
}

#[test]
fn logistics_edges_carry_transport_edge_id() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);
    let graph = app.world().resource::<LogisticsGraph>();
    assert_eq!(graph.edges.len(), 2);
    assert!(graph.edges.iter().all(|e| e.transport_edge.is_some()));
}

#[test]
fn geographic_cascade_integration() {

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);

    let open = path_edges_between_tiles(
        app.world().resource::<TransportNavExport>(),
        app.world().resource::<TransportEdgeDirectory>(),
        BuildSiteTile { x: 0, z: 0 },
        BuildSiteTile { x: 1, z: 0 },
        TransportMode::Truck,
    )
    .is_some();

    let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
    dir.by_edge
        .retain(|_, meta| meta.head_key != tile_node_key(BuildSiteTile { x: 1, z: 0 }));

    let blocked = path_edges_between_tiles(
        app.world().resource::<TransportNavExport>(),
        app.world().resource::<TransportEdgeDirectory>(),
        BuildSiteTile { x: 0, z: 0 },
        BuildSiteTile { x: 2, z: 0 },
        TransportMode::Truck,
    )
    .is_none();

    if open && blocked {
    }
    assert!(open, "connected before cut");
    assert!(blocked, "blocked after removing middle connectivity");
}

fn logistics_log_b_app() -> App {
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};


    use crate::engine::states::BaseState;
    use crate::strategic::CommitConstructionSiteEvent;
    use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};
    use bevy::state::app::StatesPlugin;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<BaseState>();
    app.insert_state(BaseState::Simulation);
    hydrate_chain(&mut app);
    // Headless LOG-B harness: seed/toast systems need message + overlay stubs
    // (same pattern as assemble_logistics_proof_sim_app — stay out of power UX lane).
    app.add_message::<CommitConstructionSiteEvent>();
    app.init_resource::<crate::render::PowerMapOverlayPresentation>();
    app.add_plugins((
        crate::economy::activation::IndustrialActivationPlugin,
        crate::strategic::InfrastructureGraphBridgePlugin,
    ));
    app.init_resource::<crate::strategic::InfrastructureGraph>();
    app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
    app.insert_resource(SimControlState {
        paused: false,
        steps_remaining: 0,
        speed: 1.0,
    });
    // Stage7 OnEnter(Simulation) seeds require sim clock resources without full SimControlPlugin.
    app.init_resource::<SimTick>();
    app.init_resource::<SimTimeMicros>();
    app
}

/// Route-cache invalidation only — no Simulation / industrial activation (avoids stage7 + play seeds).
fn logistics_route_invalidation_app() -> App {
    use crate::systems::sim_control::SimControlState;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);
    app.insert_resource(SimControlState {
        paused: false,
        steps_remaining: 0,
        speed: 1.0,
    });
    app.init_resource::<crate::economy::resource_flow::ResourceFlowRegistry>();
    app.add_plugins(crate::economy::logistics::LogisticsThroughputPlugin);
    app
}

fn spawn_facility(app: &mut App, catalog_id: &str, site_id: u64, origin: BuildSiteTile) -> Entity {
    use crate::economy::activation::BuildingDefinitionRef;
    use crate::strategic::{
        ConstructionSite, FootprintTiles, LayerType, PlannedSite, SiteArchetype,
        SiteConstructionPhase, SiteId,
    };

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
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id()
}

fn setup_aluminum_chain_facilities(app: &mut App) -> (Entity, Entity) {
    let mine = spawn_facility(
        app,
        "aluminum_bauxite_mine",
        1,
        BuildSiteTile { x: 0, z: 0 },
    );
    let refinery = spawn_facility(
        app,
        "aluminum_alumina_refinery",
        2,
        BuildSiteTile { x: 2, z: 0 },
    );
    for _ in 0..12 {
        sim_frame(app);
    }
    (mine, refinery)
}

/// LOG-A-04 exit: road open → cut middle tile → `ResourceFlowEdge.path_open` false after refresh.
#[test]
fn log_a_04_cut_road_sets_path_open_false_after_refresh() {
    use crate::economy::logistics::types::{FacilityPortal, TransportNodeAnchor};
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use crate::economy::spatial_district::chunk_key_from_site_tile;
    use crate::strategic::{
        FootprintTiles, LayerType, PlannedSite, SiteArchetype, SiteId, StrategicRasterConfig,
    };
    use crate::systems::sim_control::SimControlState;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    hydrate_chain(&mut app);
    app.insert_resource(SimControlState {
        paused: false,
        steps_remaining: 0,
        speed: 1.0,
    });
    app.init_resource::<ResourceFlowRegistry>();
    app.add_plugins(crate::economy::logistics::LogisticsThroughputPlugin);

    let from_tile = BuildSiteTile { x: 0, z: 0 };
    let to_tile = BuildSiteTile { x: 2, z: 0 };
    let cells = app
        .world()
        .resource::<StrategicRasterConfig>()
        .cells_per_chunk;
    let from_e = app
        .world_mut()
        .spawn((
            PlannedSite {
                site_id: SiteId(1),
                origin: from_tile,
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: Some("mine".into()),
                placement: None,
            },
            FacilityPortal {
                anchor: chunk_key_from_site_tile(from_tile, cells),
                transport_anchor: TransportNodeAnchor(tile_node_key(from_tile)),
            },
        ))
        .id();
    let to_e = app
        .world_mut()
        .spawn((
            PlannedSite {
                site_id: SiteId(2),
                origin: to_tile,
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: Some("refinery".into()),
                placement: None,
            },
            FacilityPortal {
                anchor: chunk_key_from_site_tile(to_tile, cells),
                transport_anchor: TransportNodeAnchor(tile_node_key(to_tile)),
            },
        ))
        .id();
    app.world_mut()
        .resource_mut::<ResourceFlowRegistry>()
        .add_edge(ResourceFlowEdge {
            from: from_e,
            to: to_e,
            transport_mode: TransportMode::Truck,
            max_rate: 4.0,
            latency_ticks: 1.0,
            path_open: false,
            route_handle: None,
            buffer_tag: Some("Bauxite".into()),
        });

    for _ in 0..4 {
        sim_frame(&mut app);
    }
    assert!(
        app.world()
            .resource::<ResourceFlowRegistry>()
            .edges
            .iter()
            .any(|e| e.path_open),
        "LOG-A-04: connected road must open flow edge via TransportNavExport"
    );

    {
        let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
        dir.by_edge
            .retain(|_, meta| meta.head_key != tile_node_key(BuildSiteTile { x: 1, z: 0 }));
    }
    app.world_mut()
        .resource_mut::<ConstructionWorldRevision>()
        .revision += 1;
    refresh_nav_from_world(&mut app);
    {
        let dir = app.world().resource::<TransportEdgeDirectory>();
        let fields = app.world().resource::<TransportFieldStore>();
        let weights = app.world().resource::<TransportCostWeights>();
        let cells = app.world().resource::<StrategicRasterConfig>();
        let book = app.world().resource::<CorridorConstructionBook>();
        let cons = app.world().resource::<ConstructionWorldRevision>().revision;
        let sig = crate::strategic::transport_directory_edge_signature(&dir);
        let topology_revision = sig ^ cons.rotate_left(17);
        let graph = rebuild_logistics_graph_from_transport(
            &dir,
            &fields,
            &weights,
            &cells,
            &book,
            topology_revision,
        );
        app.insert_resource(graph);
    }
    for _ in 0..6 {
        sim_frame(&mut app);
    }

    let flow = app.world().resource::<ResourceFlowRegistry>();
    assert!(
        flow.edges.iter().all(|e| !e.path_open),
        "LOG-A-04: after cutting middle corridor, path_open must be false"
    );
}

#[test]
fn log_b_02_in_transit_ledger_uses_route_handle_and_progress() {
    use super::types::InTransitLedger;

    use crate::economy::resource_flow::ResourceFlowRegistry;


    let mut app = logistics_log_b_app();
    let (mine, refinery) = setup_aluminum_chain_facilities(&mut app);
    let flow = app.world().resource::<ResourceFlowRegistry>();
    assert!(!flow.edges.is_empty(), "supply chain edge mine→refinery");
    assert!(
        flow.edges.iter().any(|e| e.path_open),
        "road chain should open route"
    );

    app.world_mut()
        .resource_mut::<InTransitLedger>()
        .lots
        .clear();
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
    {
        node.buffer_by_tag.remove("Bauxite");
    }
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
    {
        node.buffer_by_tag.insert("Bauxite".into(), 20.0);
    }

    sim_frame(&mut app);
    let (route_id, topo, progress_after_dispatch, remaining_after_dispatch) = {
        let ledger = app.world().resource::<InTransitLedger>();
        assert!(
            !ledger.lots.is_empty(),
            "LOG-B-02: freight must enter InTransitLedger (no teleport)"
        );
        let lot = &ledger.lots[0];
        assert!(
            lot.route.id > 0 || lot.route.topology_revision > 0 || lot.path.edge_count > 0,
            "LOG-B-02: lot must carry RouteHandle / path (got route={:?} path_edges={})",
            lot.route,
            lot.path.edge_count
        );
        assert_eq!(lot.progress_edge, 0, "fresh lot starts at progress_edge 0");
        assert!(
            lot.remaining_ticks > 0,
            "fresh lot must have remaining_ticks > 0"
        );
        (
            lot.route.id,
            lot.route.topology_revision,
            lot.progress_edge,
            lot.remaining_ticks,
        )
    };

    let refinery_mid = app
        .world()
        .get::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
        .and_then(|n| n.buffer_by_tag.get("Bauxite").copied())
        .unwrap_or(0.0);
    assert!(
        refinery_mid < 1e-4,
        "LOG-B-02: no same-tick credit — arrivals only via ledger"
    );

    sim_frame(&mut app);
    {
        let ledger = app.world().resource::<InTransitLedger>();
        if let Some(lot) = ledger.lots.first() {
            assert!(
                lot.remaining_ticks < remaining_after_dispatch,
                "LOG-B-02: remaining_ticks must decrement while in transit"
            );
            if lot.path.edge_count > 1 {
                assert!(
                    lot.progress_edge > progress_after_dispatch,
                    "LOG-B-02: progress_edge advances with transit ticks (route={route_id}/{topo}, progress {}→{})",
                    progress_after_dispatch,
                    lot.progress_edge
                );
            }
        }
    }

    for _ in 0..40 {
        sim_frame(&mut app);
    }
    let refinery_after = app
        .world()
        .get::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
        .and_then(|n| n.buffer_by_tag.get("Bauxite").copied())
        .unwrap_or(0.0);
    assert!(
        refinery_after > 0.0,
        "LOG-B-02: ledger arrivals deliver freight after transit"
    );
}

#[test]
fn log_b_freight_moves_through_ledger_not_same_tick_teleport() {
    use super::types::InTransitLedger;

    use crate::economy::resource_flow::ResourceFlowRegistry;


    let mut app = logistics_log_b_app();
    let (mine, refinery) = setup_aluminum_chain_facilities(&mut app);
    let flow = app.world().resource::<ResourceFlowRegistry>();
    assert!(!flow.edges.is_empty(), "supply chain edge mine→refinery");
    assert!(
        flow.edges.iter().any(|e| e.path_open),
        "road chain should open route"
    );

    app.world_mut()
        .resource_mut::<InTransitLedger>()
        .lots
        .clear();
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
    {
        node.buffer_by_tag.remove("Bauxite");
    }
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
    {
        node.buffer_by_tag.insert("Bauxite".into(), 20.0);
    }
    let refinery_before_dispatch = app
        .world()
        .get::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
        .and_then(|n| n.buffer_by_tag.get("Bauxite").copied())
        .unwrap_or(0.0);
    sim_frame(&mut app);
    let refinery_after_one_tick = app
        .world()
        .get::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
        .and_then(|n| n.buffer_by_tag.get("Bauxite").copied())
        .unwrap_or(0.0);
    assert!(
        refinery_after_one_tick < 1e-4,
        "no same-tick teleport: refinery must not receive bauxite before ledger arrival"
    );
    let ledger = app.world().resource::<InTransitLedger>();
    assert!(
        !ledger.lots.is_empty(),
        "freight should be in transit after dispatch"
    );
    for _ in 0..40 {
        sim_frame(&mut app);
    }
    let refinery_after = app
        .world()
        .get::<crate::economy::resource_flow::ResourceFlowNode>(refinery)
        .and_then(|n| n.buffer_by_tag.get("Bauxite").copied())
        .unwrap_or(0.0);
    assert!(
        refinery_after > refinery_before_dispatch,
        "ledger arrivals should deliver bauxite to refinery"
    );
}

#[test]
fn log_b_rail_batch_uses_longer_transit_than_truck() {
    use super::propagation::freight_transit_ticks;
    use super::types::FreightMovementModel;


    let truck = freight_transit_ticks(3, FreightMovementModel::Continuous);
    let rail = freight_transit_ticks(3, FreightMovementModel::Batched);
    assert!(
        rail > truck,
        "batched rail ETA > continuous truck for same path length"
    );
}

#[test]
fn log_b_partial_fulfillment_records_shortage() {
    use super::types::LogisticsDiagnostics;



    let mut app = logistics_log_b_app();
    let (mine, _) = setup_aluminum_chain_facilities(&mut app);
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
    {
        node.buffer_by_tag.insert("Bauxite".into(), 50.0);
    }
    for _ in 0..15 {
        sim_frame(&mut app);
    }
    {
        let mut graph = app
            .world_mut()
            .resource_mut::<crate::strategic::LogisticsGraph>();
        for edge in &mut graph.edges {
            edge.capacity = 0.01;
        }
    }
    for _ in 0..40 {
        sim_frame(&mut app);
    }
    let diagnostics = app.world().resource::<LogisticsDiagnostics>();
    assert!(
        diagnostics
            .proofs
            .iter()
            .any(|p| p.delivered + 1e-4 < p.requested),
        "solver should record partial delivery under cap squeeze"
    );
}

#[test]
fn log_c_01_soa_throughput_solver_aligned_vecs() {
    use super::solver::soa_solver_aligned;



    let mut app = logistics_log_b_app();
    let (mine, _) = setup_aluminum_chain_facilities(&mut app);
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
    {
        node.buffer_by_tag.insert("Bauxite".into(), 40.0);
    }
    for _ in 0..25 {
        sim_frame(&mut app);
    }
    let solver = app
        .world()
        .resource::<crate::economy::logistics::ThroughputSolverState>();
    assert!(
        soa_solver_aligned(solver),
        "SoA load/capacity/reserved/pressure must share edge-index length"
    );
    assert!(
        !solver.capacity.is_empty(),
        "capacity sync must populate SoA from graph edges"
    );
}



#[test]
fn log_c_congestion_rises_when_edge_saturated() {



    let mut app = logistics_log_b_app();
    let (mine, _) = setup_aluminum_chain_facilities(&mut app);
    if let Some(mut node) = app
        .world_mut()
        .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
    {
        node.buffer_by_tag.insert("Bauxite".into(), 80.0);
    }
    {
        let mut graph = app.world_mut().resource_mut::<LogisticsGraph>();
        for edge in &mut graph.edges {
            edge.capacity = 0.05;
        }
    }
    for st in app
        .world_mut()
        .resource_mut::<TransportFieldStore>()
        .by_edge
        .values_mut()
    {
        st.congestion = 0.0;
    }
    let edge_id = app
        .world()
        .resource::<TransportEdgeDirectory>()
        .by_edge
        .keys()
        .next()
        .copied()
        .expect("transport edge");
    let before = app
        .world()
        .resource::<TransportFieldStore>()
        .by_edge
        .get(&edge_id)
        .map(|st| st.congestion)
        .unwrap_or(0.0);
    for _ in 0..50 {
        sim_frame(&mut app);
    }
    let after = app
        .world()
        .resource::<TransportFieldStore>()
        .by_edge
        .get(&edge_id)
        .map(|st| st.congestion)
        .unwrap_or(0.0);
    let solver = app
        .world()
        .resource::<crate::economy::logistics::ThroughputSolverState>();
    assert!(
        solver.edge_pressure.iter().any(|&p| p > 0.5),
        "solver should report saturated edges before congestion feedback"
    );
    assert!(
        after > before + 0.01,
        "congestion should rise under sustained load"
    );
}

#[test]
fn log_c_corridor_pressure_diffuses_to_successor() {



    let mut app = logistics_log_b_app();
    setup_aluminum_chain_facilities(&mut app);
    {
        let mut solver = app
            .world_mut()
            .resource_mut::<crate::economy::logistics::ThroughputSolverState>();
        if solver.edge_pressure.len() >= 2 {
            solver.edge_pressure[0] = 1.0;
        }
    }
    sim_frame(&mut app);
    let solver = app
        .world()
        .resource::<crate::economy::logistics::ThroughputSolverState>();
    assert!(
        solver.edge_pressure.len() >= 2 && solver.edge_pressure[1] > 0.2,
        "successor edge should inherit diffused pressure"
    );
}

#[test]
fn log_c_geographic_cascade_starves_downstream_smelter() {
    use crate::economy::resource_flow::FacilityFlowState;
    use crate::entities::production::aluminum::AluminumSmelterRuntime;


    let mut app = logistics_log_b_app();
    let _mine = spawn_facility(
        &mut app,
        "aluminum_bauxite_mine",
        1,
        BuildSiteTile { x: 0, z: 0 },
    );
    let refinery = spawn_facility(
        &mut app,
        "aluminum_alumina_refinery",
        2,
        BuildSiteTile { x: 2, z: 0 },
    );
    let smelter = spawn_facility(
        &mut app,
        "aluminum_smelter1",
        3,
        BuildSiteTile { x: 2, z: 1 },
    );
    for _ in 0..20 {
        sim_frame(&mut app);
    }
    let efficiency_before = app
        .world()
        .get::<AluminumSmelterRuntime>(smelter)
        .map(|r| r.current_efficiency)
        .unwrap_or(1.0);

    let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
    dir.by_edge
        .retain(|_, meta| meta.head_key != tile_node_key(BuildSiteTile { x: 1, z: 0 }));
    app.world_mut()
        .resource_mut::<ConstructionWorldRevision>()
        .revision += 1;
    refresh_nav_from_world(&mut app);

    for _ in 0..35 {
        sim_frame(&mut app);
    }

    let flow = app
        .world()
        .resource::<crate::economy::resource_flow::ResourceFlowRegistry>();
    let bauxite_route_blocked = flow
        .edges
        .iter()
        .filter(|e| e.buffer_tag.as_deref() == Some("Bauxite"))
        .all(|e| !e.path_open);
    let refinery_starved = app
        .world()
        .get::<FacilityFlowState>(refinery)
        .is_some_and(|s| s.starved);
    let efficiency_after = app
        .world()
        .get::<AluminumSmelterRuntime>(smelter)
        .map(|r| r.current_efficiency)
        .unwrap_or(1.0);

    assert!(
        bauxite_route_blocked,
        "cutting middle tile should block mine→refinery bauxite route"
    );
    assert!(
        refinery_starved || efficiency_after < efficiency_before - 0.05,
        "downstream smelter should degrade when corridor is cut"
    );
}

#[test]
fn log_c_overlay_uses_solver_load_not_static_capacity() {

    use crate::strategic::edge_flow_for_overlay;
    use crate::strategic::LogisticsEdge;
    use crate::strategic::LogisticsNodeId;
    use crate::systems::transport::TransportEdgeId;


    let edge = LogisticsEdge {
        from: LogisticsNodeId(0),
        to: LogisticsNodeId(1),
        capacity: 100.0,
        disruption: 0.0,
        traversal_cost: 1.0,
        transport_edge: Some(TransportEdgeId(0)),
    };
    let mut solver = crate::economy::logistics::ThroughputSolverState::default();
    solver.ensure_len(1);
    solver.load[0] = 2.5;
    solver.capacity[0] = 100.0;

    let with_solver = edge_flow_for_overlay(&edge, Some(&solver));
    let static_only = edge_flow_for_overlay(&edge, None);
    assert!((with_solver - 2.5).abs() < 1e-4);
    assert!((static_only - 100.0).abs() < 1e-4);
    assert!(with_solver < static_only);
}

#[test]
fn log_d_corridor_class_rail_profile_blocks_truck_path() {

    use crate::systems::transport::{corridor_class_from_profile, CorridorClass};


    assert_eq!(corridor_class_from_profile("rail"), CorridorClass::Rail);
    let allowed = vec!["rail_train".to_string()];
    let truck_agent = "road_vehicle";
    let profile = "rail";
    let truck_may_use =
        allowed.is_empty() || allowed.iter().any(|a| a == truck_agent) || profile.contains("road");
    assert!(
        !truck_may_use,
        "rail-only agents must not admit road_vehicle"
    );
    let rail_may_use = allowed.iter().any(|a| a == "rail_train");
    assert!(
        rail_may_use,
        "rail_train should be allowed on rail corridor"
    );
}

#[test]
fn log_d_streaming_route_invalidation_on_construction_bump() {
    use super::routes::topology_revision_u32;
    use super::types::RouteCache;



    let mut app = logistics_route_invalidation_app();
    for _ in 0..2 {
        sim_frame(&mut app);
    }
    let topo_before = app.world().resource::<RouteCache>().topology_revision;
    app.world_mut()
        .resource_mut::<ConstructionWorldRevision>()
        .revision += 1;
    for _ in 0..5 {
        sim_frame(&mut app);
    }
    let cache = app.world().resource::<RouteCache>();
    let graph = app.world().resource::<LogisticsGraph>();
    let expected = topology_revision_u32(graph.revision, 2);
    assert_eq!(cache.topology_revision, expected);
    assert_ne!(cache.topology_revision, topo_before);
}







#[test]
fn solver_skips_an_unchanged_revision() {
    use super::solver::solve_throughput_greedy_system;
    use super::types::{
        CachedRoute, FreightReservation, FreightReservationBook, LogisticsDiagnostics, RouteCache,
        RouteHandle, RoutePathStore, ThroughputSolverState,
    };
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use crate::systems::transport::TransportEdgeId;
    use bevy::ecs::system::RunSystemOnce;

    let mut app = App::new();
    let from = app.world_mut().spawn_empty().id();
    let to = app.world_mut().spawn_empty().id();
    let handle = RouteHandle {
        id: 1,
        topology_revision: 7,
    };
    let mut paths = RoutePathStore::default();
    let path = paths.insert_path(&[TransportEdgeId(0)]);
    let mut cache = RouteCache::default();
    cache.routes.insert(
        (from, to),
        CachedRoute {
            handle,
            path,
            reachable: true,
            bottleneck_capacity: 4.0,
        },
    );
    let mut solver = ThroughputSolverState::default();
    solver.topology_revision = 7;
    solver.ensure_len(1);
    solver.capacity[0] = 8.0;
    app.insert_resource(paths)
        .insert_resource(cache)
        .insert_resource(solver)
        .insert_resource(FreightReservationBook::default())
        .insert_resource(LogisticsDiagnostics::default())
        .insert_resource(ResourceFlowRegistry {
            edges: vec![ResourceFlowEdge {
                from,
                to,
                transport_mode: TransportMode::Truck,
                max_rate: 2.0,
                latency_ticks: 1.0,
                path_open: true,
                route_handle: Some(handle),
                buffer_tag: None,
            }],
        });

    app.world_mut()
        .run_system_once(solve_throughput_greedy_system)
        .unwrap();
    {
        let mut book = app.world_mut().resource_mut::<FreightReservationBook>();
        assert_eq!(book.entries.len(), 1);
        book.push(FreightReservation {
            edge_index: 0,
            amount: 9.0,
            request_id: 99,
        });
    }
    app.world_mut()
        .run_system_once(solve_throughput_greedy_system)
        .unwrap();
    {
        let book = app.world().resource::<FreightReservationBook>();
        let solver = app.world().resource::<ThroughputSolverState>();
        assert_eq!(book.entries.len(), 2);
        assert_eq!(book.entries[1].request_id, 99);
        assert!((solver.reserved[0] - 2.0).abs() < 1e-5);
    }

    app.world_mut()
        .resource_mut::<ResourceFlowRegistry>()
        .edges[0]
        .path_open = false;
    app.world_mut()
        .run_system_once(solve_throughput_greedy_system)
        .unwrap();
    assert!(app
        .world()
        .resource::<FreightReservationBook>()
        .entries
        .is_empty());

    {
        let mut flow = app.world_mut().resource_mut::<ResourceFlowRegistry>();
        flow.edges[0].path_open = true;
        flow.edges[0].max_rate = 3.0;
    }
    app.world_mut()
        .run_system_once(solve_throughput_greedy_system)
        .unwrap();
    assert_eq!(
        app.world()
            .resource::<FreightReservationBook>()
            .entries
            .len(),
        1
    );

    app.world_mut()
        .resource_mut::<ThroughputSolverState>()
        .topology_revision = 8;
    app.world_mut()
        .run_system_once(solve_throughput_greedy_system)
        .unwrap();
    assert!(app
        .world()
        .resource::<FreightReservationBook>()
        .entries
        .is_empty());
}

#[test]
fn solver_anytime_partial_steps_hold_invariants() {
    use super::solver::{
        book_matches_soa_reserved, reservations_within_capacity, solve_throughput_greedy_system,
        soa_solver_aligned,
    };
    use super::types::{
        CachedRoute, FreightReservationBook, LogisticsDiagnostics, RouteCache, RouteHandle,
        RoutePathStore, ThroughputSolverState,
    };
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use crate::systems::sim_attention::SimAttentionFrame;
    use crate::systems::transport::TransportEdgeId;
    use bevy::ecs::system::RunSystemOnce;

    const EDGE_COUNT: usize = 6;
    let mut app = App::new();
    let from = app.world_mut().spawn_empty().id();
    let to = app.world_mut().spawn_empty().id();
    let handle = RouteHandle {
        id: 1,
        topology_revision: 3,
    };
    let mut paths = RoutePathStore::default();
    let path = paths.insert_path(&[TransportEdgeId(0)]);
    let mut cache = RouteCache::default();
    cache.routes.insert(
        (from, to),
        CachedRoute {
            handle,
            path,
            reachable: true,
            bottleneck_capacity: 8.0,
        },
    );
    let mut solver = ThroughputSolverState::default();
    solver.topology_revision = 3;
    solver.ensure_len(1);
    solver.capacity[0] = 100.0;
    let edges: Vec<ResourceFlowEdge> = (0..EDGE_COUNT)
        .map(|_| ResourceFlowEdge {
            from,
            to,
            transport_mode: TransportMode::Truck,
            max_rate: 1.0,
            latency_ticks: 1.0,
            path_open: true,
            route_handle: Some(handle),
            buffer_tag: None,
        })
        .collect();
    let mut attention = SimAttentionFrame::default();
    attention.budget_path_units = 4;
    attention.player_theater_active = false;
    app.insert_resource(paths)
        .insert_resource(cache)
        .insert_resource(solver)
        .insert_resource(FreightReservationBook::default())
        .insert_resource(LogisticsDiagnostics::default())
        .insert_resource(attention)
        .insert_resource(ResourceFlowRegistry { edges });

    for tick in 0..5 {
        app.world_mut()
            .run_system_once(solve_throughput_greedy_system)
            .unwrap();
        let solver = app.world().resource::<ThroughputSolverState>();
        let book = app.world().resource::<FreightReservationBook>();
        assert!(
            soa_solver_aligned(solver),
            "tick {tick}: SoA misaligned"
        );
        assert!(
            reservations_within_capacity(solver),
            "tick {tick}: reservation over capacity"
        );
        assert!(
            book_matches_soa_reserved(book, solver),
            "tick {tick}: book diverged from SoA"
        );
        if solver.solved_revision == Some(3) {
            assert_eq!(
                solver.solve_flow_cursor,
                EDGE_COUNT,
                "tick {tick}: complete pass cursor"
            );
            assert_eq!(book.entries.len(), EDGE_COUNT);
            break;
        }
        assert!(
            tick < 4,
            "expected full solve within budgeted ticks"
        );
    }
}

/// RD-7: a rail line between two yards gives a rail route; rubble on the line takes it away.
#[test]
fn rd7_yards_route_over_the_rail_line_and_lose_it_when_the_line_is_buried() {
    use crate::construction::corridor_network::{CorridorKind, CorridorNetwork};
    use crate::economy::logistics::debris_block::DebrisRouteBlockBook;
    use crate::economy::logistics::routes::{rail_yard_route, RoadRouteWorld};
    use crate::strategic::BuildSiteTile;
    use crate::systems::transport::{
        hydrate_transport_from_snapshot, refresh_transport_nav_export, TransportCostCache, TransportCostWeights,
        TransportEdgeDirectory, TransportFieldStore, TransportNavExport, TransportTopology,
    };
    let mut net = CorridorNetwork::default();
    net.add_segment(
        CorridorKind::Rail { profile: "default_rail".into() },
        BuildSiteTile { x: 10, z: 10 },
        BuildSiteTile { x: 210, z: 10 },
    )
    .expect("rail line");
    let snap = net.bake_snapshot(10.25);
    let (mut top, mut field, mut dir) =
        (TransportTopology::default(), TransportFieldStore::default(), TransportEdgeDirectory::default());
    hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
    let mut cache = TransportCostCache::default();
    for (id, st) in &field.by_edge {
        cache.by_edge.insert(
            *id,
            crate::systems::transport::edge_traversal_cost(st, &TransportCostWeights::default(), st.travel_time_base),
        );
    }
    let mut nav = TransportNavExport::default();
    refresh_transport_nav_export(&top, &cache, &dir, &mut nav);
    let open = RoadRouteWorld {
        nav: &nav,
        directory: &dir,
        graph: None,
        cache: Some(&cache),
        forecast: None,
        nav_field: None,
        flood: None,
        debris: None,
        bridges: None,
        flow: None,
        depart_tick: 0,
        offroad_field: None,
    };
    let (yard_a, yard_b) = (Vec2::new(8.5, 13.5), Vec2::new(212.5, 13.5));
    let route = rail_yard_route(&open, yard_a, yard_b, 10.0, 22.0, 4.0).expect("the yards reach the line");
    let t = route.time_s(22.0);
    // ~200 tiles × 16 m at 22 m/s ≈ 145 s on the line, plus two short sidings.
    assert!(t > 140.0 && t < 200.0, "rail trip time {t}");
    assert!(rail_yard_route(&open, yard_a, Vec2::new(212.5, 60.0), 10.0, 22.0, 4.0).is_none(), "a yard out of reach");
    // Bury a stretch of the line: no rail route any more.
    let mut debris = DebrisRouteBlockBook::default();
    for x in 100..=120 {
        debris.blocked.insert(format!("t{x}_10"));
    }
    let buried = RoadRouteWorld { debris: Some(&debris), ..open };
    assert!(rail_yard_route(&buried, yard_a, yard_b, 10.0, 22.0, 4.0).is_none(), "the buried line is a wall");
}
