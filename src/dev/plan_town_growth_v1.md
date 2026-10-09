# PLAN-TOWN-GROWTH-v1 — drawable limits, generation placement, faction growth

| Field | Value |
|:---|:---|
| **Id** | **PLAN-TOWN-GROWTH-v1** |
| **Status** | **OPEN** — slices **TWN-OV1 → TWN-OV2 → TWN-GEN → TWN-AI** |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` → `@coder` |
| **Not this program** | Play-image town stamp. **PLN-3** in [`plan_egui_game_retirement_v1.md`](plan_egui_game_retirement_v1.md) already owns `raster_tile_fallback_subregion` dropping `stamp_town_centers_subregion` on the play image. Do not redo PLN-1..3. |

Modern industrial multi-state war. Logistics, power, industrial output, conflict. No castle, keep, or wall kit. No `castle_city_template`. No medieval set.

## Summary

`TownBook` stays the town authority. `DistrictBook` stays the only zone book. A town limit is a closed ring of tiles on the existing `DistrictRecord`, and membership is “tile is inside the ring.” World generation writes towns when the map asks for settlements, onto dry land, after water exists. A faction step reads resources, threat, and ally or enemy stance, then enqueues at most one existing growth site through `CommitConstructionSiteEvent`. Preview does not touch occupation or player pending rows.

```text
⊚TownBook ═▶ ⊨town records (center, faction, access, region name)
⊚DistrictBook ═▶ ⊨zone membership (ring interior, site_ids)
⊚WorldGenParams.settlement_count ═▶ ⊨“place towns” on the full-generation job
⊚generated_town_book_on_land ⊰ water catalog of that same job
⊚FactionStanceBook ═▶ ⊨pair stance (not an access policy)
⊚town_allows_entry ∧ town_allows_vehicle ∧ haul_crosses_town_flow ◂⊳[read] TownAccessPolicy
⊚PendingConstructionQueue growth: rows ⊰ GrowthProposal
⊚CommitConstructionSiteEvent ═▶ ⊨site create
⊚LogisticsGraph ═▶ ⊨haul topology (this program only reads it)
⊚sync_world_map_town_names_and_zone_shade ═▶ ⊨town Text2d and SHADE tint
```

## Already shipped — do not rebuild

| Piece | Where | Status |
|:---|:---|:---|
| Dry centers | `generated_town_book_on_land`, `nearest_dry_town_center`, `settle_centers_onto_dry`, `snap_generated_towns_off_water`, `GeneratedTownDrySnap` | **DONE** |
| Separation (four tiles) and region names | `towns_need_separation`, `spread_generated_centers`, `region_name_for_seed` | **DONE** |
| Access policy | `TownAccessPolicy` = MilitaryClosed, RestrictedLabor, FreeLabor. `town_allows_entry`, `town_allows_logistics_entry`, `town_allows_growth_commit`, `town_allows_vehicle` | **DONE** |
| Haul gate | `haul_crosses_town_flow` / `gate_haul_through_town` in `logistics_net.rs`. Reads the graph. Does not write `LogisticsEdge.disruption` | **DONE** |
| Rect zones | `add_admin_zone`, `place_site_in_zone`, `move_building_zone`, `commit_admin_zone_drag`, `AdminZoneDrag`. Same-town moves. Foreign town sites skipped | **DONE** (the commit path stays; the shape becomes a ring) |
| Town labels | `WorldMapTownLabel` in `sync_world_map_town_names_and_zone_shade` at `world_map_tile_center` | **DONE** |
| SHADE tint writer | `WorldMapZoneTint`, `ZoneShadeToggle`. HEIGHT stays on its writer | **DONE** (the mesh follows the ring; the writer does not move) |
| Growth prefix and funnel | `pending_blueprint_from_growth_proposal` labels `growth:…`. `growth_pending_commit_event` refuses any other label. `commit_growth_pending_to_funnel_system` drains only those rows | **DONE** |
| Water tokens | `building_may_occupy_water`: port / harbor / harbour / seaport, or oil rig. `land_building_blocked_on_water` | **DONE** |
| Non-empty book skip | `seed_settlement_books_if_empty` returns when a town is present. Save hydrate in `settlement_overlay.rs` loads a saved book. City fixture keeps `saved_towns` | **DONE** |
| One play map | Tile identity `world_map_tile_center`. Minimap is the GPU compositor | **DONE** |

## Current problems

Town limits are an inclusive rectangle (`DistrictRecord.tile_rect`, `contains_tile`, `commit_admin_zone_drag`). The play session also still stamps centers onto the map image. That stamp is **PLN-3**, not a slice here. Limits the player draws are a new shape on the same district row.

Town placement is a startup dump. `SettlementPlugin` calls `seed_settlement_books_if_empty`, which writes `generated_town_book(DEFAULT_WORLD_SEED, 4)` when the book is empty, then a later update snaps wet centers. `WorldGenParams` has no settlement request. `GenerateWorldEvent` / `world_gen_pipeline_tick` never places towns. The helpers are correct. They are not part of the generation procedure.

Faction growth is two one-shots: `enqueue_town_growth_once` and `enqueue_second_town_growth`. They do not read resources, threat, allies, or enemies. `TownRecord.faction` is a slot id. `Faction` has resources and cohesion. `ShotPulse` and `ChunkStrategicOverlay.threat` exist. There is no pair stance. `GrowthReasonCode` has no threat code. `growth_proposal_tick_system` scores shops from district pressure only.

### Why the three gates cannot say “help ally” versus “fight enemy”

Do **not** add a `TownAccessPolicy` variant.

- `town_allows_entry` knows the town owner and a visitor slot. `MilitaryClosed` admits only `visitor_faction == town.faction` with clearance. A different-faction ally is refused. `FreeLabor` admits everyone, including an enemy.
- `town_allows_vehicle` has no faction argument. `MilitaryClosed` admits any military vehicle with clearance 2.
- The haul gate calls `town_allows_vehicle` only.

Those three decide whether a known visitor and vehicle kind may cross. They do not store who is an ally. Stance is a separate pair lookup. Help and fight then call the three gates with the real visitor slot and the real vehicle kind. A false gate means no help enqueue.

## Target architecture

One play map. One `TownBook`. One `DistrictBook`. One growth commit funnel. One `LogisticsGraph` writer (unchanged, not this program). One SHADE writer. One town-label writer.

**Limit.** `DistrictRecord` gains a closed ring, three or more tile vertices, serde default empty. `contains_tile` uses the ring interior (boundary included, even-odd) when the ring is closed. An empty ring still uses `tile_rect`, so generated cores and old saves keep working until a ring is written. `tile_rect` becomes the axis-aligned bounds of the ring when a ring is committed. Site include and exclude stay `place_site_in_zone` and `move_building_zone`. Same town only. Foreign sites stay skipped. Deleting a limit removes that district row and drops its `site_ids`. It does not demolish buildings.

**Draw.** Extend `AdminZoneDrag` with the vertex list. No second drag resource and no second zone book. Pointer tiles stay `admin_zone_tile_from_world`. Close the ring, move a vertex, or delete the zone. Commit writes the ring through the same include/exclude rules as the rect drag.

**SHADE.** `sync_world_map_town_names_and_zone_shade` keeps spawning `WorldMapZoneTint`. The mesh is the ring interior when a ring exists, and the rectangle when it does not. `ZoneShadeToggle` stays the visibility writer. Town `Text2d` stays on the center tile.

**Generation.** `WorldGenParams.settlement_count: u32`, default **0**. Zero means the map did not ask: do not place, and do not treat that zero as `generated_town_count` (that helper clamps 0 to 1). On **full** generation, after the water catalog for that job exists, if the count is greater than zero **and** `TownBook.towns` is empty, call `generated_town_book_on_land(params.seed, count, is_water)` and `generated_districts_for_towns`. Each new district gets a four-corner ring matching today’s center ±8 rectangle, so the limit is a polygon from the start. `GeneratedTownDrySnap.pending` stays false. A non-empty book, including a hydrated save, is left alone. A later tick must not call `generated_town_book` again once any town is stored. `seed_settlement_books_if_empty` stops being the play-session writer of a default four-town book. Tests that want a book call `generated_town_book_on_land` or pass a saved book, which the city fixture already does.

**Stance.** New `FactionStanceBook` in `src/strategic/settlement/`: pairs of faction slots to Ally, Enemy, or Neutral. Missing pair is Neutral. This is not a town policy and not a zone book. No diplomacy screen in this program. Tests insert pairs.

**Faction step.** One system on the existing `SettlementPlugin` Update chain, after pressure and saturation, before `commit_growth_pending_to_funnel_system`. It reads `TownBook`, `DistrictBook`, `Faction.resources`, town jobs and housing, `UtilityGraph` (plant still feeds), `LogisticsGraph` (read), `HaulTrafficClass`, overlay threat, and `ShotPulseBook`. It enqueues **one** `GrowthProposal` with `GrowthActorLayer::Growth`, then the existing pending row and the existing commit event. Label stays `growth:`. Player rows stay in the queue. It does not write occupation, the logistics graph, pending player labels, or a second fire extract.

Choice, first match that is legal:

1. **Defend.** A threat cell or shot target within 16 tiles (Chebyshev) of the town center, and the anchor tile is inside the town ring and dry. Catalog token `warehouse` (`BuildingUsage::Industrial`). Reason `GrowthReasonCode::ThreatInRange` (add this one variant; the current codes cannot say threat).
2. **Help an ally.** Pair stance is Ally, the anchor is inside the ally ring, and both `town_allows_entry(town, helper_faction, clearance)` and `town_allows_vehicle(town.access, kind, clearance)` are true. The haul gate would also pass that kind. Catalog `warehouse` (logistics). An enemy pair does not get this enqueue. Neutral does not get this enqueue. A refused gate does not get this enqueue. Do not pretend the helper’s faction is the town owner.
3. **Need.** Housing below population, jobs below population, plant not feeding, or freight missing, using `HousingDeficit`, `EmploymentDemand`, `UtilityMissing`, `TransportHigh`. Land tokens stay `house` or `warehouse` through `archetype_for_zoning`. A water tile is legal only when `building_may_occupy_water` is true (`harbor` or `oil_rig`). Otherwise skip that tile.

No second site in the same step. The one-shot enqueue gates stay so they do not dump a second center site on top of a step that already committed that origin.

## ECS schedule

```text
Input (admin pointer) ═▶ AdminZoneDrag vertices
  ═▶ commit ring onto DistrictBook
  ═▶ SHADE reads DistrictBook (same writer)

Full world-gen job complete (water catalog written)
  ═▶ place towns iff settlement_count > 0 and TownBook is empty

Update, SettlementPlugin chain, after InfrastructureSiteSet::Planning
  rollup → pressure → saturation → growth_proposal_tick
  ═▶ faction growth step (one proposal)
  ═▶ commit_growth_pending_to_funnel_system
  ═▶ site systems consume CommitConstructionSiteEvent
```

Faction step reads `LogisticsGraph` and overlay threat. It is not a writer of either. It does not run in extraction.

## Implementation phases

Serial: **TWN-OV1 → TWN-OV2 → TWN-GEN → TWN-AI**. OV2 and GEN both need the ring field from OV1. AI needs the ring test and the generation book.

### TWN-OV1 — ring is the membership test

**Goal.** A closed polygon of tiles replaces the rectangle as the zone test, on the same district row.

**Files.** `src/strategic/settlement/district.rs`, `src/strategic/settlement/admin_ink.rs`.

**Change.** Add the ring on `DistrictRecord` (serde default empty). `contains_tile` uses interior-plus-boundary when the ring has at least three vertices, otherwise `tile_rect`. `commit_admin_zone_drag` grows a ring argument (or a sibling `commit_admin_zone_ring` in the same module that `commit_admin_zone_drag` calls). Inside sites of this town `place_site_in_zone` or `move_building_zone`. Outside sites leave this zone and stay on the town. Other towns are not scanned into the zone. `tile_rect` is rewritten to the ring bounds. `policy_at_tile` in `logistics_net.rs` already calls `contains_tile`; do not edit that function in this slice.

**Authority.** `DistrictBook` owns membership. `AdminZoneDrag` stays a preview. Commit is the only membership write.

**Must not change.** Do not add a second zone book. Do not demolish sites. Do not write `PendingConstructionQueue`. Do not edit `src/construction/mod.rs`. Do not delete `bevy_egui`. Do not touch the play-image stamp (PLN-3). Do not change `TownAccessPolicy`.

**Proof.** `cargo test -p proc_A_dine01 --lib twn_ov_polygon_membership`

### TWN-OV2 — draw, edit, delete, tint the ring

**Goal.** The player draws a closed ring, edits vertices, and deletes the limit. SHADE shows that ring.

**Files.** `src/construction/zones/input.rs`, `src/render/world_map_network_mesh.rs`.

**Change.** Pointer path appends tiles on the existing `AdminZoneDrag`, closes the ring, moves one vertex, and deletes by removing that district from `DistrictBook` (site ids dropped, buildings left in the world). Commit uses the TWN-OV1 ring commit. `sync_world_map_town_names_and_zone_shade` builds the tint mesh from the ring when present, and from `tile_rect` when not. Town names stay `WorldMapTownLabel`.

**Authority.** `ZoneShadeToggle` still decides visibility. This system remains the only `WorldMapZoneTint` spawner.

**Must not change.** Do not add a camera writer. Do not add a second map. Do not move HEIGHT. Do not retarget the minimap compositor. Do not call `stamp_town_centers_subregion`. Do not edit `src/construction/mod.rs`.

**Proof.** `cargo test -p proc_A_dine01 --lib twn_ov_draw_commit`

### TWN-GEN — towns are part of full generation

**Goal.** When full generation is asked for settlements, towns are placed then, on dry land, separated, named. A later tick does not re-roll a book that already has towns.

**Files.** `src/terrain/generation/world_generator_enhanced.rs`, `src/gui/editor/world_preview/world_gen_bevy_shell.rs` (one knob on the existing `WgKnob` list, writing `WorldGenParams` only), `src/strategic/settlement/town.rs`, `src/strategic/settlement/mod.rs` (startup system list only).

**Change.** `settlement_count` on `WorldGenParams`, default 0. After the full job’s water catalog is stored, if count > 0 and `TownBook.towns` is empty, place with `generated_town_book_on_land` using `params.seed` and that catalog, then `generated_districts_for_towns`, and write a four-corner ring for the existing ±8 square. Leave `GeneratedTownDrySnap.pending` false. If the book is non-empty, place nothing. Remove the play-path call that fills an empty book with `DEFAULT_WORLD_SEED` at startup. Keep the function for tests. Preview generation does not place towns.

**Authority.** The full-generation completion is the only writer of a generated book in play. Save hydrate remains the writer of a loaded book. They do not both run on a non-empty book.

**Must not change.** Do not replace a non-empty book. Do not clamp a request of 0 into one town. Do not add a second map or camera. Do not edit `src/construction/mod.rs`. Do not rebuild dry-snap, separation, or region-name helpers.

**Proof.** `cargo test -p proc_A_dine01 --lib twn_gen_places_on_dry_land`

### TWN-AI — one faction step, one growth site

**Goal.** Each step looks at resources, threat, allies, and enemies, then enqueues at most one existing growth site through the commit funnel.

**Files.** `src/strategic/settlement/actors.rs` (one `GrowthReasonCode::ThreatInRange`), `src/strategic/settlement/execute.rs`, `src/strategic/settlement/mod.rs` (chain insert only). New module `src/strategic/settlement/faction_step.rs`.

**Change.** `FactionStanceBook` as specified above. The step selects one legal action (defend, else help-ally when both gates pass, else a resource need on a legal tile). It enqueues one `GrowthProposal` and leaves the existing `commit_growth_pending_to_funnel_system` to emit `CommitConstructionSiteEvent`. Defensive sites only inside the 16-tile threat range. Help only when the town allows that visitor and that vehicle. Enemy and neutral pairs do not get a help site. Water tiles only for `harbor` or `oil_rig`.

**Authority.** The step writes `GrowthProposalQueue` and then a `growth:` pending row only. `commit_growth_pending_to_funnel_system` remains the commit writer. `LogisticsGraph` rebuild stays `rebuild_logistics_graph_from_transport`.

**Must not change.** Do not add a `TownAccessPolicy` variant. Do not write player pending rows (labels must keep the `growth:` prefix so `growth_pending_commit_event` ignores them). Do not write occupation from preview. Do not add a `LogisticsGraph` writer, a fire extract, or a camera writer. Do not add a castle or wall catalog token. Do not edit `src/construction/mod.rs`.

**Proof.** `cargo test -p proc_A_dine01 --lib twn_ai_one_growth_site`

## Diagnostics

No new witness file is required to start. The city launch `city_faction_progress` already keeps player rows and refuses a non-`growth:` label. TWN-AI’s test asserts one commit event, a `growth:` label, a refused help when `town_allows_vehicle` is false, and no defensive site when the threat is outside 16 tiles.

## Edge cases

- Saved non-empty book, then a full gen with `settlement_count > 0`: leave the book. Do not snap or re-seed.
- Request count 0: book stays empty until a later full gen asks.
- `generated_town_count(0) == 1` stays as it is. The request gate is outside that helper.
- Ring with fewer than three vertices: membership falls back to `tile_rect`.
- Self-crossing ring: even-odd, boundary included, same test for commit and for the haul tile gate.
- Delete while sites are members: sites leave the zone, entities stay.
- All generated towns are faction 1 today. Help does nothing until a test or a later writer inserts an Ally pair.
- Military-closed town and a foreign ally: gates return false, so no help site. That is the rule, not a new policy.
- Threat on water: the defensive `warehouse` is skipped for that tile. Do not switch it to a keep.

## Open questions

- Who writes `FactionStanceBook` in play after this program? Not this plan. Until then only tests insert pairs, and live pairs stay Neutral.
- Should the world-gen knob default stay 0 so current full gens do not gain towns until the operator sets a count? Yes. Call that out in the knob label so an empty book is expected.

## Handoff

`@coder` implements **TWN-OV1**, then **TWN-OV2**, then **TWN-GEN**, then **TWN-AI**. No slice writes `src/construction/mod.rs`. PLN-3 still owns the play-image stamp.
