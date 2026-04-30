## Designer Questions & Next Steps v1 — **LEGACY MONOLITH**

> **⚠️ LEGACY (2026-04-26):** Prefer **split designer folders** under `prompts/designer_questions/<subsystem>/` (see each `README.md`). This file remains the **full archive**; new decisions go into subsystem docs + `implementation_questions_v1.md`.
>
> **Related:** `prompts/llm_agent_brief.md` · `prompts/README.md` · `prompts/matrix/README.md`

Date created: 2026-04-26
Last updated: 2026-04-26 (round 4 — prompts reorg + LOD/chunks/pathing/hydro closeouts)
Audience: project owner / lead designer
Goal: unblock both **prototype game engine** and **tooling** (world gen, asset handling, building) by collecting decisions on ambiguous areas.

Format:
- ~~struck-through~~ items are **resolved**; the answer is captured immediately below.
- Open items still need a decision; any new sub-questions are nested.
- Resolved entries also link the migration matrix / file that captures the decision.

---

## A. Top-level scope

1. ~~**Genre + scale**: city-builder / RTS / sim / sandbox? Single map size target (km², tile count)?~~
   - **Genre:** Logistics-warfighting simulation. Town / city building, attrition focus, organic supply chains, more complex than abstract line-war + logistics.
   - **Scale (open):** map size in km² / tile count still TBD. Plan-for-chunking is locked (see Q9), so any single hard cap is a tooling concern, not a runtime ceiling.
     - **A1.a:** ~~target chunk size~~ → **⏳ provisional 256×256 tiles** — confirm in `prompts/designer_questions/terrain_world/chunks_streaming_v1.md`.
     - **A1.b:** ~~max loaded chunks~~ → **⏳ provisional: 8 neighboring chunks + center (Moore on chunk grid)** — confirm count/radius in same file.
     - **A1.c:** "default scenario" world size (single sandbox map) — TBD.
2. ~~**Player perspective**: 2D top-down, 2.5D iso, full 3D?~~
   - **2.5D isometric primary**; minor non-iso rendering allowed for VFX / cinematic specials.
   - **A2.a:** orthographic vs perspective camera w/ iso projection? (affects shadows, parallax, future 3D dips). TBD.
   - **A2.b:** tile sprite size / art pipeline — **tolerant of mixed quality & future expansion**; see `prompts/designer_questions/terrain_world/tile_sprites_v1.md`.
3. ~~**Singleplayer-only or multiplayer**? Determinism requirements?~~
   - **Multiplayer is in scope** — not an afterthought. `systems/agents/multiplayer.rs` will be exercised.
   - **Session model (locked):** **Persistent world**, not match-based. Players **log into a shared world**, may belong to **their own faction or another player’s faction**, **build over long sessions**, and **log out** while the sim continues. Pace is **slower than a twitch RTS**; emphasis is **simulation and logistics**, not frame-perfect micro.
   - **Authority (locked):** **Server holds ground truth.** Clients receive state updates and send intents/commands; **full bit-for-bit determinism across clients is not a goal.** Small, bounded divergence (e.g. presentation, interpolation) is **acceptable by design** as long as it stays **non-game-breaking** and **does not undermine server authority** on contested outcomes (combat, economy, ownership).
   - **What this rules out (for now):** lockstep / replay-identical clients, “same inputs → identical hash” as a hard requirement. If a future mode needs deterministic replay, treat it as a **separate track** (server-side recording or deterministic server-only replay), not a constraint on live MP.
   - **A3.a:** ~~authoritative server vs lockstep vs hybrid?~~ **Authoritative server** for live play; lockstep **not** required for the main product.
   - **A3.b:** ~~strict bit-determinism or server-of-truth + prediction?~~ **Server-of-truth**; **light client prediction optional** where it improves feel (e.g. camera, local UI), with **reconciliation** to server state. No mandate for strict cross-client determinism.
   - **A3.c:** netcode tick rate; sim tick decoupled from render? **Pending** — should align with slow-sim pacing (see Q11). Likely **fixed sim step** on server, **variable render** on client.
   - **A3.d:** max concurrent players per world / per region (shard design)? **Pending.**
   - **A3.e:** reconnect & late-join — full state snapshot on join, chunk stream, or hybrid? **Pending.**
   - **A3.f:** anti-cheat / validation strictness — server validates all placement & economy commands; how much physics on server vs trusted client? **Pending.**
   - **A3.g:** interest management — AOI per player/faction, chunk subscription, bandwidth budget? **Pending** (ties to B9 chunking).
   - **A3.h:** offline / single-player fork of same world binary, or MP-only binary with local server? **Pending.**
   - **A3.i:** persistence — world runs 24/7 on dedicated host vs player-hosted “listen” server? **Pending.**
4. ~~**Save format**: JSON (current), RON, binary, mixed? Snapshot vs delta vs replay log?~~
   - **More efficient than JSON-everywhere.** Direction: binary snapshots + small RON / JSON metadata header; snapshot vs delta vs replay still TBD. **Matrix:** `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`.
   - **A4.a:** binary serializer choice — `bincode`, `postcard`, `rkyv`, custom packed? TBD.
   - **A4.b:** snapshot, delta, or replay-log primary save mode? Hybrid? TBD.
   - **A4.c:** save-file schema versioning + migration scheme (must align with Q28). TBD.

## B. World & terrain

5. ~~**Tile model**: hard tiles vs continuous height/biome blend (`BiomeWeights`)? Both?~~
   - **Continuous blend.** Authoritative model is `BiomeWeights` + `TerrainSurfaceMix` (see `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`). Hard "TerrainClass" remains a derived/quantized view for rules/UI snapping.
6. ~~**Heightmap source of truth**: noise-only, or hand-edited overrides via tools?~~
   - **Noise-only is the source of truth**, but hand-editable via tools layered on top (paint/raise/lower/flatten brush, deterministic re-noise on seed change).
   - **B6.a:** override storage — sparse delta map keyed by chunk coord, or stamped back into noise params? TBD.
   - **B6.b:** ~~brush set / undo~~ → **small initial set + expand later + undo chain** — `prompts/designer_questions/terrain_world/terrain_tools_brushes_v1.md`.
7. **Region generation**: keep all 6 Voronoi variants or pick 1–2? (`terrain/voronoi*.rs`.)
8. ~~**Rivers/lakes algorithm**: replace stub random walk with hydrology sim (flow/erosion)?~~
   - **Yes — full hydrology sim** (flow accumulation, erosion, lake basins). Stub random walk is retired.
   - **B8.a:** **Worldgen = detailed hydrology.** **Runtime = lighter flow** + **deep sim on triggers** (dam breach, earthworks, flood/drain). Detail: `prompts/designer_questions/terrain_world/hydrology_v1.md`.
   - **B8.b:** integration with biome moisture (feedback into `BiomeWeights`)? ⏳ TBD (same file).
9. ~~**Streaming/chunking**: do we need partial worlds, or single in-memory map for prototype?~~
   - **Plan for chunking from day one — worlds will be big.**
   - **B9.a–d:** **Chunk/streaming + sim partition + save granularity** — `prompts/designer_questions/terrain_world/chunks_streaming_v1.md`; **sim/render LOD** — `prompts/designer_questions/terrain_world/simulation_lod_v1.md`.

## C. Production & economy

10. **Manufacturing list to scope first**: concrete + aluminum + power exist. Steel? Food? Fuel? Electronics?
11. **Production tick rate**: per-frame vs fixed time step (e.g., 1 sim-second)?
12. **Resource flow**: `ProductionChain` exists in `core/production_utils.rs` — is it the canonical model or a placeholder?
13. ~~**Power grid**: do we want full topology sim (substations/transformers/edges) or just zone-based supply?~~
   - **Full topology sim.** Transformers, substations, edges, islanding, brownouts are gameplay first-class. Targeting and repairing the enemy grid (and your own) is core.
   - **C13.a:** **Include frequency, AC vs DC, 1φ / 3φ** in model & **tech/cost progression** — full equation fidelity ⏳; see `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`.
   - **C13.b:** per-edge capacity & failure modes (overload, sabotage, weather) — full set vs gameplay-tier subset? TBD.
   - **C13.c:** simulation cost ceiling per tick for a large grid (perf budget)? TBD.
   - **C13.d:** **Charts/graphs** (flows, power, resources over time) + **tiered, sortable, dismissible alerts** — `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`.
14. ~~**Damage/maintenance**: should `DamageState` tie into production, construction, vehicles?~~
   - **Damage + maintenance present everywhere.** Trucks, ships, rail are top priority for the first pass; buildings/grid/production follow.
   - **C14.a:** maintenance economy — consumes resources/labor, produces downtime, both? TBD.
   - **C14.b:** repair entities (workshops, depots, mobile crews)? TBD.
   - **C14.c:** ~~split tracks?~~ → **One uber damage state** (wear + stress + combat); gradients & component breakages — `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`.

## D. Vehicles & navigation

15. **Vehicle scope first**: trucks (have config), buses, trains (`Train` enum), ships? Order to implement?
   - **Per Q14**: trucks, ships, rail are top maintenance priority. Implementation order should follow.
   - **D15.a:** confirm order: Trucks → Trains/Rail → Ships → Aircraft? TBD.
16. ~~**Pathfinding**: A* per-vehicle, flow fields, or hierarchical?~~ → **Hybrid hierarchical** (network + flow + refine; handoff by context + LOD). See `prompts/designer_questions/navigation/pathfinding_hierarchical_v1.md`.
17. ~~**Road graph**: separate ECS graph entity or terrain-tile annotations?~~
   - **Separate graph entity (own ECS resource + per-edge components).** Road graph is more complex but lower-count than tiles, so shouldn't ride on the tile grid.
   - **D17.a:** **Rail ~ road graph abstraction but different rules** (signals, blocks, no passing) → separate type or heavy edge metadata; **optimize differently** — `prompts/designer_questions/navigation/pathfinding_hierarchical_v1.md`.
   - **D17.b:** intersections / interchanges — node types or sub-entities? TBD.
   - **D17.c:** dynamic edits (build/destroy roads) — incremental graph updates or rebuild? TBD.
18. **Cargo logistics**: explicit pickup/dropoff entities or implicit from `ProductionChain`?

## E. Agents / factions

19. **Agent definition**: human players, AI factions, sub-units (workers)? Currently `permissions.rs` mixes all three.
20. **Permission system**: keep current per-domain access levels, or switch to tags/roles?
21. ~~**Faction count**: max 6 (current `FactionColors`) or dynamic?~~
   - **Dynamic factions.** Color generated procedurally (HSL hue rotation), traits parameterized — no hard cap, no fixed palette.
   - **E21.a:** **Traits = placeholder**; expect **tag-oriented** flexibility — finalize in `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`.
   - **E21.b:** **Per save or scenario**; if no preset — **build from rules**. **Faction editor tool** required — same file.
   - **E21.c:** UI — faction creator panel + color preview (egui tooling); HUD swatches use the same hue. TBD.

## F. UI split (already documented in `prompts/guides/ui_boundary_guide_v1.md`)

22. **In-game HUD priorities**: resources, build menu, minimap, alerts — order? ⏳ Partial: **analytics charts + tidy alert feed** — see `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`.
23. **Editor priorities**: world gen tuning, building placer, vehicle inspector, agent permissions, save/load — order? ⏳ Include **faction editor** (`power_damage_ui_persistence_v1.md`).
24. ~~**Should `bevy_feathers` (0.18 native widgets) be adopted now** for in-game UI buttons/panels, or stay on raw `Node`?~~
   - **Use native Bevy where it fits — Bevy UI is now in good development state.** Egui stays for tooling/inspectors. Raw `Node` is fine where `bevy_feathers` adds churn; opt into `bevy_feathers` when a component needs themed widgets.
   - **F24.a:** confirm the UI boundary table in `prompts/guides/ui_boundary_guide_v1.md` matches this rule (no egui in in-game runtime). TBD audit.

## G. Asset pipeline

25. ~~**Asset folders**: `data/`, `src/data/`, `assets/` all exist. Pick canonical layout?~~
   - **Resolved 2026-04-26.** Canonical Bevy layout adopted:
     ```
     assets/
       configs/
         buildings/        ← was src/data/game_entities/buildings/
         vehicles/         ← was src/data/game_entities/vehicles/ + data/assets/vehicle_configs.json
         production/       ← was data/assets/production_configs.data
       fonts/              ← was data/assets/fonts/ + data/times.ttf
       textures/
         power/            ← was data/textures/power_stuff/
         tiles/            ← was data/textures/tiles_single/
         vehicles/         ← was data/textures/vehicles/
         misc/             ← was data/assets/unsorted_asset_tile_maps/
       splash/
         splash_01.png     ← was data/splash_01.png
       tiled/              ← was data/assets/tiled/
       game_entities/      ← was data/game_entities/ (constants)
       data/               ← .dat / .const fallbacks for std::fs::read consumers
     schemas/
       flatbuffers/        ← was data/flatbuffers/  (build-time only)
     ```
   - Code references updated: `splash.rs`, `gui_assets.rs`, `main_menu.rs`, `io/serialization/deserializers.rs`. `data/` and `src/data/` removed.
26. ~~**Asset format per type:** vehicles → JSON, buildings → JSON, terrain → ?, shaders/textures → ?~~
   - **Direction (locked):** review what we have, pick best, apply to all sample + new types.
   - **Recommendation (open for confirmation):**
     - **Configs (buildings, vehicles, production, factions):** **RON** — comments, schema-friendly, native Bevy reflection support; JSON kept transitionally.
     - **Textures:** **PNG** for sprites, **KTX2/Basis** for compressed tile atlases when size matters.
     - **Tilemaps:** Tiled `.tmx` already in tree; keep.
     - **Save snapshots:** binary (per Q4).
     - **Schemas (flatbuffers, JSON-Schema-ish):** under `schemas/`, build-time only, not asset-server-loaded.
   - **G26.a:** confirm RON vs JSON for hand-editable configs — track in `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md`.
   - **G26.b:** texture format pipeline — manual PNG, basis-universal pre-bake, runtime? TBD.
27. ~~**Hot-reload during dev**? Bevy supports it — do we wire it for tools?~~
   - **Yes, hot-reload on for dev tools.**
   - **G27.a:** restrict to dev profile only (gate via cargo feature)? TBD.
   - **G27.b:** which asset categories opt in (configs, textures, tilemaps, all)? TBD.
   - **G27.c:** how does hot-reload interact with the chunk loader (Q9)? TBD.
28. ~~**Asset versioning**: schema version field per asset, migration scripts?~~
   - **Yes — every asset / save schema gets a version field.** Direction is locked; specifics open.
   - **G28.a:** version key location — top-level `version: "1.2.0"` field, separate `schema.ron` per dir, or both? TBD.
   - **G28.b:** migration runner — at load time, offline tool, both? TBD.
   - **G28.c:** policy for breaking changes — bump major + emit migration notes in `prompts/`? TBD.

## H. Open codebase questions (need a yes/no to retire legacy)

29. ~~`src/engine/lmodels/*` (fastICA, kernel systems) — research code, ML hooks? Keep, isolate, or delete?~~
   - **Keep — research code for future agent development**, in-game faction controls, behaviour models. Stays isolated under `engine/lmodels/`; not wired into the active runtime yet.
   - **H29.a:** isolate behind a `research_lmodels` cargo feature so it doesn't pull `linfa*` deps into the default build? TBD.
   - **H29.b:** add a `LEGACY MODULE (not actively wired)` header to its files so `build.rs` ignores them, until they're refactored to current API? TBD.
30. **`src/utils/flatbuffer_rust_loader/` and `src/utils/tilemapgen/`** — still relevant?
   - Note: flatbuffer schemas now live under `schemas/flatbuffers/` (Q25). The loader util's status (still needed? rewrite? delete?) is open.
31. ~~`src/core/id_generator.rs` vs `src/idgen.rs` — which is canonical? (Both have `EntityId`.)~~
   - **Resolved 2026-04-26.** `src/idgen.rs` is canonical (`EntityId(u32)`, atomic, `Serialize`/`Deserialize`). `src/core/id_generator.rs` deleted. `core::mod.rs` carries a retirement note. `entities/entity.rs::EntityInfo::new` rewritten to use `EntityId::new()` directly (no mutable counter struct passed around).
32. ~~`src/io/templates.rs` — gated behind feature flag now; rewrite or delete?~~
   - **Resolved 2026-04-26.** **Deleted.** Referenced removed modules (`crate::io::deserialzers`, `crate::road_vehicles`) and an undefined `RoadVehicleConfigResource`. Active loading is in `crate::io::serialization::deserializers`. The `legacy_io_templates` cargo feature was removed from `Cargo.toml`; the `io/templates.rs` skip entry was removed from `build.rs`.

---

## Next steps — minimal viable prototype path

Two parallel tracks once the above decisions land.

### Track 1 — Engine prototype (in-game runtime)

Order matters; each step should produce a runnable demo.

1. **Make `cargo check` clean on Bevy 0.18** (apply small fixes surfaced after lock).
2. **Splash → Main Menu → Simulation state flow** end-to-end (already 80% wired in `main_menu.rs`, `splash.rs`, `transitions.rs`).
3. **Spawn a minimal world**: single biome, flat heightmap, fixed seed — verifies `WorldData` end-to-end.
4. **Camera + 2.5D iso renderer** (per Q2) for tiles + biome-blended colors.
5. **HUD stub showing one resource counter** driven from `ProductionRuntimePlugin` (already has `in_game_hud.rs` skeleton).
6. **One full production chain**: place concrete plant → consumes water+limestone → produces concrete; visible in HUD.
7. **One vehicle moving on a hardcoded road** between two buildings (uses the Q17 separate road-graph entity).
8. **Save/load round-trip** of `WorldData` + production state (binary snapshot per Q4).

### Track 2 — Tools (editor / world-gen)

1. **World generator tooling UI** (already exists, needs ctx/Result wiring tested at runtime).
2. **Noise stack editor** — per layer: kind, frequency, octaves, seed offset; live preview.
3. **Biome painter** overlay on the preview texture (writes to the Q6 hand-edit override layer).
4. **Hydrology preview** — show flow + lake basins from the Q8 sim.
5. **Building blueprint inspector** — read `assets/configs/buildings/*`, edit, save (with hot-reload per Q27).
6. **Vehicle config inspector** — edit `assets/configs/vehicles/vehicle_configs.json` live.
7. **Production manifest browser** — read `ProductionManifest` resource, show domain entries.
8. **Power-grid editor** — single-line diagram + topology graph (Q13).
9. **Faction creator** — hue picker + trait toggles (Q21).
10. **Agent permissions editor** (already exists, needs decisions in §E).

### Cross-cutting

- Add `cargo check`/`cargo build` CI hook running on every commit (build.rs already enforces banned imports).
- Add a single `prompts/INDEX.md` reading order (already in `prompts/README.md`).
- Define the **first playable scenario** (10-minute demo): "build one factory, deliver one resource, open the editor and tune noise, spin up two factions" — drives priorities.
- Pick chunking dimensions (Q A1.a / B9.a) before any large-world tooling lands; affects save format too.
- **Multiplayer / netcode assumptions** — see **§A3**: persistent world, **server ground truth**, **no cross-client full determinism**; protocol, AOI, and hosting choices stay **pending** (A3.c–A3.i) until sized.

---

## How to use this document

- Cross out resolved questions with answers inline (~~strikethrough~~ + indented bullet with the locked decision).
- New decisions → add to the relevant `prompts/matrix/<subsystem>/…` doc **and** the topical `prompts/designer_questions/<subsystem>/…` (design + `implementation_questions_v1.md`), not only this legacy file.
- **Single intake** for agents: `prompts/README.md` → pick one designer file + one matrix.
