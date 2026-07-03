# SCHEDULE SYNC AUDIT + IMPROVEMENT PLAN v1 — Update-schedule / SystemSet ordering
# Generated 2026-07-03 from full sweep (3-agent audit + hand verification of all H claims).
# Companions: codebase_index_v1.md (entry codes, K##) · plan_cleanup_v1.md (R#/S#/P#/T#/D#)
# Canonical references: .cursor/skills/bevy-simulation-grade/07-repo-authority-map.md ·
#                       docs/archive/2026-06-prompts-guides/runbooks/guides/ecs_systems_schedule_runbook_v1.md
# Issue codes here: SCH-E# (set-edge topology) · SCH-A# (ordering anchors) · SCH-T# (timing/pause)
#                   SCH-P# (schedule placement) · SCH-D# (docs/policy)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-SCHEDULE-SYNC-v1
# status:       PLANNED — catalog signed 2026-07-03; Wave 1 not started
# companion:    codebase_index_v1.md · plan_cleanup_v1.md (hygiene + D2/D3 gating)
# owner:        @sim-steward sequences · @coder implements fire/timing slices
# regression:   cargo test -p proc_A_dine01 --lib stage5 construction
#               validate-report cargo (compression 3) after every code slice
#               ambiguity-warn count must not increase post-SCH-E1 triage
# agent-lang:   BLANG:PRE → pipeline-preflight before Wave 1+
# index:        development_plan_index.md + HANDOFF.md § PLAN-SCHEDULE-SYNC-v1 (linked 2026-07-03)
# HANDOFF:      lease block + Wave 1 queue seed below
#
# Priority vs active lanes (human P0 wins):
#   P0  OVR-APS-PRESENCE-OPERATOR-001 — human operator lease; orthogonal
#   P1  PERF-INSTR-VFX-002 — baseline before SCH-T3 run_if(sim_ticking) additions
#   P1  PLAN-SCHEDULE-SYNC Wave 1 — SCH-E1 + SCH-T1 (low risk; safe parallel to perf baseline)
#   P2  Wave 2 SCH-E2 / SCH-A1 — fire authority + anchor sets; steward review required
#   P2  plan_cleanup Phase 1 D2/D3 — sequence SCH-A1 items 4,6 after instrumentation gating
#   P3  Wave 3 SimClock + SCH-D1 authority-map paste

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX (do not parallelize without steward sign-off)
# ═════════════════════════════════════════════════════════════════════
# Lane                         | Overlapping SCH items     | Rule
# -----------------------------|---------------------------|------------------------------------------
# plan_cleanup Phase 1 D2/D3   | SCH-A1 items 4,6          | Anchor-set migration after D3 gating
# plan_cleanup R2/R3           | SCH-E2                    | Same SIM-FIR territory — single steward owner
# PERF-INSTR-VFX-002           | SCH-T3                    | Baseline before pause run_if additions
# Stage 5 FULL_APP gate        | SCH-E2, SCH-E3, SCH-A1    | stage5 --lib + witness refresh per slice
# APS operator P0              | none direct               | Orthogonal
# plan_cleanup Phase 0         | SCH-P1, SCH-E6            | Fold dormant plugins / set hygiene into CLN-P0-*

# ═════════════════════════════════════════════════════════════════════
# SLICE TEMPLATE (copy per queue row)
# ═════════════════════════════════════════════════════════════════════
# id:            SCH-W<n>-<CODE>-001   e.g. SCH-W1-E1-001
# issue:         SCH-E1 | SCH-T1 | …
# owner:         sim-steward | coder | coder_a | coder_b
# territory:     paths from item description
# exit_witness:  validate-report cargo + optional debug_runs/* refresh
# blocks:        slice ids that must finish first (SCH-E1 before most others)
# parallel_ok:   true only if conflict matrix row is empty for both slices
# stage5_req:    true for SCH-E2, SCH-E3, SCH-A1 (fire/stage5 anchors)

# ═════════════════════════════════════════════════════════════════════
# ACTIVE PHASE
# ═════════════════════════════════════════════════════════════════════
# current:   Wave 1 PLANNED — picks in HANDOFF.md § PLAN-SCHEDULE-SYNC-v1
# next_pick: SCH-W1-E1-001 (ambiguity warn) then SCH-W1-T1-001 (pause dt)
# blocked:   SCH-E2 until fire semantics decision + debug-intelligence routing if touching R2/R3 files

# ═════════════════════════════════════════════════════════════════════
# WAVE 1 QUEUE SEED (ready for HANDOFF / agent pick)
# ═════════════════════════════════════════════════════════════════════
# id                  | issue   | owner       | effort | exit_witness / notes
# SCH-W1-E1-001       | SCH-E1  | coder       | XS+M   | ambiguity warn enabled debug; triage list; count baseline recorded
# SCH-W1-E3-001       | SCH-E3  | coder       | XS     | BuildProfiles.after(ChunkEnvironmentSet::Fire) direct edge; stage5 witness
# SCH-W1-E4-001       | SCH-E4  | coder       | XS     | HybridSimPipeline → StrategicFieldPipeline::LogisticsNetInject set edge
# SCH-W1-T1-001       | SCH-T1  | coder       | XS-S   | hybrid_emotion_drift + settlement_and_corridor_tick use ctrl.dt_scale(); pause witness
# SCH-W1-P1-001       | SCH-P1  | sim-steward | XS     | Classify dormant Aluminum/Concrete production plugins → cleanup CLN-P0 or delete
# SCH-W1-P2-001       | SCH-P2  | coder       | XS     | Document PostUpdate terrain_rebuild → network_flow_dirty 1-frame contract
#
# Wave 1 gate: E1 triage baseline recorded → unlock Wave 2 planning pick

# ═════════════════════════════════════════════════════════════════════
# SWEEP SNAPSHOT (measured 2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# add_systems: Update=80 Startup=30 OnEnter=15 PostUpdate=8 PreUpdate=5 Last=5 First=2
#              EguiPrimaryContextPass=5 ExtractSchedule=1 · FixedUpdate=0 · apply_deferred=0
# SystemSet types=35 · configure_sets sites=42 · .chain()=148 · run_if=172
# ordering edges: .after=369 .before=50 — of which 233 use BARE FN-NAME anchors, 135 use sets
#   → 63% of ordering edges violate the repo's own guardrail (07-authority-map: "use SystemSet
#     names, never ad-hoc system-name anchors")
# ambiguity detection: NOT enabled anywhere (no ScheduleBuildSettings / edit_schedule)

# ═════════════════════════════════════════════════════════════════════
# VERIFIED GLOBAL Update ORDER (reconstructed — fuller than authority map)
# ═════════════════════════════════════════════════════════════════════
# SimControlSystemSet::ApplyOperatorInput → AdvanceSimTick            (sim_control.rs:99)
#   ├─ ChunkEnvironmentSet: Lod → Weather → Ecology(⊃LandscapeBurnSet) → Fire   (chunk_environment_set.rs:26)
#   │    └─ Fire → AtmospherePipelineSet::FieldFill → WindAdvect → Emitters → Particles
#   │             → Coupling → VisualExtract → RenderPrep → Diagnostics        (atmosphere/pipeline.rs:35-49)
#   │                                          Diagnostics → TransportSchedule::Topology (engine:196-200)
#   ├─ SimEffectSystemSet::Drain (after AdvanceSimTick ONLY)          (sim/effects/mod.rs:63-67)
#   ├─ TransportSchedule: Topology → FieldIntegrate → CostCache       (transport/mod.rs:47-49)
#   │    ├─ NavSets::DamageSpeedAdjustment → MotionCalculation        (navigation/schedule_plugin.rs:16)
#   │    ├─ StrategicFieldPipeline::GraphSync … → InfrastructureSiteSet chain (strategic/plugin.rs:196-215)
#   │    └─ LogisticsSimulationSet: PortalAttach → … → Witness        (economy/logistics/mod.rs:79-89)
#   │         └─ HybridSimPipeline: IntentReset → EmotionDrift → AgentIntent (strategic/sim.rs:230-232)
#   └─ MapCameraSystemSet: ApplyInput → DeriveDesired → Smooth
#        → ViewAuthoritySystemSet::SyncViewManager → FireVisualFrameSet::BuildProfiles
#        → WorldRepresentationSystemSet::ComputeFrame → ComputeDispatchSystemSet::Dispatch
#        → FireVisualFrameSet::ProjectGpu → LocalLightExtractSet::Collect      (engine:186-238,303-314)
#
# VERIFIED NON-ISSUES (do not re-flag):
#  · behavior_sync_entity_ids_system "double registration" — behavior_entities.rs:130 is inside
#    #[cfg(test)] mod tests; sole prod registration = behavior_brain_plugin.rs:25. FALSE ALARM.
#  · Transport decay vs logistics reads — fully chained: FieldIntegrate → CostCache → RouteRefresh.
#  · Transport decay while paused — dt = delta * sim.dt_scale(); dt_scale()==0 when paused ⇒ no-op
#    (wasted schedule slot only, see SCH-T3).
#  · EguiPrimaryContextPass — read-only draw + deferred events; SCH-P3 is event-deferred (L), not direct mutation.

# ═════════════════════════════════════════════════════════════════════
# A) SET-EDGE TOPOLOGY (SCH-E#)
# ═════════════════════════════════════════════════════════════════════

SCH-E1 | H | Enable ambiguity detection in debug builds — the systemic lever
  Nothing in the repo turns on schedule ambiguity checking; every gap below was silent.
  ACTION (engine_with_worldgen.rs, early in build, cfg(debug_assertions)):
    app.edit_schedule(Update, |s| s.set_build_settings(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Warn, ..default() }));
  Then triage the warning list once; add `.ambiguous_with()` for intentional pairs so the signal
  stays clean. Effort: XS code + M triage. Do FIRST — it validates every other item here.

SCH-E2 | H | SimEffectSystemSet::Drain unordered vs ChunkEnvironmentSet fire/ecology ticks
  Drain (lightning/grid-overload → EmberSpotIgnitionEvent → fire writes) is ordered only
  .after(AdvanceSimTick) (sim/effects/mod.rs:63-67). No relative order to ChunkEnvironmentSet::Fire
  or LandscapeBurnSet, both of which read/write the same fire heat state in the same frame [K09].
  Ember application vs fire tick order is nondeterministic across schedule builds.
  ACTION: decide semantic (effects-before-tick recommended: strike lands, same-frame spread), then
    configure_sets(Update, SimEffectSystemSet::Drain.before(ChunkEnvironmentSet::Weather));
  and add a witness assert for same-frame heat consistency. Effort: S. Owner: @coder via
  @sim-steward review (fire authority).

SCH-E3 | M | Fire extraction ordering to sim is TRANSITIVE-ONLY
  FireVisualFrameSet::BuildProfiles.after(AtmospherePipelineSet::VisualExtract) (engine:186-190);
  reaches ChunkEnvironmentSet::Fire only through the atmosphere chain. If AtmospherePlugin is
  disabled/reshuffled the sim→extract ordering silently evaporates.
  ACTION: add direct edge BuildProfiles.after(ChunkEnvironmentSet::Fire) beside the existing one —
  belt-and-suspenders, zero behavior change today. Effort: XS.

SCH-E4 | M | HybridSimPipeline anchored to a bare system, not the producing set
  HybridSimPipeline::IntentReset.after(logistics_net_inject_into_overlays) (strategic/sim.rs:232)
  hides a pipeline-level dependency behind one fn.
  ACTION: replace with set edge .after(StrategicFieldPipeline::LogisticsNetInject). Effort: XS.

SCH-E5 | M | Extraction siblings unordered among themselves
  VegetationExtractFrameSet has no configure_sets edge to FireVisualFrameSet (burn overlay read may
  lag 1 frame); WaterSurfaceVisualSet lives inside TileWorldFallbackAfterFireExtract with no edge
  to vegetation/domain overlays.
  ACTION: one consolidation block in engine_with_worldgen ordering all extract-family sets relative
  to FireVisualFrameSet::BuildProfiles (they all consume the same view/sim snapshot). Effort: S.

SCH-E6 | L | Dormant / unwired sets
  ConcreteChainE2eSet — USED as .in_set/.after anchor in economy/activation/bridge.rs but has no
  configure_sets edge (set membership only). FullRenderDiagnosticSet — members wired via .in_set but
  no configure_sets ordering edge. GameSystemSet (engine/sets.rs:18) — defined, zero usages (dormant);
  NavSets in same file is live.
  ACTION: add configure_sets for ConcreteChainE2eSet + FullRenderDiagnosticSet OR migrate to parent
  pipeline sets; delete GameSystemSet or wire; fold into plan_cleanup_v1 Phase 0 hygiene. Effort: XS.

# ═════════════════════════════════════════════════════════════════════
# B) ORDERING-ANCHOR HYGIENE (SCH-A#) — 233 bare fn anchors → sets
# ═════════════════════════════════════════════════════════════════════

SCH-A1 | H | Cross-domain bare anchors (fragile: any rename/move breaks distant callers)
  Worst clusters (edges | anchor domain ← dependent domains):
   · hud_product_shell_egui_root          5 | gui/hud ← gui, render (debug_viewport_overlay.rs:137)
   · commit_construction_site_system      5 | strategic ← economy/activation (bridge.rs:60,62,67)
   · extract_fire_simulation_snapshot     6 | render ← dev (stage5_live_todos.rs:637), engine (test_harness.rs:244)
   · merge_domain_projection_into_representation 4 | render ← gui (view_representation.rs:539), dev
   · emit_frame_perf_summary              3 | render ← dev (sim_spectrum_analytics.rs:1059), engine (ux_orchestration.rs:56)
   · sync_particle_draw_dispatch_from_policy 3 | render ← render ×3 files
   · evaluate_app_stage5_readiness        3 | render ← render (stall_watch brackets)
   · strategic_fields_coupling_tick       3 | strategic ← strategic (same file)
  ACTION — introduce 6 anchor sets, place the anchor fn in the set, migrate dependents:
   1. HudRootSet            (gui/hud/hud_root_tick.rs)      ⊃ hud_product_shell_egui_root
   2. SiteCommitSet         (strategic/site/systems.rs)      ⊃ commit_construction_site_system
   3. DomainProjectionSet::Merge (render/domain_projection_frame.rs) ⊃ merge_domain_projection_…
   4. FramePerfSet::Summarize (render/frame_perf.rs)         ⊃ emit_frame_perf_summary
   5. GpuParticleDispatchSet (render/gpu_particle_draw.rs)   ⊃ sync_particle_draw_dispatch_…
   6. Stage5ReadinessSet    (render/stage5_readiness.rs)     ⊃ evaluate_app_stage5_readiness
  extract_fire_simulation_snapshot already sits in FireVisualFrameSet::BuildProfiles — migrate the
  dev/engine callers to the set anchor, no new set needed. Effort: S each, mechanical.

SCH-A2 | M | Same-domain bare anchors (~180 remaining after A1)
  Mostly intra-file chains (fire extraction, world_preview, tile fallback, construction previews).
  POLICY: same-FILE fn anchors are tolerated; cross-FILE must use a set. Enforce forward via review
  checklist + the runbook (SCH-D1); burn down existing ones opportunistically when files are touched
  (pairs naturally with plan_cleanup Phase 4 splits — a split forces the anchor question anyway).

SCH-A3 | L | Anchors on re-exported paths (crate::render::emit_frame_perf_summary etc.)
  Re-export moves break distant .after() silently at compile time (good) but produce confusing
  errors. Resolved automatically by A1 set migration; note only.

# ═════════════════════════════════════════════════════════════════════
# C) TIMING / PAUSE / DETERMINISM (SCH-T#)
# ═════════════════════════════════════════════════════════════════════
# Canonical gate: SimControlState::should_tick()/dt_scale() (sim_control.rs:44-52). Fire, weather,
# ecology, vegetation, smoke, advection, logistics comply (dt * ctrl * lod). EXCEPTIONS: SCH-T1, SCH-T2.

SCH-T1 | H | Strategic hybrid + settlement ticks ignore pause & sim speed
  hybrid_emotion_drift_system (hybrid_brain.rs:376) AND settlement_and_corridor_tick (sim.rs:501)
  use time.delta_secs().clamp(0.0,0.25) with NO SimControlState — emotions drift, settlement
  adaptation/informal_pressure advance at wall-clock speed while paused and ignore speed multipliers.
  Divergence from every other domain tick.
  ACTION: thread `dt = time.delta_secs() * ctrl.dt_scale()` (0 when paused) in BOTH systems.
  Effort: XS-S. Verify with pause witness (agent emotion + settlement snapshot stable across paused frames).

SCH-T2 | M | Hydrology tick is frame-rate dependent AND pause-immune
  substrate/hydrology/background_tick.rs:48 applies fixed `+0.03/frame` (not dt-scaled) and gates
  on BaseState::Simulation only, not SimControlState. Saturation evolves faster at 144fps than
  30fps, and keeps evolving while paused.
  ACTION: convert to dt-scaled rate (0.03/s × dt × ctrl.dt_scale()) or an explicit fixed-interval
  accumulator; gate on should_tick(). Effort: S. Retune the constant when converting.

SCH-T3 | L | Paused-frame wasted work
  Ticks that comply via dt_scale()==0 still run their full query scan every paused frame
  (transport decay, field fills). ACTION: add run_if(sim_ticking) (new canonical run-condition
  wrapping should_tick()) to the heavy ones — free CPU while paused. Effort: XS each.

SCH-T4 | M | Centralize the sim clock — remove per-system dt recomputation
  Every tick independently computes dt * ctrl.dt_scale() * lod.dt_scale(); T1/T2 show what happens
  when one forgets. PROPOSAL: `SimClock` resource written ONCE in SimControlSystemSet::AdvanceSimTick
  { dt_sim, dt_wall, ticking: bool, frame_index }; domain ticks read SimClock.dt_sim (× their local
  LOD scale only). Single authority for time [K01]; makes T1-class bugs structurally impossible.
  Migrate incrementally (helper fn first, mechanical swap per domain). Effort: M.

SCH-T5 | DECISION | No FixedUpdate anywhere — sim is frame-rate *consistent*, not frame-rate *independent*
  All diffusive integration is dt-scaled (first-order IIR / semi-Lagrangian), so results differ
  subtly between 30 and 144 fps (integration-step size). Options:
   (a) ACCEPT + document: dt-scaled Update is the design (cheap, current behavior); record in
       authority map. Recommended default.
   (b) Migrate heavy diffusive ticks (fire overlay, atmosphere advection, hydrology) to FixedUpdate
       with render-side interpolation — true determinism, significant work (extraction reads
       mid-state, LOD dt_scale interplay, pause semantics). Only worth it if replay/lockstep or
       cross-machine determinism becomes a requirement.
  ACTION: make the choice explicit (1-paragraph ADR in the runbook), don't drift into it. Effort: XS(a).

# ═════════════════════════════════════════════════════════════════════
# D) SCHEDULE PLACEMENT (SCH-P#)
# ═════════════════════════════════════════════════════════════════════

SCH-P1 | L | Dormant production plugins with PostUpdate sim systems
  AluminumProductionPlugin (production_sys.rs:10) + ConcreteProductionPlugin (concrete/sys.rs:11)
  add sim chains to Update+PostUpdate but are REGISTERED NOWHERE (live ones = *RuntimePlugin via
  systems/production/runtime.rs:14). Dead scheduling code.
  ACTION: classify + delete or fold into the live runtime plugins → add to plan_cleanup_v1 as
  R8-adjacent (EN-LEG). Effort: XS.

SCH-P2 | L | terrain_rebuild_finished_marks_network_flow_dirty in PostUpdate (strategic/plugin.rs:254)
  Marks dirty in PostUpdate; StrategicFieldPipeline consumes next frame → intentional-looking
  1-frame lag, undocumented. ACTION: comment the contract or move into Update after the rebuild
  system. Effort: XS.

SCH-P3 | L | agent_permissions_ui emits sim events from EguiPrimaryContextPass
  (gui/agent_permissions_ui.rs:545) — writes PermissionGrantEvent/PermissionRevokeEvent in egui pass;
  handle_permission_grants applies AgentPermissions mutations in Update (agent_manager.rs:242).
  Event-deferred pattern (not direct component write in egui pass). Policy nit: sim-side mutations
  should not originate in egui pass even via messages — consider moving UI to Update or documenting
  next-frame apply contract. Effort: XS doc-only unless same-frame ordering bug is proven.

SCH-P4 | L | First/Last usage is clean
  First: frame_perf reset, stall_watch reset. Last: health/trace/fence writers. All
  instrumentation — correct placement; their gating is plan_cleanup D2/D3 territory, not a
  scheduling problem.

# ═════════════════════════════════════════════════════════════════════
# E) DOCS / POLICY (SCH-D#)
# ═════════════════════════════════════════════════════════════════════

SCH-D1 | M | Authority map + runbook are behind reality
  07-repo-authority-map.md documents the view/extract spine well but omits the whole sim spine
  (SimControl → ChunkEnvironment → Atmosphere → Transport → Strategic → Logistics → Hybrid,
  SimEffect Drain, compute-dispatch weave). Anyone scheduling sim work today has no canonical map.
  ACTION: paste the VERIFIED GLOBAL ORDER section above into the authority map + runbook at
  docs/archive/2026-06-prompts-guides/runbooks/guides/ecs_systems_schedule_runbook_v1.md; add the
  cross-file-anchor policy (SCH-A2) and the SimClock/pause contract (SCH-T4). Effort: S.
  Keep this plan's reconstruction as the source until then.

SCH-D2 | L | Review checklist addition (bevy-simulation-grade skill)
  New system PR must state: schedule · set · pause gating (SimClock/should_tick) · anchor type
  (set, never cross-file fn). One-line addition to the skill's 🚦 gate.

# ═════════════════════════════════════════════════════════════════════
# EXECUTION ORDER
# ═════════════════════════════════════════════════════════════════════
Wave 1 (1 session, low risk):  SCH-E1 (ambiguity warn + triage) · SCH-E3 · SCH-E4 · SCH-T1 · SCH-P1 · SCH-P2
Wave 2:                        SCH-E2 (fire semantics — steward review) · SCH-T2 · SCH-A1 (6 anchor sets) · SCH-P3 (doc-only unless bug found)
Wave 3:                        SCH-T4 (SimClock) · SCH-E5 · SCH-T3 · SCH-D1 · SCH-D2 · SCH-T5 (ADR)
Ongoing policy:                SCH-A2 burn-down piggybacks plan_cleanup_v1 Phase 4 file splits

GATES per slice: validate-report cargo (compression 3) · ambiguity-warn count must not increase ·
fire/stage5 witness refresh on SCH-E2/E3/A1(fire) · pause witness on SCH-T1/T2.
CONFLICTS: SCH-E2 touches SIM-FIR authority — coordinate with plan_cleanup R2/R3 lane (same files);
SCH-A1 items 4,6 touch frame_perf/stage5 — sequence with plan_cleanup D3 gating so diffs stay clean.

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES (2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# Review corrections applied: runbook path fixed; SCH-E6 ConcreteChainE2eSet "used but no configure_sets";
# SCH-T1 expanded to settlement_and_corridor_tick; SCH-P3 downgraded L (event-deferred, not direct write).
# HANDOFF lease + development_plan_index link landed 2026-07-03.
# First execution pick: SCH-W1-E1-001 then SCH-W1-T1-001.
