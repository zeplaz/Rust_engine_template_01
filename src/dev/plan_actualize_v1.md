# Actualize queue — post-MAI execution

| | |
|:---|:---|
| **Status** | **QUEUE** — launch only after MAI-1, MAI-2, MAI-3 return |
| **Owner** | `@planner` (read-only) → parent launches `@coder` waves below |
| **Worktree** | `C:\dev\github\Rust_engine_template_01` · `master` only |
| **Not this file** | Do not edit Rust from this plan. Do not commit. Do not edit `src/dev/plan_macro_ai_fit_v1.md`. Do not redo an in-flight or shipped slice. |

## Summary

This is the execution queue for the parent that starts when the in-flight macro-AI coder returns. `⊚sync_logistics_graph_from_transport` stays the only writer of `LogisticsGraph` (`07-repo-authority-map`, LOG-A-01, `STR-TBR`). `⊚transport_field_integrate` stays the only writer of `EdgeFieldState.danger`. `⊚update_electrical_load_system` stays the only writer of `ElectricalComponent.current_load` in the shed. `⊚solve_throughput_greedy_system` stays the only solver writer. `⊚faction_growth_step_system` stays a reader of the logistics graph and the only growth enqueue. `⊚FireVisualFrame` (`FireVisualFrameSet` in `src/render/extraction/fire_visual_extract.rs`) stays the only fire extract. `⊚MainWorldCamera` stays the only play camera. Play tile identity stays `world_map_tile_center` (`CO-UXO` is the engine state gate; footprints stay on `src/construction/world_map_frame.rs`, which this queue does not edit).

Wave A is file-disjoint and may run as parallel coders. Wave B is one coder, after every Wave A slice is green and after MAI has left `faction_step.rs`. Order inside Wave B puts the danger write and the rebuild fold before the idle skip, the faction locks, and the stock refuse.

## In flight — do not schedule

| Id | Files the other coder owns | Status |
|:---|:---|:---|
| **MAI-1** | `src/strategic/site_sample.rs` (module declare may also touch `src/strategic/mod.rs`) | **IN FLIGHT** |
| **MAI-2** | `src/strategic/settlement/need_utility.rs` | **IN FLIGHT** |
| **MAI-3** | `src/strategic/settlement/faction_step.rs` | **IN FLIGHT** |

No slice in this queue edits those three files until MAI returns. Wave B is the first writer of `faction_step.rs` after that return. Do not retarget MAI’s sample or need ranking.

## Dropped — shipped, done, or refused

Checked on disk 2026-09-27. A symbol that already did the slice’s change is **SHIPPED** and is not queued.

| Source | Ids | Verdict |
|:---|:---|:---|
| `plan_town_growth_v1.md` | TWN-OV1, TWN-OV2, TWN-GEN, TWN-AI | **DONE**. Do not redo. The town plan file still says OPEN; this queue follows the reconciled status. |
| `plan_prediction_fit_v1.md` | all four mappings | **DEFER**. No `PRD-*` slice. Do not schedule. |
| `plan_prediction_counter_v1.md` | PRD-CUT-CART, PRD-CUT-PLACE, PRD-CUT-ARC, PRD-CUT-NET | **REFUSED** as features. Remainders are only PRD-CUT-OWN and the conditional PRD-CUT-PATH. |
| `plan_egui_game_retirement_v1.md` | EGR-0..EGR-6, UPM-1..5, city-bug pass | **SHIPPED / DONE**. Not in this queue. |
| PLN-1 symbol check | `omit_egui_pointer_preupdate_in_simulation` in `src/engine/states.rs` gates only `EguiPreUpdateSet::ProcessInput` | **NOT SHIPPED** — BeginPass / EndPass / ProcessOutput / PostProcessOutput are still ungated. Queue PLN-1. |
| PLN-2 symbol check | `src/render/fire_vfx/emit.rs` still feeds `cam.zoom_level` into `px_per_tile` | **NOT SHIPPED**. Queue PLN-2. |
| PLN-3 symbol check | `raster_tile_fallback_subregion` still calls `stamp_town_centers_subregion` for `state.image` | **NOT SHIPPED**. Queue PLN-3. |

## Current problems (only what this queue still changes)

- Play still runs bevy_egui EndPass, which can paint and capture above the Bevy shell (`CO-UXO`).
- Fire sparks still offset from extract `world_xyz` using `px_per_tile`, not `world_map_tile_center`.
- The play sprite still stamps town centers beside `WorldMapTownLabel`.
- `update_electrical_load_system` only clamps `current_load`. Overload does not shed by catalog role.
- `transport_field_integrate` only decays `danger`. `rebuild_logistics_graph_from_transport` sets disruption from `(damage + congestion * 0.45 + danger * 0.25)` and does not call `disruption_after_cell_heat`. The route walk does not skip a logistics edge at disruption `>= 1`.
- `solve_throughput_greedy_system` clears `FreightReservationBook` on every economy tick.
- `apply_arrived_shot_impacts_to_power_lines` continues when `shot.faction != side`, then still calls `damage_power_line_segment` when `shot.faction == side` (`SHOT_IMPACT_SIDE_FACTION` = 1).
- Faction perception and camera/LOD locks are not tests yet. They are not new entities.
- Nothing sums stored edge cost before the growth enqueue, and MAI’s need arm (as written) refuses a zero score or an illegal cell, not a missing `ResourceStorage`.

## Target architecture

```text
⊚omit_egui_pointer_preupdate_in_simulation ═▶ ⊨ play egui set gates (InitContexts stays)
⊚FireVisualFrame ═▶ ⊨ extract rows
   emit sparks ◂⊳[row] ⊚FireVisualFrame   place via world_map_tile_center
⊚WorldMapTownLabel ═▶ ⊨ play town mark
   play sprite stamp ⛔
⊚update_electrical_load_system ═▶ ⊨ ElectricalComponent.current_load
   ⊰ ElectricalGrid totals already written   ⛔ UtilityConnection.connected
⊚transport_field_integrate ═▶ ⊨ EdgeFieldState.danger
   ⊰ ChunkStrategicOverlay.threat slot 0
⊚sync_logistics_graph_from_transport ═▶ ⊨ LogisticsGraph
   rebuild folds danger through disruption_after_cell_heat   ⛔ overlay sample inside the rebuild
⊚solve_throughput_greedy_system ═▶ ⊨ load and reservations
   idle return ◂⊳[fingerprint] path_open + max_rate + topology_revision
⊚faction_growth_step_system ═▶ ⊨ one GrowthProposalQueue row
   ◂⊳[snapshot] LogisticsGraph, UtilityGraph, RoutePathStore, ResourceStorage
   ECS-1/ECS-2 add no system
⊚damage_power_line_segment ═▶ ⊨ PowerLineDamageBook
   shot system stops calling it for faction == side and faction != side
```

No new `SystemSet`. No second logistics writer, camera, or fire extract. `src/construction/mod.rs` is not the sole owner of any slice here, so no slice edits it.

```text
Input (plugin sets already gated)
  ═▶ Wave A parallel, disjoint files
  ═▶ Wave B serial, one coder
        ATR-2 danger + rebuild fold + route skip
        ═▶ ATR-3 idle skip (path_open in the fingerprint)
        ═▶ ECS-1 ═▶ ECS-2     tests in faction_step.rs only
        ═▶ PRD-CUT-PATH       only if MAI did not already refuse on missing stock
```

## Wave A — parallel coders

No two slices share a file. No slice writes `faction_step.rs` or `rebuild_logistics_graph_from_transport`. No intra-wave dependency. ATR-1 does not assume an edge close.

### PLN-1 — stop the play egui plugin passes

**Files.** `src/engine/states.rs`. Proof edit: `src/gui/hud/shell_diagnostics.rs` (`egr_simulation_schedule_omits_drained_egui_systems` only).

**Change.** Extend `omit_egui_pointer_preupdate_in_simulation`, which already takes `&mut App` and is called from both engines. `configure_sets` so `EguiPreUpdateSet::BeginPass`, `EguiPostUpdateSet::EndPass`, `EguiPostUpdateSet::ProcessOutput`, and `EguiPostUpdateSet::PostProcessOutput` run only when the state is not `BaseState::Simulation`. `OnEnter(BaseState::Simulation)` clears `EguiFullOutput` on the primary context, sets `EguiGlobalSettings.enable_cursor_icon_updates` false, and sets `EguiContextSettings.capture_pointer_input` false. `OnExit(BaseState::Simulation)` restores both defaults (`true`). `capture_pointer_input_system` has no set of its own; the settings flag is the cut. Leave `EguiPreUpdateSet::InitContexts` and `EguiPreUpdateSet::ProcessInput` as they are (`ProcessInput` is already skipped). Extend the existing omit test so it asserts those four set gates and the enter/exit cursor and capture flags. The test’s `include_str` of `construction/mod.rs` stays a read.

**Must not change.** Do not remove the `bevy_egui` crate. Do not gate Editor or WorldGen. Do not move `simulation_tool_pointer`. Do not edit `src/construction/mod.rs`.

**Filter.** `cargo test -p proc_A_dine01 --lib egr_simulation_schedule_omits_drained_egui_systems`

### PLN-2 — fire sparks on the same tile

**Files.** `src/render/fire_vfx/emit.rs`

**Change.** Each spark that passes the existing heat floor (`CHUNK_FIRE_OVERLAY_DISPLAY_MIN`, 0.12) is placed with `world_map_tile_center` of the cell taken from the row’s horizontal position, then a scatter offset in that cell, with z on the map plane (`PLANE_Z` in `world_map_network_mesh`, 1.04). Scatter count and `shape_fire_row_for_particle_lod` stop taking `px_per_tile`. The row still comes from the existing `FireVisualFrame` inside `emit_world_fire_particles_from_projection`. `fire_particle_view_proj` already publishes `ExtractedCameraMetrics.view_proj`, which is `MainWorldCamera` when that camera is present.

**Must not change.** Do not add a second fire extract. Do not edit `gpu_fire_particle_raster.rs`, `src/render/fire_vfx/witness.rs`, or the wgsl (`zoom_fade` is already 1.0). Do not write `SharedOverlayFieldBuffers`. Do not touch the minimap boost. Do not edit `construction/mod.rs` or `world_map_frame.rs`. Calling `world_map_tile_center` is a read.

**Filter.** `cargo test -p proc_A_dine01 --lib fire_vfx::emit`

### PLN-3 — town mark is the Text2d only

**Files.** `src/render/tile_world_fallback.rs`

**Change.** The play call of `raster_tile_fallback_subregion` on `state.image` stops stamping town centers (`stamp_town_centers_subregion`). The town picture in Simulation stays `WorldMapTownLabel`. The call under `raster_policy.cpu_minimap_pass` on `state.minimap_image` still stamps. Leave `stamp_town_centers_subregion` defined in `src/gui/map_tile_raster.rs` so its unit tests still compile. Add `pln_play_image_does_not_stamp_town_centers` beside the raster tests in this file.

**Must not change.** CPU minimap image when `cpu_minimap_pass` is on. `LogisticsVisualSnapshot.edge_rows`. Commit funnel. `MinimapCompositor`. HEIGHT and SHADE writers. Do not edit `construction/mod.rs` or `map_tile_raster.rs`.

**Filter.** `cargo test -p proc_A_dine01 --lib pln_play_image_does_not_stamp_town_centers`

### ATR-1 — drop current_load by catalog role when the grid is already overloaded

**Files.** `src/entities/production/power/systems.rs`

**Change.** `update_electrical_load_system` already clamps `current_load` to `max_transfer` and is already chained after `recalculate_grid_totals_from_members` and `emit_grid_overload_signals`. Do not add a system or a resource. Do not edit `src/infrastructure/utility/activation_link.rs` or `failure_modes.rs`. When `ElectricalGrid.total_load > total_capacity` is already true and `total_capacity > 0` (the same predicate `emit_grid_overload_signals` uses), drop `ElectricalComponent.current_load` to 0 in catalog-role order and stop once the remaining loads would no longer exceed capacity. Order, resolved with `IndustrialSupplyChainRole::resolve` and `UtilityInfrastructureRole::resolve` against `BuildingDefinitionRegistry`: `AluminumFabrication` and `AluminumSmelter` first; then `ConcreteMixer`, `CementKiln`, `AggregateMine`, `BauxiteMine`; `UtilityInfrastructureRole::PowerPlant` last. Ties inside one role drop the higher `Entity` id. Catalog id is the one already on the entity (`PlannedSite.catalog_id` when present). `IndustrialSupplyChainMembership.role` may be read only as the role `insert_supply_chain_runtime` already stored. Do not copy a role onto `UtilityGraph` or `UtilityGraphNode`. The clamp to `max_transfer` stays after the drop. This system does not write `ElectricalGrid.total_load`. `AluminaRefinery`, `IntegratedPlant`, `Transformer`, and `Substation` get no rank. New test `overload_sheds_fabrication_before_mine` lives in this file: overload already true, fabrication and smelter drop before mixer, kiln, and mine, and `PowerPlant` drops last.

**Must not change.** `circuit_open`, `load_fraction`, `preview_island_offline_from_cut`, `compute_island_partition`, `advance_power_repair_queue`, `haul_flow_while_plant_feeds`, `merge_utility_power_into_infrastructure_graph`, `apply_utility_power_with_line_load`, `sync_utility_connection_power_system`. Do not set `UtilityConnection.connected` false. Do not make the flood one-directional. Do not add a shed resource, a priority asset, a utility curve, or a military weight. Do not write `src/construction/`, `src/strategic/plugin.rs`, or `src/strategic/settlement/`. `grid_open_feeders_stop_at_the_plant` stays green and is not this filter.

**Filter.** `cargo test -p proc_A_dine01 --lib overload_sheds_fabrication_before_mine`

### PRD-CUT-OWN — refuse own-faction line damage

**Files.** `src/strategic/shot_pulse.rs`

**Change.** In `apply_arrived_shot_impacts_to_power_lines`, keep the `shot.faction != side` continue so faction 2 still does not damage faction 1. Also refuse `damage_power_line_segment` when `shot.faction == side`. `side` stays `SHOT_IMPACT_SIDE_FACTION` (1). After both refuses this function does not damage lines. Do not add a trajectory, a wall test, or a weapon. `shot_trace_tile`, `shot_travel_ticks`, and `SHOT_LINE_DAMAGE` stay. Update `link_shot_impact_on_a_line_reduces_health` so a faction-1 shot leaves `PowerLineDamageBook` without a segment. `link_other_faction_shot_does_not_damage_this_side` stays.

**Must not change.** Do not edit `src/construction/mod.rs` or `damage_power_line_segment` itself. Do not let faction 2 open a segment. Do not add a catapult, lob, or wall-clearance test.

**Filter.** `cargo test -p proc_A_dine01 --lib strategic::shot_pulse`

## Wave B — one coder, after Wave A and after MAI

One coder. Do not start until MAI has stopped writing `faction_step.rs` and every Wave A filter above is green. Serial order is the dependency order: the danger write and the rebuild fold happen before any skip or enqueue that assumes a closed edge.

### ATR-2 — threat raises danger; the rebuild closes the edge; BFS stays BFS

**Files.** `src/systems/transport/mod.rs` (`transport_field_integrate`). `src/strategic/transport_bridge.rs` (`rebuild_logistics_graph_from_transport` only; `sync_logistics_graph_from_transport` stays the caller and the sole `LogisticsGraph` writer). `src/economy/logistics/routes.rs` (the route walk in `refresh_resource_flow_routes_system`, which already has `LogisticsGraph`). Call `disruption_after_cell_heat` in `src/strategic/logistics_net.rs`. Do not edit that file.

**Change.** `transport_field_integrate` reads `ChunkStrategicOverlay` and `TransportEdgeDirectory`. Anchor cells are the directory head and tail keys, the same parse the rebuild already uses. Slot 0 only. Sample `threat[cell][0]` at the two anchors (max of the two) and write `danger = max(sampled, decayed danger)`. Decay stays `0.2 * dt` toward zero when the sample is lower. Congestion, damage, and heat decay stay. Do not write `LogisticsEdge` here. The rebuild does not take a threat lookup and does not read the overlay. It already writes disruption as `(damage + congestion * 0.45 + danger * 0.25)` clamped to `[0, 1]`. After that term, set `disruption = disruption_after_cell_heat(disruption, danger)`. Finite danger `>= 1` therefore yields disruption `1`. Non-finite danger does not raise disruption. `traversal_cost` stays `edge_traversal_cost`. In `refresh_resource_flow_routes_system`, skip a transport edge whose logistics disruption is `>= 1` before the existing BFS, so `path_edges_between_tiles` can return the other open edge between the same tiles. BFS stays BFS. `path_open` on that closed edge becomes false through the existing reachability assignment. Throughput on the closed edge stays zero through the existing `open_flow`. New test `threat_drops_the_open_edge`: two edges between the same tiles, threat 1 on one anchor copied into `danger`, rebuild disruption `>= 1` on that edge, the walk returns the other edge.

**Must not change.** Do not sample `ChunkStrategicOverlay` inside `rebuild_logistics_graph_from_transport` or `sync_logistics_graph_from_transport`. Do not add a flow field, a grid potential, a second Dijkstra, or HPA*. Do not replace `logistics_path_dijkstra`. Do not let any system other than `transport_field_integrate` write `EdgeFieldState.danger`. Do not let any system other than `sync_logistics_graph_from_transport` write `LogisticsEdge`. Do not write `threat` or call `set_faction_threat`. `feedback_congestion_from_load_system` stays congestion-only. Do not edit `src/strategic/plugin.rs` or `faction_step.rs`. Do not retarget the faction growth step. `grid_islanded_plant_stops_the_haul` must still pass and is not this filter. `NavExportEdge` stays `{ id, cost, successors, allowed_agents, profile }`. Haul rows stay `(u32, f32)`.

**Filter.** `cargo test -p proc_A_dine01 --lib threat_drops_the_open_edge`

### ATR-3 — solver returns when the tick did not change

**Files.** `src/economy/logistics/solver.rs`. `src/economy/logistics/types.rs` only to store the last-solved revision and the `path_open` / `max_rate` fingerprint on the existing `ThroughputSolverState`. `src/economy/logistics/tests.rs` for `solver_skips_an_unchanged_revision`.

**Change.** At the start of `solve_throughput_greedy_system`, if `solver.topology_revision` equals the revision stored from the previous solve and every `ResourceFlowEdge.max_rate` and `path_open` match that snapshot, return without `book.clear()`. The snapshot is fields on `ThroughputSolverState`. It is not a new resource and not a graph store. Record the fingerprint only after a real solve. A changed rate, a changed `path_open`, or a changed topology still solves. A threat close from ATR-2 that flips `path_open` is a changed tick and must still solve. `sync_solver_capacity_from_graph_system` still copies capacity when the graph changed. Do not add a timer, a frame counter, or a new set.

**Must not change.** Do not skip a tick whose revision, `path_open`, or rates changed. Do not store `LogisticsNodeId` on entities. Do not weaken `soa_solver_aligned` or `reservations_within_capacity`. Do not edit `src/strategic/plugin.rs` or `src/economy/logistics/mod.rs`. Do not edit `routes.rs` (ATR-2 already owns the skip).

**Filter.** `cargo test -p proc_A_dine01 --lib solver_skips_an_unchanged_revision`

### ECS-1 — lock the perception read against the logistics and power writers

**Files.** `src/strategic/settlement/faction_step.rs` (`#[cfg(test)]` only).

**Change.** Add `ecs_perception_does_not_write_logistics`. This is a test. It does not register a system and does not spawn a squad, unit, or stimulus bus. Build the same minimal app as `twn_ai_one_growth_site` (one town, one overlay cell with threat, one `Faction`, the player `pending` row, `CommitConstructionSiteEvent`). Also insert a `LogisticsGraph` with `revision` 7, one node, one edge (`capacity` 4, `disruption` 1, `traversal_cost` 2) and a `UtilityGraph` with one node and one `power_edges` row (`throughput` 3). Run one `Update` of `faction_growth_step_system` then `commit_growth_pending_to_funnel_system`. Assert logistics `revision`, node count, edge count, `capacity`, `disruption`, and `traversal_cost` are unchanged; utility node count, `power_edges` length, and `throughput` are unchanged; commit count is 1 and the player `pending` row remains; the test never calls `add_message` for a new stimulus type. If MAI-3’s sample and rank calls are already in the step, leave them. The test app must still satisfy whatever inputs that landed step reads so the one commit still happens. If the test fails because the step writes an edge, stop. Do not add a second writer.

**Must not change.** `sync_logistics_graph_from_transport`, `world_read_snapshot_refresh_system`, `perceive_world`, `PowerRuntimePlugin`, `src/strategic/plugin.rs`, `src/construction/mod.rs`, `src/strategic/settlement/mod.rs`, `site_sample.rs`, `need_utility.rs`, occupation, preview. Do not pass `WorldIntentField` into `FactionGrowthLook`.

**Filter.** `cargo test -p proc_A_dine01 --lib ecs_perception_does_not_write_logistics`

### ECS-2 — lock the faction step to the sim tick

**Files.** `src/strategic/settlement/faction_step.rs` (`#[cfg(test)]` only).

**Change.** Add `ecs_faction_step_ignores_camera_lod`. Same app as ECS-1, plus `ChunkSimLod::Dormant` on the overlay entity. Do not insert a camera, `ViewManager`, or viewport resource. One `Update`. Assert commit count is 1. If the test also spawns the faction kernel `Agent` from `behavior_entities.rs`, that `AgentMode` is unchanged. The step must not remove or replace that component. Do not spawn `systems::agents::permissions::Agent`. No new entity kind. If the commit disappears whenever `ChunkSimLod::Dormant` is present, the step has gained a LOD gate: remove that gate. Do not replace it with a random roll.

**Must not change.** `ChunkSimLod::dt_scale`, `chunk_sim_lod_refresh`, `strategic_fields_coupling_tick`’s dormant cell skip, `faction_growth_step_system`’s parameters (no new view or LOD argument), `src/strategic/plugin.rs`, `src/strategic/settlement/mod.rs`, `src/construction/mod.rs`. Do not add `run_if` on the settlement chain.

**Filter.** `cargo test -p proc_A_dine01 --lib ecs_faction_step_ignores_camera_lod`

### PRD-CUT-PATH — refuse enqueue when the stored path or stock is missing

**Gate.** Run only after MAI-3 has landed in `faction_step.rs`. Read the step. MAI-2/MAI-3, as planned, return none from `rank_town_need` when every score is at or below zero, and enqueue nothing when every ring cell is illegal for the token. That is not a stock refuse. If the landed step already returns before `queue.enqueue` when the world sum of `ResourceStorage.amounts` is missing, non-finite, or not `> 0`, **drop this slice**. Do not add a second refuse.

**Files.** `src/economy/logistics/routes.rs` (pure function only; do not reopen the ATR-2 skip). `src/strategic/settlement/faction_step.rs` (the enqueue gate, and the tests that expect a commit).

**Change.** Add `sum_path_traversal_cost`. Inputs: the edge-id slice already in `RoutePathStore::edge_slice` (the path `path_edges_between_tiles` stored) and a lookup of `TransportCostCache.by_edge` or `NavExportEdge.cost`. Return `Option<f32>`: `None` if the slice is empty or any id is missing or non-finite; otherwise the sum. It does not write `LogisticsGraph`. It does not call `bfs_path_to_goals`, `path_edges_between_tiles`, or `logistics_path_dijkstra`. It does not read `Time` or `delta_seconds`. `faction_growth_step_system` calls it on a path the route cache already holds. Before `queue.enqueue`: if the sum is `None`, or the present sum of `ResourceStorage.amounts` is not finite and `> 0`, return. Do not enqueue. Do not call `approve_growth_proposal_into_pending`. There is no site price. `freight_transit_ticks` stays the hop clock. `LogisticsNode.stockpile` stays unread and unwritten (`rebuild` still writes it `0.0`). Prove the two-cost sum and the missing-id `None` from the faction-step test so this slice has one filter. Any existing test that expects `CommitTally` 1, including ECS-1 and ECS-2, must insert a stored path whose costs sum finite and a `ResourceStorage` whose amounts sum `> 0`, or the assertion becomes tally 0 and the player row stays alone. A run with no `RoutePathStore` path does not enqueue.

**Must not change.** Do not change `freight_transit_ticks` into a cost sum. Do not call `approve_growth_proposal_into_pending` because a duration is small. Do not write `LogisticsGraph`, `LogisticsNode.stockpile`, or `LogisticsEdge.traversal_cost` from the growth step. Do not retarget MAI’s one-site sample or `rank_town_need`. Do not edit `src/construction/mod.rs`.

**Filter.** `cargo test -p proc_A_dine01 --lib strategic::settlement::faction_step`

## ECS schedule

No new set. Existing order, unchanged:

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
Settlement Update, after InfrastructureSiteSet::Planning, already chained:
  faction_growth_step_system
  ═▶ commit_growth_pending_to_funnel_system
Shot impact system stays the existing system. The compare refuses both sides.
```

## Diagnostics

No new witness file. Proof is the one filter on each slice.

- PLN-1: four set gates and Simulation enter/exit cursor and capture flags.
- PLN-2: spark placement on the map plane from `FireVisualFrame` rows.
- PLN-3: play image has no town stamp; CPU minimap stamp remains when the pass is on.
- ATR-1: `current_load` after the shed when `total_load > total_capacity`. `UtilityConnection.connected` unchanged.
- ATR-2: `EdgeFieldState.danger` after the threat copy, then `LogisticsEdge.disruption` after `disruption_after_cell_heat`. The walk returns the other edge.
- ATR-3: fingerprint includes `path_open`. `FreightReservationBook` length unchanged on the idle tick. A flipped `path_open` still solves.
- ECS-1: graph fields unchanged, commit count 1.
- ECS-2: commit count 1 with no view resource and `ChunkSimLod::Dormant`.
- PRD-CUT-OWN: faction 1 and faction 2 leave `PowerLineDamageBook` without a new segment.
- PRD-CUT-PATH, if not dropped: no enqueue without a finite path sum and a positive stock sum.

## Edge cases

- Empty `LogisticsGraph`: ECS-1 uses a non-empty graph. The empty live step stays as it is.
- Threat on slot 0 only. Other faction slots do not raise `danger`.
- Sampled threat of 0 leaves the existing danger decay. Sampled threat of 1 holds `danger` at 1 against that decay.
- Graph rebuild replaces node ids. Do not cache `LogisticsNodeId` on a facility.
- `ResourceStorage` is not keyed by town. The PATH refuse uses the world sum of `amounts`.
- `freight_transit_ticks` will still disagree with `sum_path_traversal_cost`. The hop clock stays.
- Preview does not write occupation. Construction still commits only through `CommitConstructionSiteEvent`.
- One fire extract. ATR-2 calls the existing heat function on `danger` inside the rebuild. It does not add a second heat reader that writes fire.
- Off-screen chunks still run the faction step. `ChunkSimLod::Dormant` may still skip coupling cell writes. Those are different writers.

## Do not build

A coder refuses these. They are not slices.

- Commander, squad, unit, `SquadObjective`, `UnitOrder`, formation buffers, vision cones, soldier entities.
- HTN, behavior tree, GOAP, blackboard, utility-curve assets, suitability book, flow field, HPA*, min-cut, max-flow, a new graph store, entity-stored `LogisticsNodeId`.
- Catapult, archer tower, lob, wall-clearance, gravity, drag, a ballistic cart, or an ODE for haul arrival. Dead reckoning, client extrapolation, a prediction buffer, harvester replicas.
- Deleting `bevy_egui`. A second logistics writer, a second camera, a second fire extract.
- `src/construction/mod.rs` in any slice in this queue. None of them is its only owner.
- PRD-CUT-CART, PRD-CUT-PLACE, PRD-CUT-ARC, PRD-CUT-NET, and the four prediction-fit mappings, as features.
- TWN-OV1, TWN-OV2, TWN-GEN, TWN-AI. MAI-1, MAI-2, MAI-3 (in flight; do not redo).
- A power agent, a shed that sets `UtilityConnection.connected` false, a role copied onto `UtilityGraph`, or an overlay sample inside `rebuild_logistics_graph_from_transport`.
- A second growth proposal, a retarget of the one growth site, or `Time.delta_seconds()` on the path sum or the stock refuse.
- Calling `logistics_path_dijkstra` or `bfs_path_to_goals` again from `sum_path_traversal_cost` or from `faction_growth_step_system`.

## Open questions

- Register this file on `src/dev/development_plan_index.md` through `@plan-orchestrator`. This file does not edit the index.
- PRD-CUT-PATH stays conditional until the landed MAI-3 step is read. A zero need score is not the stock refuse.
- `AluminaRefinery`, `IntegratedPlant`, `Transformer`, and `Substation` stay out of the ATR-1 shed order.
