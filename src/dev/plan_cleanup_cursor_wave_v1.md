# PLAN-CLEANUP-CURSOR-WAVE-v1 — clean up the 14–16 Nov Cursor-agent wave

**Status:** OPEN · **Written:** 2026-11-16 · **Owner on pickup:** `@coder` (Rust), `@coder-mcp` (tools/mcp), `@plan-orchestrator` (docs). Run deletions past `@cleanup-intelligence` first.
**Scope:** `git diff 8abc4326 HEAD` — 18 Cursor co-authored commits, 403 files, +46.5k / −2.5k lines. The last commit (`afdf586c`, "Ship defer-unlock board…") alone is 310 files / +30k lines, though its message names only three items.

---

## 0. Read this first (rules for the agent doing the cleanup)

1. **Stopping is allowed.** Ignore `plan_ref_books_upgrade_v3.md` §0.1 "Anti-stall". If an item is blocked, ambiguous, or bigger than it looks, write `BLOCKED: <reason>` in §9 and move on — or stop and ask. Marking things done that aren't is the failure this plan is cleaning up.
2. **"Done" means a production caller.** An item that adds or keeps a feature is done only when you can cite a non-test caller `file:line` (a system registered with `add_systems`, a resource that something reads in play). A unit test alone does not count.
3. **Finish, don't remove — and never leave unwired code marked shipped.** The default for unwired features is to **wire** them (operator review 2026-11-16; see §5's decision rule). Delete only true duplicates or bookkeeping that does nothing. Respect the "Product stance (operator …)" table in the owning plan; e.g. dense heat is GPU-primary by operator decision. If wiring is blocked, set the `adoption.tsv` row to `UNWIRED`, write `BLOCKED: <reason>` in §9, and move on.
4. **Verify each finding before acting.** Findings in §2–§8 came from a code-reading review (five reviewers, 2026-11-16). Items tagged **[V]** were also confirmed by a test run or by direct inspection. The others are high-confidence but check the cited lines first; line numbers may drift.
5. **Commit per item group,** with a message that matches the diff (`[cleanup] CQ-07 …`). No 300-file bundles.
6. **Validation after every group:**
   ```
   cargo check -p proc_A_dine01 --all-targets
   cargo test -p proc_A_dine01 --lib
   cd tools/mcp/python && python -m pytest -q
   ```

### Baseline at HEAD `afdf586c` (2026-11-16)

| Check | Result |
|---|---|
| `cargo check --all-targets` | passes, **427 warnings** (162 deprecated picking alias, ~45 never-used fn/const/struct, 2 unused deps) |
| `cargo test --lib` | **2072 passed / 38 failed / 3 ignored**. 23 failures were already listed in `mig_bevy_020_m0_test_defer.md`; **15 are new** (§3) |
| MCP `pytest` | **445 passed / 6 failed / 1 skipped**; the write guard rolled back 60 repo files that tests wrote |

---

## 1. Summary of what went wrong

- **Shipped-but-unwired.** Many slices landed as pub functions with unit tests but no production caller. Others are registered systems that run on empty data. 7 of the 12 "ADOPTED/CLOSED" claims audited are PARTIAL and 1 is FALSE (§6).
- **Over-claimed headlines.** TAC-03 "GPU-primary dense heat" is a CPU loop: the WGSL is never loaded, and the "parity" test compares the CPU path with a clone of itself. MOVE-10 wildlife ticks an empty book. RND-07 "HOLD" was computed from an all-zero default frame.
- **The banned witness pattern is back** (`*_live.json`, verdict strings, gate predicates), in both Rust and the APS tooling. AGENTS.md forbids this, and the 12 Nov cleanup deleted ~50k lines of it.
- **Real regressions:** mid-burn fire state is no longer saved, the route cache is cleared every sim step, terrain erosion loses mass, keyboard camera input dies over HUD chrome, 15 new lib-test failures.
- **Test inflation:** ~50 `include_str!` tests that grep source files and doc comments for book-citation strings, plus several tautological tests.
- **Doc sprawl:** 16 new `plan_*.md` (811 `.md` files under `src/dev/`), the same status kept in three ledgers that disagree, and a plan section that forbids agents from stopping to report gaps.

---

## 2. Phase P1 — correctness bugs (do first)

| ID | Sev | Where | Problem | Fix | Effort |
|---|---|---|---|---|---|
| CQ-01 **[V]** | High | `src/io/save/stand_overlay.rs:25-34,66-83`, `fire_overlay.rs:24-30`, `post_load_rebuild.rs:23-28` | Per-cell fire `heat/fuel/burn_hold` (and chunk `smoke/toxic`) became `skip_serializing`. The comment says the `chunk_fire_state` overlay is the authority, but that overlay stores only chunk means. A mid-burn save loads with cell fire zeroed, and `schema_version` was not bumped. | Restore per-cell serialization (or move it into the fire overlay), bump the schema version, keep reading v1, and add a save→load round-trip test on a burning chunk. | M |
| CQ-02 | High | `src/systems/navigation/cost_forecast.rs:215,262,271`, `src/economy/logistics/routes.rs:344` | Bucket `start_tick` is derived from `steps`, so `revision` bumps every step. `refresh_resource_flow_routes_system` then calls `route_cache.routes.clear()` every step. | Bump the revision only when the multiplier values change (or quantise ticks to `bucket_secs`). Invalidate only the affected edges. Add a test: no weather change → no revision bump. | M |
| CQ-03 | High | `src/economy/logistics/orders/demand_advert.rs:41-75`, `advisor.rs:40` | The `sort_by` comparator runs ~4 Dijkstras per comparison over an O(S²) complete site graph. It is hit by auto-supply, faction AI and `gui/logistics_orders_panel.rs:598`. | Compute one score per demand first (`sort_by_cached_key`) and reuse a single Dijkstra's distances. | S |
| CQ-04 | Med | `src/terrain/generation/hydrology/erosion.rs:66-67,91-92` | The source cell is assigned (`dem[i] = prev[i] - x`) while receivers accumulate. Same-pass deposits into `i` are overwritten, so mass is lost and the result depends on order. Erosion is on by default, so this changes every seed. | Accumulate deltas into a separate buffer, then apply them. Add a mass-conservation test. Regenerate golden seeds if they shift (`golden_seed_set.rs`). | S |
| CQ-05 | Med | `src/gui/tactical/map_camera.rs:541`, `src/render/view_runtime/plugin.rs:37-41` | `world_gameplay_input_disabled()` early-returns before **keyboard** pan/recenter/zoom-reset whenever the cursor is over HUD chrome. Capture apply is unordered against `MapCameraSystemSet::ApplyInput`. `ViewInputRoutingState` has two writers, and `render/` imports `gui::hud`. | Gate only pointer input. Add explicit `.before/.after` ordering. Use one writer for routing state. | S |
| CQ-06 | Med | `src/gui/in_game_pause_menu.rs:26-33`, `src/gui/hud/panel_state.rs:107` | One Esc can both cancel a keybinding capture and close the pause menu: the `Pause` branch runs before the `consumed_before_pause()` check. The resolver hardcodes `KeyCode::Escape` while the toggle uses a rebindable key. | Check `consumed` first and read one key source. | S |
| CQ-07 | Med | `src/strategic/site/voxel_occupation.rs:39-45,202,246`, `src/io/save/voxel_occupation_overlay.rs` | Voxel commit authority **defaults ON**, but its save overlay is a "P0 stub — not wired", so voxel occupation is not persisted. `primary_site_at` uses `find_map` over a `HashMap`, which is non-deterministic. `apply_site_commit` overwrites on a re-commit. The module docs say the default is off. | **Default:** flip both flags OFF until the overlay is wired into manifest save/load with a round-trip test. Switch to `BTreeMap` with a lowest-id tie-break, and append instead of insert. Fix the docs. | M |
| CQ-08 | Med | `src/combat/front_stock_pool.rs:140-165` | Side-wide demand is published to **every** front site, so demand is inflated N×. The pool keeps one best site per (side, tag) with no fallback. `0.5` and `draw_rate_per_min: 1` are magic numbers. | Split demand per theater/site, keep a per-site list, and move the constants to `FrontBalance`/RON. | S |
| CQ-09 | Med | `src/construction/voxel_massing.rs:118-160,356`, `src/construction/mod.rs:346` | `RUST_ENGINE_STUDIO_MASSING` loads any file path in **release** builds and permanently overrides ghost massing. | Gate it with `cfg(any(debug_assertions, feature = "sim_dump"))` like `SimRunDumpPlugin`, and scope it to the scene. | S |
| CQ-10 | Med | `src/audio/mix_frame.rs:353-365` | The ambience duck keys off `oneshots.pending`, which is drained every frame, so it reaches ~18% of target. `weather_lowpass_cutoff_hz` and `ambience_duck_gain` are computed but nothing reads them. | Hold the duck for the warning's duration. Wire the low-pass or delete it. | S |
| CQ-11 | Med | `tools/mcp/python/rust_engine_mcp/validators/studio_dcc.py:122-131` | `validate_veg_species_path(path)` ignores `path` and always validates the default RON. | Pass the path through and add a test with a bad file. | S |
| CQ-12 | Low | `tools/mcp/art_pipeline_suite/dashboard_panel.py:176-185` | The "Blender handoff (selected task)" button uses the most recent task, not the selected one. | Use the Treeview selection. | S |
| CQ-13 | Low | `src/economy/logistics/solver.rs:272-277` | `else if a.foci.is_empty() { Aggregate } else { Aggregate }` has identical branches. | Collapse them, or map focused theaters to the intended tier. | S |

---

## 3. Phase P2 — get the test suites green

**CQ-20 [V] — 15 new lib-test failures.** Most panic on Bevy 0.20 param validation because new systems require resources or messages that minimal test apps don't add: `WeatherPollingStations`, `WeatherPollSiteBook`, `LogisticsOrderRequest` messages, `AudioOneShotQueue`, drone mission `tuning`. Treat it as a design smell as well as a harness gap: combat/logistics plugins now hard-depend on weather/audio plugins. Fix each one by:
(a) making cross-domain reads `If<Res<T>>` / `Option<Res<T>>` where the consumer can sensibly skip, or
(b) having the owning plugin `init_resource`/`add_message` the type, or
(c) inserting it in the shared test harness.

```
construction::voxel_massing::tests::sync_ignores_voxel_when_flags_off
economy::activation::bridge::tests::transformer_bus_emits_overload_when_smelters_cluster
economy::logistics::orders::tests::app::{construction_site_publishes_material_demands_until_built,
  despawned_site_fails_its_orders, plant_buffers_feed_runs_and_receive_deliveries,
  requests_drive_a_multi_trip_run_to_completion_in_app, save_then_load_into_fresh_app_rebinds_sites_and_keeps_orders}
economy::logistics::tests::{log_a_04_cut_road_sets_path_open_false_after_refresh,
  log_d_streaming_route_invalidation_on_construction_bump}
gui::style::heatmap_ramp::tests::linear_blend_midpoint_between_stops
render::core::terrain_render_authority::tests::env_rollback_forces_cpu        # env-var race, see CQ-22
scenario::trigger_registry::tests::ecs12_pathway_modules_hold_no_sim_write_surface  # prod code now has `.push(` — real violation or bad lint
strategic::site::voxel_occupation::tests::voxel_occupation_p2_sole_writer_tile_derived_cache  # asserts "X2-P2"; code returns "OCC-P5"
systems::transport::persistence::tests::g4_round_trip_json_file_from_manifest   # transport save format changed?
systems::transport::snapshot::tests::hydrate_rebuilds_nodes_when_omitted_on_wire
```
Treat the two transport tests as possible save-format regressions (see CQ-01) until proven otherwise.
**Acceptance:** the new failure count is 0. Then work down the 23 older failures in `mig_bevy_020_m0_test_defer.md` (same `If<Res>` pattern) and update that file.

**CQ-21 [V] — 6 MCP pytest failures.** The failing tests are:
- `test_aps_panel_sync_characterization::test_send_to_assembly_syncs_panel_chrome_from_state` (Tk "main thread is not in main loop")
- `test_aps_runtime_callbacks::test_on_generate_with_building_grammar`
- `test_assembly_grammar_verify::test_cli_path_wrapper_full_p0`
- `test_build_worker_p0::test_style_pack_resolves_industrial_corner_production_not_victorian`
- `test_material_library_filter::{test_flat_category_filter_does_not_nameerror, test_studio_tree_category_filter_branch}`

The test files are unchanged in range, but `art_pipeline_suite/app.py` and the panels changed. First run the suite at `8abc4326` (a throwaway worktree) to split regressions from pre-existing failures.

**CQ-22 — racy env-var tests.** `src/systems/sim_control.rs:362`, `render/core/terrain_render_authority.rs` (`env_rollback_forces_cpu`), and others call `std::env::set_var` while lib tests run in parallel. Refactor to test a pure `parse(Option<&str>)` function.

**CQ-23 — tests write into the repo.** pytest wrote 60 repo files (the write guard rolled them back). An example is `param_frame_studio.py:288-307`, which writes into `tools/mcp/schemas/examples/`. Point all test writes at `tmp_path`.

---

## 4. Phase P3 — remove rule violations (AGENTS.md)

### 4a. Witness / test-shaped production code (delete)
| ID | Where | What |
|---|---|---|
| CQ-30 **[V]** | `src/strategic/site/voxel_occupation.rs:423-470,541-552,607-665`, `src/dev/sim_run_dump_occupation.rs` | Delete `VoxelOccupationAuthorityWitness`, `authority_witness_stub`, `write_authority_witness_json`, the hardcoded `issued: "2026-11-15"`, the always-true `dual_writer_guard_ok`, and the plan-phase strings. Keep counts in the sim dump only. |
| CQ-31 | `src/render/stage6_virtualization.rs:271,389` | Delete `profile_atlas_memory` / `write_atlas_memory_profile_report` and the `"OPEN_RND07"/"HOLD_RND07"` verdicts. If profiling is wanted, add counts to the dump and score them in `tools/sim_review`. |
| CQ-32 | `tools/mcp/.../building_quality_qc.py` (`refresh_pcg08_golden_report`), `param_frame_studio.py:265-307` | Remove `write_aps_live_witness(... ritual="BLANG:WIT-HON")` and the `pcg08_promote_gate_snapshot` gate. It blocks the APS enqueue button permanently (0/12 golden seeds pass) while the CLI bypasses it. Separately, find out **why every golden seed fails** (`grammar_verify_perimeter_count` ×9, `generate_no_resolvable_placements` ×3). |
| CQ-33 | `src/systems/atmosphere/gpu_field_bridge.rs:22,201`, `systems/fire/chunk_smoke_field.rs:68`, `atmosphere/visual_extract.rs:26,103`, `fire/fire_light_emission.rs:40`, `render/view_runtime/tonemap_hdr.rs` | Delete the marker constants: `GPU_FIELD_UPLOAD_PATHS` (written into diagnostics as if observed), `CHUNK_SMOKE_WRITERS`, `DEBT_011_SMOKE_UNIFIED`, `SMOKE_EXTRACT_BRIDGE`, `LGT03_TONEMAP_OWNER_TOKEN`, `PIXEL_WORLD_TONEMAPPING_NONE_TOKEN`. |
| CQ-34 | `src/compute/heat_consumers.rs` | Delete the module: a test-only mirror that doesn't match prod ordering, the `sparse_play_retired` predicate, and an `assert 32+16==48` test. |

### 4b. Single writer / layer direction
| ID | Where | Fix |
|---|---|---|
| CQ-35 | `cost_forecast.rs:32` vs `routes.rs:321,347` | `refresh_resource_flow_routes_system` writes `consumed_revision` into the forecast book. Move that field into `RouteCache`. |
| CQ-36 | `atmosphere/coupling_terms.rs` + `weather/systems.rs:163` | Two writers set `pyro_moisture_injected` in both `WeatherSourceTerms` and `AtmosphereDiagnostics`. Split out a solve-applied field. |
| CQ-37 | `src/compute/heat_weather.rs:48` | Sim reads `render::ClimateVisualAggregate`, which inverts the layers. Read `AtmosphereFrame` means instead. |
| CQ-38 | `src/render/fire_vfx/emit.rs:227,388` | Render reads `DenseHeatPlayField` directly and blends it at a magic ×0.85. **Resolved by CQ-50:** fire VFX binds the GPU dense-heat buffer in the render world through the fire projection node, so there is no main-world read and no magic blend (rule: no parallel extraction). |
| CQ-39 | `chunk_weather.rs:104`, `polling_stations.rs:106,124`, `atmosphere/frame.rs:473`, `atmosphere/advect.rs:58` | Four per-chunk `WeatherField` samplers, two of which claim to be canonical. Make `AtmosphereFrame` the single sampler that the others read. |

### 4c. Clone farms (`plan_anti_clone_kernel_v1.md`)
| ID | Where | Fix |
|---|---|---|
| CQ-40 | `combat/drone_supply.rs:29-56`, `missile_supply.rs:68-95`, `toe_demand.rs:44-90` | The `refill_*_packages_sink` functions and the drone/missile aggregation blocks are line-for-line copies. Add one `TopUpPackage` trait (`ALL`, `count`, `bump`, `toe_row`) with a generic sink and aggregator. |
| CQ-41 | `tools/mcp/python/rust_engine_mcp/studio_task.py` | The four `run_queued_*` functions are type-swapped copies. The file-based queued/running/done "bus" with handoff markers rebuilds the deleted queue system, and every caller runs it synchronously. Replace it with `run(kind, payload)` plus a dispatch table. Keep status files only if Blender truly needs async; if so, record why in the module doc. |

---

## 5. Phase P4 — finish (wire) unwired features

**Decision rule (operator review 2026-11-16).**
- **Default to wiring.** Unfinished agent work is finished, not removed. Delete only true duplicates of something already wired, or bookkeeping that does nothing.
- **Check the operator decision first.** Before changing scope or runtime, read the owning plan's "Product stance (operator …)" table.
- **CPU vs GPU follows value, not habit.** GPU for big, data-parallel fields that rendering reads, or where the operator chose GPU. CPU for sim truth that AI, logistics or saves depend on. Async GPU readback must never feed fire spread or other seeded sim truth.
- **Record every decision.** Write it in the owning plan's "Review 2026-11-16" block and in `adoption.tsv`. A row moves from `UNWIRED` to `ADOPTED` only with a production caller `file:line`.

### 5a. Wire

| ID | Feature (ledger row · plan) | State found | Do | Runtime | Effort |
|---|---|---|---|---|---|
| CQ-50 **[V]** | **TAC-03 dense heat** — `compute/heat_diffusion_gpu.rs`, `assets/shaders/compute/heat_diffusion_dense.wgsl` · `plan_dense_heat_wgsl_v1.md` (reopened) | The operator chose **GPU-primary** (CPU = parity/test only). Play runs `run_dense_heat_step` on the CPU. The WGSL is never loaded. The parity test compares a CPU clone with itself (l.226-231), and the WGSL test substring-matches constants. | **Finish the GPU path:**<br>(1) A RenderApp compute node runs inject → H → V → decay over ping-pong storage buffers. Copy `render/pipelines/gpu_spark_compute.rs` / `gpu_weather_fire_field.rs`.<br>(2) Bevy 0.20 built-in readback: `bevy_render::gpu_readback::Readback::buffer` plus an observer on `ReadbackComplete`. That observer is the **sole** main-world writer of `DenseHeatPlayField`, and it tags the result with the `SimStepStamp` it was computed from.<br>(3) Fire VFX binds the GPU buffer in the render world with no readback (this also resolves CQ-38).<br>(4) Nav and attention read the readback copy; 1–2 frames of lag is fine for soft costs.<br>(5) The CPU kernel stays only as the headless/lib-test reference and the no-GPU fallback, chosen at startup and never at the same time as the GPU writer.<br>(6) Replace the fake parity test with a real GPU-vs-CPU ≤1e-5 check that runs where a GPU exists (playcheck or an `--ignored` test).<br>(7) Fix the ROI bugs (CQ-63) in both paths.<br>**Guard:** fire spread (`ChunkFireOverlay`) must not read dense heat; it is the source. **Follow-on:** once GPU-primary works, consider per-cell instead of per-chunk resolution for VFX quality. | GPU (play) · CPU (reference) | M |
| CQ-51 **[V]** | **MOVE-10 wildlife** — `systems/ecology/wildlife_flock.rs` · `plan_move10_wildlife_v1.md` (reopened) | Operator stance: the herd must affect sim (soft path cost), not decoration. But `WildlifeHerdBook` defaults to empty, `from_seed` is test-only, and `sync_nav_wildlife_soft` / `sync_agent_frame_from_wildlife` have no caller. | Seed herds from the world seed in the ecology spawn. Schedule both syncs. Feed `wildlife_soft_samples` into the CQ-53 route cost. If herds are drawn, use instanced sprites on the existing sprite path. | CPU sim (few agents, deterministic) | M |
| CQ-52 | **DH-03b / MOVE-10 nav costs** — `NavFieldFrame.heat_cost_samples`, `mean_heat_cost`, `wildlife_soft_samples` | Written, never read. | Make them inputs to the CQ-53 route cost. No separate reader. | CPU | (in CQ-53) |
| CQ-53 | **NAV-13 / NAV-04 routing + MOVE-03/04/08 steering** — `logistics_floodfill.rs:350-640`, `path_concerns.rs:155`, `combat/supply.rs:30`, `potental_feild_nav.rs`, `road_vehicles_motion.rs` · `plan_ref_books_nav_13_v1.md` (reopened) | The Dijkstra variants (scratch array, forecast-aware) are test-only. The path-concerns loader is never scheduled. The forecast wrapper passes `None, 0`. **`potential_field_navigation_system` is never scheduled**, so separation, avoidance and arbitration never run. `smooth_vehicle_tile_path` / `filter_vehicle_steering_output` are test-only. | **Do CQ-02 first**, or wiring makes routing slower.<br>**Routing:** one route cost function used by `refresh_resource_flow_routes_system` = base + path concerns + forecast + heat (CQ-52) + wildlife (CQ-51). Schedule `load_logistics_path_concerns_on_startup` and wire `front_haul_path_concern`. Keep one Dijkstra (the NAV-04 scratch-array version) and delete the other variants once folded.<br>**Steering:** schedule the steering system for trucks (depot-queue separation). Smooth truck tile paths and cap accel/turn in `road_vehicles_motion`. This is the spacing layer for trucks, members (CQ-54) and wildlife. | CPU | M + M |
| CQ-54 | **MOVE-05 / UM-05..10 formation members** — `combat/formation_members.rs`, `combat/mod.rs:230-233` · `plan_unit_member_v1.md` (reopened) | Four systems tick on zero entities: `spawn_formation_with_members` is test-only. `OverwatchRole` flips every tick and nothing reads it. | Spawn members in the formation spawn path. Add a phase timer to the overwatch role. Use CQ-53 separation for spacing. Draw members with GPU-instanced sprites so the work is visible. **Operator choice pending:** if individual soldiers aren't wanted on the map, unschedule instead. | CPU sim · GPU instanced draw | M |
| CQ-55 | **PHYS-04 projectile hits** (wire) + **PHYS-11 pair pipeline** (delete) — `terrain/material/registry.rs:411` `probe_hits_solid`, `combat/ballistics.rs` `step_ballistic_flights`, `systems/collision/mod.rs` | `probe_hits_solid` is only called by its test, and missiles fly straight to the target cell through hills and buildings. The pair pipeline (300 lines added in `afdf586c`; the module was a 3-line stub before) pairs moving bodies, but nothing creates a body. Its "spatial hash" is O(n²), and `warm_revalidate` never re-checks the cell. | **Wire:** sample each flight's swept segment against heightfield + structure material via `probe_hits_solid`; impact early where it hits.<br>**Delete:** the coherent pair pipeline (`CoherentPairCache`, the broad/mid/narrow/warm functions). Keep `CollisionPlugin` minimal and keep the PHYS-04 material filter table. Spacing comes from CQ-53 steering. If thousands of hard-non-overlap movers ever exist, design a uniform grid then; that is where a GPU broad phase could pay off. | CPU | S–M |
| CQ-56 | **MOVE-09 missile wind** — `combat/ballistics.rs:280`, `drone_missions.rs:716` | `launch` always passes `wind: None`, so `wind_drift_tiles` / `refine_missile_aim` never run in play. | Pass the station wind from `pulse_missile_sorties`. | CPU | S |
| CQ-58 | **TAC-05 fort + chokepoints** — `orders.rs:346 → plan_fortification_rows` (`fortification.rs:157`), `TerrainAnalysisBook::chokepoint_at` | The fort path passes `None`, and `chokepoint_at` is unused. | Pass `TerrainAnalysisBook`; give fort placement and faction AI the chokepoints. | CPU | S |
| CQ-59 | **Build-readability harness** — `construction/build_readability_harness.rs`, `mod.rs:349-352` | Registered unconditionally in production `Update`. | Move it under `engine/test_harness` / scene-gated. | — | S |

### 5b. CQ-57 — smaller unwired APIs, split

| ID | Item (ledger row) | Decision | Do | Effort |
|---|---|---|---|---|
| CQ-57a | TAC-07 `TacticalFragMap` (`ai/faction/log.rs:23`) | **Wire** | `record_loss` in combat resolve on strength loss; read `heat_at` in faction decide (avoid or reinforce). | S–M |
| CQ-57b | TAC-15 `static_board_eval` (`ai/faction/decide.rs:171`) | **Wire** | Use it as a scoring input inside the existing decide step, not as a second AI. | S |
| CQ-57c | DEC-11 `arbitrate_trigger_rules` / `TriggerFactMemory` (`scenario/trigger_registry.rs:73`) | **Wire** | Resolve simultaneous scenario triggers by priority, then id. Fix the failing `ecs12_pathway_modules_hold_no_sim_write_surface` lint in the same change. Correct the "Rete alpha memory" doc claim. | S |
| CQ-57d | PHYS-08 `DestructibleDamageStage` (`construction/demolish.rs:40`) | **Wire** | Map fire integrity + blast damage to Intact → Scarred → Compromised → Collapsed. Drive the building visual state from it. Collapse goes only through `execute_demolish_at_tile`. | M |
| CQ-57e | EXEC-03 `allocate_nested_domain_budgets` (`io/streaming/budget.rs:164`) | **Wire** | Replace the `STREAMING_SPAWN_ENTITIES_PER_FRAME/PER_CHUNK` env knobs with the priority budget cascade. | M |
| CQ-57f | LOOP-09 `WorldFieldLayerEpoch` (`strategic/world_field_layers.rs:105`) | **Wire** | Bump it in the layer writers; let readers skip work when unchanged (a CPU saving). | S |
| CQ-57g | PCG-07 `enqueue_settlement_road_skeleton` (`strategic/settlement/road_skeleton.rs:214`) · `plan_town_growth_v1.md` (reopened) | **Wire** | Call it from settlement growth, through the construction funnel. | M |
| CQ-57h | IO-08 `POST_LOAD_DERIVED_STEPS` (`io/save/apply.rs:21`) | **Check, then wire** | The list claims `portal_attachment_map` and `route_dirty_tiles` are rebuilt after load; nothing rebuilds them. Test save → load → compare. If stale, add the rebuild. Then delete the string list. | S–M |
| CQ-57i | RND-15 scorch decals (`fire_vfx/emit.rs:75`, `fire/chunk_surface_fire.rs:64`) | **Wire one** | Keep one mapping and apply it as a tint in the existing overlay/terrain shader (GPU, render-only); delete the other. | S |
| CQ-57j | LGT-06 `render/view_runtime/tonemap_hdr.rs` | **Operator choice** | Wire it through Bevy's `Tonemapping` (GPU, cheap) if you want the HDR look; otherwise delete. | S |
| CQ-57k | DES-06 `ecosystem_logistic_step` (`ecology/cell_stand.rs:61`) | **Operator choice** | Makes regrowth slow down near full capacity. It changes the ecology balance, so run `playcheck.py` before and after. | S |
| CQ-57l | DEC-09 `plan_goap` (`ai/faction/decide.rs:271`) | **Delete** | A second AI decision path next to the real one. | S |
| CQ-57m | `classify_storm_with_plume`, default-k `combine_visibility_m`, `relax_sense_from_samples` | **Fold, then delete** | Copies of `classify_storm_with_context`, `combine_visibility_m_k` and the registered sense system. First move any missing input (e.g. plume membership) into the used function. | S |
| CQ-57n | TAC-10 `CombatAuthorityTier` · DES-03 `ENGINE_STARTUP_PHASES` · `VoxelOccupationConfig::from_env_*` · `sim_pause_env_latched`, `sim_time_secs_display` · `saw_forecast_route_invalidation` · ANIM-03 `quantize_unit_interval_f32` · `write_cells_mut_pub` / `swap_pub` / `chunk_key_pub` · empty `contamination_tick` (`substrate/atmosphere/contamination_tick.rs:13`) | **Delete** | Doc-only types, string lists, unused env helpers and aliases. | S |

Audio low-pass/duck wiring is in CQ-10.

## 6. Phase P5 — performance

| ID | Where | Fix | Effort |
|---|---|---|---|
| CQ-60 | `systems/fire/chunk_fire_overlay.rs:336-352` | Each tick deep-clones every chunk's `heat/fuel/burn_hold` (sleeping chunks included), then clones the whole HashMap plus `sleep_wake`. Snapshot only active chunks plus their edges, and borrow instead. Also remove the per-neighbour Vec in `source_edge_cell_indices`. Re-measure against `c5d6b5c0`'s 1.66 ms/frame. | M |
| CQ-61 | `atmosphere/update.rs:113,177`, `advect.rs:79`, `frame.rs:396` | `bump_revision()` runs unconditionally every tick, and `sim_micros` is in the key, so the AtmosphereFrame rebuild gate never skips. Bump only on real mutation. Also fix `substrate/atmosphere/mod.rs:81`, whose doc says revision is "never read". | S |
| CQ-62 | `coupling_terms.rs:248-301`, `frame.rs:545` | `detect_firestorm_plumes` runs twice per frame and allocates each time. Compute it once in the frame and expose the mask. | S |
| CQ-63 | `compute/heat_diffusion_gpu.rs:67`, `sim_attention.rs:219-225` | The ROI grows to the bounding box of all fires, never shrinks, and on expand replaces the old box instead of unioning it (heat is lost). The attention pass scans the full grid every frame with one focus per hot cell. Bound or tile the ROI, union on expand, cap the foci, and throttle. | M |
| CQ-64 | `construction/voxel_massing.rs` `sync_voxel_massing_visual_requests` | It clears `ConstructionVisualRequests` every frame, which defeats the `is_changed()` ghost-mesh gate (`pixel_world/layers.rs:519`). Clear only when something changed. | S |
| CQ-65 | `audio/mix_frame.rs` `cap_combat_oneshots_in_place`, `audio/ui_hooks.rs` `apply_alert_bark_priority_in_place` | They drain and rebuild Vecs every frame. Return early when empty, or use `retain`. | S |
| CQ-66 | `render/pixel_world/rain.rs`, `clouds.rs` | They rebuild and upload a CPU mesh every frame (a second precip path next to the GPU pass). Cache it and rebuild on change. | S |
| CQ-67 | `substrate/slab.rs` `reserve_chunks` | Forced 64-step linear growth on a HashMap. Drop it. | S |

---

## 7. Phase P6 — test hygiene

- **CQ-70 — delete source-grep tests.** There are ~50 new `include_str!("*.rs")` / `read_to_string(...rs)` tests in range that assert doc comments contain citation strings (`"gea_v2_e4"`, `"RND-04"`, `"11.3.12"`) or that literal code text exists. Biggest offenders: `construction/blast.rs` (9), `construction/demolish.rs` (8), `combat/ballistics.rs` (6), `render/probes/vfx_capture_env.rs` (5), `weather_vfx/mod.rs`, `fire_light_emission.rs`, `present.rs`, `tonemap_hdr.rs`, `voxel_massing.rs`, `core2d_overlay_order.rs`, `vegetation_species.rs`, `terrain/material/registry.rs`, `gui/hud/*`. The `repo_src` / `production_src` helpers are copy-pasted at least four times. **Keep** only genuine architecture lints, e.g. the RND-10 PSO scan; collect those in one `tests/arch_lints.rs`. Find them with: `git diff 8abc4326 HEAD -- src | grep -n '^+.*include_str!(".*\.rs")'`.
- **CQ-71 — replace tautological tests.**
  - `combat/front_stock_pool.rs:176` re-implements the decrement inline; call `try_consume_bom_unit` instead.
  - `toe_demand.rs` contains `assert!(true)`.
  - `visual_capture_probe.rs:251-266` asserts a constant equals itself.
  - `vfx_capture_env.rs` asserts `!enabled() || var_os().is_some()`.
  - `test_pcg08_generate_and_test.py:14-17` formats the same object twice; generate twice instead.
  - `test_blender_addon_package.py` only checks strings exist.
- **CQ-72 — remove `let _ = x;` noise.** Examples: `toe_demand.rs:120,128` (unused `Res<TheaterBook>`), `chunk_fire_overlay.rs:348`, `coupling_terms.rs:301` (`let _ = ATMO_GRID_SIZE; // keep … visible to reviewers`), `grammar_iterate.py` (`_ = write`).

---

## 8. Phase P7/P8 — tooling and docs consolidation

### Tooling (`@coder-mcp`)
| ID | Item |
|---|---|
| CQ-75 | Three HTML dashboard builders (`tools/sim_review/score_dump.py` `_html/_html_multi`, `tools/studio_dash/render_report.py`, `art_pipeline_suite/dashboard_panel.py`). Keep `sim_review` as the one review path, and share one HTML kit for the others. Delete `studio_dash/mission_adapter.py` (only its own test calls it). |
| CQ-76 | `floors_histogram` / `massing_qc_view` are duplicated in `art_pipeline_suite/voxel_massing_panel.py:123-170` and `tools/studio_dash/massing_qc.py`, and have drifted. Keep one, inside `rust_engine_mcp`, and remove the 5 `sys.path.insert` hacks (the core lib currently imports from the Tk GUI package). |
| CQ-77 | `building_quality_qc.py:290` re-implements the Rust BQ-A2 score with adjacency hard-coded to 0, so it can score up to +10 above Rust. `veg_species_table.py` regex-parses RON. Add shared-fixture parity tests (the Halton pattern) or call the Rust side. |
| CQ-78 | Delete or fix:<br>• `tools/validators/sole_writer_check.py` and `determinism_lint.py` always `return 0` and are never run. Either fail on new violations against a baseline file, or delete them.<br>• `tools/sim_review/tune.py` is a stub. Delete it, or mark T7 as not built.<br>• `assets/staging/reports/pcg_06_grammar_rule_coverage.json` is 877 lines, generated, stale and unread. Remove it from git and write it to `debug_runs` instead.<br>• `assets/configs/buildings/examples/massing_preview_pilot_rail_warehouse_v1.json` is byte-identical to the copy in `tools/mcp/schemas/examples/`. Keep one.<br>• `tile_map_tiles_tile_factory_floor_greybox_001.png` is a 70-byte stub. Check it. |
| CQ-79 | Small fixes:<br>• `lookup.py:25`, `build_toc.py:17`: replace the hard-coded `$REF_BOOKS_DIR` with an env var or a repo-relative path.<br>• Narrow the silent `except` blocks in `studio_task.list_tasks` and the panel pilot fallback.<br>• `playcheck.py` dump unlink: abort on failure.<br>• Prune the ~12 new "Uncategorised" CLI commands rather than adding MCP wrappers.<br>• Strip `ref_books_dual_cite` from data schemas; it is book bookkeeping in asset payloads. |

### Docs and process (`@plan-orchestrator`)
| ID | Item |
|---|---|
| CQ-80 | **Make `adoption.tsv` the single ledger.** *Re-grade done 2026-11-16:* new status `UNWIRED` (code exists, no production caller). Rows touched by §5 are now `UNWIRED`/`PARTIAL` with `REVIEW 2026-11-16 → CQ-xx` in the evidence column, and RND-07's evidence is retracted. **Rule from now on:** a row moves to `ADOPTED` only with a production caller `file:line`. **Still to do:** remove the duplicated status tables from `plan_ref_books_upgrade_v2.md` §9 and `v3`, link to the TSV, and regenerate the counts (v3 says 117/2/14; the TSV says 128/1/4). |
| CQ-81 | **Delete `plan_ref_books_upgrade_v3.md` §0.1 "Anti-stall"** and any copy in `.cursor/rules/*.mdc`, `.cursor/skills/`, `.claude/skills/`. Replace it with: "Stop and report `BLOCKED`/`UNWIRED` when a slice can't be wired honestly." |
| CQ-82 | Put a "superseded by adoption.tsv" banner on `plan_ref_books_upgrade_v2.md`. Point `.cursor/skills/ref-books/SKILL.md` and `.cursor/rules/ref-books.mdc` (plus the `.claude` copies) at the TSV and the defer-unlock plan. |
| CQ-83 | Archive the PROGRAM CLOSE plans (14, plus older ones) to `src/dev/archive/`. Fix `AGENTS.md:11`, which lists closed programs as "next product charters", by moving the plan links to `development_plan_index.md`. Fix `plan_ref_books_defer_unlock_v1.md:161` (OCC "default remains off"). Fix `plan_playtest_quadrant_world_v1.md:6` ("plan only") — `engine/playquad.rs` shipped. |
| CQ-84 | Plans cite evidence JSON under gitignored `debug_runs/`, which nobody else can check. Either cite a test name / matrix row, or stop citing. Retract the "user world-look sign-off" for erosion and terrain analysis unless a record exists. |

### Warnings (`@coder`)
**CQ-85.** Get `cargo check --all-targets` to ≤ 50 warnings:
- Migrate the 162 deprecated picking aliases to `picking::hover::Hovered` / `ui::Pressed`.
- Resolve the ~45 dead-code warnings (most are covered by CQ-57).
- Remove the 2 unused dependencies.
- Fix the 8 `private type in public API` warnings.

---

## 9. Suggested order and ledger

Order: **P1 → P2 → P3 → P4 → P5 → P6 → P7/P8**.

- CQ-80 and CQ-81 are doc-only and can run in parallel at any time. Do them early, so the next agent wave isn't steered by the anti-stall rule.
- CQ-01, CQ-07 and the two transport tests in CQ-20 all touch save/load. Have one agent do them together.
- P4 dependency chain: **CQ-02 → CQ-53 routing → CQ-51/CQ-52 feed it**. CQ-53 steering and CQ-54 members share movement code: one agent. CQ-50 (GPU heat) also covers CQ-38 and CQ-63; give all three to one agent.
- Owning plans reopened 2026-11-16 with a "Review 2026-11-16" block: `plan_dense_heat_wgsl_v1.md`, `plan_move10_wildlife_v1.md`, `plan_ref_books_nav_13_v1.md`, `plan_unit_member_v1.md`, `plan_town_growth_v1.md`, `plan_voxel_occupation_v1.md`, `plan_ref_books_defer_unlock_v1.md`, `plan_ref_books_upgrade_v3.md`. Close each one again only when its CQ items are done.
- Don't parallelise edits to the same files. In particular, CQ-30/CQ-07 (`voxel_occupation.rs`), CQ-02/CQ-35 (`cost_forecast.rs`) and CQ-61/CQ-62/CQ-39 (atmosphere) should each go to one agent.

| ID | Status | Commit | Note |
|---|---|---|---|
| | | | |

**Exit criteria:**
- lib tests: 0 new failures, and the defer list is shrinking
- pytest green
- no `*_live.json` / witness / verdict-string producers in `src/` or `tools/mcp/`
- every ADOPTED row cites a production caller
- `cargo check` ≤ 50 warnings
- `playcheck.py` matrix no worse than at `afdf586c`
