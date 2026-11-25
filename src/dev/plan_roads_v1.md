# PLAN-ROADS-v1 — roads and rail that freight uses: tiers, motorways, worn tracks, congestion, real travel time

Status: **OPEN** (2026-11-20, RD-0/1/3/4/6 + bridges + B/R items in code; see Status, Open items, Round (c) and Next at the end). Owner lanes: `coder` (sim/logistics/construction), `designer` (build picker, road card, ribbons).
Supersedes the road half of T-8 in [`plan_close_voxel_water_v1.md`](plan_close_voxel_water_v1.md) (clickable roads and
bridge decks DONE there; this plan makes roads and rail matter).

## Product stance (operator 2026-11-20)

- Logistics still moves off-road. A road is a faster, more reliable way, never the only way.
- Repeated off-road traffic wears **tracks** into the ground on its own; what wears them and how much depends on the unit.
- The player and the AI **build quality roads**: tiers (dirt, gravel, paved, concrete) and a **motorway / express** tier
  (autobahn, 401, interstate) whose real-world effect is modelled: high speed and capacity because access is limited
  to interchanges and crossings are grade-separated.
- **Congestion matters.** Every corridor has a capacity; traffic above it slows everything on it.
- **Rail is on par with roads**: same network model, same build/upgrade, same congestion and damage rules; a train is the
  rail adapter of the same kernel (big batched loads, station to station), not a separate system.
- Travel time comes from the route actually driven. No straight-line timing.
- There is **no money** in the game (checked 2026-11-20): the economy runs on goods and labour (`StockLedger`, cargo such as
  Gravel, Cement, Concrete, Labour, Diesel). Roads and rail cost **materials + labour + build time**, delivered like any site,
  and cost **diesel / power to run**.

## Where it stands (capture 2026-11-20, dam scene, `roads` dump section)

| Item | Finding |
|------|---------|
| Click picks | Bridge answers the 2D segment pick and the 3D deck ray (1/1 each) |
| Trucks on roads | **0 of 56**; 878 deliveries, all cross-country |
| Why | Only road = one 16-tile bridge at row 110; every truck leg runs rows 140..200; road join range is 24 tiles |
| Road graph | A committed road is **one edge head→tail**, no interior nodes: a truck can only join at the two ends |
| Road storage | `ExecutedRoadNetwork.tiles` is one flat list; readers pair consecutive tiles, so road A's end + road B's start = a phantom segment |
| Road types | `RoadType { Street, Highway }` in the build tool; `corridor_profiles.example.ron` has lanes/speed/surface tags, read only by spline geometry |
| Leg timing | `orders/travel.rs`: straight line × detour 1.3 ÷ speed; ignores road, terrain, slope, water, traffic |
| Off-road cost | Cross-country path grid (`logistics_floodfill`) has no terrain-speed or wetness term |
| Congestion | None |
| Worn tracks | None |
| AI roads | No faction AI road or rail building |

### Rail review (2026-11-20)

| Area | Has | Gap |
|------|-----|-----|
| Placement | `src/construction/rail/` (~500 lines): spline path, grade and curve validation, continuous placement, ghost | — |
| Junctions | `RailJunctionAuthority` registers a switch where rail meets a road tile | No level-crossing behaviour (trucks/trains don't interact), no rail-rail junction routing |
| Network | `ConstructionType::RailSegment` commits a transport edge (`CorridorClass::Rail`, agent `rail_train`) | Same one-edge-per-segment and flat `ExecutedRoadNetwork` lookups as roads |
| Profiles | `corridor_profiles.example.ron`: gauge, electrification, tracks, max speed | Read by nothing that moves freight |
| Trains | — | **No train exists.** Order-sim `Vehicle` is truck-only (speed, capacity, no mode) |
| Stations / yards | — | **None**: rail cannot load or unload at any site, so rail can carry nothing |
| Timing | Old propagation model: `Batched` ETA = 2 × path length | Order sim ignores rail entirely |
| Capacity / signals | — | No block signalling, no track capacity, no single vs double track effect |
| Look | 2D infrastructure overlay marker | Voxel view draws roads (`pixel_world/roads.rs`) but not rail; no rail bridges |
| Damage / water | — | Rail not blocked by flood/rubble books, no blast damage |
| AI | — | AI never builds rail |
| Intermodal | — | No truck → train → truck transfer |

Rail today is a placement tool and graph edges with nothing running on them. Roads are one step further (trucks exist and can
follow them). Both share the same structural gaps, so they are fixed **once, in a shared corridor kernel** (AGENTS.md
anti-clone rule), with mode adapters.

## Authority (AGENTS.md rules)

| Domain | Sole writer | Readers | Requests in |
|--------|-------------|---------|-------------|
| Built corridor network (road + rail segments, tier/profile, lanes/tracks, deck, condition) | construction execute funnel (`src/construction/`) | transport graph, render, cards, AI | `ConstructionIntent` (build / upgrade / remove) |
| Wear field (per-cell wear, emergent tracks) | `TrackWearField` owner system (transport) | transport graph, render, road card | `TrackWearQueue` (truck motion, formation motion push cells + unit wear class) |
| Off-road speed field (land class, slope, wetness, ford depth) | one publisher system (navigation) | route planner, formations | — (reads `WaterTileProbe`, ground delta, rain) |
| Corridor flow + congestion (vehicles per edge per hour) | transport owner | route planner, cards | `CorridorFlowQueue` (planned legs register their edges) |
| Transport graph + costs | transport owner | route planner, order sim | — |
| Leg route + leg time | route planner (shared kernel, per-mode adapter) | order sim, presentation | — |

The order sim never computes distance itself. It reads the leg time the planner publishes. Presentation reads both, writes neither.

## Shared corridor kernel + mode adapters

One kernel: segments → graph edges with nodes along them → costed route (time = length ÷ effective speed, effective speed =
profile speed × condition × weather × congestion) → published leg plan. Adapters say only what differs:

| | Truck (road adapter) | Train (rail adapter) |
|---|---|---|
| Ends of a leg | door to door; off-road on/off-ramp to nearest allowed node | station / yard to station; truck legs to and from the yard |
| Joins the network | any node of a non-limited-access road; motorway only at interchanges | stations only |
| Capacity unit | vehicles/h per lane | trains/h per track (block length) |
| Load | one truck load | consist of N wagons (batched, large) |
| Off network | yes (slow) | never |
| Energy | diesel | diesel or grid power (electrified profile) |

## Road tiers (RON `assets/config/infrastructure/road_tiers.ron`, first pass; tuning lives in the RON)

| Tier | Speed km/h | Lanes | Capacity veh/h/lane | Access | Wet speed × | Floods closed at | Build per 100 m |
|------|-----------:|------:|------:|--------|-----:|------|-----------------|
| Track (worn, not built) | 25 | 1 | 40 | any | 0.4 (mud) | 0.3 m | — |
| Dirt | 35 | 1–2 | 60 | any | 0.5 | 0.3 m | labour |
| Gravel | 55 | 2 | 120 | any | 0.85 | 0.5 m | gravel + labour |
| Paved | 80 | 2–4 | 300 | any | 1.0 | 0.5 m | gravel + asphalt + labour |
| Concrete | 90 | 2–6 | 400 | any | 1.0 | 0.6 m | gravel + concrete + labour |
| **Motorway / express** | 110 | 4–8 (divided) | 900 | **interchanges only** | 1.0 | 0.8 m | gravel + concrete + steel (interchanges, overpasses) + labour |

Motorway rules (the IRL effect):
- No at-grade junctions: crossing roads and rail get an overpass/underpass; trucks enter and leave only at interchanges.
- That is why it is fast and high-capacity: no cross traffic, divided carriageways. The cost is the interchanges, the
  grade separations and the detour to the nearest interchange.
- Junctions on other tiers add a delay per crossing (level crossings with rail add a train-priority delay).

Rail profiles (same RON): single / double track, gauge, diesel / electrified, speed, trains/h per track, build per 100 m
(gravel ballast + steel + sleepers + labour; electrified adds power line).

## Congestion

- Each planned leg registers its edges in `CorridorFlowQueue`; the transport owner keeps a rolling vehicles-per-hour per edge.
- Effective speed = free speed ÷ (1 + 0.15 × (flow / capacity)^4) (BPR curve, standard traffic-engineering form), floor at
  a crawl; rail uses block occupancy (a train waits for a free block instead).
- Route choice uses the congested time, so traffic spreads onto parallel roads; road card and dump show volume / capacity.
- Military convoys and refugees later feed the same flow book.

## Wear by unit (the "who makes tracks" choice)

Different movers mark the ground and roads differently. One wear queue, a wear class per mover:

| Mover | Off-road | On built roads |
|-------|----------|----------------|
| Truck (wheeled) | wears a track after repeated passes | normal |
| Tracked armour (mechanized formations) | wears fast, churns mud, flattens young stands | damages dirt/gravel/paved over time (raises repair need) |
| Infantry | light footpath wear only | none |
| Trains | — | rail wear only (maintenance) |

Formation movement already exists in combat; it pushes cells to the same `TrackWearQueue` with its class.

## Phases

### RD-0 Corridor data model (fixes the flat list, roads + rail)

- Replace `ExecutedRoadNetwork.tiles` with `CorridorNetwork { segments: Vec<CorridorSegment> }`; `CorridorSegment { id, kind: Road(tier, lanes) | Rail(profile, tracks), polyline, deck, condition }`.
  Undo/redo, snap, upgrade, rail-crossing and junction readers move over; the phantom segment goes away.
- Tier/profile RON as above. Old saves/R8 snapshots migrate from the tile list.

### RD-1 Graph edges joinable along the way

- A commit becomes a chain of edges with nodes every ≤ 8 tiles and at every junction/crossing (crossings joined; motorway
  crossings become grade separations, no node). Edge cost = time with the current congestion and condition.
- Route planner (truck adapter) joins at the nearest allowed node, on-ramp cost = off-road time. Picks by total time vs all off-road.
- Done when: a truck whose leg runs parallel to a road 6 tiles away takes the road; one with the road 60 tiles away does not; a motorway is only entered at an interchange.

### RD-2 Off-road speed field

- Base speed by land class (grass, forest, rubble, crater, rock) × slope (DEM + ground delta) × wetness × ford depth (`WaterTileProbe`; ≤ 0.5 m slow, deeper = wall).
- Cross-country grid and formations both read it. Rubble/craters block roads (D-4) and are slow off-road.

### RD-3 Leg time from the route driven

- Planner publishes `LegPlan { polyline, time_s, mode_share }`; order sim `travel_ticks` reads it (straight-line fallback only until a new leg is planned). Network change re-times the remaining leg.
- Done when: a paved detour 1.4× the straight line beats off-road on time; deliveries/hour rise when a road is built.

### RD-4 Congestion

- `CorridorFlowQueue` + per-edge flow book, BPR slow-down, congested time in route choice, volume/capacity on the card and in the dump.
- Done when: 50 trucks on one dirt road visibly slow and some take a parallel road.

### RD-5 Worn tracks (by unit class)

- Wear queue from trucks and formations with their wear class; decay without traffic; wear above a threshold over a connected run → Track segment (graph edges, worn-ground look).
- Tracked armour damages road condition; rain turns tracks to mud.
- A track is not construction; the player or AI can upgrade it to a built tier.
- Done when: in a long run with no built roads, tracks form along depot → site lines and leg times on them drop.

### RD-6 Placement tool + build/upgrade (player)

Operator 2026-11-20: the rail placement tool is "very janky"; placing roads and rail must be logical, easy, clean and
responsive, enjoyable rather than annoying.

What makes today's tools janky (review 2026-11-20, `src/construction/roads/input.rs`, `src/construction/rail/input.rs`):

| Problem | Where |
|---------|-------|
| The preview only follows the cursor while **Ctrl is held**; otherwise you click blind | `update_*_path_preview_system` (`ctrl_preview`) |
| Controls are unusual: LMB adds a point, RMB removes the last one, **Shift+LMB** commits; no Enter/Esc/double-click finish | `*_path_input_system` |
| One bad segment blocks the whole path with a generic "invalid rail segment (grade or curve)"; no where or what to do | `sync_rail_path_build_preview` |
| Grade is checked on straight chords between clicked points over the raw terrain, so any hill fails; no cut/fill, bridge or tunnel offered | `rail/validation.rs` |
| Rail snaps with the **road** snap settings and to road tiles, not to rail ends or along track | `snap_placement(.., roads)` |
| Roads and rail are two near-copies of the same tool (anti-clone rule) | `roads/` vs `rail/` |

One corridor placement tool (shared kernel, the mode adapter supplies width, min curve radius, max grade, snap rules):

- **Always-live preview.** From the first click the path follows the cursor, drawn on the ground (2D and voxel 3D) at its real width, with curves fitted to the mode's minimum radius.
- **Plain controls.** Click start → click to add a bend → **double-click, Enter, or click an existing road/rail node to finish**; **Esc** cancels; **Backspace / RMB** removes the last point; **Shift** locks straight / 15° angles; drag a placed bend to move it before committing. Same keys for every corridor type, listed in the HUD hint line and rebindable in options.
- **Route assist.** Optional "follow terrain" mode: pick start and end, the tool proposes a path that keeps within the grade (slope-cost path on the DEM), and you drag it where you want.
- **Per-section feedback, not one pass/fail.** Each stretch is coloured: green buildable; amber buildable with works (cut, fill, bridge, tunnel, level crossing, overpass) labelled on the stretch; red impossible, with the reason at the cursor ("grade 7% > rail max 3%", "curve 40 m < min 300 m"). Commit builds what is valid; amber works are part of the site.
- **Auto works.** Water → bridge deck; short steep stretch → cut/fill within limits; crossing a road → junction (road), level crossing or bridge (rail), overpass (motorway).
- **Snapping that helps.** Snap to existing ends and anywhere along a segment (makes a junction), to parallel offsets (double track, divided motorway), to grid angles with Shift; snap target highlighted before you click.
- **Live readout at the cursor.** Length, materials and labour, build time, and the travel speed you will get.
- **Upgrade / remove.** Click a segment to upgrade tier or tracks; drag along a run to upgrade it all; bulldoze tool for removal. Undo/redo per commit (exists).
- **Responsive.** Preview update ≤ 2 ms per frame; long auto-routes solve incrementally, never a hitch; the 3D pick is the voxel ray everywhere (no ortho fallback in 3D).
- Build picker: road tier + lanes, motorway (interchange placement), rail profile + tracks (play UI only).
- A road/rail build is a construction site with a material manifest delivered by trucks; it opens when done (sections open as they finish).
- Road / rail card: tier, lanes/tracks, speed now, volume/capacity, condition, trucks/trains routed.
- Measure: a `PlaceCorridor` scenario step drives the same tool code (start, bends, finish) so captures check the commit, works and junctions; frame cost of the preview in the dump; then an operator hands-on round for feel.
- Done when: placing a 100-tile road with two bends over a river and a hill takes four clicks, shows the bridge and cut before commit, and never hitches.

### RD-7 Rail runs freight (parity)

- Stations / freight yards as a site archetype (loading capacity, platform length); a site within reach of a yard can ship by rail.
- Train vehicle in the order sim (rail adapter): consist size, speed by profile, batched loads, block signalling, single-track passing loops.
- Intermodal legs: truck → yard → train → yard → truck, planner picks rail when the total time and load win.
- Voxel view draws rail (ballast, sleepers, rails, rail bridges), trains on the track.
- Floods and rubble block rail; blasts cut it; level crossings delay trucks.
- Done when: a yard pair 150+ tiles apart moves a bulk cargo (gravel) faster per unit than trucks, and blocking the line reroutes to road.

**RD-7 slices (2026-11-24):**

| Slice | What | Where |
|---|---|---|
| 7a trains in the order sim | `VehicleMode::Rail { wagons }` (vehicles default Road); `AddTrain`; rail leg times from a `SiteRailLegBook` (yard ↔ yard over rail edges, `rail_train` agent, rail speed); trains only serve yard → yard orders and win the pick when their time per unit beats the best truck; load / unload scale with wagons | `economy/logistics/orders/{model,sim,travel}.rs`, rail planner beside the truck planner |
| 7b freight yards | `SiteRole::Yard`; start-config `yards` (tile, trains, wagons) and a buildable `freight_yard_v1` that becomes a Yard site when built; a yard joins the rail node within reach | `ai/faction/start.rs`, `orders/plugin.rs`, building catalog |
| 7c intermodal | a long truck order whose ends both have a yard within reach, with rail between them and a faster total time, is replaced by truck → yard, train yard → yard, truck → sink (same cargo, same quantity / line); reverts to the direct truck order when the rail route goes (flood, rubble, cut) | `economy/logistics/intermodal.rs` |
| 7d blocks | rail edges use the existing flood / debris / bridge closures (same router); signalling and passing loops deferred (one-track capacity not modelled yet) | router (existing) |
| 7e look | trains drawn on the rail polyline (engine + wagons, 2D), clickable (vehicle card says train) | `presentation/truck_view.rs` |
| 7f measure | dam scene: two yards on the west-bank rail line + a train; dump `rail` section (train trips, units by rail vs road, per-unit times) | scenario + dump |

### RD-8 AI builds roads and rail

- Faction economy reads flow, wear and congestion: tracks and busy off-road corridors get upgrade orders (dirt → gravel → paved);
  heavy long-haul bulk lines get rail; HQ ↔ key sites get a road when built.
- Military: repair or reroute around cut roads, blown bridges and cut rail.

### RD-9 Look

- Ribbons per tier (width from lanes, colour/texture per tier, motorway divided with interchanges and overpasses), worn tracks as ground tint, lane offset for two-way traffic, decks per tier, rail as above.
- Damaged / flooded / muddy / congested states visible.

### RD-10 Weather, water, destruction

- Already: flood and rubble/crater blocks (D-4), deck pick.
- Add: mud on Track/Dirt/Gravel, blast damage to roads and rail (craters cut paved roads, bridge decks fail), flood erosion of unpaved roads. Repair stays with D-7 (deferred); meanwhile a damaged road is rebuilt by an upgrade order.

### RD-11 Measure (one round per phase, real captures)

- Dam scene gets a supply network trucks need (depot → dam, depot → town, river crossing over the bridge), one off-road-only site, and later a rail line with two yards.
- `roads` dump section grows: tier/mode share per leg, leg time vs straight-line time, volume/capacity per edge, tracks formed, blocked edges, train loads.
- Matrix rows (`tools/sim_review/expected/matrix_v1.json`): truck-km on roads > 0.5 in the dam scene; bridge carries traffic; mean leg time with roads < without; tracks formed in the natural world after N ticks; rail moves the bulk cargo.

## Order

RD-0 → RD-1 → RD-3 (trucks really use roads, faster) → RD-11 scene + dump → RD-6 placement tool (one tool for road + rail, replaces the janky ones) → RD-4 congestion → RD-2 → RD-5 → RD-7 rail → RD-9 → RD-8 → RD-10.
RD-2 and RD-9 can run in parallel with RD-5/6 (different files). RD-0/RD-1 are written for road and rail at once so RD-7 only adds the train adapter, stations and look.

## Decided (operator 2026-11-20)

1. Motorway / express tier: **yes**, limited access + grade separation.
2. Congestion: **yes**, BPR slow-down in route choice.
3. Wear: **by unit class** (trucks, tracked armour, infantry, trains), one queue.
4. Cost: **materials + labour + build time** (no money exists), diesel/power to run.
5. Rail on par: shared corridor kernel, train adapter, stations, intermodal.
6. Placement: one shared corridor tool for road, motorway and rail; always-live preview, plain controls, per-section feedback, auto works (RD-6), moved early in the order.

## Status (2026-11-20)

| Phase | Status | Evidence |
|-------|--------|----------|
| RD-0 corridor data model | DONE (code) | `construction/corridor_network.rs` (segments, nodes ≤ 8 tiles, junctions, motorway grade separation, rail never shares road nodes); `infrastructure/road_tier.rs` + `assets/config/infrastructure/road_tiers.ron`; undo/redo by segment id; rail draws as rail; tests `corridor_network::*`, `road_tier::*`, `upgrade::*` |
| RD-1 joinable roads, time-based choice | DONE (code) | road edges priced by tier speed (`hydrate_transport_from_snapshot`); `truck_road_route` joins at nodes within 48 tiles, ramps off-road; planner picks road vs cross-country by time |
| RD-3 leg time from the route | DONE (code) | `SiteLegRouteBook` (sole writer `plan_site_leg_routes_system`), `RoutedTravel` in the order sim; presentation copies the planned route |
| RD-11 dam scene roads | PARTIAL | 3 gravel supply roads + bridge; capture: 4 road segments / 11 spans, picks 11/11 (2D) and 2/2 deck rays (3D), **42 of 57 trucks routed on roads** (was 0), deliveries 879 (unchanged), all dam rows pass. Leg book was empty at the end of the run (stale-leg prune after 3 600 ticks under time skip), so trip time vs straight line is not yet in the dump |
| Note | — | Off-road speed is now 18 km/h (RON), so roadless trips are ~2.4× slower than the old straight-line estimate at truck speed |
| RD-6 placement tool | DONE (code), feel NOT checked by hand | One tool for every road tier, rail and firebreaks (`construction/corridor_tool.rs` + `roads/input.rs`); duplicate rail tool removed. Always-live preview; click start / bend, double-click / Enter / click a line to finish, Backspace / RMB undo point, Esc cancel, Shift 15° lock, drag points. Per-stretch verdict Ok / Works (bridge type, cut/fill, level crossing, overpass) / Blocked (reason). Road sheet: length, ok/works/blocked, truck trip, notes; picker offers Dirt / Gravel / Paved / Concrete / Motorway. `PlaceCorridor` scenario step drives the same code. Capture: paved road queued with a "concrete beam bridge 120 m"; gravel road to the town blocked ("climb 17.2% steeper than 12.0%"); rail down the west bank blocked (39 m / 91 m cut/fill > 18 m). Checks 7–25 µs per path |
| Bridges | DONE (code) | `infrastructure/bridge_types.rs` + `assets/config/infrastructure/bridge_types.ron` (culvert, timber beam, timber trestle, pontoon, Bailey, steel girder, prestressed concrete, steel truss, stone arch; cheapest type that fits per line); `construction/bridges.rs` (`BridgeBook`, sections pier to pier, flood damage per section, blast damage per section from `BlastCraterBook.recent`, routing open / closed); placeholder model per type and section condition (`render/pixel_world/layers.rs` `push_bridge_model`); deck height fixed at build. Capture: gravel road got a Bailey (2 sections), paved road a concrete beam bridge (4 sections); small missile hit 2 concrete sections (standing); dam burst dropped every section of both |

## Bridges (2026-11-20)

A bridge is a structure type, not a road surface. A line built across water (depth > 0.3 m at
build time) becomes a bridge bank to bank; its type is the cheapest in the line's preference list
that spans the length and depth (`choose_bridge_type`). Built in sections (one per pier span).

- **Flood:** per section, load = depth × speed² × pier exposure (single span 0.35), + 10 × overtop
  multiplier × (water above soffit) × speed²; over strength costs integrity; 0 = section falls.
  Pontoons ride the water and fail when current > 2 m/s.
- **Blast:** per section, warhead peak × linear falloff at the section ÷ blast toughness; impact
  points kept for the deck scorch. A small warhead drops a few timber spans or scars concrete; a
  large one drops a concrete span.
- **Routing:** intact bridge carries traffic over flooded river tiles; any fallen section or water
  over the deck closes it (routes re-plan via the road stamp).

## Open items (operator list 2026-11-20 + found in captures)

Round 2026-11-20 (b): code for B-1, B-2, B-4 (choice + bill), B-5, B-6, R-1, R-4, R-5, R-8; one dam
capture. Status per item:

| # | Item | Status | Notes |
|---|------|--------|-------|
| B-1 | Fire: timber bridges don't burn | DONE (code), not seen in a capture | `burns` per type in `bridge_types.ron` (timber 1.0, pontoon 0.4, Bailey 0.15, rest 0); `bridge_fire_damage_system` samples fire-overlay heat at each section once a sim second (rain ≥ 8.5 mm/h quarters it), records `fire_loss`, drops with cause `fire`; 3D model chars and shows flames; 2D mark turns orange. Burning bridges do not yet add fuel / spread fire themselves. Test `timber_burns_concrete_does_not_rain_slows_it`. Dam scene has no fire at its bridges, so no capture evidence |
| B-2 | 2D map: bridges look like roads | DONE (code), not checked on screen | `sync_world_map_bridge_marks` (`render/world_map_network_mesh.rs`): wider deck in the type's colour, casing lines, splayed wing ticks at the abutments, pier ticks at joints, damage tint, fire orange, fallen section = gap with a red cross. Capture shot is the 3D view; needs a look on the tactical map |
| B-3 | Repair: fallen sections stay down | OPEN — waits on D-7 | No change; a rebuild of the line is the only fix. Decision kept: repair belongs to the D-7 repair/rebuild program |
| B-4 | Choice and cost | PARTIAL | **Choice DONE (code):** each stretch carries its bridge (`StretchBridge`); **T** cycles the types that span every crossing of the path (`bridge_choices`, `apply_bridge_choice`); the chosen type rides on the plan (`ConstructionPlan.bridge`) to the segment (`bridge_pref`) and `register_bridges_system` (now after execute) builds it if it fits. Road sheet row "Bridge": type, "(T: n types fit)", bill. **Bill DONE (shown):** `CorridorPathSummary.bridge_materials` = `materials_per_m` × span. **Charging OPEN:** corridors build per tick with no material gate at all (`CorridorConstructionBook`), so the bill is not taken from stock — needs the RD-6 cost item (a supply gate on corridor construction) for roads, rail and bridges together. Test `the_player_can_pick_another_bridge_type_and_sees_its_bill` |
| B-5 | Flood load calibration | PARTIAL — decision needed | Current a structure takes is now capped (`structural_current_mps`: ≤ 9 m/s and ≤ 1.5·√(g·depth); the solver read up to its 50 m/s clamp over thin films), deck term ×4 (was ×10). Capture: concrete beam peak load **3 141** (was ~20 000), Bailey 2 640; both bridges still drop in the full dam burst (concrete strength 400 = 7.8× over). Open: decide whether a concrete / truss bridge should survive a full burst (raise strength, or accept wash-out as realistic) |
| B-6 | Missile vs flood not separated | DONE | Per section `blast_loss`, `flood_loss`, `fire_loss`, `dropped_by` in the dump (`roads.bridges[].section_rows`). Capture: missile took 0.23 / 0.22 of two concrete sections, the flood finished them (`dropped_by: flood`) |
| R-1 | Leg book empty at end of run | DONE | Prune by count (512, least recently driven first) instead of age. Capture: 5 legs at the end, road share 76%; haul 1→4 runs 314 m of road + 41 m off-road, 35.9 s at 12 m/s vs 28.6 s old straight-line estimate |
| R-2 | `drawn_segments` 0 in earlier captures | EXPLAINED | Surface publishes only while the pixel world is on (`pixel_world_on`); this capture had it on (`presentation.pixel_world_active`) and shows 21/21 drawn, 21 picks, 1/1 deck ray. A 0 means the dump fired outside the 3D view |
| R-3 | Placement tool feel | OPEN — operator | Hands-on round: controls, preview, readout, plus the new G (route assist) and T (bridge type) keys |
| R-4 | Route assist | DONE (code), not tried by hand | **G** routes from the last point to the cursor: A* on a 2-tile grid (16 directions, so it can zig-zag up a slope), grade ≤ 95% of the mode's limit, water only if the mode bridges (×4 cost), then simplified so every stretch passes `check_corridor_path`. Failure note in the readout. Test `route_assist_winds_up_a_slope_a_straight_line_cannot_climb`. Note: curved preview re-samples the assist points as a spline |
| R-5 | Curved path undo | DONE (code) | Plans from one placed path share a group (`ConstructionPlanQueue::begin_group`); history `RoadSegment.group`; Ctrl+Z / Ctrl+Y take the whole group (`ConstructionHistory::pop_step`). Test `a_placed_path_undoes_as_one_step` |
| R-6 | Cut/fill and crossings are labels only | OPEN | Not started: terrain cut/fill (through `ground_deform`), level crossings and overpasses in the graph |
| R-7 | Pre-existing lib test failures | OPEN — unchanged | This round's `construction::` run: 372 pass, the same 4 fail (2 integration tests on missing `FuelBreakCommitEvent` resource in `ExecuteSideEffects`, demolish funnel, voxel massing). Not checked against an older commit |
| R-8 | `step_logistics_orders_system` 13 params | DONE | Split into a chain in the same domain: `sync_logistics_order_sites_system` (8), `apply_logistics_order_requests_system` (6), `step_logistics_orders_system` (5) |

Capture (b), dam scene: all dam rows pass; deliveries path unchanged; 42 of 57 trucks on roads; debug
perf avg 37 ms / p95 70 ms (was 36 / 67; debug rows fail their thresholds as before), 0 hitches.

### Round 2026-11-20 (c): next todos + UI responsiveness

Operator: bridges should survive most floods but not all, by how hard the water hits; do every next
code todo; make the build tools and other play UIs work and respond properly.

| Item | Status | Notes |
|------|--------|-------|
| B-5 flood survival | DONE (code) | `strength` is now the design flood load, taken indefinitely; from 1× to 3× it the piers scour slowly (≈ 5 min of sustained load to fail), past 3× a span goes in seconds (`flood_integrity_after`). Strengths: culvert 60, timber beam 90, trestle 70, Bailey 300, girder 1000, truss 1200, stone arch 1400, concrete 1500. Capture: the dam-break wave stood ~18 m over the concrete deck (peak 6 033 = 4× design) and took all 4 spans; the Bailey (2 904 = 10×) went too. A smaller flood or a bridge further downstream scours instead. Test `a_big_flood_scours_concrete_slowly_only_a_far_bigger_one_drops_it_fast` |
| B-4 charging | DONE (code), seen in capture | `construction/corridor_works.rs`: a line placed to order (`CorridorSegment::charged`, set by the tool's commit) gets a bill = tier `materials_per_m` (new in `road_tiers.ron`; rail `rail_materials_per_m`) × length + its bridges' `materials_per_m` × length. Taken from the owner's nearest site with free stock (order-sim `ConsumeStock`, 40 units per cargo per sim second); the corridor build progress is capped at the scarcest delivered share (`CorridorWorksCaps` → `advance_corridor_construction_book_on_sim_tick`). Road sheet row "Materials" shows the bill. New cargo `Timber` (timber bridges, sleepers). Capture: 1 966 units taken from the HQ; its 700 Gravel ran out, so the paved line stands at 58 % and the gravel line at 90 % until more gravel arrives. Starting-world lines (fixtures, settlement skeletons) are not charged. **Hauled by trucks (2026-11-24):** each open line has a works site in the order sim at its middle (`CorridorWorksSite`, role Construction); one-off runs bring each cargo from the owner's nearest stocked site (`plan_works_runs`: short = needed − unloaded − on the way − asked; never from another works site; ≤ 400 units a run). Delivered = unloaded at the works site; a supplied line cancels leftover runs and closes, and its site is released. A deleted line's runs fail with its site. Tests `trucks_haul_a_roads_bill_to_its_works_site`, `works_runs_*` |
| RD-4 congestion | DONE (code), not yet seen under load | `economy/logistics/road_flow.rs` (`RoadFlowBook`, sole writer `publish_road_flow_system`): every truck on a leg adds its rate to each road edge of the leg's route; capacity = tier `capacity_veh_h_lane` × lanes; time × BPR `1 + 0.15 (v/c)^4` (capped at 1/0.15). Used in edge cost (`route_edge_cost_with_nav`, so routes spread to parallel roads) and in leg caps (trip time); legs re-plan when a slow-down bucket changes. Dump `road_flow` (now with run peak). Capture: book empty at dump time (trucks idle), revision moved 157 times during the run; peak v/c added for the next capture. Test `a_road_slows_as_it_fills_and_never_stops` |
| RD-2 off-road speed field | DONE (code) | `systems/navigation/offroad_speed.rs` (`OffroadSpeedField`, sole writer `publish_offroad_speed_field_system`): per tile a multiplier on the off-road speed from the existing `wheeled_logistics` mobility profile (cell tags → cost / blocked, `evaluate_tile`, until now only used by the editor preview) and slope (dense height incl. ground delta, `max_grade` 0.4), × water depth (`WaterTileProbe`: ≤ 0.05 m dry, ford ×0.35 up to 0.5 m, deeper = wall), × rubble (`DebrisRouteBlockBook` ×0.3), × mud (chunk soil moisture) and snow. Tuning `road_tiers.ron` `offroad`. Rebuilt on terrain change and every 300 sim steps (sim time only, deterministic). Read by the cross-country truck path (time-weighted A*, ±24-tile detour box, walls avoided), its trip time (`offroad_route`: each stretch at the field's harmonic-mean speed), road on/off ramps (a ramp through a wall is not taken) and formations (slowed, step round a wall, hold when boxed in). Dump `offroad_field`. Tests `the_cross_country_path_goes_round_a_river_through_its_ford`, `a_formation_steps_round_a_wall_and_holds_when_boxed_in` + 4 unit |
| RD-5 worn tracks | DONE (code), forming not yet seen in a long run | `economy/logistics/track_wear.rs` (`TrackWearBook`): trucks add passes to the off-road tiles of the legs they drive (vehicles on the leg ÷ trip time per second), moving formations by class (armour 1.5/s … infantry 0.05/s); wear halves every hour without traffic. A connected run ≥ 6 tiles at 30 passes along a planned leg becomes Track `RoadSegment`s through the construction queue (uncharged, no bill; upgradeable). Tuning `road_tiers.ron` `worn_tracks`. Dump `track_wear`. Not yet: tracked armour damaging road condition (no condition model), rain → mud on tracks (RD-10; `wet_speed_mul` is still unused) |
| RD-7 rail freight | DONE (code); look 2D only | Trains in the order sim (`VehicleMode::Rail { wagons }`, `AddTrain`; capacity wagons × `rail.wagon_capacity`, freight + tank wagons; load / unload slower with more wagons). Rail legs between a player's yards (`systems/navigation/rail_legs.rs`, sole writer of `SiteRailLegBook`; `routes::rail_yard_route`: siding to a rail node within `yard_reach_tiles`, the graph's rail route with flood / rubble / failed-bridge edges as walls, siding to the other yard). The allocator gives a yard → yard order to a train when its time per unit beats the best truck's; trains never drive truck legs, load roads or wear tracks. Yards: `SiteRole::Yard`; start-config `yards` (dam valley: North yard, South yard + 1 train of 12 wagons); buildable `freight_yard_v1` (and the rail warehouse) becomes a yard when built and brings one train. Intermodal (`economy/logistics/intermodal.rs`): a long order (≥ `intermodal_min_tiles`) with a quicker rail way is replaced by truck → yard, train yard → yard, truck → sink (a yard end skips its truck leg; standing lines publish transit demands at the yards); when the rail leg goes, the legs are cancelled and what has not arrived goes back by road. Trains drawn as a dark-red engine + wagons bar on the rail route, clickable (vehicle card "TRAIN"). Tuning `road_tiers.ron` `rail`. Dump `rail`. Tests: `rd7_done_when_rail_beats_trucks_per_unit_over_150_tiles` (rail > 3× faster for 1200 t), `rd7_yards_route_over_the_rail_line_and_lose_it_when_the_line_is_buried`, train vs truck allocation, intermodal split / revert / yard-end. **Added 2026-11-25:** signals — single track worked as token blocks, yards are the passing places: a train never leaves onto track an opposing train is on (held at the signal, shown on its card), and follows another only a headway (60 s) behind (`rail_blocked`, `rail_routes_conflict`); level crossings — a road stretch over the track gets a barrier delay (12 s + 25 s per train running that line) folded into road traffic time, so routes and trip times both pay it (`economy/logistics/level_crossings.rs`); trains in the 3D view as an engine + wagons trailing along the track round bends (`trail_along`); yards build trains (`BuildTrain`, 12 wagons for 120 Steel + 30 Machinery): automated factions when yard → yard orders wait for a train, the player with "Build train" on the yard's card (scenario yards are clickable). Tests `rd7_opposing_trains_never_share_the_single_track`, `rd7_following_trains_keep_a_headway`, `rd7_a_yard_builds_a_train_from_its_stock`, `a_road_over_a_railway_gets_a_level_crossing_delay`, `wagons_trail_along_the_track_round_a_bend`. **Not modelled:** passing loops between yards (a long single track carries one direction at a time). |
| RD-9 look | DONE (code), not yet seen by eye | `construction/road_look.rs`: committed roads drawn per tier (width from tier × lanes, colour per tier: track, dirt, gravel, paved, concrete, motorway + median strip); state tints over the tier colour — flooded (blue), buried / blocked (rust), congested v/c > 0.8 (amber), muddy unpaved road in wet ground (dark brown); rail as ballast bed + sleepers every half tile + two rails (flooded / blocked ballast tinted); worn ground (RD-5) tinted by its wear before it becomes a track; trucks keep to the right-hand lane (2.8 m off the centre line). Path requests carry a `PathLook` (colour, sideways offset, overlay); the 2D and 3D path layers honour it, overlay lines are skipped on bridges (the deck model stands in). Tests `wider_tiers_and_more_lanes_draw_wider…`, `states_tint_the_road`, `rail_is_ballast_sleepers_and_two_rails`, `worn_ground_shows_before_it_becomes_a_track`. Not done: interchange / overpass deck model for motorway grade separations, deck look per tier |
| RD-8 AI builds roads and rail | DONE (code), not yet seen in a run with an AI faction | `ai/faction/infrastructure.rs`: each enabled AI faction takes at most one action per `ai.interval_s` (120 sim s), most urgent first — clear rubble / fill craters on its blocked roads and rail near its sites (ground grade op), upgrade its busiest unpaved road at v/c ≥ 0.6 (track → dirt → gravel → paved), pave a truck leg with ≥ 600 m off-road and ≤ 30 % on roads (gravel road along the route assist), rail-link a heavy long haul (≥ 200 u/h, longer than `intermodal_min`) with no yards (a freight yard beside each end + a railway between them). Lines are billed to the faction: `CorridorSegment.owner_slot` / `ConstructionPlan.owner_slot`, the corridor works bill `OwnerId(owner_slot)` and haul with its trucks. Tuning `road_tiers.ron` `ai`. Tests `rubble_first_then_the_busiest_unpaved_road`, `a_long_mostly_off_road_leg_gets_a_road_and_a_heavy_long_haul_gets_rail`. Not done: military reroute around cut lines (the router already avoids them), bridge repair (B-3 / D-7) |
| RD-10 weather, water, destruction | DONE (code), not yet seen in a run | `economy/logistics/road_condition.rs` (sole writer `RoadConditionBook`): mud — unpaved tiers in a chunk with soil moisture past `offroad.mud_from_moisture` run down to their `wet_speed_mul` (scaled by wetness); flood erosion — water on an unpaved road adds damage per flooded minute (track ×2, dirt ×1.5, gravel ×1, paved none); blasts — damage by falloff within radius × `blast_reach_mul` (paved half); a damaged road takes `1 + damage_slow × damage` longer; damage resets when the edge's profile changes (upgrade = rebuild). Folded into `RoadFlowBook::time_mul` (routes and trip times see it); damaged roads tinted grey-brown, mud from the book too; AI (RD-8) rebuilds roads past `rebuild_damage` first. Already before: flood / crater closes (D-4), bridge sections fall to blasts / floods / fire (`bridges.rs`). Tuning `road_tiers.ron` `condition`. Tests `mud_slows_unpaved_roads_only…`, `floods_wash_out_unpaved_roads…`, `a_flooded_dirt_road_erodes_and_an_upgrade_rebuilds_it`. Not done: repair proper (D-7), rail condition (rail closes but does not degrade) |
| R-6 cut/fill | DONE (code), terrain change not measured | `construction/corridor_grading.rs`: each line placed to order is graded once: dry tiles between two nodes are cut / filled to the straight bed between the node heights, at most the mode's cut/fill limit, water and bridge spans left alone; applied through the terrain's own queue (`GroundDeformOp::Grade`, terrain stays sole writer, water/fire/voxel see it). Crossings: graph rules already in `CorridorNetwork::add_segment` (road/road junction, motorway grade separation, rail never shares road nodes = level crossing); no overpass deck model yet. Tests `a_bump_is_cut_a_dip_is_filled_water_is_left_alone`, `grading_stops_at_the_modes_cut_limit` |
| B-1 follow-up | DONE (code) | A burning timber span keeps burning while wood is left (×0.97 per s), passes fire to the spans either side (×0.6 × `burns`, water under it does not stop that), and end spans set the bank alight (scripted-ignition path, `SimEffectQueue`) |
| R-7 tests | PARTIAL | Checked against the previous commit (stash run): 36 failures there, none caused by roads work. Fixed 18: construction (fuel-break resource in 2 integration tests, demolish tripwire split, voxel massing not cleared in tactical), fire visual extract ×7 (plugin did not declare `SiteDestroyed`), scenario ×5 (runner did not declare `CommitConstructionSiteEvent`), network flow (missing `WorldFieldLayerEpoch`), plus 2 tests updated for this round (grid snap on tile centres, Esc with road sheet). **18 still fail**, other domains: economy activation / resource-flow chain (12 incl. AI faction + game loop), render pipeline tripwires (2), launch args, atmosphere revision key, strategic bundle (combat resources missing in the test app) |
| UI responsiveness | DONE (code), not tried by hand | From a read-only audit of the play UI: (1) RMB no longer drops a line tool and its path (it takes back a point on a click; a right-drag pans) and ignores clicks on panels; (2) curves sampled by distance (~2-tile pieces) so bends are not cut into "too short" stretches; (3) 2D map path layer keys every coordinate and colour (the rubber band follows the cursor, colours update) and respawns only preview ribbons on cursor moves, not the whole network; (4) Esc has one owner for line tools (first clears the path, next drops the tool; the sheet follows the tool; Esc that closed the picker or a key capture is spent); (5) the road sheet blocks map clicks over its real rect; (6) Upgrade acts on the road under the last map cursor, not under the button; (7) logistics orders panel no longer respawns every sim tick (data refresh ≤ 2/s, clicks immediate); (8) Backspace no longer wipes pending blueprints and Ctrl+Z no longer resets map zoom; (9) preview checks skipped when nothing changed; context-tray layout writes only on change; (10) rubber band freezes while the pointer is on a panel; (11) firebreak gets the sheet; (12) grid snap on tile centres (no half-tile jump on build); (13) input runs before the preview (no one-frame lag). Not covered: minimap, pause / options menus, plant card (audit found nothing there but did not look closely) |

Capture (c), dam scene: lib suite 2 329 pass / 18 fail (all pre-existing); all dam rows pass; 42 of 57
trucks on roads; debug perf avg 35 ms / p95 68 ms (debug rows fail their thresholds as before).

## Next (when work resumes)

1. Operator hands-on: line tool feel (RMB, Esc, curves, G, T), 2D bridge marks, materials row,
   orders panel.
2. Capture with congestion peak (`road_flow.peak_vc`) and grading volume; a scene with a narrow
   road and many trucks to see traffic spread.
3. ~~Works hauled by trucks (order-sim runs to a works site) instead of drawn from stock.~~ Done 2026-11-24 (B-4 row).
4. ~~R-7 remainder (economy activation chain first — 12 tests).~~ Done 2026-11-24 in PLAN-CI-TEST-TRIAGE-v1 (T-1).
5. ~~RD-2 off-road speed field~~, ~~RD-5 worn tracks~~ and ~~RD-7 rail freight~~ done (Status rows). ~~RD-9 look~~, ~~RD-8 AI builds roads / rail~~ and ~~RD-10 weather and destruction~~ done (code; to be seen in a dam run with an AI faction and a storm). Next: 
   B-3 repair with D-7; overpass deck model.

