# PLAN-ATTRITION-FIT-v1 — power shed, threat-closed hauls, idle solver skip

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ATTRITION-FIT-v1** |
| **Status** | **OPEN** — slices **ATR-1 → ATR-2 → ATR-3** |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` → `@coder` |
| **Obeys** | `src/dev/plan_attrition_systems_counter_v1.md` |
| **Not this program** | Town rings and the one growth enqueue (TWN-*). Macro ranking (MAI-1..3). Play-image and egui retirement. A second fire extract, a second camera, a suitability book, GOAP, HPA*, a blackboard, wall kits, flow fields, an HTN engine, a min-cut solver, a max-flow solver. |

Industrial power, hauls, plants, factories, rail, factions, damage. The proposals below were read against `UtilityGraph`, `LogisticsGraph`, `ThroughputSolverState`, and the power-damage book. Shipped behavior stays. The three slices are the gaps one existing writer can close without a new resource, a flow field, an HTN, or a max-flow solver.

## Summary

`UtilityGraph` is already the power graph. A cut, a partial hit, an open feeder, and an islanded plant already change what conducts and what a haul may carry. What is missing, and only when `ElectricalGrid.total_load > total_capacity` is already true, is a drop of `ElectricalComponent.current_load` in catalog-role order. That drop is a write inside `update_electrical_load_system`. It does not change `UtilityConnection.connected`. Haul search is still an unweighted walk. Overlay threat reaches cost only when `transport_field_integrate` copies it into `EdgeFieldState.danger`, which that function already owns and already decays. The logistics rebuild already copies danger into traversal cost and disruption; it folds that danger with the existing heat rule so disruption `>= 1` closes the edge. The freight solver still runs on every live economy tick, including ticks whose topology, `path_open` flags, and rates did not change. Chunk flow and the logistics graph resource are already in place. Facilities must keep resolving through `PortalAttachmentMap` after each rebuild.

```text
⊚PowerLineDamageBook ═▶ ⊨ segment hp, circuit_open, load_fraction
⊚UtilityGraph ═▶ ⊨ power nodes and power_edges
⊚merge_utility_power_into_infrastructure_graph ⊰ ⊚UtilityGraph ◂⊳[load_fraction]
⊚sync_utility_connection_power_system ═▶ ⊨ UtilityConnection.connected
⊚ElectricalGrid ═▶ ⊨ total_load, total_capacity
⊚update_electrical_load_system ═▶ ⊨ ElectricalComponent.current_load
⊚LogisticsGraph ═▶ ⊨ haul nodes and edges  (writer: sync_logistics_graph_from_transport)
⊚transport_field_integrate ═▶ ⊨ EdgeFieldState.danger
     ⊰ ⊚ChunkStrategicOverlay.threat   (copy into danger; decay stays)
⊚ChunkStrategicOverlay.threat ═▶ ⊨ faction threat     (overlay writers stay)
⊚path_edges_between_tiles ⊰ ⊚TransportNavExport adjacency  (BFS; may skip disruption >= 1)
⊚ThroughputSolverState ═▶ ⊨ capacity, load, reserved   (writer: solve_throughput_greedy_system)
⊚PortalAttachmentMap ⊰ ⊚LogisticsGraph                (rebuilt after GraphSync)
```

## Verdict

| # | Proposal | Verdict | Why |
|:---|:---|:---|:---|
| 1 | Power as a graph with capacity. Shed the lowest catalog role when `total_load > total_capacity`. Island a severed region. | **FIT** (shed only). Island, derate, cut, repair, and the islanded plant are **SHIPPED**. | `circuit_open` drops a link at zero HP or a non-finite HP. A partial hit multiplies carried load by `hp / max_hp`. `plant_reaches_consumer` is false when both feeders are blocked. `haul_flow_while_plant_feeds` stops the haul. Repair restores only a link present on the committed graph. `emit_grid_overload_signals` reports `total_load > total_capacity` and leaves every member connected. `update_electrical_load_system` today only clamps `current_load`. It does not drop a load by catalog role. |
| 2 | Haul routes that weigh throughput against threat, including flow fields for many trucks. Threat raises path cost and the route changes. | **NARROW**. Flow fields are **DEFER** and are not a slice. | `edge_traversal_cost` already adds `danger`. Graph rebuild copies that into `traversal_cost` and into `disruption`. Freight picking is `bfs_path_to_goals`, which never reads `NavExportEdge.cost`. `transport_field_integrate` writes `EdgeFieldState.danger` and today only decays it. It does not yet copy `ChunkStrategicOverlay.threat` into `danger`. A flow field would be a second pathfinder. |
| 3 | HTN for ore, smelter, steel, parts, assembly. A bottleneck spawns a repair or a reroute. | **DEFER** | Chains already exist as `IndustrialSupplyChainRole` (bauxite → alumina → smelter → fabrication, and the concrete mine → kiln → mixer). `FacilityFlowState.starved` and `RouteProof.blocked_at` / `bottleneck_capacity` already mark a short edge. `reroute_recommended` is an unused stub. Spawning a job from that mark is a second planner beside the growth enqueue and the committed power-repair queue. |
| 4 | The enemy picks a graph edge by min-cut / max-flow: the cut that drops the most throughput, minus the cost of reaching it. | **DEFER** | An arrived shot already damages the committed power segment under its impact tile (`committed_power_link_at_cell`, then `damage_power_line_segment`). A global max-flow search is a new solver and a new aim planner. It is not a function the shot path already performs. |
| 5 | Macro flow is not every frame. The graph is a resource. Entities hold an index. Updates are chunk-local. | **NARROW** (idle skip only). The resource, the portal map, and chunk-local dirty flow are **SHIPPED**. An entity-held `LogisticsNodeId` is refused. The idle skip must not add a graph store. | `LogisticsGraph` and `UtilityGraph` are resources. `rebuild_logistics_graph_from_transport` replaces node ids. `PortalAttachmentMap` is rebuilt at `GraphSync` because a stored id would be stale. `network_flow_chunk_local_solver_system` runs only when `NetworkDirtyMask` has `NETWORK_DIRTY_FLOW`. `solve_throughput_greedy_system` still runs on every tick `economy_sim_running` is true. |

## Current problems

- Reachability on `UtilityGraph` is a boolean flood. `powered_utility_node_ids` skips a blocked `link_id` and then walks `from` and `to` both ways. Residual `throughput * load_fraction` is applied later, onto `InfrastructureGraph`, and never decides who stays connected. A partial hit still conducts. An overload event does not drop `current_load`. `update_electrical_load_system` only clamps.
- Overlay threat and haul danger do not meet. Threat is on the chunk overlay. Danger on the transport field decays toward zero and is never raised from that threat. The rebuild samples neither. The BFS that sets `ResourceFlowEdge.path_open` ignores cost and ignores disruption, so a threatened edge stays the path.
- `solve_throughput_greedy_system` clears the reservation book and walks `ResourceFlowRegistry` edges whenever the economy sim is running. The file comment says the solve is on demand. The schedule does not check that the topology revision, the `path_open` flags, and the rates are unchanged.
- Storing `LogisticsNodeId` on a facility would dangle after the next full replace of `LogisticsGraph`.

## Target architecture

No new resource. No new graph store. No new `SystemSet`. No edit to `src/strategic/plugin.rs` or `src/construction/mod.rs`. No flow field, no HTN, no max-flow solver.

```text
Input (damage book, electrical loads, overlay threat)
  ═▶ FieldIntegrate  transport_field_integrate copies overlay threat into EdgeFieldState.danger, then keeps the existing decay
  ═▶ GraphSync  sync_logistics_graph_from_transport (sole LogisticsEdge writer)
        folds that danger with disruption_after_cell_heat; does not read the overlay
  ═▶ RouteRefresh  existing BFS, may skip an edge whose disruption is already >= 1
  ═▶ power Update  update_electrical_load_system drops current_load by catalog role
        only when total_load > total_capacity is already true
  ═▶ ThroughputSolve  same system, return before clearing the book when nothing changed
        fingerprint includes path_open
```

`⊚update_electrical_load_system` remains the only writer of `ElectricalComponent.current_load` in this program. `⊚sync_utility_connection_power_system` remains the only writer of `UtilityConnection.connected` for power, and ATR-1 does not call it. `⊚transport_field_integrate` remains the only writer of `EdgeFieldState.danger`. `⊚sync_logistics_graph_from_transport` remains the only writer of `LogisticsGraph`. `⊚solve_throughput_greedy_system` remains the only writer of solver load and reservations. `⊚logistics_net_inject_into_overlays` remains the writer of overlay logistics throughput. Overlay threat writers stay where they are. ATR-2 reads threat and writes `danger` only.

Shed order, locked so the slice does not invent a table and does not copy a role onto `UtilityGraph`. Resolve with `IndustrialSupplyChainRole::resolve` and `UtilityInfrastructureRole::resolve` from the building catalog row. Drop `current_load` to 0 in this order, and stop once `total_load` would no longer exceed `total_capacity`. Same inputs, same loads.

| Order | Role | Shed |
|:---|:---|:---|
| 1 | `AluminumFabrication`, `AluminumSmelter` | drop `current_load` first |
| 2 | `ConcreteMixer`, `CementKiln`, `AggregateMine`, `BauxiteMine` | drop after fabrication and smelter |
| 3 | `UtilityInfrastructureRole::PowerPlant` | sheds last |

Ties inside one role drop the higher `Entity` id. `AluminaRefinery`, `IntegratedPlant`, `Transformer`, and `Substation` are not in this order. This slice does not assign them a rank.

Threat close, locked: `transport_field_integrate` samples `ChunkStrategicOverlay.threat[cell][0]` at the edge's two anchors (max of the two) and writes `danger = max(sampled, decayed danger)`. Decay stays `0.2 * dt` toward zero when the sample is lower. The function does not write `LogisticsEdge` and does not write the overlay. `rebuild_logistics_graph_from_transport` does not sample the overlay. It already copies danger into `traversal_cost` via `edge_traversal_cost` and into `disruption`. After that existing disruption term, it calls `disruption_after_cell_heat(disruption, danger)`. Finite danger `>= 1` therefore yields disruption `1`. An edge with disruption `>= 1` is the same closed edge `open_flow` already refuses. The BFS may skip those edges and may return another path. It does not become a cost-weighted search and it is not a second Dijkstra.

Idle skip, locked: at the start of `solve_throughput_greedy_system`, if `solver.topology_revision` equals the revision stored on `ThroughputSolverState` from the previous solve and every `ResourceFlowEdge.max_rate` and `path_open` match the last solved snapshot, return without `book.clear()`. The snapshot is one field on `ThroughputSolverState`. It is not a new resource and not a graph store. A changed rate, a changed `path_open`, or a changed topology still solves that tick. A threat close that flips `path_open` is a changed tick.

## Implementation phases

```text
ATR-1 ═▶ ATR-2 ═▶ ATR-3
```

ATR-2 writes threat only as `EdgeFieldState.danger` inside `transport_field_integrate`. The rebuild, which already writes `LogisticsEdge`, folds that danger into disruption. ATR-3 does not run until route tests exist, so a skip cannot hide a path change. ATR-3 adds no graph store.

### ATR-1 — drop current_load by catalog role when the grid is already overloaded

**Goal.** When `ElectricalGrid.total_load > total_capacity` is already true (`total_capacity > 0`, the same predicate `emit_grid_overload_signals` uses), `update_electrical_load_system` drops `ElectricalComponent.current_load` to 0 in catalog-role order: fabrication and smelter before mixer, kiln, and mine; `UtilityInfrastructureRole::PowerPlant` last. Stop when the remaining loads would no longer exceed capacity. An open circuit still feeds nobody past the cut. A partial hit still conducts the derated fraction. Both open feeders still stop at the plant. `UtilityConnection.connected` does not change in this slice.

**Files.** `src/entities/production/power/systems.rs`. The system `update_electrical_load_system` is already chained after `recalculate_grid_totals_from_members` and `emit_grid_overload_signals` in `PowerRuntimePlugin`. Do not add a system. Do not add a resource. Do not edit `src/infrastructure/utility/activation_link.rs`.

**Change.** Read `ElectricalGrid` on the hosts this chain already totals. For members, resolve the catalog role with `IndustrialSupplyChainRole::resolve(catalog_id, json_role)` and `UtilityInfrastructureRole::resolve(catalog_id, json_role)` against `BuildingDefinitionRegistry`. The catalog id is the one already on that entity (`PlannedSite.catalog_id` when present). Where the chain member already carries `IndustrialSupplyChainMembership.role`, that role is the value `insert_supply_chain_runtime` stored from `IndustrialSupplyChainRole::resolve`; use it only as that resolved role, and do not write it onto `UtilityGraph` or `UtilityGraphNode`. A `PowerPlant` sheds in the last group because `insert_utility_runtime` already inserted it from `UtilityInfrastructureRole::resolve`. The clamp to `max_transfer` stays after the drop. This system does not write `ElectricalGrid.total_load`. The next tick's `recalculate_grid_totals_from_members` sees the new loads.

**Must not change.** `circuit_open`, `load_fraction`, `preview_island_offline_from_cut`, `compute_island_partition`, `advance_power_repair_queue`, `haul_flow_while_plant_feeds`, `merge_utility_power_into_infrastructure_graph`, `apply_utility_power_with_line_load`, `sync_utility_connection_power_system`. Do not set `UtilityConnection.connected` false. Do not make the flood one-directional. Do not add a shed resource, a priority asset, a utility curve, or a military weight. Do not copy a role onto `UtilityGraph`. Do not write `src/construction/`, `src/strategic/plugin.rs`, or `src/strategic/settlement/`.

**Acceptance.** `cargo test -p proc_A_dine01 --lib overload_sheds_fabrication_before_mine`

The new test beside that system: overload already true, fabrication and smelter `current_load` drop before mixer, kiln, and mine, and `PowerPlant` drops last. `grid_open_feeders_stop_at_the_plant` stays green and is not in this filter.

### ATR-2 — threat raises danger; the rebuild closes the edge; BFS stays BFS

**Goal.** `transport_field_integrate` copies overlay threat into `EdgeFieldState.danger`. The rebuild folds that danger with `disruption_after_cell_heat` so disruption `>= 1` closes the edge. `path_edges_between_tiles` may skip that edge and return another open edge between the same tiles. Throughput on the closed edge stays zero through the existing `open_flow`.

**Files.** `src/systems/transport/mod.rs` (`transport_field_integrate`, already in `TransportSchedule::FieldIntegrate`). `src/strategic/transport_bridge.rs` (`rebuild_logistics_graph_from_transport`, already called by `sync_logistics_graph_from_transport`). `src/strategic/logistics_net.rs` (`disruption_after_cell_heat`, called, not copied). `src/economy/logistics/routes.rs` (`path_edges_between_tiles` / `bfs_path_to_goals` may skip).

**Change.** `transport_field_integrate` reads `ChunkStrategicOverlay` and `TransportEdgeDirectory`. Anchor cells are the directory head and tail keys, the same parse the rebuild already uses. Slot 0 only. Write `danger` as above, then the existing congestion, damage, and heat decay stay. Do not write `LogisticsEdge` here. The rebuild does not take a threat lookup and does not read the overlay. After the existing `(damage + congestion * 0.45 + danger * 0.25)` term, set `disruption = disruption_after_cell_heat(disruption, danger)`. `traversal_cost` stays `edge_traversal_cost`. The route walk, which already receives `LogisticsGraph` in `refresh_resource_flow_routes_system`, may skip a transport edge whose logistics disruption is `>= 1`. BFS stays BFS. No second Dijkstra. `NavExportEdge` stays `{ id, cost, successors, allowed_agents, profile }`. Haul rows stay `(u32, f32)`. `logistics_path_dijkstra` stays the cell-grid read. It does not write the graph and it does not become a vector field.

**Must not change.** Do not sample `ChunkStrategicOverlay` inside `rebuild_logistics_graph_from_transport` or `sync_logistics_graph_from_transport`. Do not add a flow field, a grid potential, a regenerating vector field, a second Dijkstra, or HPA*. Do not replace `logistics_path_dijkstra`. Do not let any system other than `transport_field_integrate` write `EdgeFieldState.danger`. Do not let any system other than `sync_logistics_graph_from_transport` write `LogisticsEdge`. Do not write `threat` or call `set_faction_threat`. `feedback_congestion_from_load_system` stays congestion-only. Do not edit `src/strategic/plugin.rs`. Do not retarget the faction growth step.

**Acceptance.** `cargo test -p proc_A_dine01 --lib threat_drops_the_open_edge`

New test: two edges between the same tiles, threat 1 on one anchor copied into `danger`, the rebuild disruption is `>= 1` on that edge, the walk returns the other edge. `grid_islanded_plant_stops_the_haul` must still pass; do not put it in this filter.

### ATR-3 — solver returns when the tick did not change

**Goal.** A second economy tick with the same `topology_revision`, the same `path_open` flags, and the same `max_rate` values does not clear `FreightReservationBook` and does not rewrite load. A changed rate still solves. A threat close that flips `path_open` still solves. This slice adds no graph store.

**Files.** `src/economy/logistics/solver.rs`. `src/economy/logistics/types.rs` only if `ThroughputSolverState` needs the last-solved revision and rate fingerprint on the existing struct. `src/economy/logistics/tests.rs` for the test.

**Change.** Early return at the top of `solve_throughput_greedy_system` under the rule in Target architecture. The fingerprint includes `path_open`. Record the fingerprint only after a real solve. Do not add a timer, a frame counter, a new set, or a graph store. `sync_solver_capacity_from_graph_system` still copies capacity from the graph when the graph changed.

**Must not change.** Do not skip a tick whose revision, `path_open`, or rates changed. Do not store `LogisticsNodeId` on entities. Do not weaken `soa_solver_aligned` or `reservations_within_capacity`. Do not edit `src/strategic/plugin.rs` or `src/economy/logistics/mod.rs` unless the system stops being the sole solve writer, and that edit is not part of this slice.

**Acceptance.** `cargo test -p proc_A_dine01 --lib solver_skips_an_unchanged_revision`

## ECS schedule

Unchanged order. No new set.

```text
TransportSchedule::FieldIntegrate            transport_field_integrate
  ═▶ AdvanceSimTick
  ═▶ StrategicFieldPipeline::GraphSync       sync_logistics_graph_from_transport
  ═▶ InjectTransportScalars
  ═▶ LogisticsNetInject                      logistics_net_inject_into_overlays
  ═▶ NetworkFlow                             chunk-local, dirty mask only
  ═▶ LogisticsSimulationSet::RouteRefresh    refresh_resource_flow_routes_system
  ═▶ ThroughputSolve                         solve_throughput_greedy_system
        .run_if(economy_sim_running)
PowerRuntimePlugin on Update, already chained, .run_if(power_simulation_running):
  recalculate_grid_totals_from_members
  ═▶ emit_grid_overload_signals
  ═▶ update_electrical_load_system
Utility connection sync stays on the industrial activation schedule already in bridge.rs.
It is not the shed writer.
```

`NetworkFlow` stays a PostUpdate dirty mark and an Update solve on the next frame. Do not pull it forward.

## Diagnostics

No new witness file in these slices. The existing reads are enough:

- Power: `power_line_load_fraction` is 0 on an open circuit and `hp/max_hp` on a partial hit. `plant_reaches_consumer` is the island predicate. The shed proof is `current_load` after `update_electrical_load_system` when `total_load > total_capacity`. `UtilityConnection.connected` is unchanged by ATR-1.
- Hauls: `EdgeFieldState.danger` after the threat copy, then `LogisticsEdge.disruption` after `disruption_after_cell_heat`. `RouteProof.blocked_at` when the remaining path has no residual. Do not add a threat heatmap.
- Solver: `ThroughputSolverState.topology_revision`, the `path_open` fingerprint, and `FreightReservationBook` length unchanged on the idle tick.

`site_score` keeps reading overlay scalars. Do not store a suitability book.

## Edge cases

- Non-finite HP still opens the circuit (`circuit_open`). Finite danger `>= 1` closes the haul edge through `disruption_after_cell_heat`. Non-finite danger follows that function as it already stands: it does not raise disruption. A negative capacity stays closed (`open_flow`).
- A plant with no consumer still does not feed the haul. The shed does not change that.
- Threat on slot 0 only. Other faction slots do not raise `danger` in this program.
- Sampled threat of 0 leaves the existing decay. Sampled threat of 1 holds `danger` at 1 against that same decay, so the fold can close the edge.
- Graph rebuild replaces node ids. Portals rebind. Do not cache an id on the facility.
- Preview does not write occupation. Construction still commits only through `CommitConstructionSiteEvent`.
- One fire extract. Heat already raises disruption inside `logistics_net_inject_into_overlays`. ATR-2 calls the same heat function on `danger` inside the rebuild. It does not add a second heat reader that writes fire.
- Shot impacts stay on the tile the shot already reached. This plan does not aim them.

## Open questions

- The counter names fabrication, smelter, mixer, kiln, mine, and `PowerPlant`. `AluminaRefinery`, `IntegratedPlant`, `Transformer`, and `Substation` stay out of the shed order until a later program names them. This slice does not invent that rank.
- Register this plan on the program index through `@plan-orchestrator`. This file does not edit the index.
- ATR-3's fingerprint must include `path_open`. A threat close in ATR-2 that flips `path_open` is a changed tick and must still solve.

## SHIPPED — do not rebuild

| Behavior | Where it already lives |
|:---|:---|
| Zero HP and NaN open the circuit | `PowerLineSegmentHealth::circuit_open` |
| Partial hit derates by `hp / max_hp` | `load_fraction`, applied in `merge_utility_power_into_infrastructure_graph` |
| A cut drops load and islands | `powered_utility_node_ids`, `preview_island_offline_from_cut` |
| Repair only on a committed segment | `advance_power_repair_queue`, `POWER_REPAIR_UNCOMMITTED` |
| Both feeders open: plant does not feed | `plant_reaches_consumer`, test `grid_open_feeders_stop_at_the_plant` |
| Islanded plant stops the haul | `haul_flow_while_plant_feeds`, test `grid_islanded_plant_stops_the_haul` |
| Overload predicate | `emit_grid_overload_signals` when `total_load > total_capacity` |
| Load clamp | `update_electrical_load_system` |
| Catalog role resolve | `IndustrialSupplyChainRole::resolve`, `UtilityInfrastructureRole::resolve` |
| One logistics writer | `sync_logistics_graph_from_transport` calls `rebuild_logistics_graph_from_transport` |
| Danger field and its decay | `transport_field_integrate` |
| Cell search | `logistics_path_dijkstra` |
| Edge row type `(u32, f32)` | `play_haul_cells_at_tick`, logistics visual snapshot `edge_rows` |
| Overlay threat, power flow, logistics strength | `ChunkStrategicOverlay` |
| Site score reads those scalars | `site_score` |
| Chunk-local power/logistics flow | `network_flow_chunk_local_solver_system` and `NetworkDirtyMask` |
| Facility index after rebuild | `PortalAttachmentMap`, not a component id |
| Counter-plan cuts | suitability book, utility-curve assets, GOAP, HPA*, blackboard, wall kits (`plan_macro_ai_counter_v1.md`); power agent, flow field, HTN, max-flow, new graph store (`plan_attrition_systems_counter_v1.md`) |
| Single fire extract, one camera, preview does not mutate occupation | unchanged by ATR-* |

## Rejected by `plan_attrition_systems_counter_v1.md`

- **Old ATR-1.** Setting `UtilityConnection.connected` false for rank 1 inside `activation_link.rs`, and the rank table that kept `IndustrialSupplyChainMembership` while dropping other `ElectricalComponent` loads, are rejected. The counter allows only `update_electrical_load_system` dropping `current_load` in catalog-role order when `total_load > total_capacity` is already true. Roles stay on the catalog. They are not copied onto `UtilityGraph`. Open circuits, derate, and the islanded plant stay as they are.
- **Old ATR-2.** Sampling `ChunkStrategicOverlay` inside `rebuild_logistics_graph_from_transport` / `sync_logistics_graph_from_transport`, and the rule that this slice must not write `EdgeFieldState`, are rejected. The counter allows threat to raise cost only inside `transport_field_integrate`, by writing `EdgeFieldState.danger`. Only `sync_logistics_graph_from_transport` may write `LogisticsEdge`.

## Refused

- Flow field, HTN, min-cut / max-flow solver, blackboard, GOAP, HPA*.
- A new resource or a new graph store.
- Entity-stored `LogisticsNodeId`.
- A second writer of `LogisticsGraph`, `UtilityConnection.connected`, `EdgeFieldState.danger`, solver reservations, overlay threat, or fire.
- Copying `IndustrialSupplyChainRole` or `UtilityInfrastructureRole` onto `UtilityGraph`.
- Retargeting the one growth site per faction step.
- `src/construction/mod.rs` and `src/strategic/plugin.rs` in any ATR slice.
