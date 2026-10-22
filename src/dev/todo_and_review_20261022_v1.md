# TODO + review — 2026-10-22 (v1)

Successor to [`fleet_handoff_summary_20261009_v1.md`](fleet_handoff_summary_20261009_v1.md).
Covers the work another operator/LLM landed after `ef4e841a` (egui → Bevy UI port `51f8efb5`,
debug_runs untracking `0e199885`, churn commit `5f0ebddc`) and the cleanup done today.

## 0. State at writing

| Item | Value |
|:---|:---|
| Working tree | clean after today's commits (below) |
| Build | `cargo check --lib` green · `cargo test --lib --no-run` green |
| Lib tests | 2001 pass · **71 fail** · 3 ignored (after churn revert; was 80) — three fix lanes in flight, see §3 L0 |
| Policy | `debug_runs/**` is local-only (gitignored); tests must not depend on pre-existing witness files and must not rewrite tracked files |

Commits today: `f4a3a6a9` [effects] spark gating · `a1f3f37d` [graphics] dual-view PixelWorld3D + voxel massing + docs ·
`74cf1e4a` [tests] FULL_APP readiness harness-only · `6fd432ce` [ui] egui-stub prune ·
`d4509467` [hygiene] revert test/job churn from `5f0ebddc` (restored the `.blend` that had been overwritten with
`{"dry_run": true}`, style packs, staging assemblies, job/cache files, schema examples, `veg_resolver_known_keys_v1.md`).

## 1. Review of `ef4e841a..0e199885` (highest severity first)

Source: read-only review (code reading, verified per finding; H1 and H4 re-checked by hand).

| ID | Sev | Where | Finding | Failure scenario |
|:---|:---|:---|:---|:---|
| R-H1 | H | `src/render/world_map_network_mesh.rs` ~682–699 `sync_world_map_orders_meshes` | Logistics orders overlay ported from egui scales tile coords by `METRES_PER_TILE` (likely wrong space); change-detect key ignores vehicle positions; site markers dropped | Routes/vehicles drawn at the wrong place; vehicle dots freeze while trucks move |
| R-H2 | H | `src/gui/logistics_orders_panel.rs` ~537 | Bevy orders panel never fills `draft.source/sink/cargo`; submit always errors and the error isn't shown; pause/resume/cancel/priority controls lost | Player cannot create standing lines or one-off runs from the UI at all |
| R-H3 | H | `src/gui/editor/map_editor/mod.rs` | Map editor port dropped terrain brush, minimap road-marker painting, save/load, Play/Exit; nothing writes `MapEditorMapSnapshotIoRequest`; scenario panel lost Load/Save/Run/Stop | Editor can't save/load maps or run scenarios |
| R-H4 | H | `src/gui/hud/simulation_pointer_gate.rs` ~152–203 | Editor and dev-panel blockers set `chrome_blocks = true` from panel *presence* with no cursor-in-rect test | Editor dock always present → editor placement blocked everywhere; any open F1/F3/logistics/dev panel blocks every map click (cursor-stability P0) |
| R-H5 | H | pointer-gate scheduling | Block systems chained after gate sync, but construction consumers are not ordered after the block systems | Click passes through a panel depending on schedule order (nondeterministic, P0 class) |
| R-H6 | H | tests | Structural tests assert retired egui strings (`apply_egui_theme_system`, several `include_str!` "wired" predicates); `minimap_power_strokes_wired` is fake-green (checks the name of an empty stub) | False red and false green in the suite |
| R-H7 | H | `src/bin/world_generator.rs:18` | `KeybindingsOptionsPlugin` requires `PanelPresence`, never inserted in this binary | World generator binary likely panics on first frame |
| R-M1 | M | keybindings panel | Rebinds can't be saved/reset | Settings lost |
| R-M2 | M | floating Bevy panels | No scrolling, no Esc-to-close | Long panels clip; Esc cascade inconsistent |
| R-M3 | M | road popup | Despawn/respawn every control-point change | Flicker, wasted work |
| R-M4 | M | `landscape_grammar_sim_harness.rs:592,600` | Reads `play_scenario_live.json` / `stage5_full_app_live.json` it never writes | Fails on clean checkout |
| R-M5 | M | `runtime_witness/exceptions_manifest.json` | Containment manifest grew instead of draining (`508bc92c`) | Gate erosion |
| R-M6 | M | `AGENTS.md` | Still says "attach agent-lang every session" (contradicts new lean harness) and links now-untracked `debug_runs` JSON | Agents follow stale instructions |
| R-M7 | M | `5f0ebddc` | Committed test churn incl. a corrupted `.blend` | **Fixed** in `d4509467` |
| R-M8 | M | egui-port gaps | Dropped without replacement: agent-permissions window, F4 HUD dev overlay, most F3 diagnostics sections (strategic/doctrine, fire streaming, atmosphere, weather, recorder, ECS counts, corridors/theaters), power-overlay stroke painters, plant focus card, growth inspector, side status rail | Loss of dev/operator visibility |

Looks good: `TacticalMapPickFrame` pick math unchanged; Bevy crosshair replaces egui one; no new non-test `unwrap/expect`;
no `include_str!` to deleted files; `bevy_egui` removal from Cargo is clean; INT-001/002/006/007 fixes landed.

## 2. Status of earlier handoff items

Done: pick-frame migration, INT-001/002/006/007, MCP `micro_tool_help` → `cli_catalog`, debug_runs policy.
Partial: fire realism (spark gating only), graphics (re-scoped to dual-view PixelWorld3D — `plan_dual_view_pixel_world_v1.md`), logistics UI (ported but broken — R-H1/H2), test baseline (in progress).
Not started: INT-005, FC-2+, FAI-2+, LG-22, LG-60 stray worktree, test inventory, docs sprawl, `mcp_schema.py` CR line endings, vegetation queue drain (`coder_vegetation_drain_queue.json` 55 blocked / 26 done).

## 3. TODO for subagents (pick top-down; one lane per agent; disjoint files)

Rules for every lane: edit only the lane's files; keep the tree compiling; verify with filtered
`cargo test -p proc_A_dine01 --lib <filter>`; tests must not rewrite tracked files (`git status` clean after runs);
the parent commits per lane as `[lane] …` after checking. Cheap model (Sonnet) by default; escalate only on design-heavy items.

### L0 — Test baseline green (in flight)
- [ ] L0a construction/procedural failures (19) — module index, golden seeds (**hash updates need operator approval**), place feedback, bq128.
- [ ] L0b strategic/systems/gui failures (~25) — WorldAsset loader guards, world-gen chrome regression, spine audits, egui-era structural tests → Bevy equivalents.
- [ ] L0c dev proof/witness failures (~27) — self-contained tests vs explicit `witness-tier` (`#[ignore]` with runnable command), containment manifest, rgr_m1 budget (**decision if budget must rise**).
- Done when: `cargo test --lib` 0 failed; ignored list = INT-005 + documented witness tier only.

### L1 — Cursor & pointer stability (user P0)
- [ ] R-H4: blockers test cursor against the panel's computed rect (Bevy `ComputedNode` + `GlobalTransform` / `RelativeCursorPosition`, or `Interaction` hover on panel roots), not mere presence.
- [ ] R-H5: one ordered set — `PointerGateSet::{Sync, Block, Publish}`; all map consumers (construction, power, demolish, minimap, selection) `.after(PointerGateSet::Publish)`. Add a schedule test asserting the ordering.
- [ ] INT-005: triage the two-click place FSM (product vs harness); un-ignore the test.
- [ ] Regression tests: panel open + cursor outside panel → map click lands; cursor over panel → blocked; editor dock present → placement allowed off-dock.
- Files: `src/gui/hud/simulation_pointer_gate.rs`, `src/gui/input_pick.rs`, `src/gui/input_interaction_regression.rs`, `src/gui/panel_presence.rs`, ledger.

### L2 — Logistics UI parity (R-H1, R-H2)
- [ ] Orders panel: source/sink/cargo pickers (site list from `LogisticsOrdersSnapshotRes`), visible errors, pause/resume/cancel/priority.
- [ ] Orders overlay: correct coordinate space (match `world_map_frame` conventions), include vehicle positions in change key, restore site markers.
- [ ] Headless tests: submit one-off run and standing line through the panel's command path; overlay positions for a known snapshot.
- Files: `src/gui/logistics_orders_panel.rs`, `src/render/world_map_network_mesh.rs` (orders section only), `src/gui/logistics_targets_panel.rs`.

### L3 — Fire & sparks realism (user P0)
- [ ] Audit spark/ember particles: buoyancy, wind advection, gravity on embers, cooling colour ramp (white-yellow → orange → dark red), lifetime, flicker frequency, metric scale; coupled to `FireVisualFrame` + sim fuel/burn state.
- [ ] Prove with `VFX_CAPTURE=1` PNGs (beware Bevy 0.19 entity-level `NoCpuCulling` invisibility; probe `RTT_SPRITE_TRACE=1`).
- [ ] LG-22: `systems/fire/surface_water.rs:71` prefer the RON tag registry.
- Files: `src/render/fire_vfx/`, `src/render/extraction/fire_visual_extract.rs`, `src/systems/fire/`, `assets/shaders/fire/`.

### L4 — Trees / vegetation queue (user priority)
- [ ] Classify the 55 blocked rows in `tools/orchestrator/queues/coder_vegetation_drain_queue.json` (blocker still real? obsoleted by egui drop or dual-view?); unblock and drain; finish the 6 open rows of `landform_veg_drain_v1.json`.
- [ ] Ensure vegetation renders in both TacticalFlat2D and PixelWorld3D (`presentation` tags from `a1f3f37d`).

### L5 — Editor & tools parity (R-H3, R-H7, R-M1..M3)
- [ ] Map editor: brush, minimap road markers, save/load via `MapEditorMapSnapshotIoRequest`, Play/Exit, scenario Load/Save/Run/Stop.
- [ ] `world_generator` binary: insert `PanelPresence` (or make the plugin self-init) + smoke test that builds the binary's app for one frame.
- [ ] Keybinding save/reset; panel scroll + Esc close; road popup update-in-place.
- [ ] R-M8 decided: see L5b (keep only panels with actual use).

### L6 — Hygiene & guards
- [ ] Test write guard for Rust tests (like the MCP pytest guard): tests that write must target temp dirs or `debug_runs/`; add a CI check that `git status --porcelain` is clean after `cargo test --lib`.
- [ ] AGENTS.md: reconcile with the lean harness (R-M6); drop links to untracked debug_runs JSON.
- [x] LG-60: stray worktree removed (2026-10-22).
- [ ] `tools/mcp/python/rust_engine_mcp/validators/mcp_schema.py` → LF; run MCP pytest under the write guard.
- [ ] Rename egui-era misnomers (`*_egui.rs`, `map_egui_projection`) in a dedicated rename commit; drain containment manifest (R-M5).
- [ ] Test inventory doc (`test_inventory_v1.md`) with tiers: smoke / module / full / witness-tier.

### L7 — Game systems (after L0–L2)
- [ ] Front FC-2…FC-6 (`plan_front_combat_v1.md`): control relaxation, theaters, pressure points, formations/combat, deployment orders publishing draw demands.
- [ ] AI FAI-2…FAI-4 (`plan_ai_fleet_v1.md`): deterministic seeding, headless scenario, session seeding; then adapters to `LogisticsOrderCommand` and the deployment queue.
- [ ] Dual-view next steps per `plan_dual_view_pixel_world_v1.md` (DR-PIXEL-WORLD-APS-MESH residual 3D module-kit wave).

## 4. User decisions (2026-10-22)
1. **Golden seed hashes: APPROVED** — update `GOLDEN_SEED_SET_V1` via `bq_q3_golden_bootstrap_hashes` once the drift cause is recorded (L0a).
2. rgr_m1 `api.rs` pub-use budget (L0c): still open if the agent finds the growth legitimate.
3. **LG-60: DONE** — stray worktree `.claude/worktrees/loving-mestorf-ab9564` (clean, `d96eccb3` already in master) removed and pruned.
4. **R-M8 dev panels: keep only panels with actual use; drop everything else.** Lane L5b below.

### L5b — Dev-panel cull (from decision 4)
- [ ] Inventory every panel/overlay root, egui-retired stub and toggle (F1/F3/F4, agent permissions, plant focus card, growth inspector, side status rail, stage7 shell, shell diagnostics, info tabs, power stroke painters, `*_egui.rs` stubs).
- [ ] "Actual use" = reachable in product or editor play (keybind/menu), shows live data a player/operator acts on, or is read by a non-test system. Proof-only `include_str!` "wired" predicates do not count.
- [ ] Keep + finish the used ones (Bevy UI); delete the rest outright: module, state resources, toggle events, keybindings, plugin registration, and tests/proofs that only assert their wiring. Classify before deleting (cleanup-completion-intelligence), one commit per panel family.
- [ ] Run after L0 lands (it touches the same `src/gui` files).
