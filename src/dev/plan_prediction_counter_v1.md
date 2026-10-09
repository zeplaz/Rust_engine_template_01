# PLAN-PREDICTION-COUNTER-v1 — refuse the quadratic solvers

| Field | Value |
|:---|:---|
| **Id** | **PLAN-PREDICTION-COUNTER-v1** |
| **Status** | **REFUSAL** — four cuts, two narrow remainders |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (read-only) → `@coder` implements only **PRD-CUT-PATH** and **PRD-CUT-OWN** |
| **Not this file** | Do not edit any other plan, `src/strategic/` from this review, or `src/construction/` except the one compare named under **PRD-CUT-OWN**. Do not commit from this plan. |

Industrial power and haul. Carts ride edges that already have a cost. Sites are one overlay sample. Shots are a tick lerp onto a committed power segment. The sim is one authority on one tick. A projectile integrator, a traffic search, a high-arc weapon, and a client predictor do not belong here.

`PRD-CUT-*` ids are refusal gates. **PRD-CUT-PLACE** and **PRD-CUT-NET** have no remainder. **PRD-CUT-CART** has one remainder, **PRD-CUT-PATH**. **PRD-CUT-ARC** has one remainder, **PRD-CUT-OWN**.

```text
⊚edge_traversal_cost ═▶ ⊨ per-edge duration (travel_time_base + field weights)
⊚TransportCostCache.by_edge ═▶ ⊨ that duration     ⊚NavExportEdge.cost ⊰ ⊚TransportCostCache
⊚LogisticsEdge.traversal_cost ⊰ ⊚edge_traversal_cost
     writer: sync_logistics_graph_from_transport / rebuild_logistics_graph_from_transport
⊚path_edges_between_tiles ⊰ ⊚bfs_path_to_goals     returns Vec<TransportEdgeId> already stored
⊚RoutePathStore ═▶ ⊨ those edge ids               ⊚RoutePathStore::edge_slice reads them
⊚freight_transit_ticks ═▶ ⊨ hop count on FreightLot.remaining_ticks
     not a sum of traversal_cost
⊚faction_growth_step_system ═▶ ⊨ one GrowthProposal then approve_growth_proposal_into_pending
     reads Faction.resources as priority only
⊚ResourceStorage.amounts ═▶ ⊨ present stock
     LogisticsAiRuntime.stockpile_fill_ratio ⊰ that sum / ResourceStorageCapacity.max_amounts
     writer: strategic_fields_coupling_tick
⊚LogisticsNode.stockpile is rebuilt to 0 — not a stock authority
⊚site_score ═▶ ⊨ one suitability scalar
     strategic_city_planning_hints_tick writes CityPlanningHints.last_best_site_score
⊚apply_arrived_shot_impacts_to_power_lines ═▶ ⊨ the only shot caller of damage_power_line_segment
     side is SHOT_IMPACT_SIDE_FACTION (1)
⊚UtilityGraph ═▶ ⊨ power_edges (no faction on UtilityGraphEdge)
```

## CUT

| Id | Refuse | Already on disk | Failure if added |
|:---|:---|:---|:---|
| **PRD-CUT-CART** | A projectile solver, gravity, or drag `D = -kv - cv^2` for carts. No integrator. No `Time.delta_seconds()` on a haul. | `edge_traversal_cost` (`src/systems/transport/types.rs`) is the edge duration: `travel_time_base` plus congestion, damage, danger, and heat. `refresh_transport_nav_export` copies it to `NavExportEdge.cost`. `rebuild_logistics_graph_from_transport` (`src/strategic/transport_bridge.rs`) copies the same number to `LogisticsEdge.traversal_cost`. `bfs_path_to_goals` (`src/economy/logistics/routes.rs`) already returns `Vec<TransportEdgeId>`. `path_edges_between_tiles` is that path. Arrival along that path is the sum of those stored costs. `freight_transit_ticks` (`src/economy/logistics/propagation.rs`) is a separate hop clock (`path_len`, or `path_len * 2` when batched). It is not a physics step. `haul_phase_after_cost` is `#[cfg(test)]` and scales one phase by one cost. It is not a path arrival. `logistics_path_dijkstra` (`src/systems/navigation/logistics_floodfill.rs`) returns `LogisticsPathResult.total_cost` as a sum of cell `cost_mul` on a dense grid. Production haul does not call it. That total is not `traversal_cost`. | A second motion model beside the edge cost. A cart that leaves the graph. Replacing `freight_transit_ticks` with an integrator. Calling `logistics_path_dijkstra` to time a cart. |
| **PRD-CUT-PLACE** | A binary-search mini-simulation of worker traffic to place markets. No second sim. | There is no worker entity and no citizen entity. `site_score` (`src/strategic/runbook_rounds.rs`, `city_planning`) is one function of `logistics_strength`, `flood_risk`, and `mean_threat`. `strategic_city_planning_hints_tick` (`src/strategic/sim.rs`) calls it once per existing settlement overlay and stores the max on `CityPlanningHints.last_best_site_score`. `workers_required` on the residential menu is a label count. `PreviewWorkerState` (`src/preview/assembly_worker.rs`) is a preview job, not a resident. | A loop that spawns simulated traffic to score a footprint. A market search beside `site_score`. |
| **PRD-CUT-ARC** | Catapults, archer towers, high-arc lobs over walls. No new weapon. No arc solver. | Shots are `ShotPulse` tiles. `shot_trace_tile` (`src/strategic/shot_pulse.rs`) lerps origin to target across `shot_travel_ticks` (0 if origin equals target, otherwise `SHOT_TRACE_TICKS`). Range is Chebyshev `shot_range_tiles` against `SHOT_MAX_RANGE`. A landed impact calls `damage_power_line_segment` from `apply_arrived_shot_impacts_to_power_lines` only. Facade module ids (`wall_brick_1u`, `wall_concrete_2u` in the module index) are art modules. The shot path does not test them. There is no siege wall kit on the shot. Faction 2 must not damage faction 1: when `side` is `SHOT_IMPACT_SIDE_FACTION` (1), `shot.faction != side` already `continue`s. `link_other_faction_shot_does_not_damage_this_side` locks that. | A gravity lob, a wall-clearance test, a second weapon, or a change that lets faction 2 call `damage_power_line_segment` on faction 1's lines. |
| **PRD-CUT-NET** | Dead reckoning, client extrapolation, and a bandwidth buffer for 5,000 harvesters. No prediction buffer. | There is no net session and no citizen or harvester entity. `MultiplayerPlugin` (`src/systems/agents/multiplayer.rs`) spawns local `PlayerConnection` entities and `CreateAgentCommand` from `AddPlayerCommand`. It has no socket, no replication, and no tick buffer. `Faction` (`src/strategic/behavior_entities.rs`) is a pressure component (`resources`, `cohesion`), not a person. Freight state is `FreightLot.remaining_ticks` advanced by one in `advance_lot_progress` on the sim tick. The sim is deterministic and single-authority. A client pose that runs ahead of that tick diverges from it. | A prediction buffer, an interpolation delay, or harvester replicas. |

## NARROW

Only these two. Both are gaps in the code above. They do not lift the cuts.

| Id | Gap the code shows | Only behavior allowed |
|:---|:---|:---|
| **PRD-CUT-PATH** | Nothing sums `NavExportEdge.cost` / `TransportCostCache.by_edge` / `LogisticsEdge.traversal_cost` along the `TransportEdgeId` slice `bfs_path_to_goals` already returned. `freight_transit_ticks` counts hops. `faction_growth_step_system` (`src/strategic/settlement/faction_step.rs`) enqueues whenever `select_faction_growth` returns, sets `GrowthProposal.priority` from `Faction.resources`, then calls `approve_growth_proposal_into_pending` in the same system. It does not read a path duration or `ResourceStorage`. `LogisticsNode.stockpile` is written `0.0` in `rebuild_logistics_graph_from_transport` and is not stock. | One pure function. Suggested name `sum_path_traversal_cost`. Inputs: the edge-id slice already in `RoutePathStore::edge_slice` (the path `path_edges_between_tiles` stored), and a lookup of `TransportCostCache.by_edge` or `NavExportEdge.cost` for those ids. Return `Option<f32>`: `None` if the slice is empty or any id is missing or non-finite; otherwise the sum. It does not write `LogisticsGraph`. It does not call `bfs_path_to_goals`, `path_edges_between_tiles`, or `logistics_path_dijkstra`. It does not read `Time` or `delta_seconds`. `faction_growth_step_system` calls it on a path the route cache already holds. Before `queue.enqueue`: if the sum is `None`, or the present sum of `ResourceStorage.amounts` is not finite and `> 0`, return. Do not enqueue. Do not call `approve_growth_proposal_into_pending`. There is no site price. Do not add one. Present `ResourceStorage.amounts` is the stock that covers the site. The duration sum is not a reason to approve on a later tick or earlier than the existing approve. `freight_transit_ticks` stays the hop clock. `LogisticsNode.stockpile` stays unread and unwritten. |
| **PRD-CUT-OWN** | `apply_arrived_shot_impacts_to_power_lines` skips `shot.faction != side`, then calls `damage_power_line_segment` for the rest. `side` is `SHOT_IMPACT_SIDE_FACTION` (1). Faction 1 therefore damages every committed segment `committed_power_link_at_cell` returns. `UtilityGraphEdge` has no faction, so those lines are this side's lines. `link_shot_impact_on_a_line_reduces_health` locks that hit. | The existing compare is the fix. In that function, also refuse `damage_power_line_segment` when `shot.faction == side`. Keep the `shot.faction != side` continue, so faction 2 still does not damage faction 1. Do not add a trajectory, a wall test, or a weapon. `shot_trace_tile`, `shot_travel_ticks`, and `SHOT_LINE_DAMAGE` stay. After both refuses this function does not damage lines. Update `link_shot_impact_on_a_line_reduces_health` so a faction-1 shot leaves `PowerLineDamageBook` without a segment. `link_other_faction_shot_does_not_damage_this_side` stays. |

## ECS schedule

No new `SystemSet`. No new resource.

```text
GraphSync
  ═▶ sync_logistics_graph_from_transport   sole LogisticsGraph writer (unchanged)
RouteRefresh
  ═▶ refresh_resource_flow_routes_system   bfs_path_to_goals stores edge ids (unchanged)
FreightDispatch
  ═▶ dispatch_freight_from_solver_system   freight_transit_ticks stays hop count
Growth
  ═▶ faction_growth_step_system
        reads RoutePathStore + TransportCostCache (or NavExportEdge.cost) via sum_path_traversal_cost
        reads ResourceStorage.amounts
        may return before enqueue
        else the existing enqueue and approve_growth_proposal_into_pending
Shot
  ═▶ apply_shot_impacts_to_power_lines_system
        same system; the compare in apply_arrived_shot_impacts_to_power_lines refuses own side and other factions
```

`⊚sync_logistics_graph_from_transport` stays the only writer of `LogisticsGraph`. `⊚strategic_fields_coupling_tick` stays the writer of `LogisticsAiRuntime.stockpile_fill_ratio`. The growth step reads storage amounts. It does not write them. `⊚damage_power_line_segment` stays the damage function. The shot system stops calling it for faction 1 and faction 2.

## Acceptance

- `cargo test -p proc_A_dine01 --lib economy::logistics::routes` — path bytes unchanged. A unit test of `sum_path_traversal_cost` on two known cache costs equals their sum. A missing id returns `None`.
- `cargo test -p proc_A_dine01 --lib strategic::settlement::faction_step` — the chained app test that expects `CommitTally` 1 must insert a stored path whose costs sum finite, and a `ResourceStorage` whose amounts sum `> 0`, or the assertion becomes tally 0 and the player `house` row still alone. A run with no `RoutePathStore` path does not enqueue.
- `cargo test -p proc_A_dine01 --lib strategic::shot_pulse` — faction 2 still does not open a segment. Faction 1 no longer reduces HP. `shot_trace_tile` tests stay.
- No new file under `src/systems/agents/`. `MultiplayerPlugin` is untouched.

## Do not build

A coder refuses these from this file. They are not tasks.

- Do not add drag, gravity, a ballistic cart, or an ODE for haul arrival.
- Do not change `freight_transit_ticks` into a cost sum or an integrator.
- Do not call `logistics_path_dijkstra` from growth or from freight dispatch. Do not treat `LogisticsPathResult.total_cost` as edge duration.
- Do not run `bfs_path_to_goals` again inside `sum_path_traversal_cost` or inside `faction_growth_step_system`.
- Do not write `LogisticsGraph`, `LogisticsNode.stockpile`, or `LogisticsEdge.traversal_cost` from the growth step.
- Do not call `approve_growth_proposal_into_pending` because a duration is small. No present stock, or no stored path, means return.
- Do not add `Time.delta_seconds()` to the sum or the refuse.
- Do not add a worker, citizen, harvester, or market-traffic loop. Do not call `site_score` in a search. One sample in `strategic_city_planning_hints_tick` stays the placement read.
- Do not add a catapult, an archer tower, a lob, or a wall-clearance test. Do not let faction 2 damage faction 1.
- Do not add a prediction buffer, client extrapolation, or harvester replicas. Do not read `MultiplayerManager` as a net session.
- Do not schedule **PRD-CUT-CART**, **PRD-CUT-PLACE**, **PRD-CUT-ARC**, or **PRD-CUT-NET** as features. Schedule only **PRD-CUT-PATH** and **PRD-CUT-OWN**.

## Open questions

- `ResourceStorage` is not keyed by the growth town. The refuse uses the world sum of `amounts`. A per-town stock split is out of this plan.
- `freight_transit_ticks` will still disagree with `sum_path_traversal_cost` on the same edges. That hop clock stays. This plan does not unify them.
