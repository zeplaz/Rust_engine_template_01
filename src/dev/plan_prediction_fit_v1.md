# PLAN-PREDICTION-FIT-v1 — closed-form haul time, footprint search, arcs, worker prediction

| Field | Value |
|:---|:---|
| **Id** | **PLAN-PREDICTION-FIT-v1** |
| **Status** | **CLOSED as DEFER** — no `PRD-*` slices |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (this file only). `@coder` does not implement from it. |
| **Obeys** | `src/dev/plan_attrition_systems_fit_v1.md` **ATR-2** (threat copies into `EdgeFieldState.danger` inside `transport_field_integrate` only). `src/dev/plan_macro_ai_fit_v1.md` **MAI-1** / **MAI-3** (pure `sample_overlay_site` / `site_score`; one growth enqueue). |
| **Not this program** | Catapults, archers, grenades, castle walls, siege kits, arc solvers, worker entities, a 5,000-worker server, dead reckoning, a suitability book, a second edge-cost writer, a second Dijkstra, a traffic sim per footprint guess. |

Industrial war sim: power, hauls, plants, factions, towns. Projectile math was mapped onto those systems and checked against live code. Each mapping is **DEFER**. There is no `PRD-*` id.

```text
⊚sync_logistics_graph_from_transport ═▶ ⊨ LogisticsEdge
   traversal_cost ⊰ edge_traversal_cost(EdgeFieldState, weights, travel_time_base)
   disruption ⊰ damage, congestion, danger
⊚transport_field_integrate ═▶ ⊨ EdgeFieldState.danger
   today: decay only (danger -= 0.2 * dt). ATR-2 is the only allowed threat copy.
⊚transport_cost_cache_refresh ═▶ ⊨ TransportCostCache
   ⊰ travel_time_base (polyline length, floor 0.01) then edge_traversal_cost
⊚bfs_path_to_goals ⊰ TransportNavExport adjacency
   does not read NavExportEdge.cost
⊚logistics_path_dijkstra ═▶ cell-grid read only
   weight 0.5 * (cost_mul(a) + cost_mul(b)). Does not write LogisticsGraph.
⊚freight_transit_ticks ═▶ FreightLot.remaining_ticks
   hop count, or twice that on rail. Does not read traversal_cost or travel_time_base.
⊚commit_freight_arrivals_system ═▶ ⊨ SiteStagingStock credit (deployable tags)
   and destination ResourceFlowNode buffers (other tags)
⊚apply_deployable_staging_gate ⊰ SiteStagingStock + PendingSiteStagingDebits + InTransitLedger
   allows_commit stays false while the tag is still in transit or short of staged stock
⊚select_faction_growth ═▶ one Option<GrowthProposal>
⊚commit_growth_pending_to_funnel_system ═▶ ⊨ one CommitConstructionSiteEvent
⊚sample_overlay_site / site_score   planned MAI-1..3, not present in src/ yet
    precinct site_score is a pure scalar in runbook_rounds::city_planning
⊚shot_pulse::apply_arrived_shot_impacts_to_power_lines
   ═▶ damage_power_line_segment once, side SHOT_IMPACT_SIDE_FACTION = 1
   faction 2 does not match book_impact_for_faction(..., 1)
⊚TownRecord.population ═▶ a u32 count
   ReplicationInterestPolicy is a distance/weight stub. No worker replica.
```

Haul rows that paint stay `(u32, f32)`: `LogisticsVisualSnapshot.edge_rows` is `(edge_id, traffic)`. `haul_marker_t` lives on a separate `Vec<f32>`. `over_snapshot_keeps_haul_off_the_tuple` locks that split.

## Verdict

| # | Proposal | Verdict | Why |
|:---|:---|:---|:---|
| 1 | Closed-form arrival from edge cost (terrain or road class as drag) so a build waits until that delivery. | **DEFER** | An edge duration exists and is already used. It is not an unused clock. |
| 2 | Iterative mini-sims that slide a building footprint by simulating traffic on each guess. | **DEFER** | MAI-3 already plans the search: one `sample_overlay_site` / `site_score` per ring candidate, keep the winner. A traffic loop per guess is cut and is not a slice. |
| 3 | High-arc shots over friendly walls, skipping own-faction damage. | **DEFER** | Impact and the faction filter already exist. Faction 2 does not damage the faction-1 line. There is no wall and no arc. |
| 4 | Client-side dead reckoning of workers. | **DEFER** | No worker entity and no replication stream. `TownRecord.population` is a count. |

No `PRD-*` ids.

## 1 — arrival from edge cost — DEFER

`EdgeFieldState.travel_time_base` is the duration field. Hydration sets it to polyline length (`polyline_length`, floor `0.1`) in `src/systems/transport/snapshot.rs`. `transport_cost_cache_refresh` feeds `travel_time_base.max(0.01)` into `edge_traversal_cost`. `rebuild_logistics_graph_from_transport` does the same and stores the result on `LogisticsEdge.traversal_cost`. Capacity is `(2.0 / cost) * traffic_factor`. Danger, damage, congestion, and heat are already inside that cost. `CorridorClass` is a profile label (`Road`, `Rail`, `Maritime`, `Conveyor`, `Power`, `Pipeline`). It is not a second drag table, and nothing leaves it unread as a wait time.

The delivery clock that construction can see is `freight_transit_ticks` in `src/economy/logistics/propagation.rs`: `path_len` ticks, or `path_len * 2` when the mode is rail. `dispatch_freight_from_solver_system` writes that into `FreightLot.remaining_ticks`. It does not read `traversal_cost`, `travel_time_base`, or `CorridorClass`. While `remaining_ticks > 0`, `commit_freight_arrivals_system` does not credit `SiteStagingStock`. `apply_deployable_staging_gate` sets `allows_commit = false` and `stock_still_in_transit` when the tag is still on the ledger, and `insufficient_staged` when staged stock plus the pending debit queue are short. Preview does not write `TileOccupationBook` (`sweep_preview_move_does_not_change_occupation_or_pending`).

`haul_phase_after_cost` divides a visual phase by `traversal_cost` and is `#[cfg(test)]` only. The comment on it says the overlay inject does not read traversal cost. Promoting it into a build wait would start construction from a predicted phase. The gate above already waits on stock or the existing debit queue.

`logistics_path_dijkstra` (`src/systems/navigation/logistics_floodfill.rs`) is the cell-grid cost search. Freight reachability is the unweighted `bfs_path_to_goals` / `path_edges_between_tiles` (`src/economy/logistics/routes.rs`). The BFS never reads `NavExportEdge.cost`. The production writer of `LogisticsEdge` is `sync_logistics_graph_from_transport`. Fixture graphs in tests are not a second runtime writer.

A drag formula on top of `remaining_ticks` would be a second ETA writer. ATR-2 remains the only threat-cost change, and it writes `danger` inside `transport_field_integrate`, not a duration.

## 2 — footprint mini-sims — DEFER

`site_score` in `src/strategic/runbook_rounds.rs` is a pure scalar: logistics strength up, flood and threat down. `sample_overlay_site` is not in `src/` yet. **MAI-1** plans that function as one cell in, one scalar plus a forward/internal label out, with nothing stored. **MAI-3** plans the faction step calling it once per candidate inside the town ring and keeping the highest scalar (tie `(y, x)`). That step still returns one proposal. `select_faction_growth` already returns the first defend, help, or need hit. `twn_ai_one_growth_site` locks one `growth:` warehouse. `commit_growth_pending_to_funnel_system` is the commit. Preview occupation stays untouched until that event.

A loop that places a footprint, runs a haul search, and slides the footprint is a second search. It would write either the overlay or a suitability book. Both are rejected by MAI. This program does not narrow MAI-3 to a single cell, and it does not add the traffic loop beside it.

## 3 — high arc over friendly walls — DEFER

`shot_impact_at` exists. `apply_arrived_shot_impacts_to_power_lines` calls `damage_power_line_segment` once on the arrival tick when `book_impact_for_faction` is positive for `side`. The system passes `SHOT_IMPACT_SIDE_FACTION` (`1`). A shot whose `faction` is not that side is skipped. `link_other_faction_shot_does_not_damage_this_side` lands a faction-2 shot on a faction-1 line and leaves `PowerLineDamageBook` without that segment. `cont_other_faction_impact_stays_off_this_side` keeps `book_impact_for_faction(..., 1)` at `0` for a faction-2 shot while the faction-2 read stays positive.

Travel is not an arc. `shot_travel_ticks` is `0` when origin equals target, otherwise the constant `SHOT_TRACE_TICKS` (`8`). Range is Chebyshev tiles, cap `SHOT_MAX_RANGE` (`24`). There is no wall kit and no loft solver. The only gap that would have allowed a slice — faction 2 damaging faction 1 — is already closed.

## 4 — dead reckoning of workers — DEFER

`TownRecord.population` in `src/strategic/settlement/town.rs` is a `u32`. Housing and jobs compare against that count. There is no citizen component and no worker entity to extrapolate. `ReplicationInterestPolicy` in `src/systems/sim_frame_delta.rs` stores `max_tick_lag`, `distance_weight`, and `simulation_weight`. It does not replicate entities and it has no client prediction step. A dead-reckoning buffer would invent both the replica and the server.

## Must not change

- Do not add a field to `LogisticsEdge`. Tuple stays `(u32, f32)` on `edge_rows`.
- Do not let any system other than `sync_logistics_graph_from_transport` write `LogisticsEdge` in production.
- Do not let any system other than `transport_field_integrate` write `EdgeFieldState.danger`. ATR-2 owns the threat copy. Do not add a drag or duration writer beside it.
- Do not change `freight_transit_ticks` to read `traversal_cost`, `travel_time_base`, or `CorridorClass`.
- Do not start a site from a predicted tick. `allows_commit` stays on staged stock and `PendingSiteStagingDebits`. In-transit stock stays `stock_still_in_transit`.
- Do not add `SiteSuitabilityBook`, a placement search that writes `ChunkStrategicOverlay`, or a per-guess traffic simulation. MAI-1 stays one pure sample. MAI-3 stays the ring search.
- Do not enqueue more than one growth site per faction step. Preview does not mutate occupation.
- Do not add an arc solver, a wall kit, or a second damage path beside `damage_power_line_segment` from `shot_pulse`.
- Do not plan worker replicas or dead reckoning.
- No slice edits `src/construction/mod.rs` or `src/strategic/plugin.rs`. This program has no slice, so those files stay untouched by it.

## Existing locks (do not re-author as PRD tests)

| Lock | Filter |
|:---|:---|
| Haul phase stays off the `(u32, f32)` row | `cargo test -p proc_A_dine01 --lib over_snapshot_keeps_haul_off_the_tuple` |
| One staging debit caller | `cargo test -p proc_A_dine01 --lib site_staging_try_debit_has_one_caller` |
| One growth site | `cargo test -p proc_A_dine01 --lib twn_ai_one_growth_site` |
| Preview does not write occupation | `cargo test -p proc_A_dine01 --lib sweep_preview_move_does_not_change_occupation_or_pending` |
| Faction 2 does not damage the faction-1 line | `cargo test -p proc_A_dine01 --lib link_other_faction_shot_does_not_damage_this_side` |
| Faction 2 impact stays off faction 1 | `cargo test -p proc_A_dine01 --lib cont_other_faction_impact_stays_off_this_side` |

## Open questions

None. The FIT gate for a closed-form wait required an unused edge duration; `travel_time_base` is the cost base and `remaining_ticks` is the hop clock. The arc slice required a remaining friendly-fire gap; faction 2 already misses faction 1.
