//! Construction integration tests (Phase 2 P8).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::construction_pipeline::{
    validate_road_segment, ConstructionIntent, ConstructionPlanQueue, ConstructionStatus,
    ConstructionType,
};
use super::build_tool_authority::{
    shift_lmb_applies_to_active_tool, shift_lmb_queues_building_blueprint, BuildTool,
    BuildingArchetypeId, ZoneTool,
};
use super::pending_construction::PendingConstructionQueue;
use super::roads::{regenerate_road_segments, IntersectionRegistry};
use super::zones::{commit_painted_zones_to_pending, ActiveZonePaint};

#[test]
fn road_e2e_queue_validate_segments() {
    let params = WorldGenParams {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let pts = vec![
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(4.0, 0.0, 1.0),
        Vec3::new(7.0, 0.0, 1.0),
    ];
    let segs = regenerate_road_segments(&pts, None, 8.0, &params, false);
    assert!(segs.len() >= 2);
    let mut queue = ConstructionPlanQueue::default();
    for w in segs.windows(2) {
        if !w[0].valid {
            continue;
        }
        let head = BuildSiteTile {
            x: w[0].start.x.floor() as u32,
            z: w[0].start.z.floor() as u32,
        };
        let tail = BuildSiteTile {
            x: w[0].end.x.floor() as u32,
            z: w[0].end.z.floor() as u32,
        };
        if validate_road_segment(head, tail, &params).valid {
            queue.enqueue(ConstructionIntent {
                entity_type: ConstructionType::RoadSegment { head, tail },
                world_position: Vec2::ZERO,
                rotation: 0.0,
            });
        }
    }
    assert!(!queue.plans.is_empty());
    for plan in &queue.plans {
        assert_eq!(plan.status, ConstructionStatus::Requested);
    }
}

#[test]
fn hydro_coupling_execute_plan_emits_preview_validate_does_not() {
    use super::construction_pipeline::{
        execute_construction_plans_system, validate_construction_plans_system,
        ConstructionPlanQueue, ConstructionWorldRevision,
        ExecutedRoadNetwork,
    };
    use crate::substrate::hydrology::{
        HydrologyConstructionCouplingWitness, HydrologyDirtyReason, HydrologyEventQueue,
    };
    use crate::systems::transport::{
        TransportCostCache, TransportCostWeights, TransportEdgeDirectory, TransportFieldStore,
        TransportTopology,
    };
    use crate::strategic::CorridorConstructionBook;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use bevy::state::app::StatesPlugin;

    let params = WorldGenParams {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let mut queue = ConstructionPlanQueue::default();
    queue.enqueue(ConstructionIntent {
        entity_type: ConstructionType::RoadSegment {
            head: BuildSiteTile { x: 1, z: 1 },
            tail: BuildSiteTile { x: 3, z: 1 },
        },
        world_position: Vec2::ZERO,
        rotation: 0.0,
    });

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<ConstructionPlanQueue>()
        .init_resource::<ExecutedRoadNetwork>()
        .init_resource::<ConstructionWorldRevision>()
        .init_resource::<TransportTopology>()
        .init_resource::<TransportFieldStore>()
        .init_resource::<TransportEdgeDirectory>()
        .init_resource::<TransportCostCache>()
        .init_resource::<TransportCostWeights>()
        .init_resource::<CorridorConstructionBook>()
        .init_resource::<super::rail::ActiveRailPlacement>()
        .init_resource::<super::roads::IntersectionRegistry>()
        .init_resource::<super::history::ConstructionHistory>()
        .init_resource::<HydrologyEventQueue>()
        .init_resource::<HydrologyConstructionCouplingWitness>()
        .insert_resource(params)
        .insert_resource(queue)
        .add_systems(Update, validate_construction_plans_system);

    {
        let mut coupling = app.world_mut().resource_mut::<HydrologyConstructionCouplingWitness>();
        coupling.bridge_registered = true;
    }

    app.update();
    assert!(
        app.world()
            .resource::<HydrologyEventQueue>()
            .pending
            .is_empty(),
        "validate-only tick must not emit hydrology events"
    );

    app.add_systems(Update, execute_construction_plans_system);
    app.update();
    let hydro = app.world().resource::<HydrologyEventQueue>();
    assert!(!hydro.pending.is_empty(), "execute must enqueue hydrology dirty events");
    assert!(hydro.pending.iter().all(|e| matches!(
        e.reason,
        HydrologyDirtyReason::ConstructionComplete { .. }
    )));
    let coupling = app.world().resource::<HydrologyConstructionCouplingWitness>();
    assert_eq!(coupling.preview_emit_count, 0);
    assert!(coupling.execute_emit_count > 0);
}

#[test]
fn infra_e2_003_execute_road_increments_transport_binds_markers_and_paths() {
    use super::construction_pipeline::{
        execute_construction_plans_system, validate_construction_plans_system,
        ConstructionPlanQueue, ConstructionWorldRevision, ExecutedRoadNetwork, SimRoadSegmentMarker,
    };
    use super::corridor_transport::SimCorridorEdgeBinding;
    use crate::economy::logistics::routes::path_edges_between_tiles;
    use crate::economy::resource_flow::TransportMode;
    use crate::systems::transport::{
        edge_traversal_cost, refresh_transport_nav_export, TransportCostCache, TransportCostWeights,
        TransportEdgeDirectory, TransportEdgeId, TransportFieldStore, TransportNavExport, TransportTopology,
    };
    use crate::strategic::CorridorConstructionBook;
    use bevy::state::app::StatesPlugin;

    let params = WorldGenParams {
        width: 64,
        height: 64,
        ..Default::default()
    };
    let mut queue = ConstructionPlanQueue::default();
    queue.enqueue(ConstructionIntent {
        entity_type: ConstructionType::RoadSegment {
            head: BuildSiteTile { x: 1, z: 1 },
            tail: BuildSiteTile { x: 3, z: 1 },
        },
        world_position: Vec2::ZERO,
        rotation: 0.0,
    });

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<ConstructionPlanQueue>()
        .init_resource::<ExecutedRoadNetwork>()
        .init_resource::<ConstructionWorldRevision>()
        .init_resource::<TransportTopology>()
        .init_resource::<TransportFieldStore>()
        .init_resource::<TransportEdgeDirectory>()
        .init_resource::<TransportCostCache>()
        .init_resource::<TransportCostWeights>()
        .init_resource::<TransportNavExport>()
        .init_resource::<CorridorConstructionBook>()
        .init_resource::<super::rail::ActiveRailPlacement>()
        .init_resource::<super::roads::IntersectionRegistry>()
        .init_resource::<super::history::ConstructionHistory>()
        .insert_resource(params)
        .insert_resource(queue)
        .add_systems(
            Update,
            (
                validate_construction_plans_system,
                execute_construction_plans_system,
            )
                .chain(),
        );

    app.update();

    let topo = app.world().resource::<TransportTopology>();
    assert_eq!(topo.neighbors.len(), 1, "execute should add one transport edge");

    let bindings: Vec<TransportEdgeId> = {
        let world = app.world_mut();
        world
            .query_filtered::<&SimCorridorEdgeBinding, With<SimRoadSegmentMarker>>()
            .iter(world)
            .map(|b| b.edge_id)
            .collect()
    };
    assert_eq!(bindings.len(), 2, "head + tail markers bind to edge");
    assert!(bindings.iter().all(|id| *id == bindings[0]));

    let book = app.world().resource::<CorridorConstructionBook>();
    assert!(book.rows.contains_key(&bindings[0]));

    let top = app.world().resource::<TransportTopology>();
    let field = app.world().resource::<TransportFieldStore>();
    let dir = app.world().resource::<TransportEdgeDirectory>();
    let weights = app.world().resource::<TransportCostWeights>().clone();
    let mut cache = TransportCostCache::default();
    for (id, st) in &field.by_edge {
        cache.by_edge.insert(*id, edge_traversal_cost(st, &weights, st.travel_time_base));
    }
    let mut nav = TransportNavExport::default();
    refresh_transport_nav_export(top, &cache, dir, &mut nav);

    let path = path_edges_between_tiles(
        &nav,
        dir,
        BuildSiteTile { x: 1, z: 1 },
        BuildSiteTile { x: 3, z: 1 },
        TransportMode::Truck,
    );
    assert!(path.is_some(), "logistics path should exist after corridor execute");
}

#[test]
fn zone_paint_queues_zone_pending_kind() {
    let mut pending = PendingConstructionQueue::default();
    let mut paint = ActiveZonePaint::default();
    paint.zone = Some(ZoneTool::ResidentialLow);
    paint.push_unique(BuildSiteTile { x: 2, z: 3 });
    let n = commit_painted_zones_to_pending(
        ZoneTool::ResidentialLow,
        &paint.painted,
        &mut pending,
    );
    assert_eq!(n, 1);
    assert_eq!(pending.entries.len(), 1);
    assert!(matches!(
        pending.entries[0].kind,
        super::pending_construction::PendingEntryKind::ZonePaint(_)
    ));
}

fn scan_rs_for_legacy_gui_build(dir: &std::path::Path, violations: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "dev") {
                continue;
            }
            scan_rs_for_legacy_gui_build(&path, violations);
            continue;
        }
        if path.extension().is_some_and(|e| e == "rs")
            && path.file_name() != Some(std::ffi::OsStr::new("integration_tests.rs"))
        {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let legacy = ["use crate::gui::build", "use super::gui::build", "crate::gui::build::"];
            if legacy.iter().any(|needle| text.contains(needle)) {
                violations.push(path.display().to_string());
            }
        }
    }
}

#[test]
fn no_legacy_gui_build_placement_in_src() {
    let mut violations = Vec::new();
    scan_rs_for_legacy_gui_build(std::path::Path::new("src"), &mut violations);
    assert!(
        violations.is_empty(),
        "legacy gui::build references: {violations:?}"
    );
}

#[test]
fn input_conflict_matrix_gates() {
    assert!(shift_lmb_applies_to_active_tool(BuildTool::Zone(
        ZoneTool::ResidentialLow
    )));
    assert!(!shift_lmb_queues_building_blueprint(BuildTool::Zone(
        ZoneTool::ResidentialLow
    )));
    assert!(!shift_lmb_queues_building_blueprint(BuildTool::Building(
        BuildingArchetypeId::Housing
    )));
}

#[test]
fn intersection_registry_dedupes_tile() {
    let mut reg = IntersectionRegistry::default();
    let tile = BuildSiteTile { x: 1, z: 1 };
    let e1 = Entity::from_bits(1);
    let e2 = Entity::from_bits(2);
    let id1 = reg.register_or_extend(tile, e1);
    let id2 = reg.register_or_extend(tile, e2);
    assert_eq!(id1, id2);
    assert_eq!(reg.degree_at_tile(tile), 2);
}
