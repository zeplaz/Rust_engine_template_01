# PLAN-ATTRITION-SYSTEMS-COUNTER-v1 — cut the second war brain

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ATTRITION-SYSTEMS-COUNTER-v1** |
| **Status** | **REFUSAL** — no new systems |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (read-only review) |
| **Not this file** | Do not edit `plan_attrition_systems_fit_v1.md`, `plan_macro_ai_fit_v1.md`, `plan_macro_ai_counter_v1.md`, `src/strategic/`, or `src/construction/` from this plan. Do not commit. |

Industrial war: power, hauls, plants, factories, rail, damage. Panzer quotas, ammo blackboards, radar/SAM priority lists, and diesel island managers are the wrong game even when a reflection names them.

`ATR-CUT-*` ids are refusal gates. They do not add systems. A fit plan must not schedule one. Each cut condition below is already met in the named file.

```text
⊚UtilityGraph ═▶ ⊨power topology          ⊚PowerLineDamageBook ═▶ ⊨segment hp, circuit_open, load_fraction
⊚sync_logistics_graph_from_transport ═▶ ⊨LogisticsEdge (only writer)
⊚transport_field_integrate ═▶ ⊨EdgeFieldState.danger (decay today; the only threat-cost writer allowed)
⊚ThroughputSolverState ⊰ LogisticsGraph     ⊚solve_throughput_greedy_system ═▶ ⊨path residual, not max-flow
⊚ChunkStrategicOverlay ═▶ ⊨threat, logistics_throughput, power_flow
⊚PowerRepairQueue ═▶ ⊨one committed-line repair
⊚LogisticsVisualSnapshot.edge_rows ═▶ ⊨(u32, f32)
```

One growth site per faction step stays **TWN-AI** in `src/dev/plan_town_growth_v1.md`. This plan does not raise that cap.

## CUT

| Proposal | Already does the job | Failure if added |
|:---|:---|:---|
| **ATR-CUT-GRID** — a power-grid agent, a utility-curve load shed, a per-island blackboard, a second power graph | `UtilityGraph` (`src/infrastructure/utility/graph.rs`) is the power topology. It stores nodes (`id`, `key`, `position`) and `power_edges` (`from`, `to`, `link_id`, nominal `throughput`). `PowerLineSegmentHealth::circuit_open` and `load_fraction` (`src/construction/power_lines/damage.rs`) open a line at hp 0 and conduct `hp / max_hp` otherwise. `apply_utility_power_with_line_load` in that same graph file writes `InfrastructureGraph` throughput as `edge.throughput * frac` and writes 0 on an open line. `powered_utility_node_ids` / `plant_reaches_consumer` (`src/infrastructure/utility/activation_link.rs`) walk the live feeder and skip blocked link ids. `haul_flow_while_plant_feeds` (`src/strategic/logistics_net.rs`) zeros haul flow into `ChunkStrategicOverlay` when the plant no longer reaches a consumer. `compute_island_partition` (`src/render/pipelines/infrastructure_overlay.rs`) is the island read. Overload is already `ElectricalGrid.total_load > total_capacity` via `emit_grid_overload_signals` (`src/entities/production/power/grid_topology.rs`). `update_electrical_load_system` (`src/entities/production/power/systems.rs`) is the writer of `ElectricalComponent.current_load`. Catalog roles already exist as `UtilityInfrastructureRole` (`src/construction/utility_infrastructure_role.rs`) and `IndustrialSupplyChainRole` (`src/construction/supply_chain_role.rs`). They are resolved from the building catalog. They are not a field on `UtilityGraphNode`. | A second graph. A blackboard copy of the island. A curve asset. An agent that re-decides power the damage book already decides. Diesel generators as an island manager. |
| **ATR-CUT-FLOW** — flow fields and a regenerating vector field for trucks | The haul route is `LogisticsGraph`. `sync_logistics_graph_from_transport` (`src/strategic/transport_bridge.rs`) is the only writer of `LogisticsEdge` (`capacity`, `disruption`, `traversal_cost`). It copies `edge_traversal_cost` and folds `EdgeFieldState.danger` into `disruption`. Freight reachability is `path_edges_between_tiles` on `TransportNavExport` (`src/economy/logistics/routes.rs`), which reads `TransportCostCache`. `logistics_path_dijkstra` (`src/systems/navigation/logistics_floodfill.rs`) is the existing cell-grid cost search. It is not a vector field and it does not write `LogisticsGraph`. `transport_field_integrate` (`src/systems/transport/mod.rs`, `TransportSchedule::FieldIntegrate`) already writes `TransportFieldStore` and today only decays `danger`. `edge_traversal_cost` already adds `field.danger * weights.danger`. | A flow field, a regenerating vector field, or any system besides `sync_logistics_graph_from_transport` writing `LogisticsEdge`. A second route authority next to the graph. |
| **ATR-CUT-HTN** — an HTN for ore-to-tank pipelines and emergency objectives on a global blackboard | Chain order is `IndustrialSupplyChainRole` on the catalog. Throughput shortfall is `solve_throughput_greedy_system` (`src/economy/logistics/solver.rs`): it reserves along an open path, stores residual on `ThroughputSolverState`, and records `RouteProof.bottleneck_capacity`. Stock is `LogisticsNode.stockpile`. A line repair is `PowerRepairQueue::enqueue_damaged_segment` (`src/construction/power_lines/repair.rs`). `advance_power_repair_queue` calls `restore_segment` on the committed `PowerLineDamageBook` and does not insert graph edges. | An HTN crate, a global blackboard, or a new objective type beside the repair queue. |
| **ATR-CUT-MAXFLOW** — min-cut / max-flow sabotage as a new solver, and a bomber entity | Power cut is the open feeder: `circuit_open` plus `compute_island_partition` (destroyed link ids leave the BFS). Logistics flow is the greedy residual in `solve_throughput_greedy_system` and the chunk diffusion in `network_flow_chunk_local_solver_system` (`src/strategic/network_flow.rs`), which writes overlay `power_flow` / `logistics_flow` only. Neither is a max-flow. A shot that already arrived calls `damage_power_line_segment` from `apply_arrived_shot_impacts_to_power_lines` (`src/strategic/shot_pulse.rs`). Haul derate is `open_flow(capacity, disruption)` after GraphSync copies field danger. Threat to read is `ChunkStrategicOverlay.threat` or `book_danger_at` on `ShotPulseBook`. | Edmonds-Karp, a cut library, a second residual graph, a bomber entity. |
| **ATR-CUT-STORE** — petgraph, a CSR matrix resource, `NodeIndex` components, a new 64×64 influence chunk, a new macro schedule | Stores that exist: `UtilityGraph`, `LogisticsGraph`, `ChunkStrategicOverlay` (per chunk, default raster `StrategicRasterConfig` 32×32). Macro work already ticks in place: `TransportSchedule` then `StrategicFieldPipeline::GraphSync` after `SimControlSystemSet::AdvanceSimTick` (`src/strategic/plugin.rs`); `PowerRuntimePlugin` on `Update` gated by `power_simulation_running`; `growth_proposal_tick_system`; `tick_strategic_command_queue_system` after `AdvanceSimTick`. | A graph crate, a matrix resource, node-index components, a 64×64 influence chunk, or a move of those systems onto a new set. |
| **ATR-CUT-WRONG-GAME** — Panzer quotas, ammo blackboards, radar/SAM priority lists, diesel island managers | Damage is `damage_power_line_segment` and haul `disruption`. Supply steps are `IndustrialSupplyChainRole`. Cooling-diesel notes in nuclear failure modes are plant failure flags, not an island manager. | Those four systems. They have no remainder. |

## NARROW

| Id | Only behavior allowed | Cut condition (already met — do not schedule) |
|:---|:---|:---|
| **ATR-CUT-GRID** | No agent, no curve, no blackboard, no second graph. The only extra behavior is a fixed priority order on the existing shed. When `total_load > total_capacity` is already true, `update_electrical_load_system` in `src/entities/production/power/systems.rs` may drop `current_load` in catalog-role order: fabrication and smelter before mixer, kiln, and mine; `UtilityInfrastructureRole::PowerPlant` sheds last. Resolve the role with `IndustrialSupplyChainRole::resolve` and `UtilityInfrastructureRole::resolve` from the building catalog. Do not copy the role onto `UtilityGraph`. Do not change `apply_utility_power_with_line_load`. Line hp derate and the islanded-plant haul stop stay as they are. | Circuit open, hp derate, load drop, and islanded-plant haul stop already run. |
| **ATR-CUT-FLOW** | No flow field and no regenerating vector field. Threat may raise cost only inside `transport_field_integrate`, by increasing `EdgeFieldState.danger` on `TransportFieldStore`. That function already writes `danger`. `sync_logistics_graph_from_transport` is the only system that may write the edge (`LogisticsEdge.traversal_cost` and `disruption`). `feedback_congestion_from_load_system` stays congestion-only. `logistics_path_dijkstra` stays a cell-grid read. It must not write the graph and must not become a vector field. | `LogisticsGraph` already has one writer, and `edge_traversal_cost` already includes `danger`. |
| **ATR-CUT-HTN** | No HTN engine and no blackboard. A bottleneck is a read of `RouteProof.bottleneck_capacity`, `ThroughputSolverState` residual (`capacity - load - reserved`), or `LogisticsNode.stockpile`. That read may call `PowerRepairQueue::enqueue_damaged_segment` once for one committed link. It may not enqueue a new objective type, and it may not insert an edge. | The repair queue already restores a committed segment. |
| **ATR-CUT-MAXFLOW** | No second max-flow and no bomber. One target score: existing edge throughput (`open_flow` or solver residual) times existing threat (`ChunkStrategicOverlay.threat` or `book_danger_at`). The score may only feed a path that exists: `damage_power_line_segment` (already used by `apply_arrived_shot_impacts_to_power_lines`) or the haul derate in **ATR-CUT-FLOW** (`EdgeFieldState.danger` inside `transport_field_integrate`). | Island partition and the greedy residual already expose the weak edge. |
| **ATR-CUT-STORE** | No remainder. Do not add petgraph, a CSR resource, `NodeIndex` components, or a 64×64 chunk. Macro systems stay on `TransportSchedule`, `StrategicFieldPipeline`, the power `Update` chain, the growth tick, and the strategic command tick. | The three stores and those ticks already exist. |
| **ATR-CUT-WRONG-GAME** | No remainder. | The shot and the haul derate are the attrition tools. |

## Do not build

A coder follows this list. None of it is a task. Refuse the work from this file alone.

- Do not add a power-grid agent, a utility-curve asset, a per-island `Blackboard`, or a second power graph. The shed file is `src/entities/production/power/systems.rs`. The topology file stays `src/infrastructure/utility/graph.rs`.
- Do not add a flow field or a regenerating vector field. Do not let any system except `sync_logistics_graph_from_transport` write `LogisticsEdge`. Threat cost is `transport_field_integrate` writing `EdgeFieldState.danger` only.
- Do not add an HTN crate or a global blackboard. One repair is `PowerRepairQueue::enqueue_damaged_segment`.
- Do not add a min-cut or max-flow solver, a residual graph, or a bomber entity. One score, then `damage_power_line_segment` or the haul derate.
- Do not add petgraph, a CSR matrix, `NodeIndex` components, or a 64×64 influence chunk. Do not move macro systems onto a new schedule.
- Do not add Panzer quotas, an ammo blackboard, a radar or SAM priority list, or a diesel island manager.
- Do not enqueue more than one growth site in a faction step. That cap is **TWN-AI**.
- Do not add a second fire extract. Fire stays `FireVisualFrameSet` in `src/render/extraction/fire_visual_extract.rs`.
- Do not add a second camera. View stays `ViewManager` in `src/gui/view_authority.rs`.
- Do not change the logistics visual edge. `LogisticsVisualSnapshot.edge_rows` stays `Vec<(u32, f32)>` in `src/render/extraction/logistics_visual_snapshot.rs`.
- Do not schedule `ATR-CUT-GRID`, `ATR-CUT-FLOW`, `ATR-CUT-HTN`, `ATR-CUT-MAXFLOW`, `ATR-CUT-STORE`, or `ATR-CUT-WRONG-GAME`. Their cut conditions are met.
