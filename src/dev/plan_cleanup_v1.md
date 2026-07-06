# CLEANUP PLAN v1 — Rust codebase
# Generated 2026-07-03 from full sweep. Companion: codebase_index_v1.md (entry codes CO-*/RN-*/… and concepts K##).
# Issue codes here (R#/S#/P#/T#/D#) are referenced from the index.
#
# GROUND RULES (apply to every item)
# 1. Classify before delete (obsolete vs transitional vs dormant vs incomplete) — prefer completion/migration over destruction.
# 2. One authority per resource; never move a writer without mapping current writers first [K01].
# 3. Every slice ends with `cargo check` → structured validation report, plus witness refresh where a witness exists [K02].
# 4. No behavior change inside a "split file" commit; mechanical moves only, re-exports preserved.
# 5. Severity: H = do in next working session(s), M = schedule, L = opportunistic.

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-CLEANUP-v1
# status:       PLANNED — catalog signed 2026-07-03; execution not started
# Companion:    codebase_index_v1.md (CO-*/RN-* entry codes, K## concepts)
# Deferrals:    plan_deferral_registry_v1.md — DR-CLEANUP-P2 (Phase 2+) · Phase 0 unblocked (MIG-V1 green)
# owner:        @sim-steward sequences slices · @coder implements · @planner for Phase 4 splits
# regression:   cargo test -p proc_A_dine01 --lib stage5 construction
#               validate-report cargo (compression 3) after every code slice
# agent-lang:   BLANG:PRE → pipeline-preflight → witness-brief before Phase 1+
#               intel-officer-sweep after any bulk "done" claim on this program
# index:        development_plan_index.md + HANDOFF.md § PLAN-CLEANUP-v1 (linked 2026-07-03)
# HANDOFF:      lease block PLAN-CLEANUP-v1 — active phase + slice id; do not reopen closed slices
#
# Priority vs active lanes (human P0 wins):
#   P0  OVR-APS-PRESENCE-OPERATOR-001 — human operator lease; orthogonal to cleanup
#   P1  PERF-INSTR-VFX-002 / plan_visual_perf_production_exec_001 — capture baseline BEFORE Phase 1 D2/D3
#   P1  PLAN-CLEANUP Phase 0 — zero-risk hygiene (this program)
#   P2  PLAN-CLEANUP Phase 1 — prod/debug separation (after perf baseline or in parallel with doc-only D2 registry)
#   P3  PLAN-CLEANUP Phase 2+ — authority/perf/splits; **DR-CLEANUP-P2** · @sim-steward review required

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX (do not parallelize without steward sign-off)
# ═════════════════════════════════════════════════════════════════════
# Lane                    | Overlapping items        | Rule
# ------------------------|--------------------------|------------------------------------------
# PERF-INSTR-VFX-002      | D2, D3, P4, P9           | Baseline witness BEFORE gating instrumentation
# Stage 5 FULL_APP gate   | R3, R2, S10, S5, P4      | stage5 --lib after each slice; refresh stage5_full_app_live.json on R3/S10
# APS presence (operator) | none direct              | Do not use cleanup to skip OVR-APS-PRESENCE-OPERATOR-001
# GPU terrain exec        | R6, T1, RN-TWF, P7       | Coordinate with plan_gpu_terrain_production_exec_001 before R6/T1
# VM-* multiview          | S4a, S10, R6             | Defer S4a until HUD/minimap owner signs split plan
# G-PLAY / --test harness | D1, D3, S5               | No harness behavior change without test scenario witness refresh
# OPS witness spine       | T2                       | T2 deferred — renames churn witness paths (refresh same PR only)

# ═════════════════════════════════════════════════════════════════════
# DEFER REGISTRY (explicit — not in active queue)
# ═════════════════════════════════════════════════════════════════════
# S4a   simulation_shell_phase2.rs split (3045 LOC) — DEFER until GU-MMS/minimap owner sign-off
# S11 + S1c building_grammar — ROUTED to plan_city_grammar_upgrade_v1 CITY-G0a/G0b (authoritative owner)
# R7    particle spine unify — DEFER until D3 gating lands (perf comparisons must be honest)
# T2    delivery-wave naming sweep — DEFER until product lane quiet; refresh all witnesses same PR
# R3    atmosphere dual-path — BLOCKED on debug-intelligence routing packet (see R3-Routing below)
# R6+T1 minimap source ladder — DEFER until GPU terrain exec decision on TWF vs MMC (**DR-GPU-TERRAIN-P0C**)

# R3-Routing-Package (required before any R3 code):
#   1. Refresh debug_runs/stage5_full_app_live.json baseline
#   2. Emit debug-intelligence YAML: writers on AtmosphereField + AtmosphereClipmapStack + gpu_weather_fire_field consumers
#   3. Define mutual-exclusion witness schema (p2h_authoritative vs legacy bridge active)
#   4. Rollback criterion: gpu_field_authoritative regression in VT matrix or stage5 gate false

# ═════════════════════════════════════════════════════════════════════
# SLICE TEMPLATE (copy per queue row)
# ═════════════════════════════════════════════════════════════════════
# id:            CLN-<PHASE>-<CODE>-001   e.g. CLN-P0-T6-001
# issue:         T6 | R1 | …
# owner:         sim-steward | coder | coder_a | coder_b
# territory:     paths from codebase_index entry code
# exit_witness:  validate-report cargo + optional debug_runs/* refresh
# blocks:        slice ids that must finish first
# parallel_ok:   true only if conflict matrix row is empty for both slices
# stage5_req:    true for Phase 2+ authority/perf/render slices

# ═════════════════════════════════════════════════════════════════════
# ACTIVE PHASE
# ═════════════════════════════════════════════════════════════════════
# current:   Phase 0 PLANNED — picks in HANDOFF.md § PLAN-CLEANUP-v1
# next_pick: Phase 0 — CLN-P0-* rows below
# blocked:   Phase 1 D2/D3 until PERF-INSTR-VFX-002 baseline captured OR env-latch registry doc-only slice ships first

# ═════════════════════════════════════════════════════════════════════
# PHASE 0 QUEUE SEED (ready for agent-queue-update / HANDOFF)
# ═════════════════════════════════════════════════════════════════════
# id                  | issue | owner       | effort | exit_witness / notes
# CLN-P0-T6-001       | T6    | operator    | XS     | disk freed; no code change; delete stale target_* dirs
# CLN-P0-R1-001       | R1    | coder       | XS     | validate-report cargo; confirm no legacy_engine doc refs
# CLN-P0-R4-001       | R4    | sim-steward | XS     | classify IO-SER legacy_drez → delete or archive doc
# CLN-P0-R8-001       | R8    | sim-steward | XS     | classify EN-LEG → delete or fix stale impl + MIGRATION note
# CLN-P0-S8-001       | S8    | coder       | XS     | CO-EVT traits/spacial.rs — remove println! defaults
# CLN-P0-T4-001       | T4    | sim-steward | XS     | classify empty placeholders → scaffold doc or delete
# CLN-P0-P10-001      | P10   | coder       | XS     | EC-LOG frequency comment / assertion only
# CLN-P0-T7-001       | T7    | sim-steward | XS     | fold floating TODOs into DV-TODO boards or fix 3 named items
#
# Phase 0 gate: all 8 rows done OR explicitly DEFER with steward note → unlock Phase 1 planning pick

# ═════════════════════════════════════════════════════════════════════
# A) REDUNDANCY (R#)
# ═════════════════════════════════════════════════════════════════════

R1 | L | CO-ENL src/engine/engine.rs (63 LOC, feature legacy_engine)
  Legacy EnginePlugin stub, 90% comments, dead-code marked. Canonical = CO-ENG engine_with_worldgen.rs.
  ACTION: delete file + feature flag once no doc references it; or keep 1 paragraph in CO-ENG header. Effort: XS.

R2 | M | SUB-SHM substrate/shim.rs ← SIM-FIR
  Dual-write mirrors ChunkSurfaceFire→ThermalState, ChunkSmokeField→ContaminationState [K09]. Drift compare exists
  (compare_dual_write_drift_system) but no same-frame double-write guard; 1-frame divergence window.
  ACTION: (1) document intended migration end-state (slab authoritative? ECS authoritative?); (2) order the mirror
  system explicitly after ALL fire writers via SystemSet; (3) add witness assertion for same-frame drift. Effort: S.

R3 | H | SUB-ATM bridge_legacy.rs vs SIM-ATM incremental_schedule.rs
  TWO live advection paths for smoke: ECS advect_atmosphere_field + clipmap advect_l0_preserving_mass. The "legacy"
  ECS→clipmap bridge runs EVERY frame even when P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE=true; clipmap smoke diverges
  from ECS after first bridge (no re-sync). Fallback render path (gpu_weather_fire_field) can read stale clipmap.
  ACTION: pick one authority per consumer. Gate legacy_atmosphere_bridge_system behind run_if(!p2h_authoritative)
  OR retire clipmap smoke and keep clipmap for contamination only. Add mutual-exclusion witness. Effort: M. RISK: render drift — route through debug-intelligence triage first.

R4 | L | IO-SER legacy_drez.rs (145 LOC, #![allow(dead_code)])
  Quarantined .dat loaders, unwrap-chained CSV parse (S7). Never called.
  ACTION: delete after confirming migration complete, or move to docs/migration reference. Effort: XS.

R5 | L | TER-HYD flow.rs (gen-time) vs SUB-HYD background_tick.rs (runtime)
  Two hydrology algorithms, intentional (offline vs live) but zero shared config; rehydrated chunks can mismatch.
  ACTION: extract shared HydrologyRuntimeConfig; add rehydrate witness check. Effort: S.

R6 | M | RN-TWF tile_world_fallback (1644) vs RN-TMA tilemap_adapter (485) vs RN-MMC minimap compositor [K20]
  Not true duplicates: TWF is the CANONICAL always-on path (raster + minimap image + heat overlay); TMA is opt-in
  feature; MMC is GPU overlay compositor. But consumers must choose per-frame (resolve_minimap_texture_source),
  and "fallback" naming is actively misleading (T1).
  ACTION: (1) decide stage-7 end state: is TWF minimap_image dead once MMC is authoritative? classify, then remove
  or mark transitional; (2) rename per T1; (3) document the source-resolution ladder in map_view/backend. Effort: M.

R7 | M | RN-PRT (fire particles ~2830 LOC) vs RN-WPT (water particles ~1640 LOC)
  Near-identical emit→dispatch→raster scaffolding (~1970 LOC of parallel infra); divergence is only class vs profile.
  ACTION: unify buffer/dispatch/raster scaffold into a generic instanced-particle spine over RN-GPU; keep domain
  emitters separate. ~500+ LOC net removal, one code path to maintain. Effort: M-L. Do AFTER D3 gating so perf
  comparisons are clean.

R8 | L | EN-LEG prod_comps.rs + legacy_transport_stubs.rs
  Both marked legacy, gated/unwired; prod_comps has a stale trait impl referencing a nonexistent field.
  ACTION: archive or delete; if kept as reference, fix the stale impl and link a MIGRATION note. Effort: XS.

R9 | M | BN-SUB bevysubengines/world_generator_plugin.rs (734 LOC)
  Parallel worldgen implementation, not wired anywhere, header says "do not reintroduce"; kept for save-format
  experiments. Classification: dormant-reference.
  ACTION: extract the save-format knowledge it protects into a doc/test, then delete the module. Effort: S.

NON-ISSUES verified during sweep (do NOT "clean" these):
  - IF-TRG vs SIM-TRN: authoring vs runtime split is correct layering [K19].
  - IF-SET vs STR-SET: attachment stub vs full sim — zero type overlap.
  - IO save/apply vs streaming/apply: shared core fn, intentional reuse.
  - CB roads/rail/power_lines/zones lanes: tool-layer duplication is per-lane by design.

# ═════════════════════════════════════════════════════════════════════
# B) SMELLS (S#)
# ═════════════════════════════════════════════════════════════════════

S1 | H | God-file cluster: generators & grammars
  a. TER-GEN world_generator_enhanced.rs 1517 — noise+Voronoi+hydrology+biome+strategic in one file, 38 unwraps (S6).
     Split: hydrology_gen_pipeline / strategic_field / voronoi driver / passes glue.
  b. SIM-ECO landscape_grammar_lg2.rs 785 + _map.rs 565 — eval + disturbance queue + witness interleaved.
     Split: eval / disturbance_queue / rollout_engine / witness.
  c. CB-GRM building_grammar.rs 965 — ~20 types + 500 LOC eval + deserialization.
     Split: grammar_types / grammar_deserialize / grammar_evaluation. Pair with S11.
  Mechanical splits, re-export from original path. Effort each: M.

S2 | H | Dual writers to sim resources [K09]
  ChunkSurfaceFire, ChunkSmokeField, AtmosphereField each have ≥2 writers (tick systems + mirrors/partial-GPU).
  Covered operationally by R2/R3; additionally: declare explicit SystemSets so writer order is schedule-enforced,
  not incidental. Effort: S per resource.

S3 | M | Witness-collector god files (>700 LOC boards)
  IO-SVA wave_s_artifacts 468, EC-ACW 799, EC-LGW witness_collectors 740, CB-DBG witness_collectors 783.
  Same shape everywhere: board + todo tracking + snapshot serialization in one file.
  ACTION: extract a shared witness_board base (register/refresh/serialize) in DV; per-domain files keep only their
  metrics. Do once, apply to all four. Effort: M.

S4 | H/M | God-file cluster: UI & tools
  a. GU-SHL simulation_shell_phase2.rs 3045 (H) — ops strip + context tray + build rail + minimap chrome + 200 types.
     Split: ops_strip / context_tray / build_rail / minimap_chrome (coordinate with GU-MMS minimap_shell — chrome
     currently overlaps it). Root file keeps layout composition + plugin.
  b. GU-REP world_representation.rs 1343 (M) — split types (bands/zones/bubbles) from policy engine.
  c. GU-EDM map_editor/mod.rs 1640 (M) — split by lane: terrain_brush(M3) / road_markers+bake(M4,R9) / snapshot(M5);
     scenario tools belong in scenario_script_panel.
  d. CB-GHO staged_ghost_panel.rs 730 (M) — split UI / state machine / input.
  e. CB-DBG placement_debug.rs 837 (M) — split probe / gizmo / witness.
  Also RN-TWF 1644 (M) — split raster / minimap image / heat-overlay blend after R6 decision.

S5 | H | RN-S5 stage5_full_app_harness.rs 2508
  ~60% serialization boilerplate (LogE01CaptureLane arms + snapshot structs) around a readiness probe.
  ACTION: move proof types → dev/stage5_proof_types.rs, snapshot structs → extraction side, keep orchestrator ≤600
  LOC. Confirm registration intent per D3 first. Effort: M.

S6 | H | TER-GEN unwrap density (38 in one file)
  tiles.get_mut(idx).unwrap() in tight loops, channel .unwrap() that panics if receiver drops. Worldgen failure
  currently = panic with no diagnostics.
  ACTION: convert to Result + emit failure into worldgen debug report (world_gen_diagnostics). Effort: S-M.

S7 | L | IO-SER legacy_drez unwrap-chains + stringly "Civilian"/"Military" parse — dead code; resolved by R4.

S8 | M | CO-EVT traits/spacial.rs println! in trait default methods (lines 11,16,21)
  Unguarded stdout in potentially hot trait calls.
  ACTION: replace with debug_assert!/warn_once or remove the unsupported-dimension defaults. Effort: XS.

S9 | M | CO-THN test_harness.rs 1912 always compiled into prod binary
  Plugin registration is conditional (test_mode only) but TestWorldHarness resource is always init'd and ~1400 LOC
  of harness logic ships in release. See D1 for the gating action; smell part = move file under engine/test/ and
  split scenario seeders from harness control. Effort: M.

S10 | M | GU-EDP PreviewCameraState authority drift
  Defined in preview_render_contract.rs but written from gpu_preview.rs, preview_lifecycle.rs AND
  render/full_render_diagnostic.rs (cross-module writer) [K01].
  ACTION: declare single writer (gpu_preview), make others readers; add authority header comment to mod.rs. Effort: S.

S11 | M | CB-GRM stringly-typed grammar ids
  id/usage/footprint_mode/massing_id/slot are raw Strings; corridor_type_for_profile does contains("rail") dispatch.
  Typos are silent bugs.
  ACTION: newtype ids (MassingId, SlotId, UsageId) + enum CorridorType, validate on deserialize. Effort: S-M.

S12 | L | STR-SIM sim.rs 788 + STR-S7 stage7_behavioral.rs 605
  Heatmap kinds (control/threat/recon/logistics) and HUD+witness+state mixed. Split per-heatmap / hud-vs-witness
  before any GPU-compute migration lands. Effort: M, defer until that migration is scheduled.

# ═════════════════════════════════════════════════════════════════════
# C) PERFORMANCE (P#)
# ═════════════════════════════════════════════════════════════════════
# Rule: measure with PerfScope/frame_perf before+after; ignore STALL substage_* artifacts (checkpoint-interval wall,
# see memory note) — trust upd_* + gpu_gap_ms.

P1 | H | SIM-FIR ember_spot_ignition.rs — O(n_chunks × n_embers) linear neighbor search via nested HashMaps.
  50ms+ spikes during intense fire. ACTION: spatial hash over 4-cell radius. Effort: S-M.

P2 | M | SIM-ECO landscape grammar witnesses rebuilt every frame (refresh_lg1/lg3_witness), no Changed<>/event gate.
  ~20ms on large maps; disturbances are sparse. ACTION: gate witness refresh on disturbance events / chunk dirty.
  Effort: S. (Overlaps D5.)

P3 | M | SIM-ATM atmosphere_field_fill_from_chunks iterates ALL chunks per frame, no dirty filter.
  5–10ms CPU at 100 chunks. ACTION: filter on ChunkEnvironmentDirty.smoke / Changed<>. Effort: S.

P4 | M | RN-EXT fire extraction snapshot — chunk heat maps collected/cloned per frame into FireSimulationSnapshot;
  SharedOverlayFieldBuffers partially duplicates it. ACTION: A/B buffer swap instead of clone; gate on
  Changed<ChunkSurfaceFire>; audit overlap with overlay_field_buffers. Effort: M.

P5 | M | SIM-TRN transport_topology_tick + field decay run unconditionally every frame on static graphs.
  O(edges) rebuild, ~10ms on large maps. ACTION: Changed<TransportGraph> guard + skip decay when field settled.
  Effort: S.

P6 | L | TER-GEN Voronoi centroidal relaxation is sequential while noise sampling is rayon-parallel; dominates
  worldgen (2–5s of ~10s). Offline path, no frame pressure. ACTION: par_iter over sites when touching S1a. Effort: S.

P7 | M | RN-TWF raster path — verify TileWorldFallbackRasterDirty marks only truly-changed chunks; fuse base+heat
  overlay into single pass; sparse texture updates. Effort: S-M (verify first — may already be correct).

P8 | L | GU-MAP consumers — verify egui texture paint skips when handle unchanged; wire HudEguiTextureCache into
  map_view consumers. Effort: S.

P9 | M | GU-REP LOD resolve — check for per-frame Vec/HashMap allocs and missing input change detection in
  WorldLodPolicyEngine::resolve (runs before render projection every frame). Effort: S (audit) + S (fix).

P10 | L | EC-LOG route solver allocates HashMap/HashSet per solve + string-key clones — acceptable if on-demand;
  add a frequency assertion/comment; pool if it ever runs per-frame. Effort: XS.

# ═════════════════════════════════════════════════════════════════════
# D) TIDY (T#)
# ═════════════════════════════════════════════════════════════════════

T1 | M | Rename RN-TWF "tile_world_fallback" → terrain/tile raster pipeline name that reflects its canonical role
  (it is the default path; the "fallback" name misleads). Do together with R6 decision.

T2 | L | Delivery-wave naming sweep: stage5_*→readiness_*, stage6_*→residency_*, stage7_*→operational_*; document
  M1-M4 / R9 / P2A / Wave P / F7 / LG# notation once in a root NOTATION doc [K22]. Multi-PR mechanical rename,
  ~15 files. Do LAST (churns witness names — refresh witnesses in same PR).

T3 | L | CO-UXO legacy states.rs (BaseState…) vs ux_states.rs (AppState…) — bridge is live; when migration
  completes, retire states.rs + stray `//gui_main::Gui;` line at states.rs:1.

T4 | L | Empty placeholders: traits/{rates,region,vehicles}.rs, engine/{sets,transitions,utils}.rs, utils/events.rs
  stub hook, construction/{build_validation,path_feedback,build_ghost}.rs near-empty. Classify: scaffold-intentional
  → keep with 1-line doc; abandoned → delete. Effort: XS.

T5 | L | GU-HUD mod.rs re-exports 80+ symbols untaxonomized; DV mod.rs similar. Add grouped sections (shell/panels/
  diagnostics) and consider gating diagnostic panel re-exports with the D3 outcome. Effort: XS.

T6 | M | Repo root: ~12 stale target_* build dirs (May–Jun, ~5–10 GB). Gitignored, zero code risk.
  ACTION: delete all but active `target/`. Effort: XS. (Also delete .claude_tmp_render_gui_report.md if present.)

T7 | L | TODO/FIXME triage: 226 hits repo-wide, but most are board-tracked [K14]; genuine actionables noted:
  manufacturing_plugin.rs throughput TODO, infrastructure/settlement distance stub, p5_agent_overlay placeholder.
  Fold into DV-TODO boards or fix; don't leave floating.

# ═════════════════════════════════════════════════════════════════════
# E) DEBUG/TEST vs PRODUCTION BUILD (D#)
# ═════════════════════════════════════════════════════════════════════
# Verified good already: 108 dev live_proof modules are cfg(test)-gated (zero prod impact); witness file IO master-
# gated by witness_writes_enabled() (release writes need RUNTIME_WITNESS_WRITES=1); features all default-off [K13];
# scenario/logistics/procedural/integration test modules cfg(test)-gated; both bins clean of dev deps.

D1 | M | CO-THN TestHarness in prod binary
  TestHarnessStatePlugin always registered (TestWorldHarness resource always present); TestHarnessPlugin gated on
  test_mode() at runtime, but ~1400 LOC of harness code ships in every release binary.
  ACTION: put the whole harness behind `feature = "test_instrumentation"` (or new `harness` feature) with a thin
  always-compiled launch-arg shim; CI --test jobs enable the feature. Effort: M.

D2 | H | Unconditional dev plugins in CO-ENG (engine_with_worldgen.rs:189-190,259-260)
  TestRunInstrumentationPlugin, SimSpectrumAnalyticsPlugin (1093 LOC, rolling-window stats + disk flush machinery),
  OrchestratorHealthPlugin added unconditionally; cost is latch-gated but systems are scheduled every frame.
  ACTION: (1) wrap SimSpectrumAnalyticsPlugin + TestRunInstrumentationPlugin registration in the same feature as D1
  or an is-instrumented launch check; (2) create ONE env-latch registry doc/module listing all 12+ latches
  (RUST_ENGINE_DEEP_DEBUG[_JSONL|_MINIMAP_ONLY|_FLUSH_EVERY], SIM_ANALYTICS[_FRAMES|_QUIET|_STRIDE|_FLUSH_SECS],
  STAGE5_PER_FRAME_HOOKS, RUNTIME_WITNESS_WRITES[_FORCE_OFF], ORCHESTRATOR_EXPORT_HEALTH, PERF_NO_VSYNC,
  MINIMAP_GPU_DEBUG, VIEW_RUNTIME_AUDIT…) with cost class per latch [K12]; (3) assert in CI that release pipelines
  never set the HIGH-cost ones (RUST_ENGINE_DEEP_DEBUG=1, SIM_ANALYTICS_FRAMES=1, STAGE5_PER_FRAME_HOOKS=1). Effort: M.

D3 | H | Render/GUI diagnostics registered unconditionally
  CO-ENG lines 112–119 add WorldGenChromeDebugPlugin, UiLayoutTreeDebugPlugin, SimViewSyncDebugPlugin,
  DebugViewportOverlayPlugin, VisualDiagnosticsPlugin with no gate. Also verify registration/gating of:
  FullRenderDiagnosticPlugin (933 LOC, serde_json::to_string_pretty per frame when armed, no change detection),
  FramePerfPlugin + RenderSchedulePerfPlugin + StallWatchPlugin (wall-clock stamps run regardless of verbosity),
  HudDevOverlayPlugin (584), FrameBudgetDiagnostics (492), stage5 harness probe (S5).
  ACTION: audit each → either run_if(env latch) at system level (cheap, keeps binaries identical) or feature-gate
  (smaller binary). Keep frame_perf as the ONE always-on perf spine (it feeds perf attribution), gate the rest.
  Effort: M. This is the highest-leverage prod-hygiene item together with D2.

D4 | M | SIM-FIR witness_collectors write_fire_ecology_live_proof_system wired in FirePlugin without feature guard
  (~50KB JSON serialization on hot path when witness gate is open; gate protects release-by-default but debug builds
  pay it every frame). Same pattern: SIM-ATM diagnostics resource always init'd.
  ACTION: throttle via LiveProofCadence everywhere (some already are), serialize only on cadence tick, and skip
  serialization entirely when unchanged. Effort: S.

D5 | M | DV-TODO boards (~3.6k LOC) always compiled + STAGE5_PER_FRAME_HOOKS INFO-logs every frame when set
  Boards are prod-inert but ship in binary and several register runtime hooks unconditionally.
  ACTION: fold under the D1/D2 instrumentation feature; ensure per-frame hook logging stays opt-in. Effort: S-M.

# ═════════════════════════════════════════════════════════════════════
# EXECUTION ORDER (phased, each phase independently shippable)
# ═════════════════════════════════════════════════════════════════════
Phase 0 — zero-risk hygiene (1 session):     T6, R1, R4, R8, S8, T4, P10, T7
Phase 0b — RTT viewport hygiene (parallel):  RTT-A1-* — **SHIPPED 2026-07-04** (witness `debug_runs/rtt_lane_witness_live.json`)
Phase 1 — prod/debug separation (H):         D2, D3, D1, D4, D5   ← do before perf work so measurements are honest
Phase 2 — correctness/authority (H):         R3 (route via debug-intelligence), R2, S2, S10, S6
Phase 3 — perf passes (measured):            P1, P3, P5, P2, P4, P9, P7, P8, P6
Phase 4 — structural splits (mechanical):    S5, S4a, S1a, S1b, S1c, S4c, S4d, S4e, S4b, S3, S11, S12
Phase 5 — consolidation & naming (churn):    R6+T1, R7, R9, R5, T3, T5, T2

VERIFICATION GATE per slice: cargo check (validation-first report), affected witness refresh green, frame_perf
before/after on Phase 3 items (PERF_NO_VSYNC=1 probe, compare upd_* not STALL substage_*).

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES (2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# Catalog accuracy: spot-checks confirm R3 (legacy_atmosphere_bridge_system ungated in substrate/mod.rs),
# D2/D3 (unconditional plugins in engine_with_worldgen.rs). NON-ISSUES block is correct.
# Gaps closed in this revision: PROGRAM METADATA, CONFLICT MATRIX, DEFER REGISTRY, ACTIVE PHASE,
# PHASE 0 QUEUE SEED, R3-Routing-Package, SLICE TEMPLATE.
# Remaining before execution: seed CLN-P0-* into HANDOFF or machine queue; link from development_plan_index.md
# when Phase 0 starts; capture PERF-INSTR-VFX-002 baseline before Phase 1 D2/D3 code gates.
# 2026-07-03: HANDOFF lease + development_plan_index link landed — Phase 0 ready for picks.
