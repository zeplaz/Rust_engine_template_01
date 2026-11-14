# PLAN-REF-BOOKS-UPGRADE-USE-v1 — keyword lookup → engine-wide ECS architecture

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-UPGRADE-USE-v1** |
| **Status** | **SHIPPED** — Orders 0–8 closed (HAUL-AMPLIFY-1 + play UX keep-working) |
| **Date** | 2026-11-14 (PLAYCHECK-1 close → haul-amplify) |
| **P0 Bevy** | [`plan_bevy_020_migration_v1.md`](plan_bevy_020_migration_v1.md) — **PROGRAM CLOSE**; residual lib-test defer |
| **Index hub** | [`INDEX_MATRIX.md`]($REF_BOOKS_DIR/lookup/INDEX_MATRIX.md) → matrix: `$REF_BOOKS_DIR\lookup\keyword_book_matrix.csv` (**158** keywords · **4** books) |
| **Spine** | [`plan_production_restock_spine_v1.md`](plan_production_restock_spine_v1.md) — **P0–P7 SHIPPED**; residual **OPEN**: [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md) |
| **Architecture truth** | [bevy-simulation-grade](.cursor/skills/bevy-simulation-grade/SKILL.md) · [`07-repo-authority-map.md`](../../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md) · [`plan_schedule_sync_v1.md`](plan_schedule_sync_v1.md) |
| **Obeys** | `AGENTS.md` — **Sim → View → Render → UI** never inverted; one writer per resource; named SystemSets only |
| **Game fit** | City / logistics / ecology Bevy sim — formations, order-sim haul, MissionBook air, weather/smoke, construction nets, industrial MFG, play UI only |
| **Legacy Bevy** | Keep-in-mind only (pre-0.20 leftovers on touched paths) — **never** outranks 0.20; see bevy-020 plan § Legacy |

## 0. Scope rule — engine-wide systems architecture

Book patterns apply to the **whole Update/PostUpdate graph**, not a logistics-only corner:

```text
⊚Sim ═▶ ⊚View ═▶ ⊚Render ═▶ ⊚UI
  AdvanceSimTick → ChunkEnvironment / Transport / Logistics / Combat / MFG
  ViewRepresentation / MapCamera / ViewAuthority → ViewManager (sole)
  FireVisualFrame / atmosphere extract / world representation (read snaps)
  HUD / picker / overlays (read snaps · never write sim truth)
```

| Layer | What books may change | What books must not invent |
|:---|:---|:---|
| **Sim** | SystemSet edges vs `AdvanceSimTick`; single writers (`MissionBook`, `LogisticsOrderSim`, …) | Second fleet / mission / vehicle ECS |
| **View** | Order vs `SyncViewManager` / `ViewProjectionAuthority` | Parallel ViewManager writers |
| **Render** | Extract **after** sim+view snaps; FireVisualFrame placement | Sim mutation during extract |
| **UI** | Intent messages into existing ingress | Direct sim resource mutation |

**Slice A exit = whole-layer graph**, not “economy after tick” alone. Production-spine leftovers (missile catalog, haul-amplify) stay **orthogonal product work** — they inherit this architecture; they do not redefine it.

Canonical schedule paste: `07-repo-authority-map.md` (sim spine + view spine). Large Wave 2+ schedule rewrites still gate on **DR-SCHED-W2** / [`plan_schedule_sync_v1.md`](plan_schedule_sync_v1.md) — Slice A documents + bounded set-edge fixes only.

## 1. Lookup workflow

```text
keyword
  → keyword_book_matrix.csv  (gea_v1_e4 | gea_v2_e4 | gea_e3 | aig_e3)
  → keyword_lookup.csv       (section title + pdf_page)
  → catalog.json               (PDF path under $REF_BOOKS_DIR\)
  → extract 1–3 patterns       (layer, authority, schedule, data layout, decision shape)
  → map onto engine-wide layer + existing SystemSet / sole writer
  → reject if it invents a parallel stack or inverts Sim→UI
```

Prefer **gea_v1_e4** for foundations/runtime; **gea_v2_e4** for **graphics / animation / physics / audio / gameplay runtime** (primary over gea_e3); **aig_e3** for decision/message shape on MissionBook / fronts / order-sim. Use **gea_e3** only when Vol I/II cells are empty.

**SURFACE** = chapter hubs. **DEEP** = niche/nuanced (Vol II sweep + prior) — **cite-first**. Vol II niche examples: `motion_matching`, `stochastic_ray_tracing`, `radiometry_light_transport`, `pipeline_app_layer`, `overlays_ui`, `concurrent_object_updates`.

## 2. Spine reconciliation (not “pre-P4”)

| Spine phase | Book-driven follow-up |
|:---|:---|
| **P0–P3** | Consumers sit **under** engine-wide Slice A graph (tick + order-sim + UAS) |
| **P4–P6** | **SHIPPED** — book work is **architecture hygiene**, not new spine phases |
| **DEFER (spine)** | Deployable **dual-bus** merge (haul-amplify / missile catalog / picker / P7 **SHIPPED**) |

Do **not** reopen spine as “blocked on books.” Books inform **engine architecture** first; product DEFERs follow that map.

## 3. Keyword → layer / module map (engine-wide)

Each row names **layer**, **single authority** (resource + writer), or read-only consumer. **Reject** any proposal that adds a second writer or inverts Sim→UI.

| # | Keyword | Book / page | Layer | Pattern | Authority (writer) | Module / SystemSet |
|:--|:---|:---|:---|:---|:---|:---|
| 1 | `runtime_architecture` | **gea_v1_e4 p47** | **All** | Layered runtime; sim vs presentation | Plugin roots · `07-repo-authority-map` | `src/lib.rs` · `sim_control` · view/render plugins |
| 2 | `game_loop` | **gea_v1_e4 p505** | Sim (+ cadence) | Fixed-step / real-time tick | `SimStepClock` · `SimControlSystemSet::AdvanceSimTick` | `src/systems/sim_control/` |
| 3 | `concurrency` | **gea_v1_e4 p203** | **All** | Job ownership; named sets only | SystemSet chains ¬ string `.after` | Sim after `AdvanceSimTick`; View after `SyncViewManager`; Extract after snaps |
| 4 | `memory_architectures` | **gea_v1_e4 p183** | Sim→View | Frame snaps / batch resources | Per-frame resources · immutable downstream | `MissionBook`, order-sim buffers, `ViewRepresentationSnapshot`, `DroneMissionDump` (RO) |
| 5 | `hid` | **gea_v1_e4 p538** | View | Input → authority first | `ViewProjectionAuthority` · `MapCameraSystemSet` | `src/gui/map_camera.rs` · `view_authority.rs` |
| 6 | `decision_making` | **aig_e3 p318** | Sim | Intent → sole executor | **`MissionBook`** ← `apply_drone_mission_intents` | `src/combat/drone_missions.rs` |
| 7 | `strategy` | **aig_e3 p506** | Sim | Tactical aggregate, not unit micro | Front pressure / balance **readers** | `src/combat/front.rs`, `pressure.rs`, `balance.rs`, `src/strategic/frontline.rs` |
| 8 | `world_interface` | **aig_e3 p856** | UI→Sim ingress | Blackboard / message ingress | **`LogisticsOrderRequest`** → order sim | `src/economy/logistics/orders/plugin.rs` |
| 9 | `pathfinding` | **aig_e3 p216** | Sim | Graph on existing nets | Route graph + construction pathing | `roads/pathing.rs`, `routes.rs`, `orders/travel.rs` |
| 10 | `perception` | **aig_e3 p44** | Sim | Sensing → modifiers (read) | Weather envelopes → **read** `MissionBook` / sortie gates | `src/systems/weather/air_ops.rs` |
| 11 | `profiling` / `debug_tools` | **gea_v1_e4 p106** / **p566** | Tools | Structured metrics | External tools only (not ECS writers) | `tools/sim_review/` · `validate-report` |

**Out of matrix scope (explicit DEFER — do not implement from books):**

| Topic | Matrix keyword | Why DEFER |
|:---|:---|:---|
| SIMD / vector hot paths | `simd`, `gpgpu` | No new SIMD/GPGPU ECS; perf via existing sim_review first |
| ML / learning AI | `learning` | No neural/GA stack; MissionBook + aggregates only |
| Second fleet / unit ECS | — | **Rejected** — `VehicleFleet` + `LogisticsOrderSim` already own haul; formation mul is spine DEFER |
| Greenfield BT/GOAP | `decision_making` / `goap` | **Rejected** as new stacks — extend `MissionIntent` / `MissionBook` only |

## 3b. Impact mass — two independent rankings

These tables are **not competing**. Engine-wide ranks schedule/authority leverage across the whole graph. Niche ranks payoff for *this* city/logistics/ecology/air sim. A topic can score high on both; pick by **current lane**, not by cross-table winner.

Impact score (1–5): **1** = cite-only / no near-term edit · **3** = bounded harden of existing writer · **5** = changes how whole layer(s) behave if applied correctly. Effort: **S** doc/edge · **M** contract + tests · **L** multi-file / steward gate.

### 3b-0. A11 re-review (book ch.10 Execution Management) — **promoted**

Earlier A11 was scored **2** (“future AI budget; don’t invent a second scheduler”). That undersells **this** game.

**Why the books matter here:** AI for Games §10 is not “add a custom AI OS.” It is how to spend CPU when many agents/factions/decision tiers all want a tick. We already have that pressure:

| Decision level (ours) | Examples | Full fidelity every frame? |
|:---|:---|:---|
| Strategic field | `ChunkStrategicOverlay`, threat/recon per `faction_slot`, front derive | No — scale with `ChunkSimLod` / interest |
| Theater / front | pressure points, supply draw, stance | Near theaters yes; far = aggregate |
| Formation / logistics | `Formation`, `LogisticsOrderSim`, `AutoSupplyPolicy` | Active factions / player theater |
| Air / magazines | `MissionBook`, drone/missile packages, weather gates | Active AOs only |
| Hybrid / agent scores | `HybridSimPipeline`, decision explainability | Budgeted / cadence |
| Ecology / fire | chunk weather/fire (already `ChunkSimLod::{Dormant,Normal,Active}`) | Already LOD’d |

**Book → our ECS (do this, not a second scheduler):**

| Book pattern (aig_e3) | Page | Map onto existing | Do **not** |
|:---|:---|:---|:---|
| Load-balancing / priority scheduling | p825–839 | Round-robin **faction slots** / theaters across frames; Bevy `SystemSet` + `run_if` / cadence | New AI thread pool / fiber OS |
| Interruptible processes | p832 | Path/order work that yields; align with `interruptible_pathfinding` | Blocking per-faction mega-tick |
| Hierarchical scheduling | p837 | **Different cadences per decision level** (strategic slower than MissionBook) | One flat “AI update” system for everything |
| Anytime algorithms | p841 | Partial path / pressure refresh under frame budget; improve next frame | Must-finish GOAP planners |
| AI LOD / Scheduling LOD | p843–844 | Skip or cheapen far-faction AI when `ChunkSimLod::Dormant` or off-interest | Graphics-only LOD thinking |
| Behavioral LOD | p845 | Far = influence/front aggregates only; near = full MissionBook + auto-supply | Per-unit micro AI at any LOD |
| Group LOD | p850 | Formation / AO as the AI atom (we already reject airframe entities) | Entity swarm AI |
| Multi-tier AI | p575 | Strategic → front → formation → mission intents (already layered) | Collapsing tiers into one brain |

**Verdict:** A11-class patterns are **high impact** for multi-faction + multi-level decisions. Promote on **both** rankings (architecture cadence **and** game AI fidelity). Still **reject** inventing a parallel scheduler ECS — extend `ChunkSimLod` / interest / SystemSet cadence / faction round-robin.

### Ranking A — Engine-wide systems architecture

| Rank | Keyword(s) | Cite | Impact | Effort | What high-impact looks like here |
|:--|:---|:---|:--|:--|:---|
| **A1** | `runtime_architecture` + `concurrency` + `game_loop` | gea p47 / p203 / p505 | **5** | M | Whole **Sim→View→Render→UI** graph: `AdvanceSimTick` fences, named SystemSets — Slice A core |
| **A2** | `memory_architectures` + `abstract_timelines` + `time_measurement` | gea p183 / p511 / p514 | **5** | M | Frame snaps + pause/`dt_scale`; immutable downstream reads |
| **A3** | `ai_lod` + `load_balancing_scheduler` + `anytime_algorithms` (+ hierarchical / behavioral LOD) | aig **p835–850** | **5** | M | **Engine-wide decision cadence:** tier fidelity by interest/`ChunkSimLod`; faction work load-balanced across frames; **no** second scheduler — see §3b-0 |
| **A4** | `events_message_passing` + `world_interface` | **gea_v2** p632 · aig p856 | **4** | M | One ingress shape for intents/orders (no UI→sim writers) |
| **A5** | `multiprocessor_game_loops` + `concurrent_object_updates` | gea_v1 p524 · **gea_v2** p620 | **4** | L | Parallel-safe SystemSets / SCH — steward + **DR-SCHED-W2** |
| **A6** | `subsystem_lifecycle` + `engine_configuration` | gea p404 / p454 | **3** | S | Plugin boot / config single-writer |
| **A7** | `hid` + `debug_cameras_pause` | gea p538 / p582 | **3** | S | Input → `ViewProjectionAuthority`; pause vs sim clock |
| **A8** | `world_streaming` + `world_chunk_formats` | **gea_v2** p589 / p582 | **3** | L | Chunk/load vs existing slab authority |
| **A9** | `rendering_pipeline` + `pipeline_app_layer` + `overlays_ui` / `geometry_visual_effects` | **gea_v2** p121 / p133 / p245 / p141 | **4** | M | Extract after snaps; FireVisualFrame read-sim; app-layer ownership |
| **A10** | `rng` | gea p401 | **3** | S | Seeded determinism |
| **A11** | `profiling` / `debug_tools` | gea p106 / p566 | **2** | S | `sim_review` + validate-report |
| **A12** | `simd` / `gpgpu` / `lock_free` | gea p324+ | **1** | — | Architecture DEFER |

### Ranking B — Niche (this game: factions / logistics / ecology / air / MFG)

| Rank | Keyword(s) | Cite | Impact | Effort | What high-impact looks like *for us* |
|:--|:---|:---|:--|:--|:---|
| **B1** | `decision_making` + `action_execution` | aig p318 / p490 | **5** | M | `MissionIntent` → sole **`MissionBook`** (drones + missiles) |
| **B2** | `ai_lod` + `behavioral LOD` + `multi_tier_ai` + `group LOD` | aig p843–850 · p575 | **5** | M | **Per-faction / per-theater fidelity:** far slots = pressure aggregates + cheap auto-supply; near/player = full MissionBook, fronts, formations — mirrors `ChunkSimLod` |
| **B3** | `perception` / `perception_window` + `sense_management` | aig p44 / p870 | **5** | M | Weather/smoke → sortie gates; read MissionBook modifiers |
| **B4** | `strategy` + `influence_maps` + `terrain_analysis` | aig p506 / p533 / p540 | **4** | M | Front/pressure aggregates — no per-unit micro |
| **B5** | `world_interface` + `event_managers` + `polling_stations` | aig p856 / p859 / p867 | **4** | M | Demand/orders/picker → `LogisticsOrderRequest` only |
| **B6** | `load_balancing_scheduler` + `anytime_algorithms` + `interruptible_pathfinding` | aig p835 / p841 / p298 | **4** | M | Spread AI-faction logistics/combat work across frames; path/orders under budget |
| **B7** | `formation_steering` + `slot_assignment` + `tactical_pathfinding` | aig p164 / p181 / p569 | **4** | M | Haul-amplify on **`LogisticsOrderSim` only** (spine DEFER until playcheck) |
| **B8** | `pathfinding` + `hierarchical_pathfinding` | aig p216 / p278 | **4** | M | Roads / routes / construction graphs only |
| **B9** | `firing_solution` + `waypoint_tactics` | aig p141 / p507 | **3** | M | AirOpZone / magazines / AO hygiene |
| **B10** | `map_flooding` + `tactical_convolution` + `cellular_automata_tactics` | aig p549+ | **3** | L | Ecology/fire/strategic fields — existing pipelines |
| **B11** | `region_sense_manager` / `fem_sense_manager` | aig p878 / p886 | **3** | L | Only if weather LOD needs more than `WeatherField::sample` |
| **B12** | `shape_grammars` / `l_systems` / PCG erosion | aig p697+ | **2** | M/L | MCP/world-gen — not runtime ECS art |
| **B13** | `goap` / `fuzzy_logic` / `mcts` / `learning` | aig p426+ | **1** | — | Rejected greenfield stacks |

### Ranking C — Presentation / sensory (Vol II: anim · ballistics · audio)

**Correction:** these are **product needs**, not “later literacy.” Today: tile/fire/air **frame anim** exists; **ember ballistic** exists; **no** `bevy_audio` / spatial mix. Trucks, buildings, missiles, weather, and combat all need visual + sonic fidelity gated by interest/LOD.

| Rank | Keyword(s) | Cite (gea_v2) | Impact | Effort | What high-impact looks like *for us* |
|:--|:---|:---|:--|:--|:---|
| **C1** | `audio` + `audio_engine_architecture` + `audio_3d_render` + `audio_game_features` | p442 / p502 / p484 / p522 | **5** | L | **Audio stack from scratch:** UI chrome → ambient (rain/wind) → spatial (zoom hear truck / plant / shot) → one-shots (explosion, missile launch) → warning/alert bus. Sole **`AudioMixFrame`** (or named) writer; reads sim events + view pose; **never** writes sim |
| **C2** | `animation_systems` + `action_state_machines` + `animation_pipeline` | p253 / p314 / p313 | **5** | M | **Sprite/tile/vehicle/building motion:** atlas frames, damage/power states, truck facings, building idle/work — extend `tile_visual_state` / air presentation caps; ASM for visual phases. **Not** full skeletal skinning / motion-matching first |
| **C3** | `physics_integration` + `advanced_physics` + `rigid_body_dynamics` (selective) | p423 / p440 / p387 | **4** | L | **Sim ballistics + blast/destruction:** missile/drone flight envelopes, crater/blast on construction/env — deterministic sim truth → presentation reads. Extend ember-ballistic pattern; middleware only if a named collision sole writer is chartered |
| **C4** | `collision_detection` | p359 | **3** | M | AO / footprint / blast radius queries — prefer existing graphs/fields over a full rigid world |
| **C5** | `skeleton_skinning` / `animation_blending` / `motion_matching` | p258 / p285 / p343 | **2** | — | DEFER until hero 3D units exist; sprites/atlases win first |
| **C6** | `audio_math` | p454 | **1** | — | Cite when implementing DSP; not a greenfield crate |

**Audio event classes (target catalog — one ingress shape):**

| Class | Examples | Drive from |
|:---|:---|:---|
| **UI** | click, confirm, deny, panel open | UI intents only |
| **Ambient** | rain, wind, distant city bed | weather / `ChunkSimLod` + camera |
| **Spatial continuum** | truck rumble, plant hum, fire crackle | nearby entities × zoom / `ViewProjectionAuthority` |
| **One-shot combat** | shot, explosion, missile launch | MissionBook / combat resolve events (RO) |
| **Warning / alert** | launch warning, low ammo, weather gate | dumps / explain bus → HUD + audio, no sim mut |

Fidelity follows **§3c/§3d**: far/dormant = bed + rare alerts; near/player theater = spatial + one-shots. Same interest idea as AI LOD — **not** a second audio OS.

### How to use the rankings

1. **Architecture lane** → **Ranking A** (Slice A = A1–A2; fold **A3** AI-LOD cadence after graph is written).
2. **Sim/product lane** → **Ranking B** (Slice B = B1+B3–B5 contracts; **B2/B6** = faction/theater LOD + frame budget).
3. **Sensory / presentation lane** → **Ranking C** (audio C1, sprite anim C2, ballistics/blast C3) — after Slice A graph; share attention/LOD with A3.
4. Spine leftovers (missile catalog, picker, P7, dual-bus) inherit A/B/C — not book ranking rows.
5. Never build a second AI/audio/physics scheduler world; never promote B13 into new ECS because it is “interesting.”
6. Cross-domain uses of the **same idea** → §3c (not only faction AI).

## 3c. Same book idea → other game aspects (cross-domain)

Core pattern from AIG §10 (+ related GEA time/LOD): **fidelity and work rate follow interest and tier** — spend full sim/decision/render budget where the player (or active theater) is looking; degrade elsewhere without inventing a second OS.

We already partially do this in pieces. Books help **unify the policy** across domains so faction AI LOD does not fight ecology LOD, streaming interest, or visual bands.

| Domain | Already have | Book pattern that helps | Concrete upgrade (still single-writer ECS) |
|:---|:---|:---|:---|
| **Ecology / weather / fire sim** | `ChunkSimLod::{Dormant,Normal,Active}` + `dt_scale` | Scheduling LOD · hierarchical cadence | Share **interest** with combat theaters (active front → bump chunk sim LOD); keep ecology as sole writer of chunk weather/fire |
| **Chunk streaming (IO)** | Wave C interest → hydrate → residency budgets | Load-balancing · priority scheduling · interruptible | Treat hydrate queue as anytime work under `STREAMING_*_BUDGET`; priority = camera + active theaters + construction ghosts — not a new task runtime |
| **World / view LOD** | `WorldLodBand`, `TacticalLodBubble`, representation policy | Graphics LOD vs **AI/sim LOD** split (book §10.3.1 vs 10.3.2+) | Keep view bands presentation-only; **do not** let HUD zoom mute sim truth — instead drive **behavioral** cheapness separately (A3/B2) |
| **Fire / atmosphere **extract**** | FireLodBand, fire streaming, atmos page residency, visual cadence | Group LOD · scheduling LOD | Far views: fewer fire instances / coarser atmos pages; near: full extract — already directionally correct; cite books for **budget arbitration** between fire vs atmos vs overlays in one frame |
| **Air presentation (drones)** | `AirPresentationBand`, hero voxel budget, swarm caps | Group LOD · perception window | Swarm = group LOD; “hero” drones = near band — aligns with MissionBook extract; book warns complex AI that *looks* dumb (perception window) → prefer readable few sorties over invisible micro |
| **Logistics / order-sim** | `LogisticsOrderSim`, `AutoSupplyPolicy`, fleet growth | Behavioral LOD · load-balance factions | Far AI factions: batch/cheap auto-supply ticks; player/near: full haul resolve; round-robin faction builds (B6) |
| **Manufacturing / power** | `should_tick` / `dt_scale` on MFG | Hierarchical scheduling | Off-interest plants: slower effective tick (pause-aware); never a second MFG clock |
| **Construction / roads pathing** | pathing + travel on existing nets | Interruptible pathfinding · anytime | Corridor plans yield under frame budget; resume next tick — sole writer stays construction execute funnel |
| **Strategic fields / fronts** | overlays, pressure, front derive | Influence maps · multi-tier · map flooding | Far theaters: update influence/pressure on slow cadence; contested cells full rate — fold into `StrategicFieldPipeline` sets |
| **Hybrid / agent scores** | `HybridSimPipeline`, decision explainability | Anytime · AI LOD | Score/emotion drift at reduced rate when agents off-screen; explain panel reads snapshots only |
| **Multi-faction doctrine** | `FactionDroneDoctrine`, stances, slots | Multi-tier AI · coordinated action | Doctrine/stance = strategic tier; MissionBook = tactical; don’t run both at full cost for idle slots |
| **Player UX / AI explain** | AI explain panel, dumps | Perception window (ch.2) · tools | Show *why* a cheap far-faction looked “dumb” (LOD tier) so operators trust the sim; don’t fake full AI in UI |
| **GEA time / multiprocessor** | `SimStepClock`, `dt_scale`, SCH waves | Abstract timelines · multiprocessor loops | One sim timeline; presentation cadence separate (`VisualCadence`) — books reinforce not mixing clocks |

**Unifying policy (target, not a new crate):** **two** interest frames (sim vs view), not one blob that every layer mutates — see **§3d** for ECS contract.

```text
SimAttentionFrame  (Sim sole writer)  → ChunkSimLod hints · faction/theater AI fidelity · logistics/MFG cadence · path/order budgets
View / WorldLod     (View sole writers) → extract budgets · air presentation · HUD explain (read snaps)
Streaming interest  (IO sole writer)    → hydrate priority; may **read** both frames, never writes sim truth
```

**Explicit non-goals still:** second AI scheduler, per-unit micro at any LOD, view/HUD writing sim, GOAP/MCTS stacks, one god-resource with N writers.

## 3d. ECS contract — interest / fidelity (bevy-simulation-grade)

Book ch.10 patterns must land as **Resources + SystemSets + single writers**, never as a custom scheduler world or shared mutable “AI manager.”

### Layer rules

| Layer | May write | May read | Must not |
|:---|:---|:---|:---|
| **Sim** | Domain truth (`MissionBook`, `LogisticsOrderSim`, chunk weather/fire, strategic overlays, `ChunkSimLod`, construction execute) + **one** `SimAttentionFrame` | Other sim resources; optionally **view pose authority** as a *hint* (not HUD widgets) | Read egui/HUD state; mute sim solely because zoom band is Strategic |
| **View** | `ViewProjectionAuthority`, `ViewManager`, `WorldRepresentationFrame` / lod bands | Sim snaps + `SimAttentionFrame` (RO) | Write MissionBook / order-sim / chunk sim |
| **Render** | Extract/GPU frames only | View snaps + sim visual extracts | Write any sim resource mid-extract |
| **UI** | Intent **messages** (`MissionIntent`, `LogisticsOrderRequest`, viewport requests) | Dumps / explain snapshots | `ResMut` on sim attention or domain truth |

### Single-writer table (target)

| Resource / component | Sole writer (system or set) | Consumers (RO) |
|:---|:---|:---|
| **`SimAttentionFrame`** (new or thin wrap) | **One** system in a named set, e.g. after `AdvanceSimTick` / early `StrategicFieldPipeline` — builds from fronts, active AOs, fire/heat, construction focus, optional camera focus hint | Faction AI cadence, logistics cheap-tick gates, MFG rate, path budgets, `ChunkSimLod` **refresh inputs** (ecology still writes `ChunkSimLod`) |
| `ChunkSimLod` | Existing `chunk_sim_lod_refresh` (ecology/weather/fire inputs; may **read** `SimAttentionFrame`) | Chunk environment dt |
| `WorldLodBand` / `WorldRepresentationFrame` | Existing view representation pipeline | Extract, HUD |
| Streaming interest / hydrate budget | Existing IO streaming systems | Hydrate only |
| `MissionBook` | `apply_drone_mission_intents` only | Air extract, dumps, UI RO |
| `LogisticsOrderSim` | Order-sim plugin writers only | Fleet growth, UI snaps |
| Fire / atmos visual frames | Existing extract sets | GPU |

### Schedule shape (ECS-native “hierarchical scheduling”)

```text
SimControlSystemSet::AdvanceSimTick
  → build_sim_attention_frame          // sole writer · immutable for rest of tick
  → ChunkEnvironment / StrategicField / Logistics / Combat / MFG
       (each .after(attention); Res<SimAttentionFrame>; own ResMut only)
ViewRepresentation / MapCamera / SyncViewManager
  → FireVisualFrame / air presentation  // read view lod + sim snaps; never ResMut sim
UI
  → emit intents only; explain panel reads DroneMissionDump / attention dump
```

Use **named SystemSets** and `.after(AttentionSet::Build)` — never `.after("build_sim_attention_frame")` strings. No `par_iter_mut` on attention; parallel systems may only **read** the frame.

### Fidelity = data, not a second world

| Book idea | ECS form |
|:---|:---|
| Behavioral / AI LOD | `match attention.tier(faction_or_chunk) { Full => …, Aggregate => … }` inside **existing** systems |
| Load-balancing | Deterministic round-robin over `faction_slot` using `SimStepClock` / tick index — still one system |
| Anytime / interruptible | Stateful path/order jobs as components/resources advanced each tick until done; budget field on attention frame |
| Hierarchical scheduling | Different systems in different sets / `run_if` cadences — **not** a nested custom scheduler |
| Group LOD | Operate on `Formation` / AO / packages (already) — never spawn airframe entities for LOD |

### Hard rejects (ECS drift)

- Dual writers on `SimAttentionFrame` or “everyone pushes interest”
- View/UI `ResMut` into sim attention or MissionBook
- New `AiScheduler` / fiber / job-steal crate beside Bevy
- Using `WorldLodBand` as the **only** gate for combat/logistics truth (presentation ≠ sim)
- Per-entity micro AI “because LOD tier is Full”

Slice **A3** must publish this contract (authority map row + set name) before coding B2/B6.

## 3e. GEA Vol II Ed4 — niche / unusual / nuanced (matrix fold-in)

**Catalog:** `gea_v2_e4` — *Game Engine Architecture*, Volume II, 4th ed. (716 pp). Primary cite for chs **11–18** (rendering → gameplay runtime). Prefer **gea_v2_e4** over **gea_e3** wherever both cells hit.

### Vol II–only DEEP hits (no Vol I / AIG outline match)

| Keyword | v2 page | Engine map | Reject |
|:---|:---|:---|:---|
| `motion_matching` | p343 | **DEFER** (C5) — hero 3D only; sprites/ASM first | Second animation ECS / per-agent motion world |
| `stochastic_ray_tracing` | p219 | Lighting literacy for atmos/fire overlays — **read** theory; GPU path stays Bevy | Custom RT sim resource |
| `radiometry_light_transport` | p158 | Same — shapes how FireVisualFrame / lighting *reads* energy | Sim-side radiometry ECS |
| `rendering_equation` / `shading_equation` | p174 / p185 | Shader/material literacy; no new extract writers | Parallel material authority |
| `pipeline_app_layer` | p133 | Clarify **app** vs render-world ownership (Bevy Extract / `extract_app`) | Dual pipeline managers |
| `geometry_visual_effects` | p141 | VFX / blast FX after view snaps | Sim mutation in VFX |
| `overlays_ui` | p245 | HUD/overlays **read** snaps only | Overlay writing MissionBook / attention |
| `animation_post_processing` | p303 | Sprite post (tint/damage overlays) on presentation | Anim writer into sim tick |
| `render_debug_tools` | p248 | Prefer `sim_review` + validate-report over in-engine debug panels | Play-UI debug toggles |

### Shared Vol II SURFACE (also on gea_e3 — use v2 page first)

`rendering_pipeline` p98 · `lighting_post` p153 · `post_processing` p153 · `animation_systems` p253 · `physics_collision` p352 · `audio` / `audio_3d_render` p442/p484 · `world_streaming` p589 · `concurrent_object_updates` p620 · `events_message_passing` p632 · `runtime_object_model` p563 · `game_object_update` p606

**Impact:** A9 = **4** (pipeline + overlays). **Ranking C** elevates **audio (C1)**, **sprite/building/truck anim (C2)**, **ballistics/blast (C3)** — see table above. Skeleton/motion-matching stay DEFER; full Rapier-for-everything rejected unless chartered.

## 4. Session slices (ECS exit criteria)

### Slice A — Engine-wide GEA runtime / schedule hygiene

| | |
|:---|:---|
| **Owner** | `@planner` / `@sim-steward` (whole-graph map, ≤1 session) → `@coder` (bounded set-edge fixes) |
| **Read** | gea_v1_e4 **p47**, **p183**, **p203**, **p505** (+ **p538** if input/authority drift); render/overlay edges → **gea_v2_e4** **p133** / **p245** (§3e) |
| **Authority sources** | `07-repo-authority-map.md` (sim + view spines) · `plan_schedule_sync_v1.md` (SCH codes; Wave 2+ stays **DR-SCHED-W2**) |
| **Write authority** | Docs + optional `SystemSet` comment/edge anchors; **no new resources** unless single writer named in authority map |
| **Do (whole architecture)** | 1) Paste/refresh engine-wide graph: **Sim** (`AdvanceSimTick` → ChunkEnvironment / Transport / Logistics / Combat / MFG) · **View** (`ViewRepresentation` / `MapCamera` / `ViewAuthority` → sole `ViewManager`) · **Render** (`FireVisualFrame` / extract after snaps) · **UI** (read-only + intent ingress). 2) Dual-writer risk list across those layers. 3) Bounded `.in_set` / `.after(SystemSet)` fixes only where edges are wrong or string-anchored. |
| **Exit** | Written **four-layer** graph with one writer per listed resource; known SCH gaps tagged (not silently “fixed”); any touched `src/` → `validate_cargo_report` green |
| **Don't** | New vehicle/mission ECS; haul-amplify; start SCH Wave 2+ without steward; invert UI→Sim; raw `.after("system_name")` strings |

### Slice A3 — Interest → fidelity (ECS-native)

| | |
|:---|:---|
| **Owner** | `@planner` / `@sim-steward` (name resource + set) → `@coder` |
| **Read** | aig p835–850 · §3c–§3d · `07-repo-authority-map` |
| **Write authority** | **Sole** writer of `SimAttentionFrame` (or chosen name); domain writers unchanged |
| **Do** | Add attention build system in named `SystemSet` after `AdvanceSimTick`; wire RO reads into cadence gates; document dual-frame split (sim attention vs view lod); update authority map |
| **Exit** | One writer; consumers `Res` only; validate_cargo_report green; no view→sim mut; no second scheduler |
| **Don't** | God-object interest with N writers; HUD zoom as sole sim mute; GOAP/MCTS |

### Slice C — Sensory (audio + sprite anim + ballistics) — after Slice A; shares A3 attention

| | |
|:---|:---|
| **Owner** | `@planner` (authority rows) → `@coder` / `@designer` (audio UX classes); MCP for SFX assets |
| **Read** | gea_v2 **p442–522** (audio) · **p253/p314** (anim/ASM) · **p423/p440** (physics integrate/advanced) · Ranking C · §3d |
| **Write authority** | Sole **`AudioMixFrame`** (name TBD); presentation anim writers only; **sim** owns ballistic/blast truth |
| **Do** | 1) Charter audio event classes (UI / ambient / spatial / combat one-shot / warning). 2) Sprite/ASM plan for trucks, buildings, missile/air near-band (extend existing tile + air caps). 3) Ballistic/blast: sim resolve → visual+audio consumers; no dual physics world. |
| **Exit** | Authority map rows; no audio→sim mut; anim stays presentation; validate_cargo when code lands |
| **Don't** | Skeleton-first; Rapier for every truck; audio writing MissionBook; play-UI debug SFX panels |

### Slice B — AIG decision / fronts / order-sim messages (under Slice A graph)

| | |
|:---|:---|
| **Owner** | `@coder` (MissionBook, fronts, order adapter) |
| **Read** | aig_e3 **p318**, **p506**, **p856** (optional **p44** if weather→sortie drift) |
| **Write authority** | `MissionBook` / `DroneCombatModifiers` (`drone_missions.rs`); `LogisticsOrderSim` (`orders/plugin.rs`); fronts **read** aggregates |
| **Do** | Tighten intent/message contracts **inside** the Sim layer of the Slice A graph: `MissionIntent` → `MissionBook`; demand → `LogisticsOrderRequest`; front systems read snapshots only; UI remains ingress-only |
| **Exit** | No UI/command path mutates sim truth; lib tests on `drone_missions` / order adapter if touched; no second mission queue resource; still conforms to four-layer map |
| **Don't** | BT/GOAP crate, per-airframe pathfinding entities, picker/missile catalog (asset lane), new presentation writers |

Validate with `validate_cargo_report` / targeted `cargo test -p proc_A_dine01 --lib` — not raw cargo walls.

## 5. Ordered next steps (after this hygiene pass)

| Order | Work | Owner | Notes |
|:--|:---|:---|:---|
| **0** | Index + deep matrix + rankings + A11 re-review + §3c/§3d + **Vol II §3e** + **Ranking C** | **DONE** | Planning only · **158** kw · 4 books · anim/physics/audio elevated |
| **0b** | **Bevy 0.20** PROGRAM CLOSE | `@coder` | **DONE** — book slices unblocked |
| **1** | **Slice A** — Ranking **A1–A2** | `@coder` | **DONE** — four-layer graph + dual-writer list in authority map |
| **2** | **Slice A3** — ECS **`SimAttentionFrame`** + SystemSet (§3d) | `@coder` | **DONE (scaffold)** — plugin + Lod upgrade + AO/fire foci + `tier_for_faction_slot` |
| **2b** | **Slice C** — Ranking **C1–C3** | `@coder` | **DONE (scaffold)** — SFX WAVs + playback, blast funnel, truck presentation ASM; richer art/sprites later |
| **3** | **Slice B** — Ranking **B1 + B3–B5** (MissionBook + perception + order ingress) | `@coder` | **CHECKED** — MissionIntentQueue / order request remain sole ingress; no new dual writers |
| **4** | **Phase FIDELITY-1** — Slice **B2/B6** + sensory productize (§5b) | `@coder` | **DONE** — Bevy 0.20 `SystemParam` groups + launch apply split (§5c); attention gates AI/logistics/MFG/path; sensory stack live |
| **5** | Missile **catalog** + chain | `@coder` + assets | **DONE** — MISSILE-CATALOG-1 (§5d) |
| **6** | Missile **picker** | `@coder` / `@designer` | **DONE** — industrial menu auto-lists `missile_*` |
| **7** | **P7** + dual-bus | `@coder` | **DONE** — `catalog_id_allows_manufacturing_node` + dual-bus note |
| **7b** | **PLAYCHECK-1** — UAS+missile dump/score | `@coder` | **SHIPPED** — [`plan_playcheck_uas_missile_v1.md`](plan_playcheck_uas_missile_v1.md) |
| **8** | **Haul-amplify** | `@coder` | **DONE** — [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md); Formation mul on order-sim only |

## 5b. Next major phase — **FIDELITY-1** (Slice B2/B6 + sensory productize)

**Goal:** `SimAttentionFrame` stops being a scaffold and **gates real work** (faction AI, logistics, MFG, path, audio/spatial, ballistics rate). Sensory stack becomes playable (UI clicks, sortie audio, truck rumble, crater FX read).

**Exit (phase green):**

1. Named consumers `Res<SimAttentionFrame>` with `AttentionTier` branches (not just Lod bump).
2. Far factions = aggregates / cheap tick; near/player = full MissionBook + order-sim.
3. Audio: UI panels enqueue clicks; missile sortie → launch/warning; ambient follows weather.
4. Ballistics: at least one MissionBook/combat path launches flights; craters visible or scored.
5. Truck proxies: atlas/facing or debug sprite; spatial audio tag when zoomed.
6. `validate_cargo_report` 0 errors; targeted lib tests for attention cadence + blast + truck sync.
7. Authority map rows updated for every new sole writer.

**Out of phase (do not pull in):** SCH Wave 2+, haul-amplify, skeleton/motion-matching, full WeatherField rewrite (see `plan_weather_wind_v1.md` — separate lane), GOAP/MCTS.

## 5c. RESOLVED — Bevy **0.20** SystemParam budget on `step_drone_missions_system` (2026-11-14)

**Symptom (misread as schedule bug):** `E0599` on `.chain()` / `.before()`, `E0277` “fn is not a system set”. **Root cause:** rustc arity — function systems accept at most **16 top-level parameters** ([docs.rs/bevy/0.20.0 `bevy::ecs::system`](https://docs.rs/bevy/0.20.0/bevy/ecs/system/index.html) — “All tuples between 1 to 16 elements…”). Same limit in 0.18/0.19; **not** fixed by migration. Nested-tuple packing is the old workaround; **do not prefer it**.

### Forward pattern (Bevy 0.18–0.20, this repo)

| Pattern | Use | Reject |
|:---|:---|:---|
| **`#[derive(SystemParam)]` groups** | `MissionStepWriters` / `MissionStepReads` / `MissionStepQueries` — one arg each; derive folds >16 **fields** internally ([`SystemParam` derive](https://docs.rs/bevy/0.20.0/bevy/ecs/system/trait.SystemParam.html)) | Flat 17+ `fn` args; hand-nested `(a,(b,(c,…)))` “bunk” style |
| **Split by side-effect** | Mission step **enqueues** `MissileSortieLaunchQueue`; `apply_missile_sortie_launches_system` owns `BallisticFlightBook` + `AudioOneShotQueue` | Stuffing more `ResMut` into the mission step |
| **Attention as RO** | `MissionStepReads.attention` → `allows_sortie_pulse` / faction gate | Parallel AI scheduler |
| **Atmos** | `MissionStepReads.atmos: Option<Res<AtmosphereClipmapStack>>` (restored — headroom from derive) | Dropping atmos to “fix” param count |

**0.19/0.20 note:** `SystemParam::get_param` now returns `Result` (validation merged); custom hand-impls must migrate — prefer **derive**, which this engine already uses (`AiWorld`, `ExecuteHydroOpt`, …). Exclusive systems unified in 0.20 (`&mut World` is a normal param) — irrelevant here.

### FIDELITY-1 checklist

1. ~~Param budget~~ — Bevy **0.20** `MissionStepWriters` / `Reads` / `Queries` + `apply_missile_sortie_launches_system`.
2. ~~B2/B6~~ — mission/pressure/faction AI + logistics/MFG/auto-supply + `budget_path_units` → compute heat/nav.
3. ~~C1–C3~~ — UI clicks, sortie audio, truck spatial, blast crater dump, ballistics launch path.
4. Stamp: phase **DONE** when `validate_cargo_report` reports 0 errors (schema-green).

## 5d. Next major phase — **MISSILE-CATALOG-1** (Order 5 → 6 → 7)

**Goal:** Mirror the shipped UAS industrial path for missiles: catalog buildings + chain hops → activation/`ManufacturingNode` → order-sim stock → play picker. Then P7 catalog_id discipline + dual-bus note. **Haul-amplify stays DEFER** until playcheck.

**Template:** UAS (`uas_*` JSON + `industrial_supply_chains.json` `uas_assembly` + industrial menu). Spine exit: [`plan_production_restock_spine_v1.md`](plan_production_restock_spine_v1.md) “Missile plants — DEFER exit”.

**Exit (phase green):**

1. `assets/configs/buildings/missile_*.json` + index rows; chain hops for `missile_body_plant` / assembly.
2. Activation maps `catalog_id` → `mfg_missile_body_v1`; output tag `missile_body` bridges to order-sim / MissilePackage refill (already P6).
3. Play build picker / industrial rail lists missile plants (same gate as UAS — no fake empty buildings).
4. P7: no `ManufacturingNode` without `catalog_id`; dual-bus deployable debt documented or narrowed.
5. `validate_cargo_report` 0 errors; lib/config tests for catalog load + chain role.

**Out of phase:** haul-amplify, SCH Wave 2+, skeleton/MM, new missile entity ECS.

## 6. How to add a new book to the index

1. Drop PDF into `$REF_BOOKS_DIR\`.
2. Append a row to `CATALOG` in `_build_index.py` (rebuild refreshes `catalog.json`).
3. Add `(keyword, title_regex, [book_id, …])` to `KEYWORD_RULES`.
4. Rebuild: `python "$REF_BOOKS_DIR\lookup\_build_index.py"` → `catalog.json`, `keyword_lookup.csv`, `keyword_book_matrix.csv`, `*_toc_summary.json`.
5. Update §3 queue rows only when a keyword maps to an **existing** layer + named writer (engine-wide map, not a silo).

## 7. Rejected (ECS / spine)

- Treating Slice A as “logistics/combat only” — **must** cover Sim→View→Render→UI
- Re-OCR / full-text PDF extraction
- Porting GEA C++ allocators or custom memory pools into Rust ECS
- Greenfield behavior trees / GOAP / second `VehicleFleet` or drone entity micro-sim
- Book-driven **learning**, **SIMD**, or **GPGPU** chapters → new engine subsystems
- Using ref books to justify **haul-amplify** before playcheck (spine DEFER stands)
- Starting **SCH Wave 2+** / fire-authority rewrites without **DR-SCHED-W2** steward gate
- Multi-writer or UI-driven **interest** blobs; using `WorldLodBand` as the only sim gate; custom AI scheduler beside Bevy SystemSets (§3d)