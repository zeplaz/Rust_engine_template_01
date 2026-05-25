//! **LOGISTICS_THROUGHPUT_GREEN** — real transport-graph freight causality (post industrial activation).
//!
//! Spec: [`super::logistics_throughput_phase_todos.md`](super::logistics_throughput_phase_todos.md)  
//! Architecture: [`super::Logistics throughput architecture.md`](super::Logistics%20throughput%20architecture.md)  
//! **Prerequisite:** `INDUSTRIAL_ACTIVATION_GREEN`. **Not** Stage 5.

use bevy::log::info;
use bevy::prelude::{App, Resource};
use std::sync::atomic::AtomicBool;

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

pub const LOGISTICS_THROUGHPUT_TODO_COUNT: usize = 24;

/// Set by `infra_pairs_by_transport_edge_not_directory_iteration_order` (LOG-A-07).
pub static LOG_A_07_INFRA_PAIRING_TEST_PASSED: AtomicBool = AtomicBool::new(false);

pub static LOGISTICS_THROUGHPUT_TODOS: &[Stage5LiveTodo] = &[
    // ── LOG-A Authority & wiring ───────────────────────────────────────────
    Stage5LiveTodo {
        id: "LOG-A-01",
        status: TodoStatus::Open,
        file: "src/strategic/mod.rs",
        system: "DerivedLogisticsGraph",
        goal: "`LogisticsGraph` derived-only with `revision`; no solve-time mutation or facility append in bridge.",
        runtime_check: "Rebuild replaces graph; `logistics_bridge` stops pushing orphan facility nodes.",
        failure_mode: "Graph rebuild corrupts IDs; async solve races on mutable graph.",
    },
    Stage5LiveTodo {
        id: "LOG-A-02",
        status: TodoStatus::Open,
        file: "src/economy/logistics_bridge.rs",
        system: "FacilityPortalAttachment",
        goal: "`FacilityPortal { anchor, transport_anchor }` + `PortalAttachmentMap` rebuilt at GraphSync.",
        runtime_check: "Facilities lack persistent `LogisticsNodeId`; map resolves after each rebuild.",
        failure_mode: "Streaming/rebuild breaks facility node handles.",
    },
    Stage5LiveTodo {
        id: "LOG-A-03",
        status: TodoStatus::Open,
        file: "src/strategic/transport_bridge.rs",
        system: "LogisticsEdgeTransportId",
        goal: "`LogisticsEdge.transport_edge: Option<TransportEdgeId>` on every corridor edge.",
        runtime_check: "Unit test: rebuild pairs logistics edge to directory id.",
        failure_mode: "InfrastructureGraph sorted-index pairing drift.",
    },
    Stage5LiveTodo {
        id: "LOG-A-04",
        status: TodoStatus::Open,
        file: "src/economy/resource_flow.rs",
        system: "PathOpenFromNav",
        goal: "`path_open` from `TransportNavExport` reachability — kill default `true`.",
        runtime_check: "Road connects facilities → open; remove segment → blocked after route refresh.",
        failure_mode: "I4-03 stub: chain edges ignore geography.",
    },
    Stage5LiveTodo {
        id: "LOG-A-05",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "VersionedRouteHandle",
        goal: "`RouteHandle { id, topology_revision }` invalidates on `ConstructionWorldRevision`.",
        runtime_check: "Execute road plan bumps revision → stale routes refresh.",
        failure_mode: "Ghost routes after topology change.",
    },
    Stage5LiveTodo {
        id: "LOG-A-06",
        status: TodoStatus::Open,
        file: "debug_runs/logistics_throughput_live.json",
        system: "LogisticsProofJson",
        goal: "Machine proof JSON: routes_open/blocked, topology_revision, board snapshot.",
        runtime_check: "Written in sim test or live proof harness.",
        failure_mode: "LOG closure not measurable.",
    },
    Stage5LiveTodo {
        id: "LOG-A-07",
        status: TodoStatus::Open,
        file: "src/strategic/infrastructure_graph.rs",
        system: "InfraTransportPairing",
        goal: "Infrastructure mirror uses `transport_edge` not sorted directory index.",
        runtime_check: "Permuted edge order test: integrity links stay correct.",
        failure_mode: "Wrong edge maintenance/damage target.",
    },
    // ── LOG-B Freight ledger ───────────────────────────────────────────────
    Stage5LiveTodo {
        id: "LOG-B-01",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "RoutePathStore",
        goal: "Centralized `RoutePathStore` — no `Vec<TransportEdgeId>` per freight lot.",
        runtime_check: "Lots store `RouteHandle` + progress only.",
        failure_mode: "Memory blow-up at scale.",
    },
    Stage5LiveTodo {
        id: "LOG-B-02",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "InTransitLedger",
        goal: "`InTransitLedger` with compact lots + edge progress.",
        runtime_check: "Transfer moves through ledger, not direct inventory snap.",
        failure_mode: "Teleport persists.",
    },
    Stage5LiveTodo {
        id: "LOG-B-03",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "FreightMovementModel",
        goal: "`FreightMovementModel::Continuous | Batched` for trucks vs trains/convoys.",
        runtime_check: "Batched route longer tick latency than continuous at same distance.",
        failure_mode: "Rail and ports feel like trucks.",
    },
    Stage5LiveTodo {
        id: "LOG-B-04",
        status: TodoStatus::Open,
        file: "src/economy/resource_flow.rs",
        system: "ArrivalsOnlyPropagation",
        goal: "Refactor propagation: gather → solve → commit arrivals only.",
        runtime_check: "Unit test: no same-tick credit without ledger arrival.",
        failure_mode: "Instant magical transfer.",
    },
    Stage5LiveTodo {
        id: "LOG-B-05",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "PartialFulfillment",
        goal: "Partial delivery + shortage witness when route saturated.",
        runtime_check: "requested > delivered → deficit in witness.",
        failure_mode: "All-or-nothing hides bottlenecks.",
    },
    // ── LOG-C ThroughputSolver ─────────────────────────────────────────────
    Stage5LiveTodo {
        id: "LOG-C-01",
        status: TodoStatus::Open,
        file: "src/economy/logistics/throughput_solver.rs",
        system: "SoaThroughputSolver",
        goal: "SoA `load` / `capacity` / `reserved` `Vec<f32>` indexed by edge id.",
        runtime_check: "Solve hot path avoids HashMap.",
        failure_mode: "Perf collapse with thousands of edges.",
    },
    Stage5LiveTodo {
        id: "LOG-C-02",
        status: TodoStatus::Open,
        file: "src/economy/logistics/throughput_solver.rs",
        system: "FreightReservations",
        goal: "`FreightReservationBook` + solve pass enforces capacity.",
        runtime_check: "Invariant: sum reservations per edge ≤ capacity.",
        failure_mode: "Unlimited edge throughput.",
    },
    Stage5LiveTodo {
        id: "LOG-C-03",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "CongestionFeedback",
        goal: "`feedback_congestion_from_load_system` writes `TransportFieldStore.congestion`.",
        runtime_check: "High load edge → congestion rises next ticks.",
        failure_mode: "Freight does not stress corridors.",
    },
    Stage5LiveTodo {
        id: "LOG-C-04",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "CorridorPressure",
        goal: "`propagate_corridor_pressure_system` diffuses saturation to neighbors.",
        runtime_check: "Saturated edge raises neighbor pressure scalar.",
        failure_mode: "Reroute oscillation when routing added.",
    },
    Stage5LiveTodo {
        id: "LOG-C-05",
        status: TodoStatus::Open,
        file: "src/economy/logistics/diagnostics.rs",
        system: "RouteProof",
        goal: "`RouteProof` ring: requested, delivered, blocked_at, bottleneck_capacity.",
        runtime_check: "Proof JSON includes per-request trace before async solve.",
        failure_mode: "Months debugging opaque shortages.",
    },
    Stage5LiveTodo {
        id: "LOG-C-06",
        status: TodoStatus::Open,
        file: "src/strategic/logistics_net.rs",
        system: "OverlaySolverLoad",
        goal: "Inject solver `load` into `logistics_throughput` overlay (not static capacity).",
        runtime_check: "Overlay changes when solve load changes, capacity constant.",
        failure_mode: "Heatmap lies about actual freight.",
    },
    Stage5LiveTodo {
        id: "LOG-C-07",
        status: TodoStatus::Open,
        file: "src/economy/resource_flow.rs",
        system: "GeographicCascadeTest",
        goal: "Integration: cut road → refinery starved → smelter efficiency drop.",
        runtime_check: "`cargo test` aluminum chain with transport gap.",
        failure_mode: "Starvation from empty buffer only, not corridor cut.",
    },
    // ── LOG-D Scale & futures ──────────────────────────────────────────────
    Stage5LiveTodo {
        id: "LOG-D-01",
        status: TodoStatus::Open,
        file: "src/systems/transport/types.rs",
        system: "CorridorClass",
        goal: "`CorridorClass` on transport meta/field: Road/Rail/Maritime/Conveyor/Power/Pipeline.",
        runtime_check: "Rail edge rejects road_vehicle agent in route test.",
        failure_mode: "Mode collapse until rework.",
    },
    Stage5LiveTodo {
        id: "LOG-D-02",
        status: TodoStatus::Open,
        file: "src/economy/spatial_district.rs",
        system: "DistrictScopedSolve",
        goal: "Throughput solve limited to active industrial districts.",
        runtime_check: "District with no facilities skips solve subgraph.",
        failure_mode: "Global solve does not scale.",
    },
    Stage5LiveTodo {
        id: "LOG-D-03",
        status: TodoStatus::Open,
        file: "src/economy/logistics/routes.rs",
        system: "StreamingRouteInvalidation",
        goal: "Route cache invalidates on chunk bbox + transport hydrate.",
        runtime_check: "Partial hydrate bumps revision for affected routes.",
        failure_mode: "Stale cross-chunk routes.",
    },
    Stage5LiveTodo {
        id: "LOG-D-04",
        status: TodoStatus::Open,
        file: "src/economy/logistics/",
        system: "AsyncDistrictSolve",
        goal: "Async district job scaffold — apply reservations main thread only.",
        runtime_check: "Job completes next frame; fields not mutated off-thread.",
        failure_mode: "Frame spikes on large worlds.",
    },
    Stage5LiveTodo {
        id: "LOG-D-05",
        status: TodoStatus::Open,
        file: "src/gui/diagnostics_ui.rs",
        system: "LogisticsDiagnosticsPanel",
        goal: "UI panel: top saturated edges, starved facilities, last RouteProof.",
        runtime_check: "Visible in diagnostics overlay when board present.",
        failure_mode: "No operator visibility into logistics.",
    },
];

#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsThroughputWitness {
    // LOG-A
    pub derived_logistics_graph: bool,
    pub facility_portal_attachment: bool,
    pub logistics_edge_transport_id: bool,
    pub path_open_from_nav: bool,
    pub versioned_route_handle: bool,
    pub logistics_proof_json: bool,
    pub infra_transport_pairing: bool,
    // LOG-B
    pub route_path_store: bool,
    pub in_transit_ledger: bool,
    pub freight_movement_model: bool,
    pub arrivals_only_propagation: bool,
    pub partial_fulfillment: bool,
    // LOG-C
    pub soa_throughput_solver: bool,
    pub freight_reservations: bool,
    pub congestion_feedback: bool,
    pub corridor_pressure: bool,
    pub route_proof: bool,
    pub overlay_solver_load: bool,
    pub geographic_cascade_test: bool,
    // LOG-D
    pub corridor_class: bool,
    pub district_scoped_solve: bool,
    pub streaming_route_invalidation: bool,
    pub async_district_solve: bool,
    pub logistics_diagnostics_panel: bool,
}

#[derive(Resource, Default)]
pub struct LogisticsThroughputTodoBoard {
    pub status: Vec<TodoStatus>,
}

#[must_use]
pub fn logistics_throughput_todo_predicate(id: &str, w: &LogisticsThroughputWitness) -> bool {
    match id {
        "LOG-A-01" => w.derived_logistics_graph,
        "LOG-A-02" => w.facility_portal_attachment,
        "LOG-A-03" => w.logistics_edge_transport_id,
        "LOG-A-04" => w.path_open_from_nav,
        "LOG-A-05" => w.versioned_route_handle,
        "LOG-A-06" => w.logistics_proof_json,
        "LOG-A-07" => w.infra_transport_pairing,
        "LOG-B-01" => w.route_path_store,
        "LOG-B-02" => w.in_transit_ledger,
        "LOG-B-03" => w.freight_movement_model,
        "LOG-B-04" => w.arrivals_only_propagation,
        "LOG-B-05" => w.partial_fulfillment,
        "LOG-C-01" => w.soa_throughput_solver,
        "LOG-C-02" => w.freight_reservations,
        "LOG-C-03" => w.congestion_feedback,
        "LOG-C-04" => w.corridor_pressure,
        "LOG-C-05" => w.route_proof,
        "LOG-C-06" => w.overlay_solver_load,
        "LOG-C-07" => w.geographic_cascade_test,
        "LOG-D-01" => w.corridor_class,
        "LOG-D-02" => w.district_scoped_solve,
        "LOG-D-03" => w.streaming_route_invalidation,
        "LOG-D-04" => w.async_district_solve,
        "LOG-D-05" => w.logistics_diagnostics_panel,
        _ => false,
    }
}

impl LogisticsThroughputTodoBoard {
    pub fn sync_from_witness(&mut self, w: &LogisticsThroughputWitness) {
        debug_assert_eq!(self.status.len(), LOGISTICS_THROUGHPUT_TODO_COUNT);
        debug_assert_eq!(LOGISTICS_THROUGHPUT_TODOS.len(), LOGISTICS_THROUGHPUT_TODO_COUNT);
        for (slot, row) in self.status.iter_mut().zip(LOGISTICS_THROUGHPUT_TODOS.iter()) {
            *slot = if logistics_throughput_todo_predicate(row.id, w) {
                TodoStatus::Done
            } else {
                TodoStatus::Open
            };
        }
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        self.open_count() == 0
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }
}

pub fn register_logistics_throughput_todo_hooks(app: &mut App) {
    app.init_resource::<LogisticsThroughputTodoBoard>()
        .init_resource::<LogisticsThroughputWitness>()
        .init_resource::<crate::economy::logistics::LogisticsThroughputRuntimeWitness>();
    let mut board = LogisticsThroughputTodoBoard::default();
    board.status = vec![TodoStatus::Open; LOGISTICS_THROUGHPUT_TODO_COUNT];
    app.insert_resource(board);
}

pub fn sync_logistics_throughput_board_from_witness(
    witness: &LogisticsThroughputWitness,
    board: &mut LogisticsThroughputTodoBoard,
) {
    board.sync_from_witness(witness);
    if board.is_green() {
        info!(
            target: "logistics_throughput_todos",
            "LOGISTICS_THROUGHPUT_GREEN"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_row_count_matches_static_table() {
        assert_eq!(LOGISTICS_THROUGHPUT_TODOS.len(), LOGISTICS_THROUGHPUT_TODO_COUNT);
    }

    #[test]
    fn all_ids_have_predicates() {
        for row in LOGISTICS_THROUGHPUT_TODOS {
            let w = LogisticsThroughputWitness::default();
            let _ = logistics_throughput_todo_predicate(row.id, &w);
        }
    }
}
