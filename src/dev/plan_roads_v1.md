# PLAN-ROADS-v1 — roads and rail that freight uses: tiers, motorways, worn tracks, congestion, real travel time

Status: **OPEN** (2026-11-20). Owner lanes: `coder` (sim/logistics/construction), `designer` (build picker, road card, ribbons).
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
