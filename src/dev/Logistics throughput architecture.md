Logistics throughput architecture — review & target design
This document is grounded in the current codebase: construction → transport hydrate → strategic LogisticsGraph → economy ResourceFlowRegistry. Industrial activation is green; the gap is causality, not placement.

1. Architecture review — what exists today
Layer stack (actual data flow)
Construction lane
Transport spine - authoritative topology
Strategic / overlay
Economy - facility flow
no edges
ExecutedRoadNetwork
bake_snapshot_from_ordered_tile_markers
hydrate_transport_from_snapshot
TransportEdgeDirectory
TransportFieldStore
TransportCostCache
TransportNavExport
LogisticsGraph rebuild
ChunkStrategicOverlay fields
ResourceFlowNode on entities
ResourceFlowRegistry edges
propagate_resource_flow_system
FacilityLogisticsNode on LogisticsGraph
Layer	Authority	What it models today
Transport (src/systems/transport/)
Topology + per-edge fields + costs
Roads/rails as TransportEdgeId, congestion/damage/danger decay, nav export
Construction (ExecutedRoadNetwork → bake → hydrate)
Adds edges when segments execute
Ordered tile chain → snapshot → directory
Strategic LogisticsGraph
Derived from transport endpoints
Chunk-cell nodes at head/tail; capacity from cost + corridor book
Economy ResourceFlow*
Facility inventories + chain edges
Produce/consume/starve; direct entity→entity transfer
Grid logistics (logistics_floodfill)
Standalone grid Dijkstra
Terrain/mobility paths — not wired to facility flow
Transport spine (solid foundation)
Construction execution already writes the transport truth:


construction_pipeline.rs
Lines 289-300
    let snap = bake_snapshot_from_ordered_tile_markers(
        &marker_tiles,
        |_x, _z| 0.5,
        ROAD_MARKER_Y_SCALE,
        ROAD_MARKER_Y_BIAS,
    );
    // ...
    if hydrate_transport_from_snapshot(topology, fields, directory, &snap).is_err() {
        return None;
    }
transport_bridge rebuilds LogisticsGraph from TransportEdgeDirectory + TransportFieldStore + CorridorConstructionBook, with capacity scaled by construction phase and disruption from field state:


transport_bridge.rs
Lines 122-135
        let state = fields.by_edge.get(&eid).cloned().unwrap_or_default();
        let cost = edge_traversal_cost(&state, weights, state.travel_time_base);
        let tf = book.traffic_factor(eid);
        let capacity = ((2.0 / cost.max(0.08)).min(3.0)) * tf;
        let disruption =
            (state.damage + state.congestion * 0.45 + state.danger * 0.25).clamp(0.0, 1.0);
Schedule order is correct for coupling: TransportSchedule after AdvanceSimTick, then StrategicFieldPipeline::GraphSync, then overlays.

Economy flow (causality stub)
Nodes and edges exist; propagation is not graph-limited:


resource_flow.rs
Lines 248-258
            flow.add_edge(ResourceFlowEdge {
                from: from_e,
                to: to_e,
                transport_mode: TransportMode::Truck,
                max_rate: 4.0,
                latency_ticks: 1.0,
                path_open: true,  // always true at link time
                buffer_tag,
            });

resource_flow.rs
Lines 305-324
fn transfer_along_edge(...) -> f32 {
    if !edge.path_open {
        return 0.0;
    }
    // instant inventory debit/credit same tick — no in-transit, no edge load
latency_ticks is stored but never applied. Starvation works (downstream scaling) but upstream can be “full” while geographically isolated.

Facility ↔ transport disconnect
register_facility_logistics_system appends nodes to the same LogisticsGraph resource transport rebuild overwrites structurally — facilities are extra nodes with no edges to corridor nodes:


logistics_bridge.rs
Lines 36-47
        let id = LogisticsNodeId(graph.nodes.len() as u32);
        graph.nodes.push(LogisticsNode { id, throughput, stockpile: 0.0, anchor });
        commands.entity(entity).insert((FacilityLogisticsNodeId(id), ...));
After sync_logistics_graph_from_transport, facility nodes are either wiped (full rebuild replaces graph) or live in a parallel namespace with no LogisticsEdge to the road graph. I4-03 “path gate” is a boolean stub, not reachability on TransportNavExport.

Overlay vs sim flow
logistics_net_inject_into_overlays paints static capacity × (1 − disruption) at anchors — not solved freight, not reservations:


logistics_net.rs
Lines 32-49
    for edge in &graph.edges {
        let eff = edge.capacity * (1.0 - edge.disruption.clamp(0.0, 1.0));
        // split half to each endpoint cell
Useful for AI heatmaps; misleading if read as “tons moved this tick.”

Secondary leaks
Issue	Location	Effect
InfrastructureGraph edge ↔ transport id by sorted index
infrastructure_graph.rs
Wrong pairing when edge order ≠ logistics edge order
transport_topology_tick no-op
transport/mod.rs
Junction saturation not in topology
Field integrate = decay only
transport_field_integrate
Congestion never rises from freight
Two path models
transport graph vs logistics_path_dijkstra
Duplicate semantics; economy uses neither
Global ResourceFlowRegistry
one-shot if !flow.edges.is_empty()
No per-facility routes, no reroute
2. Abstraction leaks & hidden assumptions
Fake logistics (must eliminate)
Teleport transfers — propagate_resource_flow_system moves buffer/inventory between entities with no path, vehicle, or edge utilization.
Chain topology ≠ geography — link_supply_chain_edges_system links catalog order only; distance and corridors irrelevant.
path_open without path — witness checks file exists; runtime always true at link.
Anchor proximity fiction — facility and road nodes share ChunkCellKey type but are not connected; “same chunk” ≠ connected corridor.
Capacity without allocation — LogisticsEdge.capacity is not a ledger; no FreightReservation competes for it.
Latency unused — no pipeline delay, no cascading wave propagation.
Mode-agnostic edges — TransportMode::Truck hardcoded; rail/port/power corridors not distinguished in flow solver.
Hidden assumptions
One aluminum/concrete chain per world (edges built once globally).
throughput_limit derived from power_consumption, not from JSON logistics or building footprint.
Smelter starvation via FacilityFlowState is correct gameplay signal but misattributes cause (empty buffer vs blocked corridor).
Strategic overlays and economy sim can diverge without violating tests.
Full LogisticsGraph rebuild each frame is OK at current edge counts (will not scale).
3. Target ECS architecture — single causality spine

**Principle (non-negotiable):**

| Layer | Owns |
|-------|------|
| **Transport** | Connectivity, traversal, costs, disruption, congestion, topology revision |
| **Economy** | Facility demand, buffers, production/consumption contracts |
| **ThroughputSolver** | Reservations, edge load, bottlenecks, in-transit progress |
| **LogisticsGraph** | **Derived cache only** — rebuilt at `GraphSync`, never mutated during solve |

Failure today is entirely in **economy causality** (no routing, reservations, in-transit, edge saturation, facility↔transport attachment, persisted solve state). Construction and transport spine are not the blocker.

### Graph roles (final)

| Graph | Owner | Role |
|-------|-------|------|
| **Transport** (`systems/transport`) | `TransportEdgeDirectory`, `TransportFieldStore`, `TransportCostCache`, `TransportNavExport` | **Only authority** for topology + fields |
| **LogisticsGraph** (`strategic/transport_bridge` + builder) | `revision`, junction nodes, `LogisticsEdge { transport_edge }` | Derived solve graph; **read-only** after rebuild |
| **PortalAttachmentMap** | Rebuilt each `GraphSync` | `Entity → LogisticsNodeId` transient; facilities store **anchors only** |
| **Facility flow** (`economy/logistics`) | `ResourceFlowNode`, versioned `RouteHandle` | Demand contracts, not geometry |
| **Freight ledger** | `InTransitLedger`, `RoutePathStore`, `FreightReservationBook` | Physical movement state |

### Authority correction — `LogisticsGraph` is not sim state

```rust
/// Rebuilt from transport each GraphSync — DO NOT mutate during solve/async jobs.
#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsGraph {
    pub revision: u64,  // == ConstructionWorldRevision or transport directory signature
    pub nodes: Vec<LogisticsNode>,
    pub edges: Vec<LogisticsEdge>,
}

#[derive(Clone, Debug)]
pub struct LogisticsEdge {
    pub from: LogisticsNodeId,
    pub to: LogisticsNodeId,
    pub transport_edge: Option<TransportEdgeId>,  // mandatory pairing (LOG-A-03)
    pub capacity: f32,
    pub disruption: f32,
    pub traversal_cost: f32,
}
```

Runtime mutation lives only in: `ThroughputSolverState`, `FreightReservationBook`, `InTransitLedger`, `RouteCache`, `PortalAttachmentMap`.

### Types (review-corrected)

```rust
/// Stable facility identity — survives graph rebuilds and chunk streaming.
#[derive(Component, Clone, Copy, Debug)]
pub struct FacilityPortal {
    pub anchor: ChunkCellKey,
    pub transport_anchor: TransportNodeAnchor, // tile key or nearest junction key
}

#[derive(Resource, Default)]
pub struct PortalAttachmentMap {
    pub revision: u64,
    pub facility_to_graph: HashMap<Entity, LogisticsNodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RouteHandle {
    pub id: u32,
    pub topology_revision: u32,
}

#[derive(Clone, Debug)]
pub struct ResourceFlowEdge {
    pub from: Entity,
    pub to: Entity,
    pub mode: TransportMode,
    pub route: RouteHandle,
    pub path_open: bool,  // reachability AND solver — never default true
    pub max_demand_rate: f32,
    pub buffer_tag: Option<String>,
}

/// Centralized path storage — lots must NOT own Vec<TransportEdgeId>.
#[derive(Clone, Copy, Debug)]
pub struct RoutePath {
    pub first_edge: u32,
    pub edge_count: u16,
}

#[derive(Resource, Default)]
pub struct RoutePathStore {
    pub edges: Vec<TransportEdgeId>,
    pub paths: Vec<RoutePath>,
}

#[derive(Clone, Debug)]
pub struct FreightLot {
    pub route: RouteHandle,
    pub progress_edge: u16,
    pub remaining_ticks: u16,
    pub amount: f32,
    pub movement: FreightMovementModel,
}

pub enum FreightMovementModel {
    Continuous,  // trucks, pipelines
    Batched,     // ore trains, convoys, port loads
}

/// SoA solver — hot loops index by TransportEdgeId.0 (sparse HashMap diagnostics only).
#[derive(Resource, Default)]
pub struct ThroughputSolverState {
    pub load: Vec<f32>,
    pub capacity: Vec<f32>,
    pub reserved: Vec<f32>,
    pub topology_revision: u32,
}

/// Deterministic debug trace — add in LOG-C before async (LOG-C-05).
#[derive(Clone, Debug)]
pub struct RouteProof {
    pub request_id: u64,
    pub requested: f32,
    pub delivered: f32,
    pub blocked_at: Option<TransportEdgeId>,
    pub bottleneck_capacity: f32,
}
```

**Rule:** No inventory credit on facility B unless (a) local buffer consume, or (b) `FreightLot` **arrived** after reservations on a version-valid `RouteHandle`.

### Transport edge classification (LOG-D-01, spec in LOG-A)

Add `CorridorClass` on transport metadata / field state (not logistics-only): `Road | Rail | Maritime | Conveyor | Power | Pipeline`. Drives cost cache, route legality (`allowed_agents`), congestion curves, maintenance.

4. Modules, systems, schedules, resources
Proposed crate layout
src/
  systems/transport/          # unchanged authority (topology, fields, cost, nav)
  strategic/
    transport_bridge.rs       # rebuild junction graph from transport
    logistics_portals.rs      # NEW: facility ↔ nearest portal edges
    logistics_witness.rs      # overlay: solved load not static capacity
  economy/
    logistics/                # NEW package (split resource_flow)
      mod.rs
      types.rs                # ResourceFlowNode/Edge, Freight*, RouteHandle
      routes.rs               # path_open, route build, invalidation
      requests.rs             # facility RequestSlot / buffer / reserve
      throughput_solver.rs    # ThroughputSolver + LogisticsTick phases
      propagation.rs          # arrivals → inventory, starvation
      diagnostics.rs          # witness + trace events
      plugin.rs
    resource_flow.rs          # thin: re-export or deprecate gradually
    logistics_bridge.rs       # → portals only
System sets (Bevy 0.18)
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsSimulationSet {
    /// After TransportSchedule::CostCache — routes use fresh costs.
    RouteRefresh,
    /// Collect facility pull/push; no inventory commit.
    RequestGather,
    /// Max-flow / min-cost flow on capacitated graph; write reservations.
    ThroughputSolve,
    /// Advance in-transit; apply arrivals; partial fulfillment.
    FreightCommit,
    /// Push congestion from edge_load into TransportFieldStore.
    FieldFeedback,
    /// Diffuse saturation to neighbors — prevents reroute oscillation (LOG-C-04).
    CorridorPressure,
    /// Facility produce/consume with starvation cascade.
    FacilityBalance,
    /// Witness JSON + overlay injection.
    Diagnostics,
}
Schedule placement (relative to existing)
SimControlSystemSet::AdvanceSimTick
  → TransportSchedule::Topology
  → TransportSchedule::FieldIntegrate
  → TransportSchedule::CostCache
  → StrategicFieldPipeline::GraphSync          # LogisticsGraph + portals
  → LogisticsSimulationSet::RouteRefresh     # path_open, RouteHandle cache
  → LogisticsSimulationSet::RequestGather
  → LogisticsSimulationSet::ThroughputSolve
  → LogisticsSimulationSet::FreightCommit
  → LogisticsSimulationSet::FieldFeedback      # closes loop: load → congestion
  → LogisticsSimulationSet::CorridorPressure   # neighbor spillover
  → InfrastructureSiteSet::… (unchanged)
  → Economy: FacilityBalance (after FreightCommit)
  → LogisticsSimulationSet::Diagnostics
  → StrategicFieldPipeline::LogisticsNetInject # use edge_load not static cap
Resources & caches
Resource	Purpose
ThroughputSolverState
SoA `load` / `capacity` / `reserved` indexed by edge id
RouteCache + RoutePathStore
Versioned `RouteHandle` → compact `RoutePath` in centralized store
InTransitLedger
`FreightLot` with `progress_edge` + `FreightMovementModel`
FreightReservationBook
Per-tick reservations (diagnostics may use sparse map)
LogisticsDiagnostics + RouteProof ring
Per-request delivered/blocked/bottleneck
PortalAttachmentMap
Rebuilt at GraphSync; entities keep `FacilityPortal` anchors only
TransportTopologyRevision
Increments with `ConstructionWorldRevision` / directory signature
Portal attachment (fixes facility disconnect)
On activate / graph sync:

1. Rebuild derived `LogisticsGraph` from transport (existing `transport_bridge`).
2. Rebuild `PortalAttachmentMap` — map facility entities to transient `LogisticsNodeId`; **never** store `LogisticsNodeId` on facility components.
3. Add portal stub edges portal ↔ nearest junction (off-network cost, dock capacity from JSON).
Reachability for `path_open` (**kill `path_open: true` default**):

Multi-source Dijkstra / A* on TransportNavExport from from portal junction, mode-filtered allowed_agents (road_vehicle, rail_train, …).
Optional: fallback logistics_path_dijkstra only for last mile off-network (portal → site), not whole chain.
5. ThroughputSolver — algorithm sketch
Per LogisticsTick (can be sub-sampled every N sim ticks):

Phase A — Requests
Each ResourceFlowNode emits FreightRequest { commodity, amount_wanted, priority, from, to } from consumption rates minus buffer.

Phase B — Routes
For each active ResourceFlowEdge, if !path_open skip. Else capacity bound = min(edge.max_demand_rate, route_bottleneck_capacity).

Phase C — Solve (hierarchical for scale)

Local (LOG-B): greedy on path edges — flow_e = min(request, cap_e - reserved_e) in path order; O(path length × requests).
Regional (LOG-C): for hotspots, push requests sharing edges into a small min-cost max-flow on subgraph (≤ few thousand edges per industrial district from IndustrialDistrictSnapshot).
Feedback: edge_load / capacity → congestion increment in TransportFieldStore (replaces decay-only loop for freight-driven congestion).
Partial fulfillment: if flow < request, source buffer retains remainder; downstream marks starved proportionally; emit FreightShortageEvent { edge, commodity, deficit }.

Reroute: invalidate RouteCache on ConstructionWorldRevision, corridor phase change, or damage > threshold; re-run RouteRefresh before solve.

Prioritization: military > grid fuel > construction concrete > bulk ore (enum FreightClass); ties broken by distance then age.

Chain-specific behavior (transformer / concrete / aluminum / power)
Chain	Mode	Notes
Aggregate → kiln
Truck / conveyor stub
Short haul; high volume, low value → low priority unless construction starved
Cement → mixer
Truck
Time-sensitive → ConcreteBatchState ties to arrival not inventory fiction
Bauxite → refinery → smelter
Rail preferred
allowed_agents filter; smelter pull gated on path + grid
Fabrication
Truck/rail
Downstream of smelter starvation cascade
Transformers / substations
Power corridor edge profile
Not bulk freight — separate PowerFlowSolver slice on CorridorType::PowerTransmission (I3 already has grid); logistics shares corridor capacity for maintenance convoys only
Coal plant
Rail + port
Import fuel as high-priority FreightClass::Energy
Geography: route cost = sum edge_traversal_cost + portal penalties; terrain enters via existing cost weights and optional LogisticsTile last-mile multiplier.

6. Code examples (Bevy 0.18 style)
Plugin registration
// src/economy/logistics/plugin.rs
use bevy::prelude::*;
use crate::systems::transport::TransportSchedule;
use crate::strategic::plugin::StrategicFieldPipeline;
pub struct LogisticsThroughputPlugin;
impl Plugin for LogisticsThroughputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThroughputSolverState>()
            .init_resource::<InTransitLedger>()
            .init_resource::<FreightReservationBook>()
            .init_resource::<RouteCache>()
            .init_resource::<LogisticsDiagnostics>()
            .init_resource::<LogisticsTick>()
            .configure_sets(
                Update,
                (
                    LogisticsSimulationSet::RouteRefresh
                        .after(TransportSchedule::CostCache)
                        .after(StrategicFieldPipeline::GraphSync),
                    LogisticsSimulationSet::RequestGather.after(LogisticsSimulationSet::RouteRefresh),
                    LogisticsSimulationSet::ThroughputSolve.after(LogisticsSimulationSet::RequestGather),
                    LogisticsSimulationSet::FreightCommit.after(LogisticsSimulationSet::ThroughputSolve),
                    LogisticsSimulationSet::FieldFeedback.after(LogisticsSimulationSet::FreightCommit),
                    LogisticsSimulationSet::FacilityBalance.after(LogisticsSimulationSet::FreightCommit),
                    LogisticsSimulationSet::Diagnostics.after(LogisticsSimulationSet::FacilityBalance),
                ),
            )
            .add_systems(
                Update,
                (
                    refresh_facility_routes_system,
                    gather_freight_requests_system,
                    solve_throughput_system,
                    commit_freight_arrivals_system,
                    feedback_congestion_from_load_system,
                    balance_facility_inventories_system,
                    record_logistics_diagnostics_system,
                )
                    .chain()
                    .run_if(economy_sim_running),
            );
    }
}
Path gate (replaces path_open: true stub)
pub fn refresh_facility_routes_system(
    nav: Res<TransportNavExport>,
    portals: Query<(Entity, &FacilityPortal)>,
    mut routes: ResMut<RouteCache>,
    mut edges: ResMut<ResourceFlowRegistry>,
    revision: Res<ConstructionWorldRevision>,
) {
    if routes.revision != revision.revision {
        routes.clear();
        routes.revision = revision.revision;
    }
    for edge in &mut edges.edges {
        let Some(route) = routes.get_or_compute(edge.from, edge.to, edge.mode, &nav, &portals) else {
            edge.path_open = false;
            continue;
        };
        edge.route = route.handle;
        edge.path_open = route.reachable && route.bottleneck_capacity > 0.0;
    }
}
Throughput solve + reservation
pub fn solve_throughput_system(
    mut solver: ResMut<ThroughputSolverState>,
    mut reservations: ResMut<FreightReservationBook>,
    graph: Res<LogisticsGraph>,
    transport_cap: Res<ThroughputSolverState>, // edge_capacity filled from graph+fields
    requests: Res<FreightRequestQueue>,
    tick: Res<LogisticsTick>,
) {
    if tick.phase != 1 {
        return;
    }
    reservations.clear();
    solver.edge_load.clear();
    for req in &requests.0 {
        let Some(path) = req.path_edges() else { continue };
        let mut flow = req.amount;
        for &eid in &path {
            let cap = solver.edge_capacity.get(&eid).copied().unwrap_or(0.0);
            let used = solver.edge_load.get(&eid).copied().unwrap_or(0.0);
            flow = flow.min((cap - used).max(0.0));
            if flow <= 0.0 { break; }
        }
        if flow > 0.0 {
            for &eid in &path {
                *solver.edge_load.entry(eid).or_insert(0.0) += flow;
                reservations.push(FreightReservation { edge_id: eid, amount: flow, holder: req.holder });
            }
            req.commit_in_transit(flow);
        }
    }
}
Arrival commit (no teleport)
pub fn commit_freight_arrivals_system(
    mut ledger: ResMut<InTransitLedger>,
    mut nodes: Query<&mut ResourceFlowNode>,
) {
    let mut i = 0;
    while i < ledger.lots.len() {
        if ledger.lots[i].eta_ticks > 0 {
            ledger.lots[i].eta_ticks -= 1;
            i += 1;
            continue;
        }
        if let Ok(mut node) = nodes.get_mut(ledger.lots[i].destination) {
            node.credit(&ledger.lots[i].commodity, ledger.lots[i].amount);
        }
        ledger.lots.swap_remove(i);
    }
}
Congestion feedback (closes transport loop)
pub fn feedback_congestion_from_load_system(
    solver: Res<ThroughputSolverState>,
    mut fields: ResMut<TransportFieldStore>,
) {
    for (eid, &load) in &solver.edge_load {
        let cap = solver.edge_capacity.get(eid).copied().unwrap_or(1.0);
        let saturation = (load / cap.max(1e-6)).clamp(0.0, 2.0);
        if let Some(state) = fields.by_edge.get_mut(eid) {
            state.congestion = (state.congestion + 0.15 * saturation).min(1.0);
        }
    }
}
7. Phased migration — LOG-A … LOG-D

**Live board:** [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs) · spec [`logistics_throughput_phase_todos.md`](logistics_throughput_phase_todos.md)  
**Exit:** `LOGISTICS_THROUGHPUT_GREEN` when all rows reconcile **Done** (witness predicates, not prose).  
**Not** Stage 5 · **not** construction lane · prerequisite: `INDUSTRIAL_ACTIVATION_GREEN`.

| Phase | Goal | Key rows |
|-------|------|----------|
| **LOG-A** | Authority & wiring | Derived graph + revision; `FacilityPortal` + `PortalAttachmentMap`; `transport_edge` on `LogisticsEdge`; `path_open` from nav; versioned `RouteHandle`; `logistics_throughput_live.json`; infra pairing fix |
| **LOG-B** | Freight ledger | `RoutePathStore`; compact `FreightLot`; `FreightMovementModel`; arrivals-only propagation; partial fulfillment tests |
| **LOG-C** | ThroughputSolver | SoA solver; reservations; congestion + **corridor pressure**; `RouteProof`; overlay load injection; cut-road cascade test |
| **LOG-D** | Scale & futures | `CorridorClass` on transport; district partition; streaming route invalidation; async district scaffold; diagnostics UI |

### LOG-A acceptance (highest leverage — implement first)

- Road connects mine → refinery → `path_open == true`; remove middle edge → `false` after route refresh.
- `sync_logistics_graph_from_transport` does not orphan facilities (`PortalAttachmentMap` rebuild).
- `link_supply_chain_edges_system` never sets `path_open: true` without reachability proof.
- Proof JSON: `debug_runs/logistics_throughput_live.json` with `routes_open`, `routes_blocked`, `topology_revision`.

### Recommended topology (final)

```text
Construction → TransportAuthority (topology, fields, costs, nav, revisions)
    → StrategicGraphBuilder (derived LogisticsGraph, portals, districts)
    → RouteSystem (cache, reachability, versioned invalidation, RouteProof)
    → ThroughputSolver (SoA requests, reservations, bottlenecks, pressure)
    → FreightLedger (in-transit, ETA, arrivals)
    → FacilityEconomy (inventories, starvation, production)
```
8. Performance strategy (large worlds)
Technique	Use
District partitioning
Reuse IndustrialDistrictSnapshot / chunk anchors — solve only subgraphs with active requests
Route cache
Invalidate by ConstructionWorldRevision + dirty corridor chunks
Hierarchical graph
Junction-level LogisticsGraph for solve; facility detail only at portals
Tick throttling
LogisticsTick.interval_ticks = 4 for bulk; priority-1 requests every tick
Solver hot path
SoA `Vec<f32>` only — HashMap for diagnostics overlays, not solve loop
Incremental rebuild
Diff TransportEdgeDirectory signature (already in sync_construction_book_after_transport_changes) → patch graph nodes/edges
SOA edge arrays
Parallel capacity/load arrays indexed by TransportEdgeId.0 for solver hot loop
Cap solver size
If district > 8k edges, fall back greedy path flow + warn once
Target: < 2 ms district solve @ 10³ edges; full world amortized across ticks.

9. Chunk streaming compatibility
Authority per loaded chunk: transport edges whose endpoints map to loaded ChunkCellKey participate in solve; portals on operational facilities in loaded chunks emit requests.
Streaming in: hydrate transport slice for chunk → GraphSync for affected bbox → invalidate RouteCache region.
Streaming out: freeze in-transit lots (serialize into chunk snapshot) or force delivery to nearest buffer depot.
Cross-chunk routes: route cache key includes portal pair only — path may span unloaded chunks if nav export stores global topology (current model); for true streaming, store prefix route in snapshot and complete on load.
Align StrategicRasterConfig.cells_per_chunk with terrain matrix (already done in ensure_chunk_strategic_overlays).
10. Simulation throttling & async solving
Mode	When
Sync greedy
Default LOG-B; same frame as sim
Sync MCMF
LOG-C districts with contention
Deferred district
AsyncLogisticsJob resource: main thread posts district id + request snapshot; background thread returns edge_load next frame (Bevy task pool or std::thread channel)
Coarse global tick
Strategic AI-only regions: update every N ticks, hold facility buffers
Guardrails: never mutate TransportFieldStore off main thread; async only returns reservation deltas applied in ThroughputSolve entry.

11. Diagnostics, witnesses, overlays
Proof JSON (debug_runs/logistics_throughput_live.json):

{
  "tick": 1204,
  "routes_open": 42,
  "routes_blocked": 3,
  "edge_saturation_max": 0.91,
  "starvation_events": 2,
  "bottleneck_edges": [{"id": 17, "load": 0.91, "profile": "default_road"}],
  "chains": {
    "aluminum_primary": {"refinery_starved": true, "smelter_input_deficit": 12.5}
  }
}
Tracing: LogisticsTraceBuffer ring — last N shortages with { facility, commodity, blocking_edge, deficit }.

Overlays:

routing_congestion ← existing transport inject (keep).
logistics_throughput ← solver edge_load mapped to cells (replace static capacity in LOG-C).
Debug UI panel in diagnostics_ui.rs: top saturated edges, starved facilities.
Witness board: [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs) (24× `LOG-*` rows).

12. Future compatibility (agents, traffic, warfare)
Future system	Hook
Road agents / traffic
W5 FreightReservation + allowed_agents; agents consume edge capacity before industrial solve (priority lane)
Trains
Rail profile edges, separate mode solver pass, station portals
Shipping / ports
TransportMode::Maritime + port portal nodes, multi-hop routes
Dynamic routing
RouteCache invalidation + alternate path scoring (disruption-aware Dijkstra)
Maintenance / attrition
Write EdgeFieldState.damage from logistics + construction; reduces capacity in rebuild
Sanctions / warfare
disruption + danger spikes on strategic corridors; AI reads same ThroughputSolverState
Economic AI
WorldReadSnapshot includes district load_ratio and bottleneck edge ids
13. Immediate implementation order (recommended)
LOG-A — LogisticsEdge.transport_edge_id, portal sync after GraphSync, path_open from nav, fix infrastructure pairing.
LOG-B — split propagation: requests → in-transit → arrivals; wire latency_ticks from path length.
LOG-C — ThroughputSolver + congestion feedback + overlay load injection.
LOG-D — district async, proof JSON, chunk invalidation.
This preserves construction + transport topology as foundation, extends industrial activation without reverting green boards, and replaces magical inventory transfer with infrastructure-bounded causality.

**Next in-repo:** LOG-A-01…A-07 (see phase todos). RouteProof (LOG-C-05) before async district solve (LOG-D-04).

---

## Appendix — review notes (incorporated above)

Post-design review (lines 582+) identified eight scaling traps; all are merged into §3–§7:

1. **LogisticsGraph derived-only** — no solve-time mutation; runtime in solver resources.
2. **FacilityPortal anchor-only** — `PortalAttachmentMap` rebuilt at GraphSync.
3. **RouteHandle versioned** — invalidate on `ConstructionWorldRevision` / topology signature.
4. **RoutePathStore** — no `Vec<TransportEdgeId>` per `FreightLot`.
5. **CorridorClass on transport** — early, not logistics-only.
6. **SoA ThroughputSolver** — `Vec<f32>` indexed by edge id.
7. **FreightMovementModel** — `Continuous` vs `Batched` latency.
8. **RouteProof + CorridorPressure** — deterministic traces before async; pressure diffusion against reroute oscillation.

Authoritative spec: §3–§7 + [`logistics_throughput_phase_todos.md`](logistics_throughput_phase_todos.md) + live board [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs).