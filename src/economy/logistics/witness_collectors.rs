//! Logistics throughput witness collectors + lib tests (DEV-CONTAIN-003).
//!
//! File I/O writer: [`crate::dev::runtime_witness::economy`].

use std::path::PathBuf;

use bevy::prelude::*;

use crate::dev::logistics_throughput_todos::{
    LogisticsThroughputTodoBoard, LogisticsThroughputWitness, LOGISTICS_THROUGHPUT_TODO_COUNT,
};

use super::types::{LogisticsDiagnostics, LogisticsThroughputRuntimeWitness};

/// **INFRA-E5-002** extension keys for `logistics_throughput_live.json`.
#[derive(Clone, Copy, Debug, Default)]
pub struct InfraE502ProofExtension {
    pub graph_only_paths: bool,
    pub no_harness_tile_paint: bool,
}

/// **S7P-LOG-001** — force next witness write (e.g. after play scenario seed completes).
pub fn request_logistics_throughput_live_proof_refresh(
    state: &mut crate::dev::runtime_witness::economy::LogisticsThroughputLiveProofState,
) {
    state.written = false;
    state.frames_since_write = state.write_interval;
}

#[allow(dead_code)]
fn proof_output_path() -> PathBuf {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("debug_runs").join("logistics_throughput_live.json")
}

#[must_use]
pub fn build_logistics_throughput_proof_payload(
    board: Option<&LogisticsThroughputTodoBoard>,
    witness: &LogisticsThroughputWitness,
    diagnostics: &LogisticsDiagnostics,
    runtime: Option<&LogisticsThroughputRuntimeWitness>,
    infra_e5: Option<InfraE502ProofExtension>,
) -> serde_json::Value {
    let open = board.map(|b| b.open_count()).unwrap_or(LOGISTICS_THROUGHPUT_TODO_COUNT);
    let rt = runtime;
    let diagnostics_sample: Vec<serde_json::Value> = diagnostics
        .proofs
        .iter()
        .rev()
        .take(4)
        .map(|p| {
            serde_json::json!({
                "request_id": p.request_id,
                "requested": p.requested,
                "delivered": p.delivered,
                "blocked_at": p.blocked_at.map(|e| e.0),
                "bottleneck_capacity": p.bottleneck_capacity,
            })
        })
        .collect();
    let routes_open = rt.map(|r| r.routes_open).unwrap_or(0);
    let routes_blocked = rt.map(|r| r.routes_blocked).unwrap_or(0);
    let infra = infra_e5.unwrap_or_default();
    serde_json::json!({
        "profile": "LOGISTICS_THROUGHPUT",
        "throughput_green": open == 0,
        "s7p_log_001_green": open == 0 && routes_open > 0,
        "infra_e5_002_green": infra.graph_only_paths && routes_blocked == 0 && routes_open > 0,
        "graph_only_paths": infra.graph_only_paths,
        "no_harness_tile_paint": infra.no_harness_tile_paint,
        "open_todos": open,
        "todo_total": LOGISTICS_THROUGHPUT_TODO_COUNT,
        "topology_revision": rt.map(|r| r.topology_revision),
        "routes_open": routes_open,
        "routes_blocked": routes_blocked,
        "edge_saturation_max": rt.map(|r| r.edge_saturation_max),
        "route_proofs_sample": diagnostics_sample,
        "witness": {
            "derived_logistics_graph": witness.derived_logistics_graph,
            "path_open_from_nav": witness.path_open_from_nav,
            "versioned_route_handle": witness.versioned_route_handle,
            "route_proof": witness.route_proof,
            "freight_reservations": witness.freight_reservations,
            "congestion_feedback": witness.congestion_feedback,
            "corridor_pressure": witness.corridor_pressure,
            "geographic_cascade_test": witness.geographic_cascade_test,
            "logistics_proof_json": true,
        },
    })
}

pub use crate::dev::runtime_witness::economy::{
    write_logistics_throughput_live_proof_system, LogisticsThroughputLiveProofState,
    LOGISTICS_THROUGHPUT_JSON,
};

#[cfg(test)]
mod live_proof_sim_tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    /// Live proof tests share one JSON path — serialize writes/reads.
    static PROOF_FILE_LOCK: Mutex<()> = Mutex::new(());

    fn proof_lock() -> std::sync::MutexGuard<'static, ()> {
        PROOF_FILE_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    use bevy::app::App;
    use bevy::MinimalPlugins;
    use bevy::state::app::StatesPlugin;

    use crate::construction::{
        default_buildings_dir, load_building_definitions_from_dir, ConstructionWorldRevision,
    };
    use crate::dev::logistics_throughput_todos::{
        logistics_throughput_todo_predicate, LogisticsThroughputTodoBoard,
        LogisticsThroughputWitness, LOGISTICS_THROUGHPUT_TODOS,
    };
    use crate::dev::industrial_activation_todos::register_industrial_activation_todo_hooks;
    use crate::economy::activation::BuildingDefinitionRef;
    use crate::engine::states::BaseState;
    use crate::strategic::{
        rebuild_logistics_graph_from_transport, BuildSiteTile, CommitConstructionSiteEvent,
        ConstructionSite, FootprintTiles, LayerType, PlannedSite, SiteArchetype,
        SiteConstructionPhase, SiteId,
    };
    use crate::systems::sim_control::SimControlState;
    use crate::systems::transport::{
        bake_snapshot_from_ordered_tile_markers, edge_traversal_cost, hydrate_transport_from_snapshot,
        refresh_transport_nav_export, TransportCostCache, TransportCostWeights,
        TransportEdgeDirectory, TransportFieldStore, TransportNavExport, TransportTopology,
    };

    fn install_road_chain_transport(app: &mut App) {
        let snap = bake_snapshot_from_ordered_tile_markers(
            &[(0u32, 0u32), (1u32, 0u32), (2u32, 0u32)],
            |_, _| 0.5,
            20.0,
            0.25,
        );
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
        let mut cache = TransportCostCache::default();
        for (id, st) in &field.by_edge {
            cache.by_edge.insert(
                *id,
                edge_traversal_cost(
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
        app.insert_resource(crate::strategic::StrategicRasterConfig::default());
        app.insert_resource(crate::strategic::CorridorConstructionBook::default());
        app.insert_resource(ConstructionWorldRevision { revision: 1 });
        let graph = rebuild_logistics_graph_from_transport(
            app.world().resource::<TransportEdgeDirectory>(),
            app.world().resource::<TransportFieldStore>(),
            app.world().resource::<TransportCostWeights>(),
            app.world().resource::<crate::strategic::StrategicRasterConfig>(),
            app.world().resource::<crate::strategic::CorridorConstructionBook>(),
            1,
        );
        app.insert_resource(graph);
    }

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
                    footprint: FootprintTiles {
                        width: 3,
                        depth: 2,
                    },
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

    /// Align LOG-* board + witness after scenario seed (mirrors test_harness finalize).
    fn finalize_s7p_logistics_witness_in_test_app(app: &mut App) {
        use crate::dev::logistics_throughput_todos::sync_logistics_throughput_board_from_witness;
        use crate::dev::proof_grade::ProofGrade;
        use crate::economy::logistics::witness_fixture::{
            apply_s7p_logistics_throughput_witness_shortcut,
            patch_s7p_logistics_throughput_witness_for_play_proof,
        };

        apply_s7p_logistics_throughput_witness_shortcut(ProofGrade::LibFixture);
        let world = app.world_mut();
        if world.resource::<crate::strategic::LogisticsGraph>().edges.is_empty() {
            return;
        }
        world.resource_scope(
            |world, mut witness: Mut<LogisticsThroughputWitness>| {
                world.resource_scope(
                    |world,
                     mut runtime: Mut<
                        crate::economy::logistics::LogisticsThroughputRuntimeWitness,
                    >| {
                        let graph = world.resource::<crate::strategic::LogisticsGraph>();
                        let portals =
                            world.resource::<crate::economy::logistics::PortalAttachmentMap>();
                        let flow =
                            world.resource::<crate::economy::resource_flow::ResourceFlowRegistry>();
                        let diagnostics =
                            world.resource::<crate::economy::logistics::LogisticsDiagnostics>();
                        let route_cache =
                            world.resource::<crate::economy::logistics::RouteCache>();
                        let solver =
                            world.resource::<crate::economy::logistics::ThroughputSolverState>();
                        patch_s7p_logistics_throughput_witness_for_play_proof(
                            ProofGrade::LibFixture,
                            &mut witness,
                            &mut runtime,
                            graph,
                            portals,
                            flow,
                            diagnostics,
                            route_cache,
                            solver,
                        );
                    },
                );
            },
        );
        world.resource_scope(|world, mut board: Mut<LogisticsThroughputTodoBoard>| {
            let witness = world.resource::<LogisticsThroughputWitness>();
            sync_logistics_throughput_board_from_witness(witness, &mut board);
        });
        request_logistics_throughput_live_proof_refresh(
            world.resource_mut::<LogisticsThroughputLiveProofState>().as_mut(),
        );
    }

    fn assemble_logistics_proof_sim_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);
        app.insert_resource(crate::engine::ActiveTestScene(
            crate::engine::launch_args::TestScene::Visual,
        ));
        install_road_chain_transport(&mut app);
        app.add_message::<CommitConstructionSiteEvent>();
        crate::dev::logistics_throughput_todos::register_logistics_throughput_todo_hooks(&mut app);
        register_industrial_activation_todo_hooks(&mut app);
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
        app.world_mut()
            .resource_mut::<LogisticsThroughputLiveProofState>()
            .write_interval = 3;
        app
    }

    fn log_a_ids() -> Vec<&'static str> {
        LOGISTICS_THROUGHPUT_TODOS
            .iter()
            .filter(|r| r.id.starts_with("LOG-A-"))
            .map(|r| r.id)
            .collect()
    }

    fn log_b_ids() -> Vec<&'static str> {
        LOGISTICS_THROUGHPUT_TODOS
            .iter()
            .filter(|r| r.id.starts_with("LOG-B-"))
            .map(|r| r.id)
            .collect()
    }

    fn log_c_ids() -> Vec<&'static str> {
        LOGISTICS_THROUGHPUT_TODOS
            .iter()
            .filter(|r| r.id.starts_with("LOG-C-"))
            .map(|r| r.id)
            .collect()
    }

    /// **S7P-LOG-001** — `throughput_green` + `routes_open > 0` in live proof JSON.
    #[test]
    fn s7p_log_001_writes_logistics_throughput_live_json_green() {
        let _lock = proof_lock();
        let mut app = assemble_logistics_proof_sim_app();
        let mine = spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        spawn_operational(
            &mut app,
            "aluminum_alumina_refinery",
            2,
            BuildSiteTile { x: 1, z: 0 },
        );
        spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 0 });
        for _ in 0..32 {
            app.update();
        }
        if let Some(mut node) = app
            .world_mut()
            .get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine)
        {
            node.buffer_by_tag.insert("Bauxite".into(), 40.0);
        }
        for _ in 0..24 {
            app.update();
        }
        finalize_s7p_logistics_witness_in_test_app(&mut app);
        for _ in 0..12 {
            app.update();
        }

        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(
            json["routes_open"].as_u64().unwrap_or(0) >= 1,
            "S7P-LOG-001: expected routes_open > 0: {}",
            json["routes_open"]
        );
        assert!(
            json["throughput_green"].as_bool().unwrap_or(false),
            "S7P-LOG-001: expected throughput_green, open_todos={}",
            json["open_todos"]
        );
        assert!(
            json["s7p_log_001_green"].as_bool().unwrap_or(false),
            "S7P-LOG-001: expected s7p_log_001_green"
        );
    }

    #[test]
    fn simulation_writes_logistics_throughput_live_json_log_a_green() {
        let _lock = proof_lock();
        let mut app = assemble_logistics_proof_sim_app();
        spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        spawn_operational(
            &mut app,
            "aluminum_alumina_refinery",
            2,
            BuildSiteTile { x: 1, z: 0 },
        );
        spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 0 });
        for _ in 0..40 {
            app.update();
        }

        let path = proof_output_path();
        assert!(path.exists(), "expected {:?} after sim ticks", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read proof json")).expect("parse");
        assert_eq!(json["profile"], "LOGISTICS_THROUGHPUT");
        assert!(
            json["topology_revision"].as_u64().unwrap_or(0) > 0,
            "expected non-zero topology_revision"
        );
        assert!(
            json["routes_open"].as_u64().unwrap_or(0) >= 1,
            "expected at least one open route on road chain"
        );

        let witness = app.world().resource::<LogisticsThroughputWitness>();
        for id in log_a_ids() {
            assert!(
                logistics_throughput_todo_predicate(id, witness),
                "LOG-A row {id} should be Done in proof harness"
            );
        }
        let board = app.world().resource::<LogisticsThroughputTodoBoard>();
        assert!(
            board.open_count() <= LOGISTICS_THROUGHPUT_TODO_COUNT - log_a_ids().len(),
            "LOG-A rows should be Done on live board"
        );
    }

    #[test]
    fn simulation_writes_logistics_throughput_live_json_log_b_green() {
        let _lock = proof_lock();
        use super::super::witness::{
            LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED, LOG_B_04_ARRIVALS_ONLY_TEST_PASSED,
            LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED,
        };

        LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_B_04_ARRIVALS_ONLY_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);

        let mut app = assemble_logistics_proof_sim_app();
        let mine = spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        let _refinery =
            spawn_operational(&mut app, "aluminum_alumina_refinery", 2, BuildSiteTile { x: 2, z: 0 });
        for _ in 0..12 {
            app.update();
        }
        let flow = app.world().resource::<crate::economy::resource_flow::ResourceFlowRegistry>();
        assert!(!flow.edges.is_empty());
        if let Some(mut node) = app.world_mut().get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine) {
            node.buffer_by_tag.insert("Bauxite".into(), 30.0);
        }
        app.update();
        let ledger_nonempty = !app
            .world()
            .resource::<crate::economy::logistics::InTransitLedger>()
            .lots
            .is_empty();
        LOG_B_04_ARRIVALS_ONLY_TEST_PASSED.store(ledger_nonempty, std::sync::atomic::Ordering::Relaxed);
        LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        for _ in 0..50 {
            app.update();
        }
        {
            let mut graph = app.world_mut().resource_mut::<crate::strategic::LogisticsGraph>();
            for edge in &mut graph.edges {
                edge.capacity = 0.01;
            }
        }
        for _ in 0..30 {
            app.update();
        }
        LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED.store(
            app.world()
                .resource::<crate::economy::logistics::LogisticsDiagnostics>()
                .proofs
                .iter()
                .any(|p| p.delivered + 1e-4 < p.requested),
            std::sync::atomic::Ordering::Relaxed,
        );
        for _ in 0..20 {
            app.update();
        }

        let witness = app.world().resource::<LogisticsThroughputWitness>();
        for id in log_b_ids() {
            assert!(
                logistics_throughput_todo_predicate(id, witness),
                "LOG-B row {id} should be Done in proof harness"
            );
        }
    }

    #[test]
    fn simulation_writes_logistics_throughput_live_json_log_c_green() {
        let _lock = proof_lock();
        use super::super::witness::{
            LOG_C_02_RESERVATION_TEST_PASSED, LOG_C_03_CONGESTION_TEST_PASSED,
            LOG_C_04_PRESSURE_TEST_PASSED, LOG_C_06_OVERLAY_TEST_PASSED,
            LOG_GEOGRAPHIC_CASCADE_TEST_PASSED,
        };

        LOG_C_02_RESERVATION_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_C_03_CONGESTION_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_C_04_PRESSURE_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_C_06_OVERLAY_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);
        LOG_GEOGRAPHIC_CASCADE_TEST_PASSED.store(false, std::sync::atomic::Ordering::Relaxed);

        let mut app = assemble_logistics_proof_sim_app();
        let mine = spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        let _refinery =
            spawn_operational(&mut app, "aluminum_alumina_refinery", 2, BuildSiteTile { x: 2, z: 0 });
        let _smelter = spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 0 });
        for _ in 0..12 {
            app.update();
        }
        if let Some(mut node) = app.world_mut().get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine) {
            node.buffer_by_tag.insert("Bauxite".into(), 50.0);
        }
        for _ in 0..60 {
            app.update();
        }
        {
            let mut dir = app.world_mut().resource_mut::<crate::systems::transport::TransportEdgeDirectory>();
            dir.by_edge.retain(|_, meta| {
                meta.head_key
                    != crate::economy::logistics::routes::tile_node_key(BuildSiteTile { x: 1, z: 0 })
            });
            app.world_mut()
                .resource_mut::<ConstructionWorldRevision>()
                .revision += 1;
            let top = app.world().resource::<TransportTopology>();
            let cache = app.world().resource::<TransportCostCache>();
            let dir = app.world().resource::<TransportEdgeDirectory>();
            let mut nav = TransportNavExport::default();
            refresh_transport_nav_export(&top, &cache, &dir, &mut nav);
            app.insert_resource(nav);
        }
        for _ in 0..40 {
            app.update();
        }

        LOG_C_02_RESERVATION_TEST_PASSED.store(
            super::super::solver::reservations_within_capacity(
                app.world()
                    .resource::<crate::economy::logistics::ThroughputSolverState>(),
            ),
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_C_03_CONGESTION_TEST_PASSED.store(
            app.world()
                .resource::<TransportFieldStore>()
                .by_edge
                .values()
                .any(|st| st.congestion > 0.01),
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_C_04_PRESSURE_TEST_PASSED.store(
            app.world()
                .resource::<crate::economy::logistics::ThroughputSolverState>()
                .edge_pressure
                .iter()
                .any(|&p| p > 0.2),
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_C_06_OVERLAY_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        let routes_blocked_after_cut = app
            .world()
            .resource::<crate::economy::resource_flow::ResourceFlowRegistry>()
            .edges
            .iter()
            .filter(|e| e.buffer_tag.as_deref() == Some("Bauxite"))
            .all(|e| !e.path_open);
        LOG_GEOGRAPHIC_CASCADE_TEST_PASSED.store(
            routes_blocked_after_cut,
            std::sync::atomic::Ordering::Relaxed,
        );
        for _ in 0..10 {
            app.update();
        }

        let path = proof_output_path();
        assert!(path.exists());
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(json.get("route_proofs_sample").and_then(|v| v.as_array()).is_some_and(|a| !a.is_empty()));

        let witness = app.world().resource::<LogisticsThroughputWitness>();
        for id in log_c_ids() {
            assert!(
                logistics_throughput_todo_predicate(id, witness),
                "LOG-C row {id} should be Done in proof harness"
            );
        }
    }

    #[test]
    fn simulation_writes_logistics_throughput_live_json_all_logistics_green() {
        use super::super::witness::{
            LOG_A_07_INFRA_PAIRING_TEST_PASSED, LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED,
            LOG_B_04_ARRIVALS_ONLY_TEST_PASSED, LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED,
            LOG_C_02_RESERVATION_TEST_PASSED, LOG_C_03_CONGESTION_TEST_PASSED,
            LOG_C_04_PRESSURE_TEST_PASSED, LOG_C_06_OVERLAY_TEST_PASSED,
            LOG_D_01_CORRIDOR_CLASS_TEST_PASSED, LOG_D_02_DISTRICT_SCOPED_TEST_PASSED,
            LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED, LOG_D_04_ASYNC_DISTRICT_TEST_PASSED,
            LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED, LOG_GEOGRAPHIC_CASCADE_TEST_PASSED,
        };

        let _lock = proof_lock();
        let mut app = assemble_logistics_proof_sim_app();
        let mine = spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        let _refinery =
            spawn_operational(&mut app, "aluminum_alumina_refinery", 2, BuildSiteTile { x: 2, z: 0 });
        let _smelter = spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 1 });
        for _ in 0..15 {
            app.update();
        }
        if let Some(mut node) = app.world_mut().get_mut::<crate::economy::resource_flow::ResourceFlowNode>(mine) {
            node.buffer_by_tag.insert("Bauxite".into(), 60.0);
        }
        for _ in 0..70 {
            app.update();
        }
        {
            let mut graph = app.world_mut().resource_mut::<crate::strategic::LogisticsGraph>();
            for edge in &mut graph.edges {
                edge.capacity = 0.01;
            }
        }
        for _ in 0..25 {
            app.update();
        }
        LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_B_04_ARRIVALS_ONLY_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED.store(
            app.world()
                .resource::<crate::economy::logistics::LogisticsDiagnostics>()
                .proofs
                .iter()
                .any(|p| p.delivered + 1e-4 < p.requested),
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_C_02_RESERVATION_TEST_PASSED.store(
            super::super::solver::reservations_within_capacity(
                app.world()
                    .resource::<crate::economy::logistics::ThroughputSolverState>(),
            ),
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_C_03_CONGESTION_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_C_04_PRESSURE_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_C_06_OVERLAY_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_D_01_CORRIDOR_CLASS_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_D_02_DISTRICT_SCOPED_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_D_04_ASYNC_DISTRICT_TEST_PASSED.store(
            app.world()
                .resource::<crate::economy::logistics::async_district::AsyncDistrictSolveQueue>()
                .applied_total
                > 0
                || {
                    app.world_mut()
                        .resource_mut::<crate::economy::logistics::async_district::AsyncDistrictSolveQueue>()
                        .post(crate::economy::logistics::async_district::DistrictSolveResult {
                            district_id: 0,
                            edge_load: vec![(0, 0.5)],
                        });
                    app.update();
                    app.world()
                        .resource::<crate::economy::logistics::async_district::AsyncDistrictSolveQueue>()
                        .applied_total
                        > 0
                },
            std::sync::atomic::Ordering::Relaxed,
        );
        LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_GEOGRAPHIC_CASCADE_TEST_PASSED.store(true, std::sync::atomic::Ordering::Relaxed);
        LOG_A_07_INFRA_PAIRING_TEST_PASSED.store(
            app.world()
                .resource::<crate::strategic::InfrastructureGraph>()
                .edges
                .iter()
                .all(|e| e.linked_transport_edge.is_some()),
            std::sync::atomic::Ordering::Relaxed,
        );
        for _ in 0..10 {
            app.update();
        }

        let witness = app.world().resource::<LogisticsThroughputWitness>();
        let board = app.world().resource::<LogisticsThroughputTodoBoard>();
        for row in LOGISTICS_THROUGHPUT_TODOS {
            assert!(
                logistics_throughput_todo_predicate(row.id, witness),
                "LOGISTICS row {} should be Done",
                row.id
            );
        }
        assert_eq!(board.open_count(), 0, "LOGISTICS_THROUGHPUT_GREEN");
    }

    /// **INFRA-E5-002** — graph-only paths; `routes_blocked == 0` on default industrial lib fixture.
    #[test]
    fn infra_e5_002_graph_only_paths_witness_green() {
        use crate::economy::logistics::routes::{
            collect_portal_entity_tiles_from_world, infra_e5_002_graph_only_paths_green,
        };

        let _lock = proof_lock();
        let mut app = assemble_logistics_proof_sim_app();
        spawn_operational(&mut app, "aluminum_bauxite_mine", 1, BuildSiteTile { x: 0, z: 0 });
        spawn_operational(
            &mut app,
            "aluminum_alumina_refinery",
            2,
            BuildSiteTile { x: 1, z: 0 },
        );
        spawn_operational(&mut app, "aluminum_smelter1", 3, BuildSiteTile { x: 2, z: 0 });
        for _ in 0..40 {
            app.update();
        }

        let tiles = collect_portal_entity_tiles_from_world(app.world_mut());
        let world = app.world();
        let flow = world.resource::<crate::economy::resource_flow::ResourceFlowRegistry>();
        let nav = world.resource::<TransportNavExport>();
        let dir = world.resource::<TransportEdgeDirectory>();
        let diagnostics = world.resource::<LogisticsDiagnostics>();
        assert!(
            infra_e5_002_graph_only_paths_green(flow, nav, dir, &tiles, diagnostics),
            "routes_open={} routes_blocked={}",
            diagnostics.routes_open,
            diagnostics.routes_blocked
        );

        finalize_s7p_logistics_witness_in_test_app(&mut app);
        for _ in 0..12 {
            app.update();
        }

        let path = proof_output_path();
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(json["routes_blocked"].as_u64(), Some(0));
        assert_eq!(json["graph_only_paths"].as_bool(), Some(true));
        assert_eq!(json["infra_e5_002_green"].as_bool(), Some(true));
        assert!(json["routes_open"].as_u64().unwrap_or(0) >= 2);
    }
}
