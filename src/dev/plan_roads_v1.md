# PLAN-ROADS-v1 — roads that trucks use: tiers, worn tracks, real travel time

Status: **OPEN** (2026-11-20). Owner lanes: `coder` (sim/logistics/construction), `designer` (build picker, road card, ribbons).
Supersedes the road half of T-8 in [`plan_close_voxel_water_v1.md`](plan_close_voxel_water_v1.md) (clickable roads and
bridge decks DONE there; this plan makes roads matter).

## Product stance (operator 2026-11-20)

- Logistics still moves off-road. A road is a faster, more reliable way, never the only way.
- Repeated off-road traffic wears **dirt tracks** into the ground on its own.
- The player and the AI **build quality roads**: tiers (dirt, gravel, paved, concrete), lane counts, bridges. A road's tier
  sets its speed, capacity and how it handles weather, flooding and damage.
- Travel time comes from the route actually driven. No straight-line timing.

## Where it stands (capture 2026-11-20, dam scene, `roads` dump section)

| Item | Finding |
|------|---------|
| Click picks | Bridge answers the 2D segment pick and the 3D deck ray (1/1 each) |
| Trucks on roads | **0 of 56**; 878 deliveries, all cross-country |
| Why | Only road = one 16-tile bridge at row 110; every truck leg runs rows 140..200; road join range is 24 tiles |
| Road graph | A committed road is **one edge head→tail**, no interior nodes: a truck can only join at the two ends |
| Road storage | `ExecutedRoadNetwork.tiles` is one flat list; readers pair consecutive tiles, so road A's end + road B's start = a phantom segment |
| Road types | `RoadType { Street, Highway }` in the build tool; `corridor_profiles.example.ron` has lanes/speed/surface tags, read only by spline geometry |
| Leg timing | `orders/travel.rs`: straight line × detour 1.3 ÷ speed; ignores road, terrain, slope, water |
| Off-road cost | Cross-country path grid (`logistics_floodfill`) has no terrain-speed or wetness term |
| Worn tracks | None |
| AI roads | No faction AI road building |

## Authority (AGENTS.md rules)

| Domain | Sole writer | Readers | Requests in |
|--------|-------------|---------|-------------|
| Built road network (segments, tier, lanes, deck, condition) | construction execute funnel (`src/construction/`) | transport graph, render, road card, AI | `ConstructionIntent` (build / upgrade / remove) |
| Track wear field (per-cell wear, emergent tracks) | new `TrackWearField` owner system (logistics/transport) | transport graph, render, road card | `TrackWearQueue` (truck motion pushes cells driven) |
| Off-road speed field (land class, slope, wetness, ford depth) | one publisher system (navigation) | route planner | — (reads `WaterTileProbe`, ground delta, rain) |
| Transport graph + costs | transport owner | route planner, order sim | — |
| Leg route + leg time | route planner (`sync_truck_motion_waypoints_system` successor) | order sim, truck presentation | — |

The order sim never computes distance itself. It reads the leg time the planner publishes. Presentation reads both, writes neither.

## Phases

### RD-0 Road data model (fixes the flat list)

- Replace `ExecutedRoadNetwork.tiles` with `RoadNetwork { segments: Vec<RoadSegment> }`; `RoadSegment { id, polyline: Vec<IVec2>, tier, lanes, deck: Option<Deck>, condition }`.
  Undo/redo, snap, upgrade and rail-crossing readers move over; the phantom segment goes away.
- Tier table in RON `assets/config/infrastructure/road_tiers.ron` (replaces the example profiles for roads):

| Tier | Speed km/h | Lanes | Capacity trucks/h/lane | Wet speed × | Floods as | Build per 100 m |
|------|-----------:|------:|-------:|-----:|-----------|-----------------|
| Track (worn, not built) | 25 | 1 | 40 | 0.4 (mud) | blocked > 0.3 m | — |
| Dirt | 35 | 1–2 | 60 | 0.5 | blocked > 0.3 m | labour only |
| Gravel | 55 | 2 | 120 | 0.85 | blocked > 0.5 m | gravel |
| Paved | 80 | 2–4 | 300 | 1.0 | blocked > 0.5 m | gravel + asphalt |
| Concrete | 90 | 2–6 | 400 | 1.0 | blocked > 0.6 m | gravel + cement |

  The numbers are the first pass; tuning lives in the RON. Bridges: a deck on any built tier, with a load limit by tier.
- Done when: two separate roads give exactly their own segments; old saves/R8 snapshots load (migration from the tile list).

### RD-1 Graph edges a truck can join anywhere

- A road commit becomes a chain of edges with nodes every ≤ 8 tiles and at every junction and crossing (roads that cross
  or touch get joined). Edge cost = length ÷ (tier speed × wet factor × congestion factor).
- Route planner joins the network at the nearest node of a **nearby** road (on-ramp cost = off-road time to that node), not only within a fixed 24 tiles. Pick by total time (off-road on-ramp + road + off-road off-ramp) vs all off-road.
- Done when: a truck whose leg runs parallel to a road 6 tiles away takes the road; one with the road 60 tiles away does not.

### RD-2 Off-road speed field

- One published field: base speed by land class (grass, forest, rubble, crater, rock), × slope factor (from DEM + ground delta),
  × wetness (rain/infiltration soil moisture), water depth via `WaterTileProbe` (ford ≤ 0.5 m slow, deeper = wall).
- The cross-country path grid uses it as its cost. Rubble and craters already block roads (D-4); off-road they become slow, not walls.
- Done when: trucks route around a flooded valley floor and steep valley walls; rain visibly slows dirt and off-road legs.

### RD-3 Leg time from the route driven

- The planner publishes `LegPlan { polyline, time_s, tier_share }` per vehicle leg; the order sim's `travel_ticks` reads it
  (falls back to the straight-line estimate only until a plan exists for a brand-new leg).
- Re-plan on network change (stamp already exists) also re-times the remaining part of the leg.
- Done when: a paved detour 1.4× longer than the straight line beats the off-road straight line on time, and deliveries per hour go up when the road is built.

### RD-4 Worn dirt tracks

- Truck motion pushes the cells it drives to `TrackWearQueue`; the wear owner adds wear per pass, decays it slowly without traffic.
- Wear above a threshold over a connected run turns it into a **Track** segment (graph edges, drawn as worn ground). Track speed beats raw off-road, so tracks reinforce themselves along real supply lines.
- Below the decay threshold a track fades back. Rain turns tracks to mud (wet factor).
- A track is not construction: it never takes building materials. The player or AI can **upgrade** it to a built tier (construction funnel).
- Done when: in a long run with no built roads, tracks form along the depot → site lines and leg times on them drop.

### RD-5 Build and upgrade (player)

- Build picker: road tool gets tier + lanes (play UI only; no debug toggles). Upgrade an existing segment or track in place (`upgrade.rs` grows to tiers).
- Building a road is a construction site with a material manifest (gravel, asphalt, cement per 100 m), delivered by trucks like any site; it opens when done.
- Road info card: tier, lanes, speed now (wet, damaged), capacity use, condition (open / flooded / debris / damaged), trucks routed.
- Done when: a player builds a gravel road, trucks deliver its material, it opens, and traffic shifts onto it.

### RD-6 AI builds roads

- Faction economy reads per-edge traffic and track wear: a track or off-road corridor above a traffic level gets an upgrade order (dirt → gravel → paved) within its budget; HQ ↔ key sites get a road when they are built.
- Military: the AI repairs or reroutes around cut roads and blown bridges (reads the same blocks).
- Done when: in the natural playquad world the AI's depot → site lines become gravel roads over a long run.

### RD-7 Look

- 2D and voxel ribbons per tier: width from lanes, colour/texture per tier, worn tracks as a ground tint (no ribbon), lane offset for trucks (two-way traffic does not overlap), bridge decks per tier.
- Damaged/flooded/muddy states visible.

### RD-8 Weather, water, destruction

- Already: flood and rubble/crater blocks (D-4), deck pick.
- Add: mud from wetness on Track/Dirt/Gravel, road damage from blasts (craters cut paved roads; bridge decks fail and fall), flood erosion of unpaved roads.
- Repair stays with D-7 (deferred program) — damaged roads can be rebuilt as an upgrade order meanwhile.

### RD-9 Measure (one round per phase, real captures)

- Dam scene gets a supply road network that trucks need (depot → dam, depot → town, a river crossing over the bridge) plus one off-road-only site.
- `roads` dump section grows: per-truck route tier share, leg time vs straight-line time, tracks formed, wear, congestion, blocked edges.
- Matrix rows (`tools/sim_review/expected/matrix_v1.json`): share of truck-km on roads > 0.5 in the dam scene; bridge carries traffic; mean leg time with roads < without; tracks formed in the natural world after N ticks.

## Order

RD-0 → RD-1 → RD-3 (trucks really use roads and get faster) → RD-9 scene + dump → RD-2 → RD-4 → RD-5 → RD-7 → RD-6 → RD-8.
RD-2 and RD-7 can run in parallel with RD-4/5 (different files).

## Open choices (operator)

1. Tier list and lanes as above, or add a separate highway/motorway tier?
2. Congestion: simple per-edge capacity slow-down (planned) or none for now?
3. Tracks: only from trucks, or also from formations/infantry movement?
4. Should roads cost money as well as materials?
