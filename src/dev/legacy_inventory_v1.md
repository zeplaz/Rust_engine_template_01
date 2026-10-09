# Legacy inventory v1 — fleet `legacy` (2026-10-08)

Classification pass (cleanup-completion-intelligence vocabulary) of old, dormant, half-wired and
confused systems, config files and repo-root loose files. Cross-refs: `codebase_index_v1.md` (module
codes), `plan_cleanup_v1.md` (R#/S#/P#/T#/D#), `fleet_charter_20261008_v1.md` (owners).

**Verdicts:** `wire-in` (finish + register) · `migrate` (move to the current authority) · `park`
(keep, explicitly marked dormant, with an unblock condition) · `archive` (move out of the live tree,
keep history) · `delete-candidate` (only after a classification packet + owner/user OK).

**Classes:** OBSOLETE (superseded, no live role) · TRANSITIONAL (scaffold on the way to an
authority) · DORMANT (complete-ish but not registered/used) · INCOMPLETE (registered, but behaviour
is a stub).

Status column: ● done in a `[legacy]` checkpoint · ◐ in progress / partially · ○ open (board ask or later slice).

---

## A. Owned code — `src/infrastructure/`, `src/bevysubengines/`

| ID | Item | Evidence | Class | Verdict | Owner | St |
|----|------|----------|-------|---------|-------|----|
| LG-01 | Transport + utility networks missing from the Wave S world save | `pipeline.rs` manifest overlays listed settlement / fire / atmosphere only; `write_transport_snapshot_ron` was only called from a live proof; `UtilityNetworkSnapshot` had no save path at all → player roads, rails, corridor construction rows and power lines were lost on save/load | INCOMPLETE | wire-in: `transport_r8` + `utility_network` overlays (write before flush, hydrate once per bundle dir, same pattern as settlement books) | legacy (`src/io/save/{transport,utility}_overlay.rs`, `pipeline.rs` — no other fleet owns `src/io`) | ● |
| LG-02 | `TransportGraph` resource (IF-TRG) is never populated at runtime | Only `init_resource` in `InfrastructureTransportPlugin`; every runtime writer (construction pipeline, history, map editor, play scenario, logistics routes) writes `TransportTopology`/`TransportEdgeDirectory`. Readers that see the empty graph: `strategic/settlement/block_frame.rs` (junction lookup), `systems/transport::transport_topology_tick` (no-op) | INCOMPLETE (authority in name only) | migrate: make the graph a **derived mirror** of the directory (rebuild on `Changed<TransportEdgeDirectory>`), and change `transport_topology_tick` so it does not clear `TransportFieldStore` every frame when the graph is non-empty (P5). The tick change is logistics-owned → board | legacy (mirror) + logistics (tick) | ○ |
| LG-03 | `CorridorAuthoringSession` / `confirm_corridor_authoring` (INFRA-E2-001) | Resource initialised, nothing outside its own tests reads it; confirm path hard-codes a `RoadProfile` (2 lanes, 50 km/h, `truck`) and ignores the loaded `ProfileRegistry` | DORMANT + INCOMPLETE | wire-in the profile lookup now (resolve turn radius / allowed agents from `ProfileRegistry`); map-editor tool attachment is tooling's | legacy / tooling | ○ |
| LG-04 | `attach_settlement_to_nearest_transport_node` (INFRA-E5-001, T7) | Distance is `origin.length()` for every node ("coarse stub"), so it returns arbitrary keys; no callers outside tests | INCOMPLETE + DORMANT | wire-in: measure distance to edge endpoints (`control_points` first/last), keep API | legacy | ○ |
| LG-05 | `ProfileRegistry` path is compile-time `env!("CARGO_MANIFEST_DIR")` | `profiles/mod.rs::load_default_example`; same pattern in all terrain registries (`material_plugin.rs`) → a binary run outside the source tree finds no config; terrain registries `.expect()` → Startup panic | TRANSITIONAL | migrate: shared runtime config-root resolver (env override → exe-relative `assets/` → manifest dir fallback) | legacy (profiles + resolver); terrain loader adoption → board | ○ |
| LG-06 | `ProfileRegistry` under-used | Registry consulted only by `construction/corridor_transport.rs` (allowed agents); `corridor_class_from_profile` string heuristic still decides road/rail everywhere else | TRANSITIONAL | migrate call sites incrementally (logistics/construction) | logistics / interaction | ○ |
| LG-07 | `UtilityNetworkSnapshotResource::default()` = test fixture | Runtime starts with a phantom 3-node network (`sub_a`, `sub_b`, `plant_1`) at hashed pseudo-positions (`node_position_for_key`) | TRANSITIONAL scaffold | migrate: start empty (or scenario-seeded) once power HUD / activation witnesses stop depending on the fixture; now that LG-01 saves the network, a real seed can come from a save/scenario | legacy + logistics (econ activation) | ○ |
| LG-08 | `merge_utility_power_into_infrastructure_graph` runs every frame | Unconditional retain + re-push of power nodes/edges into `InfrastructureGraph` each `Update` | INCOMPLETE (perf) | change-gate on `UtilityGraph` / `PowerLineDamageBook` change — only after confirming who else rewrites `InfrastructureGraph` each frame | legacy | ○ |
| LG-09 | `WorldGeneratorSubenginePlugin` (BN-SUB, R9) | Never `add_plugins`'d (only `lib.rs` re-export); own `WorldGenParams` duplicates `terrain::generation::world_generator_enhanced::WorldGenParams`; its `WorldData` save format, `export_world_data_binary`, `convert_to_game_engine_format` have zero callers; `#[deprecated] BiomeType`; two `include_str!` guard tests pin its run-condition text (`world_gen_bevy_shell.rs:1831`, `hud/shell_diagnostics.rs:233`) | OBSOLETE (dormant-reference) | park now (module header marks it parked); R9 delete after the save-format knowledge is captured and the two guard tests are retargeted (graphics/tooling files) | legacy | ◐ |
| LG-10 | `infra_e0_003` / feature `legacy_transport_ecs_stubs` | `entities/structure/legacy_transport_stubs.rs` gated off by default; witness asserts it stays off (R8) | OBSOLETE | archive with R8 (entities area) | hygiene / legacy | ○ |

## B. Owned config — `assets/config/`

| ID | File | Consumer (runtime / test) | Class | Verdict | St |
|----|------|---------------------------|-------|---------|----|
| LG-20 | `infrastructure/corridor_profiles.example.ron` | runtime `InfrastructureProfilesPlugin` (+ tests) | live | keep; path → LG-05 | ○ |
| LG-21 | `power/plant_definitions.json` | runtime, `include_str!` in `entities/production/power/plant_registry.rs` (rebuild to change); generated by `utils/gen_plant_definitions_json.py` | live (JSON by design: Python generator + editor) | keep JSON per AGENTS.md interchange rule | — |
| LG-22 | `terrain/*.example.ron` vs `*.example.json` (family, tag, material registry) | 4 loaders prefer RON, but `terrain/material/rules.rs:42`, `systems/fire/surface_water.rs:71`, `terrain/generation/p4_hydrology.rs:90` hard-code the **.json** → both formats are live and can drift | TRANSITIONAL (RON migration half done) | migrate the three hard-coded readers to the prefer-RON helper; then JSON twins become test fixtures | ○ (board: terrain/effects) |
| LG-23 | `terrain/profiles/default.world_profile.ron` | `WorldProfileLoader` registered + `WorldProfileSelector` initialised, nothing loads a profile, `apply_profile` never called (U7) | DORMANT | park: wiring needs registry files renamed to the loader extension convention (`<name>.material_registry.ron` …) — the `*.example.ron` names it points at match no registered loader | ○ |
| LG-24 | `world_gen_tuning.example.{ron,json}` | test + Python tool only; runtime overlay reads `assets/config/world_gen_tuning.{ron,json}` (CWD-relative, optional, never committed) | TRANSITIONAL by design | keep; document that copying the example activates it; resolve path via LG-05 resolver | ○ |
| LG-25 | `terrain/material_registry_minimal.example.ron` | test-only fixture | live (fixture) | keep | — |
| LG-26 | `terrain/material_registry.example.json` | fallback only; tests | TRANSITIONAL | becomes fixture after LG-22 | ○ |
| LG-27 | Unreferenced examples outside `assets/config`: `assets/configs/buildings/visual_config_warehouse_industrial_west_v2.example.ron`, `assets/staging/specs/tile_batch_landscape_expanded_v1.example.json`, `tools/mcp/schemas/examples/roof_pitched_lod0.example.json` | no consumer | DORMANT | tooling to adopt in a schema test or archive | ○ (board: tooling) |

## C. Repo-root loose files (user-authored — never deleted by fleets)

| ID | File | What it is | Intent the code is missing | Verdict |
|----|------|-----------|----------------------------|---------|
| LG-40 | `buildlog.txt` | 2024-era failed `cargo build` log (76 errors: `pub pub mod`, missing `entities::production`, `EventWriter`, …) | none — superseded by the current green build | archive → `docs/archive/` (user OK) |
| LG-41 | `engine_model_plan.md` | early engine sketch: GUI thread → dispatch stack → render/engine; error path *log → recover → popup → try save-and-shutdown*; audio system + receiver; parallel I/O receivers | (a) **no audio** anywhere in `src/`; (b) no engine-level error-recovery path (panic hook → user popup → emergency save); only `preview/assembly_worker.rs` sets a panic hook | keep; intent recorded here as product gaps (G-AUDIO, G-RECOVER) |
| LG-42 | `evning and other notes for progress` | personal, non-engineering note | none | leave untouched; owner may prefer to keep it outside the repo |
| LG-43 | `gptpromot_draft.txt` | prompt draft: generic, reusable systems; stay on latest Bevy; `bevy_egui` + `bevy_ecs_tilemap`; reflection-driven auto-UI (`#[reflect]` → egui inspector) | `bevy_egui` ✓; `bevy_ecs_tilemap` ✓ (opt-in feature `bevy_tilemap_adapter`); **reflection-driven inspector** ✗ (`bevy-inspector-egui` commented out in `Cargo.toml`) | keep / archive (user); inspector gap → tooling |
| LG-44 | `migrate.sh` | 2024 reorg script: `mv src src_old; mv src_reorganized src` | none — `src_reorganized/` no longer exists; running it would fail midway after the backup copy | archive (user OK); OBSOLETE |
| LG-45 | `model.html` | saved 2023 Cloudflare "Worker threw exception" error page (contains network metadata) | none | delete-candidate (user decision); OBSOLETE |
| LG-46 | `REORGANIZATION.md` | 2024 reorg description (core/, engine/lmodels, entities/types, utils/events) | mostly realised; stale tree (no `src/infrastructure`, `src/strategic`, `src/construction` …) | archive with LG-44 |
| LG-47 | `the_remnete_temple_pligm_rite.md` | lore fragment: semi-secret temple sites that became pilgrimages; old games with gods; runes reaching civilisations | world-lore seed — no lore/landmark system in code | keep; candidate seed for a lore/landmark doc |

## D. Elsewhere (reported to owning fleets)

| ID | Item | Owner |
|----|------|-------|
| LG-60 | Stray git worktree `.claude/worktrees/loving-mestorf-ab9564` (detached `d96eccb3`) — violates `single_worktree_policy_v1.md`; duplicates every grep hit | hygiene / tooling |
| LG-61 | P5: `transport_topology_tick` clears + rebuilds `TransportFieldStore` from the graph each frame (would wipe congestion/damage once LG-02 populates the graph) | logistics |

Plugin / dead-code sweep results (repo-wide) are appended in §E when the sweep lands.

## Product gaps surfaced from old notes

- **G-AUDIO** — no audio system (`engine_model_plan.md`).
- **G-RECOVER** — no crash → popup → emergency-save path (`engine_model_plan.md`).
- **G-INSPECT** — no reflection-driven inspector (`gptpromot_draft.txt`).
