# PLAN-MACRO-AI-FIT-v1 — sampled site score and plain need ranking

| Field | Value |
|:---|:---|
| **Id** | **PLAN-MACRO-AI-FIT-v1** |
| **Status** | **OPEN** — slices **MAI-1 → MAI-2 → MAI-3**, after **TWN-AI** is green |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` → `@coder` |
| **Not this program** | Town rings, generation, and the one growth enqueue ([`plan_town_growth_v1.md`](plan_town_growth_v1.md) TWN-OV1, TWN-OV2, TWN-GEN, TWN-AI). Play-image stamp and egui passes (PLN-1..3 in [`plan_egui_game_retirement_v1.md`](plan_egui_game_retirement_v1.md)). A stored suitability book is rejected below. GOAP, behavior trees, hierarchical pathfinding, and a blackboard stay **DEFER**. |

Modern industrial multi-state war. Logistics, power, industrial output, conflict, settlements. This plan only keeps Millington techniques that read authorities the sim already publishes, and only inside the narrow left by [`plan_macro_ai_counter_v1.md`](plan_macro_ai_counter_v1.md).

## Summary

The cell grid already exists on `ChunkStrategicOverlay`. `site_score` in `src/strategic/runbook_rounds.rs` (`city_planning`) already folds three scalars into one number. The frontline writer already marks contested chunks. The growth API is already one `growth:` proposal per faction step through `CommitConstructionSiteEvent`. What is missing is a pure call: score the candidate cell the growth step is about to use, and label that same cell forward or internal. The score is not stored. The label is not a grid.

```text
⊚ChunkStrategicOverlay ═▶ ⊨ cell control, threat, logistics_strength, fire_risk, civilian_stability, power_flow, artillery_danger
⊚site_score(logistics_strength, flood_risk, mean_threat) ⊰ one sampled cell
   fire_risk on that cell is the flood_risk argument (the overlay has no flood field)
   threat[slot] on that cell is the mean_threat argument
   logistics_strength[slot] on that cell is the logistics_strength argument
⊚sample_overlay_site ═▶ a scalar plus a forward|internal label for that cell only
   label ⊰ FrontlineState.contested_chunks ∨ (faction_control[slot] − faction_control[other])
   ⛔ SiteSuitabilityBook · refresh_site_suitability_system · plugin.rs
⊚derive_frontline_from_control_system ═▶ ⊨ FrontlineState (contested chunks only)
⊚world_read_snapshot_refresh_system ═▶ ⊨ WorldReadSnapshot
⊚strategic sim tick ═▶ ⊨ LogisticsAiRuntime (stockpile_fill_ratio, industrial_output_proxy)
⊚UtilityGraph ═▶ ⊨ power topology
⊚rebuild_logistics_graph_from_transport ═▶ ⊨ LogisticsGraph
⊚TransportGraph ∧ TransportEdgeDirectory ∧ TransportCostWeights ═▶ ⊨ corridor movement
⊚TownBook ═▶ ⊨ TownAccessPolicy
⊚town_allows_entry ∧ town_allows_vehicle ⊰ TownAccessPolicy
⊚faction growth step ═▶ ⊨ the one GrowthProposal (growth: label)
⊚commit_growth_pending_to_funnel_system ═▶ ⊨ CommitConstructionSiteEvent
⊚DistrictBook ═▶ ⊨ ring membership
```

## Fit table

| Technique | Existing authority it must read | Verdict | Why |
|:---|:---|:---|:---|
| Influence / territory (safety, resource density, threat, forward vs internal on the sampled cell, next-site score) | `ChunkStrategicOverlay` (`logistics_strength`, `threat`, `fire_risk`, `faction_control`). `site_score(logistics_strength, flood_risk, mean_threat)`. `FrontlineState.contested_chunks`. `DistrictBook` ring is the membership test. | **FIT** | MAI-1 calls `site_score` on one cell and attaches a label. It does not allocate a map. |
| Stored influence heatmap (`SiteSuitabilityBook`, one record per overlay cell, `refresh_site_suitability_system`) | Same overlay fields, already published. | **REJECTED** | [`plan_macro_ai_counter_v1.md`](plan_macro_ai_counter_v1.md) **MAI-CUT-INFLUENCE** and **MAI-CUT-FRONT**. Not a slice. |
| GOAP / STRIPS build orders | `GrowthProposalQueue`, `growth:` pending rows, `commit_growth_pending_to_funnel_system`, `CommitConstructionSiteEvent`. Also `BuildOrderQueue` → `ApprovedBuildOrders` (bounds check only). | **DEFER** | A precondition search would be a second planner beside the one growth enqueue. `BuildOrderQueue` is already a separate intent path and does not commit sites. |
| Plain need ranking (housing, storage, power) | `TownRecord` housing, jobs, population. `LogisticsAiRuntime.stockpile_fill_ratio` and `industrial_output_proxy`. `ChunkStrategicOverlay.power_flow`, `threat`, `artillery_danger`. `UtilityGraph` (plant still feeds). `GrowthReasonCode` (`HousingDeficit`, `UtilityMissing`, `TransportHigh`, plus TWN-AI `ThreatInRange`). Catalog tokens already used by growth: `house`, `warehouse`, and water-only `harbor` / `oil_rig`. | **FIT** | MAI-2 is `rank_town_need` with plain weights. Defense stays the TWN-AI arm (`warehouse` when threat is within 16 tiles). Curve RON/JSON and a utility-AI crate stay out (**MAI-CUT-UTILITY**). |
| Behavior trees / FSMs for workers | `TownRecord.population` is a count. `Agent` / `Faction` in `behavior_entities.rs` are narrative cohesion, not crews. | **DEFER** | No citizen, crew, or worker entity exists. Do not invent a population. |
| Hierarchical pathfinding | `TransportGraph`, `TransportEdgeDirectory`, `TransportCostWeights`, `CorridorClass` (road, rail, maritime, conveyor, power, pipeline). `LogisticsGraph` is rebuilt by `rebuild_logistics_graph_from_transport`. `systems/navigation/potental_feild_nav.rs` is local steering. | **DEFER** | The corridor graph is the macro path. A second A* is not required. |
| Blackboard | `WorldReadSnapshot`, `OperationalTheaterSummary`, `ShotPulseBook`, `FrontlineState`, `LogisticsAiRuntime`, `Faction.resources`, `TownAccessPolicy`, `UtilityGraph`. | **DEFER** | Those resources already publish threat, stock, and policy. A new singleton board would be a second copy. |

## Already shipped or owned elsewhere — do not rebuild

| Piece | Where | Status |
|:---|:---|:---|
| Dry centers, separation, region names | `generated_town_book_on_land`, `settle_centers_onto_dry` | **DONE** |
| `TownAccessPolicy` (MilitaryClosed, RestrictedLabor, FreeLabor), `town_allows_entry`, `town_allows_vehicle`, haul gate | `src/strategic/settlement/`, `logistics_net.rs` | **DONE** |
| Rect admin zones, town `Text2d`, SHADE tint | settlement ink, `sync_world_map_town_names_and_zone_shade` | **DONE** — ring shape is TWN-OV1 / TWN-OV2 |
| Polygon membership, draw/delete, generation placement, one growth site | TWN-OV1, TWN-OV2, TWN-GEN, TWN-AI | **OWNED ELSEWHERE** — this program calls that API |
| `growth:` prefix, `CommitConstructionSiteEvent` funnel | `execute.rs` | **DONE** |
| Water occupancy | `building_may_occupy_water`, `land_building_blocked_on_water` | **DONE** — call them, do not edit them |
| Contested-chunk frontline | `derive_frontline_from_control_system` → `FrontlineState.contested_chunks` | **DONE** — the label reads this list. Do not replace it with a zero-crossing book |
| Hint scalar `site_score` | `strategic_city_planning_hints_tick` → `CityPlanningHints` | **DONE** — that writer stays the only writer of the hints resource. MAI-1 calls the function. It does not retune the weights |
| Egui passes, fire spark tile lock, play-image town stamp | PLN-1..3 | **OWNED ELSEWHERE** |
| Logistics graph rebuild | `sync_logistics_graph_from_transport` | **DONE** — read only |

`ChunkFieldCell` / `WorldFieldLayerEpoch` are not a second live grid. The sample reads `ChunkStrategicOverlay` only, through `site_score`.

## Current problems

`FrontlineState.contested_chunks` is a chunk list where slot 0 and slot 1 are both above a tie band. It is the forward label for a cell the step already chose to look at. It is not a per-cell map to allocate.

`site_score` folds logistics strength, flood risk, and threat into `CityPlanningHints.last_best_site_score` for the settlement tick and the HUD. The growth step does not call it when it picks the one anchor inside the town ring.

TWN-AI’s faction step (once landed) picks defend, else help-ally when both town gates pass, else a resource need, and enqueues one site. The need arm is first-match. It does not rank housing against storage against power from stockpile fill, industrial output, and power flow. Defense stays that first arm.

`BuildOrderQueue` drains into `ApprovedBuildOrders` when the tile is inside the world. That queue is not the growth funnel. Growing it into a planner would fork construction intent.

## Target architecture

One pure sample. One pure need ranking. The existing faction step remains the only growth enqueue. The existing commit system remains the only growth commit. No new resource.

**Sample.** A function reads one overlay cell the caller names. Friendly slot is the town’s faction slot. The other slot is the remaining compared slot (frontline already compares slot 0 and slot 1). On that cell only:

- `site_score(logistics_strength[slot], fire_risk, threat[slot])`. `fire_risk` is passed as the `flood_risk` argument. There is no flood field on `ChunkStrategicOverlay`.
- `friendly_minus_enemy` = `faction_control[slot] - faction_control[other]` on that cell. It is a number in the return value, not a buffer.
- The label is `Forward` when the cell’s chunk is in `FrontlineState.contested_chunks`, or when `friendly_minus_enemy` is at or below 0. Otherwise the label is `Internal`.
- The function does not walk 4-neighbors, does not record a sign-change map, and does not write `ChunkStrategicOverlay`, `FrontlineState`, or `CityPlanningHints`.

**Need ranking.** A pure function, no resource of its own. It ranks the need arm only. The TWN-AI defend arm (threat or shot within 16 tiles → `warehouse`, `ThreatInRange`) and the help-ally arm stay in front of it, with the same gates. Defense is that existing arm, scored from the cell `threat` and `artillery_danger` the step already reads. This program does not add a second defense picker and does not add a reason-code variant. Weights are plain scalars. There is no curve asset and no utility crate.

Inputs the caller passes: housing, population, `stockpile_fill_ratio`, `industrial_output_proxy`, mean `power_flow` on ring cells, and whether `UtilityGraph` still feeds the town. Outputs are one existing token and one existing reason:

- Housing rises as housing falls below population → `house`, `HousingDeficit`. Population 0 scores 0.
- Storage rises as stockpile fill falls. High `industrial_output_proxy` with low fill raises the same score → `warehouse`, `TransportHigh`.
- Power: low mean `power_flow`, or a plant that does not feed, raises `warehouse` with `UtilityMissing`. It does not add a plant token.

Ties break in a fixed order: housing, then storage, then power. Equal scores do not roll a seed. If every score is at or below zero, the function returns none and the step enqueues nothing.

**Wire.** The faction step still enqueues at most one proposal. Defend (threat or shot within 16 tiles, anchor inside the ring, dry, `warehouse`) still wins. Help-ally still wins only when `town_allows_entry` and `town_allows_vehicle` both pass, and the enemy and neutral pairs still do not. Otherwise `rank_town_need` picks the token. The anchor is the ring cell for which `sample_overlay_site` returns the highest scalar. The step may call that function once per candidate inside the ring and keep the winner only (tile, scalar, label). It does not retain a grid. Tie `(y, x)`. Water stays illegal unless `building_may_occupy_water` is true for `harbor` or `oil_rig`. Label stays `growth:`. The forward/internal label rides on that one proposal as a name for the chosen cell. It is not written back onto the overlay.

## Rejected — not a slice

`SiteSuitabilityBook` and `refresh_site_suitability_system`, registered in `src/strategic/plugin.rs`, one record per overlay cell (`friendly_minus_enemy`, `frontline`, `suitability`, length `len_cells()`), including a 4-neighbor sign-change zero crossing stored on that book. Rejected by [`plan_macro_ai_counter_v1.md`](plan_macro_ai_counter_v1.md) **MAI-CUT-INFLUENCE** and **MAI-CUT-FRONT**. A second grid of overlay numbers drifts from `ChunkStrategicOverlay`. Do not schedule it. Do not add the system to `StrategicFieldPipeline`.

## Implementation phases

Serial: **TWN-AI green → MAI-1 → MAI-2 → MAI-3**. MAI-1 does not call the growth step and does not register a system. MAI-2 does not register a system. MAI-3 is the only slice that touches the step, and only after `twn_ai_one_growth_site` is green.

### MAI-1 — sample one overlay cell through `site_score`

**Goal.** A pure function scores one cell with `site_score(logistics_strength, flood_risk, mean_threat)` and returns a forward or internal label for that cell. No resource. No system. The growth step is unchanged.

**Files.** `src/strategic/site_sample.rs` (new). Declare the module from `src/strategic/mod.rs` and export the function only. Do not edit `src/strategic/plugin.rs`.

**Change.** `sample_overlay_site(...)` takes the caller’s scalars for one cell: `logistics_strength`, `fire_risk` (passed as `flood_risk`), `threat` (passed as `mean_threat`), the two `faction_control` slots, and whether that chunk is in `FrontlineState.contested_chunks`. It returns `site_score(...)` plus `Forward` or `Internal`. `friendly_minus_enemy` is computed on those two slot values for the label. Nothing is stored. `CityPlanningHints` is not written. Weights inside `site_score` stay as they are in `runbook_rounds.rs`.

**Authority.** No new writer. `ChunkStrategicOverlay` field writers stay where they are (`apply_zones_to_strategic_overlays_system`, logistics inject, network flow, the sim tick for `civilian_stability`). `derive_frontline_from_control_system` stays the only writer of `FrontlineState`. `strategic_city_planning_hints_tick` stays the only writer of `CityPlanningHints`.

**Must not change.** Do not edit `src/strategic/plugin.rs`, `src/strategic/settlement/`, `src/construction/`, or `src/strategic/frontline.rs`. Do not add a resource, a `SystemSet` entry, or a `ChunkFieldCell` buffer. Do not write `LogisticsGraph`, occupation, `fire_risk`, or call `set_faction_threat`. Do not read egui. No unseeded randomness. Do not allocate a suitability map.

**Proof.** `cargo test -p proc_A_dine01 --lib mai_sample_site_score`

### MAI-2 — need curves from stock, power, and output

**Goal.** A pure ranking from resources the sim already stores, returning one existing token and one existing reason code.

**Files.** `src/strategic/settlement/need_utility.rs` (new). No schedule edit. No edit to `faction_step.rs`, `actors.rs`, `execute.rs`, or `mod.rs`.

**Change.** `rank_town_need(...)` reads numbers the caller passes (housing, population, stockpile fill, industrial output, mean power flow, plant-feeds, ring threat, ring artillery). It returns `house` or `warehouse` plus `HousingDeficit`, `TransportHigh`, or `UtilityMissing`. Fixed tie order. If every score is at or below a zero threshold, it returns none.

**Authority.** No resource. The caller in MAI-3 remains the writer of `GrowthProposalQueue`.

**Must not change.** Do not add a reason-code variant. Do not add a catalog token. Do not write `LogisticsAiRuntime`, `UtilityGraph`, or `CityPlanningHints`. Do not edit `src/construction/`. Do not enqueue.

**Proof.** `cargo test -p proc_A_dine01 --lib mai_need_utility_from_stock`

### MAI-3 — the one growth step calls the sample and the ranking

**Goal.** After TWN-AI, the existing step picks the anchor by calling `sample_overlay_site` inside the town ring and the need-branch token from `rank_town_need`, and still enqueues one `growth:` site. It does not read a suitability book.

**Files.** `src/strategic/settlement/faction_step.rs` only.

**Change.** Defend and help stay first, with the same gates and the 16-tile threat test. The need arm calls `rank_town_need`. For each candidate tile inside the ring the step calls `sample_overlay_site` and keeps the highest scalar. Tie `(y, x)`. The retained label is `Forward` or `Internal` on that winning cell only. A land token on water is skipped via the existing `building_may_occupy_water` / `land_building_blocked_on_water`. `harbor` or `oil_rig` only when that predicate is true. Help still requires both entry and vehicle gates. The proposal still uses `GrowthActorLayer::Growth` and the `growth:` label. `commit_growth_pending_to_funnel_system` still emits `CommitConstructionSiteEvent`. There is no `SiteSuitabilityBook` read.

**Authority.** The faction step remains the only writer of that one growth proposal. The commit system remains the only growth commit. `sample_overlay_site` and `rank_town_need` stay pure.

**Must not change.** Do not add a `TownAccessPolicy` variant. Do not write player pending rows. Do not write occupation from preview. Do not write `LogisticsGraph`, fire extract, the camera, or `DistrictBook`. Do not edit `src/construction/mod.rs` or `src/strategic/plugin.rs`. Do not add a second proposal in the same step. Do not retarget the one-shot enqueue gates. Do not insert a suitability resource.

**Proof.** `cargo test -p proc_A_dine01 --lib mai_one_site_uses_sample`

## ECS schedule

MAI-1 and MAI-2 add no system. The schedule is unchanged.

```text
StrategicFieldPipeline::ZoneAndReadModel
  apply_zones_to_strategic_overlays_system
  ═▶ derive_frontline_from_control_system
  ═▶ world_read_snapshot_refresh_system
  ═▶ process_build_order_queue_system         unchanged, not growth

InfrastructureSiteSet::Planning
  ═▶ commit_construction_site_system

SettlementPlugin Update .after(Planning) .chain()
  rollup → pressure → saturation → growth_proposal_tick
  ═▶ faction growth step                       TWN-AI, then MAI-3 calls the pure functions
  ═▶ commit_growth_pending_to_funnel_system
  ═▶ site systems consume CommitConstructionSiteEvent
```

The sample does not run in extraction. The step does not run in extraction. `refresh_site_suitability_system` is not inserted.

## Diagnostics

No new witness file. MAI-1 asserts the same three scalars twice return the same `site_score`, a chunk listed in `contested_chunks` labels `Forward`, and a cell whose friendly slot leads the other slot outside that list labels `Internal`. No book is compared. MAI-2 asserts low stockpile fill picks `warehouse` over `house` when housing is full, and a full stockpile with housing below population picks `house`. MAI-3 asserts one `growth:` commit, the anchor is the higher `site_score` cell inside the ring, a refused help when `town_allows_vehicle` is false, and no land token on a water cell.

## Edge cases

- No other control slot: pass enemy control as 0. The label is `Internal` unless the chunk is in `contested_chunks`.
- Several enemy slots: the caller passes the compared other slot (frontline uses slot 0 and slot 1). The function does not sum slots and does not scan the overlay.
- Missing overlay cell: the step skips that candidate. There is no book entry to omit.
- Empty ring: membership stays `tile_rect` (TWN-OV1). The sample does not invent a ring.
- Every ring cell illegal for the token (water for a land token): the need arm enqueues nothing.
- `FrontlineState` absent in a test: pass `chunk_is_contested = false` and label from the two control values only.
- `BuildOrderQueue` may still approve a `StructureType` row. This program does not read or drain that queue.
- Player pending rows stay queued because the label prefix filter is unchanged.

## Left for later — still DEFER

These stay **DEFER** in the fit table. [`plan_macro_ai_counter_v1.md`](plan_macro_ai_counter_v1.md) already cuts them. This program does not schedule them.

- A GOAP or STRIPS planner, goal stack, or action schema with preconditions and effects.
- Any second build queue, including growing `BuildOrderQueue`, `ApprovedBuildOrders`, or `StructureType` (`Fortification`, `CommandPost`, `FireSupport`, `SensorTower`) into the commit funnel.
- Behavior trees, worker FSMs, citizen entities, construction crews, or a population of agents. `Agent` and `Faction` stay the narrative cohesion components they are.
- Hierarchical pathfinding, a navmesh, or a second search over `TransportGraph` / `LogisticsGraph`. Local potential-field steering stays local.
- A blackboard singleton, a copy of `WorldReadSnapshot`, or a new policy map beside `TownAccessPolicy`.
- Castle vocabulary: keeps, walls as a kit, knights, quarries, sawmills, timber, granaries, “flee to town hall”, `castle_city_template`, and `SettlementArchetype::MilitaryFortress` as a build order. That archetype remains a hint label inside `CityPlanningHints` only.
- A second fire extract, a second `LogisticsGraph` writer, a second camera, a second `DistrictBook`.
- Replacing polygon rings, town labels, SHADE, or the one growth enqueue.
- PLN-1..3 (egui plugin passes, fire spark tile lock, play-image town stamp).
- Curve assets (RON or JSON) and a utility-AI crate. MAI-2 weights stay plain numbers in the function.

## Open questions

- Who writes faction stance pairs in play is still the open question on the town plan. MAI-1 does not read `FactionStanceBook`. Until a writer exists, the caller passes the other slot as 0 when no pair is known, so `friendly_minus_enemy` is just the friendly slot and the label follows `contested_chunks` when that list is non-empty.
- `WorldReadSnapshot` does not gain a suitability mean. There is no book to copy. The snapshot writer stays `world_read_snapshot_refresh_system`.

## Handoff

`@coder` waits until `cargo test -p proc_A_dine01 --lib twn_ai_one_growth_site` is green, then implements **MAI-1**, then **MAI-2**, then **MAI-3**. No slice writes `src/construction/mod.rs` or `src/strategic/plugin.rs`. `SiteSuitabilityBook` stays rejected.
