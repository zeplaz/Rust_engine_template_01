# Fleet handoff summary — 2026-10-09 (v1)

| | |
|:---|:---|
| **Date** | 2026-10-09 |
| **Baseline** | `374b0494` (checkpoint of prior WIP, 2026-10-08 23:10) |
| **HEAD at writing** | `5d5c54b9` (6 fleet commits on top of baseline, all dated 2026-10-08) |
| **Build** | `cargo check --lib` **green** (39 s, finished, 0 errors) on the **working tree as it stands**, i.e. including uncommitted work. It was not checked against a clean `HEAD` checkout, so some uncommitted files may be required for HEAD to compile (see section 3). |
| **State** | All 11 fleets stopped by the user. Branch `master`, single canonical checkout, no side branches. |
| **Resume** | Read, in order: [fleet charter](fleet_charter_20261008_v1.md) (ownership + commit protocol), [fleet board](fleet_board_20261008_v1.md) (asks + user priorities), then this file. Then the per-fleet logs in [`fleet_logs/`](fleet_logs/) and the plan docs in section 5. |

Evidence base: `git log 374b0494..HEAD`, `git status --porcelain` (240 entries), the four existing logs, plans and ledgers, and file contents on disk. Anything not backed by a commit or a file is marked "unknown".

## 0. Headline

The run lasted about one hour and ended early (usage limit hit with 11 concurrent sessions, per the caller's account; not provable from the repo). Only **4 of 11 fleets committed anything** (`logistics`, `front`, `ai`, `hygiene`, plus the orchestrator's charter/board commits), and every one of those committed **plan documents or config only, no code**. Substantial **code is on disk but uncommitted**: logistics order sim (about 2,000 lines), faction AI (about 1,100 lines), combat/front (about 1,000 lines plus RON balance), interaction picking (about 500 lines), legacy save-overlay wiring, MCP CLI/test guard. `tooling`, `effects`, `graphics`, `drain`, `tests` have **no commits, no logs, and no identifiable files on disk** (some may be hidden inside generated churn; unknown).

## 1. User priorities (verbatim intent)

1. **Interaction (P0):** cursor/pointer stability across the world and menus must be smooth and correct. One authoritative pointer resource.
2. **Effects (P0):** fire and sparks/embers must be realistic and correct: plausible physics, sim-coupled, metric scale, proven with `VFX_CAPTURE`.
3. **Drain:** queued trees/vegetation work moves to the front: `coder_vegetation_drain_queue.json`, `landform_veg_drain_v1.json`, `parallel_wave_aps_veg_*` (all in `tools/orchestrator/queues/`).
4. **Logistics design:** distinguish **continuous supply lines (standing)** from **one-off orders** that use available trucks and repeat trips until fulfilled, then complete; heavy emphasis on **draw demands** (inspired by a MarsTactics developer write-up; the write-up itself could not be found, the plan is built from the user brief).
5. **Graphics:** move from iso sprites toward blended 2.5D/3D models on the iso world.
6. **Front:** frontlines, theaters, pressure points, deployment, fortifications, faction symbols, balance, combat.
7. **Process:** checkpoints as git commits; standing order "commit each slice, log a short report, continue without waiting".

## 2. Commits since baseline

| SHA | Tag | What |
|:---|:---|:---|
| `c647ddc5` | orchestrator | Fleet charter + cross-fleet board |
| `febb2f9b` | front | `plan_front_combat_v1.md` (FC-0) |
| `4902c69b` | logistics | `plan_logistics_orders_v1.md` (LO-0) |
| `f6998633` | hygiene | `.gitattributes` (LF, CRLF for cmd/bat, binaries) |
| `0e3d6458` | ai | `plan_ai_fleet_v1.md` (FAI-0) |
| `5d5c54b9` | orchestrator | User priorities recorded on the board |

Fleet logs `ai.md`, `front.md`, `hygiene.md`, `logistics.md` were committed with those slices (the `front` and `logistics` logs say next = FC-1 / LO-1; both slices exist only as uncommitted files).

## 3. Per fleet

### 3.1 `tooling` (APS/editors/MCP CLI/orchestrator scripts)
- **Mission:** dev tooling, editors, studios/suites.
- **DONE:** nothing committed; no log.
- **IN PROGRESS / UNCOMMITTED (likely tooling):**
  - `tools/mcp/python/rust_engine_mcp/cli_catalog.py` (new, 180 lines) + `cli.py` (+72/-4): `build_parser()`, `commands` and `help` subcommands; grouped catalog derived from live argparse (also meant to back MCP `micro_tool_help`, which previously listed non-existent commands). State: looks complete, not verified by test run.
  - `tools/mcp/python/rust_engine_mcp/test_write_guard.py` (new, 205 lines), `tests/test_write_guard.py` (new, 96 lines), `__init__.py` (+8), `tests/conftest.py` (+34): pytest audit-hook guard that snapshots and rolls back any repo file written during a pytest session (including child Python processes). Purpose: stop test runs churning `debug_runs/*_live.json`, job status files, staging assemblies, style packs. Env vars `RUST_ENGINE_TEST_WRITE_GUARD`; keep-switch constant `ENV_KEEP`. Not confirmed it was ever run to completion; the churn below suggests earlier tests ran **before** the guard existed.
  - `tools/mcp/schemas/aps_suite_state_mutation_allowlist_v1.json` and four `tools/mcp/schemas/examples/*_production_run001.json`: modified; likely a side effect of tests (or EOL churn), not authored work. Unknown.
- **NEXT:** verify `cli_catalog` + write guard with `pytest tools/mcp/python/tests/test_write_guard.py`; wire `micro_tool_help` in `server.py` to `cli_catalog` (not seen done); fix `tools/mcp/python/rust_engine_mcp/validators/mcp_schema.py` which has `\r\r\n` endings (git sees binary; hygiene ask); adopt or archive the three unreferenced examples (legacy ask LG-27); reflection-driven inspector gap G-INSPECT (`bevy-inspector-egui` commented out in `Cargo.toml`); remove stray worktree `.claude/worktrees/loving-mestorf-ab9564` (LG-60, still registered at `d96eccb3`).
- **Dependencies:** board asks from hygiene (mcp_schema.py EOL) and legacy (LG-27, LG-60, G-INSPECT).

### 3.2 `effects` (fire / sparks / embers, P0)
- **Mission:** VFX/fire/particles/weather/sim-effect spine.
- **DONE:** nothing committed; no log. No uncommitted file under its ownership dirs (`src/render/fire_vfx`, `fx_spine`, `particle_domains`, `weather_vfx`, `src/sim/effects`, `src/systems/fire`, `assets/shaders/fire`).
- **IN PROGRESS:** unknown. Some `debug_runs/*` witnesses changed (`product_fire_vfx_live.json`, `fire_streaming_live.json`, `landscape_grammar_fire_harvest_wire_live.json`) but timestamps and content cannot attribute them to this fleet; likely test-run churn.
- **NEXT (P0 from the user):** audit fire front spread, spark/ember particle physics (ballistics, wind advection, buoyancy, cooling/lifetime, metric scale, coupling to `FireVisualFrame` and the sim), then prove with `VFX_CAPTURE=1` real PNGs (see memory note on Bevy 0.19 NoCpuCulling invisibility: check `RTT_SPRITE_TRACE=1` render-world trace). Use existing `src/dev/fire_ecology_f1_todos.md`. Fix legacy ask LG-22 (`src/systems/fire/surface_water.rs:71` hard-codes `tag_registry.example.json`; prefer the RON twin).
- **Dependencies:** legacy LG-22; perf-attribution memory note (STALL numbers are not system cost).

### 3.3 `logistics`
- **Mission:** standing supply lines vs one-off multi-trip orders; draw demands.
- **DONE:** `4902c69b` plan [`plan_logistics_orders_v1.md`](plan_logistics_orders_v1.md) (LO-0): research note, survey, isolation contract (sim -> snapshot -> UI), domain model, two order kinds, trip planning, draw demands with allocation/fairness, command API `LogisticsOrderCommand`, UI plan, phases LO-1..LO-6.
- **IN PROGRESS / UNCOMMITTED:** `src/economy/logistics/orders/` (new): `mod.rs` 17, `model.rs` 474, `sim.rs` 916, `tests.rs` 651 lines; plus `pub mod orders;` in `src/economy/logistics/mod.rs`. This looks like LO-1 and probably LO-2 (pure Rust deterministic `LogisticsOrderSim`, `LogisticsOrderCommand`, `TravelModel`, tests for multi-trip etc.). Compiles in the current tree; test status unknown (run `cargo test -p proc_A_dine01 --lib economy::logistics::orders`). Log not updated (still says "next LO-1").
- **NEXT:** run and commit LO-1/LO-2; LO-3 snapshot + command API + Bevy adapter plugin (sim tick under `economy_sim_running`, site registry, `speed_mps` to ticks); LO-4 `src/gui/logistics_orders_panel.rs` (needs an `InputBindings` hotkey; ask interaction with exact name); LO-5 map overlay + site discovery + one-way stock bridge to `ResourceFlowNode.buffer_by_tag`; LO-6 cross-fleet helpers.
- **Dependencies:** answer the ai ask (make `owner` the overlay faction slot `u8`, expose rear depot / front supply `SiteId` lookup); tell `front` which entity/site a front supply point is (front publishes `PublishDemand{...}` upserts per tick); gate `transport_topology_tick` on `graph.is_changed()` (legacy LG-61/P5).

### 3.4 `ai`
- **Mission:** strategic/faction AI that plays through the player's order APIs.
- **DONE:** `0e3d6458` plan [`plan_ai_fleet_v1.md`](plan_ai_fleet_v1.md) (FAI-0): five fit/counter pairs reconciled; slices FAI-1..FAI-9.
- **IN PROGRESS / UNCOMMITTED:** `src/ai/faction/` (new): `mod.rs` 228, `decide.rs` 307, `log.rs` 121, `orders.rs` 177, `perception.rs` 295; `pub mod faction;` and `pub mod orders;` lines in `src/ai/mod.rs`; `src/strategic/settlement/mod.rs` now `pub use faction_step::{FactionPairStance, FactionStanceBook}` (needed by `front`'s `frontline.rs` too; not clearly owned by either). Looks like FAI-1 (perceive, decide with named score terms, plan, log, order sinks via `StrategicCommandQueue`). Whether the plugin is registered in `SimulationPlugin` is not confirmed: `src/strategic/simulation_plugin.rs` has a +2 diff, content not attributed. Test status unknown (`cargo test -p proc_A_dine01 --lib ai::faction`).
- **NEXT:** FAI-2 deterministic seeding of `hybrid_resolve_and_feedback_system` (no `thread_rng`); FAI-3 headless scenario + witness; FAI-4 session seeding at `OnEnter(Simulation)`; FAI-5 explainability UI; FAI-6 per-faction growth (claimed `src/strategic/settlement/faction_step.rs`; no one objected on board); FAI-7/8 adapters to `DeploymentOrderQueue` (front) and `LogisticsOrderCommand` (logistics) once those APIs land.
- **Dependencies:** logistics (`owner: u8`, depot ids), front (`issued_by: u8`, formation `faction_slot`, pure `enqueue(order)`).

### 3.5 `graphics` (2.5D/3D on the iso world)
- **Mission:** blended 2.5D/3D models on the iso world.
- **DONE:** nothing committed; no log; **no `plan_graphics_25d_overhaul_*` file exists** (checked `src/dev`).
- **IN PROGRESS:** none identifiable. Only trace: `interaction` wrote a pick contract note in `src/gui/input_pick.rs` saying a perspective/3D main camera only requires replacing `screen_to_world_xy` / `world_xy_to_screen` with ray-ground intersection.
- **NEXT:** write the plan first (authority: `RepresentationResult`, projection graph, registry GPU upload; no parallel extraction or duplicate LOD; `ScaffoldContract` for transitional scaffolds; iso tile bake spine in `docs/archive/2026-06-src-dev/plans/design_tile_bake_spine_convergence_v1.md`); decide camera model (ortho kept with 3D mesh pass vs perspective); coordinate pick contract with interaction; render symbols/front polylines from `front` (FC-9).
- **Dependencies:** interaction (pick frame), front (FC-8/9 symbols and polylines), tooling/APS (GLB module kits).

### 3.6 `interaction` (cursor/pointer P0)
- **Mission:** adversarial test + fix of mouse, menus, icon and building placement.
- **DONE:** nothing committed; no log.
- **IN PROGRESS / UNCOMMITTED:**
  - [`interaction_bug_ledger_v1.md`](interaction_bug_ledger_v1.md) (new, 19 lines): only **INT-001** (draw tools pick with the desired-camera projection while buildings pick with the rendered ortho: drift when RTT is clamped at 4096 px or lags a resize) and **INT-002** (cursor west/south of world clamps to tile 0 via `floor().max(0.0)`, no upper bound reject): both `open`. The test suite refers to **INT-005** (first LMB locks without committing; locked origin survives cursor motion) which is not in the ledger table: ledger is behind the code.
  - `src/gui/input_pick.rs` (new, 235 lines): `TacticalMapPickFrame` as the single screen to world pick authority (rendered ortho first, desired camera fallback).
  - `src/gui/input_interaction_regression.rs` (new, 255 lines, `#[cfg(test)]`): headless harness (`set_cursor`, `click`, `step`, `screen_for_world`) and two `int_005_*` tests.
  - `src/gui/mod.rs` registers both modules (+3). **Call sites are likely not yet migrated** to `TacticalMapPickFrame` (draw tools, building ghost, demolish): unknown; check `grep TacticalMapPickFrame src/`.
- **NEXT (P0):** migrate all picks onto the frame; fix INT-001/002 with tests; run the test with `cargo test -p proc_A_dine01 --lib interaction_regression`; audit pointer ownership between egui and world (single pointer resource), Escape/focus, menus, strategic icon anchoring (`strategic_icon_instances.rs`), construction ghost/snap/place; update ledger (statuses `fixed <sha>`); give logistics an `InputBindings` hotkey name when asked.
- **Dependencies:** graphics (3D camera pick), logistics (panel hotkey), front (deployment/fortify placement UX, `DefenseKind::Wire`).

### 3.7 `front` (frontlines, theaters, combat)
- **Mission:** frontlines, theaters, pressure points, deployment, fortifications, symbols, balance, combat.
- **DONE:** `febb2f9b` plan [`plan_front_combat_v1.md`](plan_front_combat_v1.md) (FC-0): slices FC-1..FC-10 (balance table + cell-level fronts, control relaxation, theaters, pressure points, formations + combat resolution, deployment orders + draw demands, fortification, insignia + APP-6-style symbols, map presentation, witness + headless scenario).
- **IN PROGRESS / UNCOMMITTED:** looks like FC-1 complete or nearly: `src/combat/{mod.rs 41, balance.rs 467, front.rs 492}`; `assets/configs/combat/front_balance_v1.ron` (97 lines, compiled in via `include_str!`, disk override); `src/strategic/frontline.rs` (+148): `FrontlineState` gains `segments`, `revision`, `derived_at_tick`; `hostility_from_stances`, `control_grid_from_overlays`, cadence-gated derivation; `src/strategic/faction_plugin.rs` adds `CombatPlugin`; `src/lib.rs` adds `pub mod combat;`. Tests unknown (`cargo test -p proc_A_dine01 --lib combat::`). Log not updated.
- **NEXT:** commit FC-1; FC-2 control relaxation (fronts move); FC-3 theaters; FC-4 pressure points (`PressurePointBook`); FC-5 formations + combat (replace `src/systems/damage` TODO stub, `src/systems/collision` is empty); FC-6 `DeploymentOrderQueue` with pure `enqueue` + `FrontDrawDemandBook`; FC-7..FC-10.
- **Dependencies:** logistics supply-point site per front; ai asks (`issued_by: u8`, `faction_slot`); graphics for symbols/polylines; interaction placement UX.

### 3.8 `drain` (queued vegetation/trees now first)
- **Mission:** drain `tools/orchestrator/queues/` rows.
- **DONE:** nothing committed; no log; queue files are unmodified in `git status`.
- **IN PROGRESS:** none identifiable. `src/dev/veg_resolver_known_keys_v1.md` has an uncommitted rewrite (removes the Charter/Engine(extract)/Witness rows, trims text); owner unclear (hygiene or drain); it also says `Parity | PASS` with a witness row removed: verify before committing. Vegetation witnesses in `debug_runs/` changed (`veg_*_live.json`, `vegetation_*_live.json`, `aps_veg_*`), likely test churn.
- **NEXT (priority 3):** start from `coder_vegetation_drain_queue.json`, `landform_veg_drain_v1.json` (+ [`landform_veg_drain_todos_v1.md`](landform_veg_drain_todos_v1.md)), `parallel_wave_aps_veg_dispatch_v1.json` and `parallel_wave_aps_veg_wave0_orders_v1.md`; follow the code ownership table for edits; claim rows in HANDOFF before work.
- **Dependencies:** effects (burn/fire harvest wiring), tooling (APS veg state axis).

### 3.9 `legacy`
- **Mission:** find old/half-wired systems and loose config; wire in or retire.
- **DONE:** nothing committed. Log absent; the inventory file stands in for it.
- **IN PROGRESS / UNCOMMITTED:**
  - [`legacy_inventory_v1.md`](legacy_inventory_v1.md) (new, 73 lines): LG-01..LG-10 code, LG-20..LG-27 config, LG-40..LG-47 repo-root loose files, LG-60/61 elsewhere, product gaps (no audio system G-AUDIO, no engine-level error-recovery path G-RECOVER, no reflection inspector G-INSPECT).
  - **LG-01 (marked complete in inventory):** world save now writes/hydrates roads, rails, corridor construction rows and power lines. Files: `src/io/save/utility_overlay.rs` (new, 206), `src/io/save/transport_overlay.rs` (+about 225), `src/io/save/mod.rs`, `src/io/save/pipeline.rs` (overlay refs, hydrate-state resources, write-before-flush and hydrate systems). Claimed on board (no fleet owns `src/io`).
  - LG-03 (partial): `src/infrastructure/authoring/mod.rs` `confirm_corridor_authoring_with_profiles` resolves `ProfileRegistry`; LG-04 likely in `src/infrastructure/settlement/mod.rs` (+61); LG-09: `src/bevysubengines/mod.rs` now headed "PARKED". Test status unknown.
- **NEXT:** LG-02 make `TransportGraph` a derived mirror of `TransportEdgeDirectory` (blocked on logistics gating `transport_topology_tick`); LG-05 runtime config-root resolver; LG-06/07/08; LG-22 prefer-RON in three terrain/fire readers; LG-23; LG-09 R9 delete after retargeting two `include_str!` guard tests; repo-root loose files need **user decisions** (archive `buildlog.txt`, `migrate.sh`, `REORGANIZATION.md`; `model.html` is a delete-candidate; leave the personal note file untouched).
- **Dependencies:** logistics (LG-61), effects (LG-22), tooling (LG-27, LG-43), hygiene (LG-60, LG-10).

### 3.10 `tests`
- **Mission:** build health first, then test cleanup/consolidation.
- **DONE:** nothing committed; no log; **no `test_inventory_v1.md`** exists.
- **IN PROGRESS:** none identifiable. Observation: `cargo check --lib` is green; `cargo test` build (all test targets) was **not** run for this handoff.
- **NEXT:** run `cargo test -p proc_A_dine01 --lib --no-run` to find test-compile breaks (new `#[cfg(test)]` modules in logistics/ai/combat/interaction are untested by any fleet gate); run the new fleets' filters; then write the test inventory and consolidate. The MCP write guard (tooling) addresses the test-induced file churn.
- **Dependencies:** all code fleets (red build owned by `tests` per charter); tooling write guard.

### 3.11 `hygiene`
- **Mission:** file management, docs sprawl, debug_runs policy, line endings, consistency in shared code.
- **DONE:** `f6998633` `.gitattributes` and log [`fleet_logs/hygiene.md`](fleet_logs/hygiene.md): `* text=auto eol=lf`, CRLF for cmd/bat, binaries explicit. Index was already LF; the working tree had about 5,347 clean CRLF files rewritten to LF in place (only when bytes hash-matched the staged blob).
- **IN PROGRESS:** none beyond the above. Remaining CRLF warnings appear for files modified by Windows tools (e.g. `src/gui/mod.rs`, schemas); they normalize on commit.
- **NEXT:** debug_runs policy (witness JSON should not be tracked churn: gitignore, or regenerate-on-demand, or a single rollup); docs sprawl in `src/dev` (hundreds of files); archive candidates from legacy (`buildlog.txt`, `migrate.sh`, `REORGANIZATION.md`, `model.html`: ask user first); remove stray worktree LG-60 (coordinate with tooling); R8 `legacy_transport_stubs.rs` archive (LG-10); `src/utils` consistency; clippy.
- **Dependencies:** tooling (`mcp_schema.py` EOL), legacy (LG-60, LG-10).

## 4. Uncommitted files (240 entries: 226 modified, 14 untracked)

### 4.1 Real work (about 50 paths)

| Path | Likely owner | Kind | Recommend |
|:---|:---|:---|:---|
| `src/economy/logistics/orders/` (4 files), `src/economy/logistics/mod.rs` | logistics | code | Run `economy::logistics::orders` tests; if green commit as `[logistics]` LO-1/2 |
| `src/ai/faction/` (5 files), `src/ai/mod.rs` | ai | code | Run `ai::faction` tests; commit as `[ai]` FAI-1 |
| `src/combat/` (3 files), `assets/configs/combat/front_balance_v1.ron` | front | code + data | Run `combat::` tests; commit as `[front]` FC-1 |
| `src/strategic/frontline.rs`, `src/strategic/faction_plugin.rs`, `src/lib.rs` | front | code | Commit with FC-1 (they depend on `combat`) |
| `src/strategic/settlement/mod.rs` (re-export), `src/strategic/simulation_plugin.rs` | ai and/or front | code | Inspect `git diff`; commit with whichever slice needs it (frontline.rs uses `FactionStanceBook`) |
| `src/gui/input_pick.rs`, `src/gui/input_interaction_regression.rs`, `src/gui/mod.rs`, `src/dev/interaction_bug_ledger_v1.md` | interaction | code + doc | Run `interaction_regression` tests; commit as `[interaction]`; update ledger with INT-005 |
| `src/io/save/{transport_overlay,utility_overlay,mod,pipeline}.rs` | legacy | code | Run save/overlay tests; commit as `[legacy]` LG-01 |
| `src/infrastructure/authoring/mod.rs`, `src/infrastructure/settlement/mod.rs`, `src/bevysubengines/mod.rs` | legacy | code | Run their tests; commit as `[legacy]` LG-03/04/09 |
| `src/dev/legacy_inventory_v1.md` | legacy | doc | Commit |
| `tools/mcp/python/rust_engine_mcp/{cli.py,cli_catalog.py,__init__.py,test_write_guard.py}`, `tools/mcp/python/tests/{conftest.py,test_write_guard.py}` | tooling | code | Run pytest on those tests; commit as `[tooling]` |
| `src/dev/veg_resolver_known_keys_v1.md` | unknown (drain/hygiene) | doc | Review diff; unclear, possibly discard |

### 4.2 Generated / test churn (about 190 paths): recommend discard or ignore, do not act automatically

| Group | Count | Note |
|:---|:---|:---|
| `debug_runs/*.json`, `debug_runs/agent_ops/*` (16), `debug_runs/art_pipeline/*` (33), `debug_runs/bq_q2_screens/*`, `debug_runs/mig_bevy_019/*` (6), captures, thumb caches | about 160 | Witness JSON regenerated by test/proof runs; timestamp-only or env-only changes. Likely noise. If not wanted: `git restore -- debug_runs` (**user decision; not done**). Consider a hygiene policy first. |
| `tools/mcp/jobs/*.status.json`, `tools/mcp/cache/**` | 10 | Pytest/job runs |
| `assets/staging/assemblies/*.json`, `*.blend`, `*.snap_hash`; `assets/staging/tiles/tile_landscape_lg5_pilot_v1/{atlas_meta,batch_status}.json`; `assets/models/modules/*_production_run001/manifest.json`; `assets/archive/*/MOVED_LOG.json`; `assets/configs/buildings/style_packs/style_{rural,victorian}.ron` | 17 | Almost certainly test-induced regeneration (the write guard docstring names exactly these classes). Compare to HEAD before keeping; recommend discard unless an art task is known. |
| `tools/mcp/schemas/aps_suite_state_mutation_allowlist_v1.json`, `tools/mcp/schemas/examples/*_production_run001.json` (4) | 5 | Same churn pattern; CRLF warnings on them. |

Do not run a blanket commit. Use the charter protocol: `git add <new paths>`, then `git commit -m ... -- <pathspec>` per fleet.

## 5. Key docs index

- Charter: [fleet_charter_20261008_v1.md](fleet_charter_20261008_v1.md); board: [fleet_board_20261008_v1.md](fleet_board_20261008_v1.md); logs: [fleet_logs/](fleet_logs/)
- Plans: [plan_logistics_orders_v1.md](plan_logistics_orders_v1.md), [plan_front_combat_v1.md](plan_front_combat_v1.md), [plan_ai_fleet_v1.md](plan_ai_fleet_v1.md)
- Ledgers/inventories (uncommitted): [interaction_bug_ledger_v1.md](interaction_bug_ledger_v1.md), [legacy_inventory_v1.md](legacy_inventory_v1.md)
- Not existing yet: graphics 2.5D plan, tooling fleet plan, test inventory.
- Related existing: [single_worktree_policy_v1.md](single_worktree_policy_v1.md), [fire_ecology_f1_todos.md](fire_ecology_f1_todos.md), [landform_veg_drain_todos_v1.md](landform_veg_drain_todos_v1.md), [construction_invariants.md](construction_invariants.md), [plan_cleanup_v1.md](plan_cleanup_v1.md), [codebase_index_v1.md](codebase_index_v1.md), [../../AGENTS.md](../../AGENTS.md)
- Queues: [`tools/orchestrator/queues/`](../../tools/orchestrator/queues/) (`coder_vegetation_drain_queue.json`, `landform_veg_drain_v1.json`, `parallel_wave_aps_veg_dispatch_v1.json`, `HANDOFF.md`)

## 6. Open cross-fleet asks (from the board, none answered in writing)

- logistics to ai/front/interaction: single `LogisticsOrderCommand` API; fronts publish `PublishDemand`; hotkey for orders panel.
- ai to logistics: owner as faction slot `u8`; rear-depot/supply `SiteId` lookup. ai to front: `issued_by` as `u8`, formation `faction_slot`, pure `enqueue`.
- ai to all: ownership of `src/strategic/settlement/faction_step.rs` (ai claims, no objection).
- legacy to logistics: gate `transport_topology_tick` (LG-61). legacy to effects: prefer-RON (LG-22). legacy to hygiene: stray worktree, R8 stubs. legacy to tooling: LG-27, inspector.
- hygiene to tooling: `mcp_schema.py` line endings.
- legacy claimed `src/io/save/*` overlays.

## 7. Risks and gotchas

- **Shared checkout, shared index:** fleets used pathspec commits; any `git add -A`, `commit -a`, stash, reset or checkout of others' files would destroy work. Large amount of valuable code is **uncommitted** and has no backup except the disk; commit it soon.
- **HEAD may not build alone:** `HEAD` contains only docs/`.gitattributes`, so it builds as the baseline did; but commits of code must include their dependent files (e.g. `frontline.rs` depends on `combat/` and on the `FactionStanceBook` re-export; `faction_plugin.rs` and `lib.rs` register `combat`). Commit in dependency order.
- **Build evidence is for the dirty tree** (green). Test targets were not compiled or run; new tests in orders (651 lines), combat, ai and interaction are unverified.
- **Cargo lock contention:** one shared `target/`; 11 sessions serialize on it. Use `cargo check --lib` and filtered `--lib <module>` tests.
- **Witness JSON churn:** every proof/test run rewrites tracked `debug_runs/*` and asset staging files, polluting each fleet's `git status`; the tooling write guard is the fix but is itself uncommitted and unverified.
- **Line endings:** `.gitattributes` forces LF; Windows tools writing CRLF produce warnings; `mcp_schema.py` has `\r\r\n`.
- **Usage limit:** the run was stopped with 11 concurrent sessions (reportedly a usage limit); 5 fleets left no trace at all, so their work is unknown, not necessarily absent. Prefer fewer concurrent fleets, commit per slice immediately, and update each fleet log with every commit (the existing logs are already stale versus disk).
- **Stray worktree:** `.claude/worktrees/loving-mestorf-ab9564` still registered (detached `d96eccb3`); violates the single-worktree policy and doubles repo-wide greps.
- **Per-user decisions pending:** repo-root loose files (archive/delete) and whether to restore churned `debug_runs`/assets.
