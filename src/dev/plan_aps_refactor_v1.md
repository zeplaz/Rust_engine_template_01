# APS PROFESSIONAL-TOOL REFACTOR v1 — clean, organized, predictable
# Generated 2026-07-03 from full-architecture audit (53 files, ~11.9k LOC Python/Tk,
# tools/mcp/art_pipeline_suite/).
# **Companion plan** — execution hub + integrated queue seed live in
# [`plan_building_quality_v1.md`](plan_building_quality_v1.md) (BQ↔APSR coupling, Week 0 picks).
# aps-design-ux skill (tag surfaces, tooltip rules, operator rubric — authority for copy/UX rules).
# Issue codes: APSR-S# (state/services) · APSR-P# (panels) · APSR-D# (design system) ·
#              APSR-T# (tests/specs) · APSR-Q# (quality surfaces)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-APS-REFACTOR-v1
# status:       PLANNED — audit signed 2026-07-03; execution sequenced via plan_building_quality_v1.md
# integration:  plan_building_quality_v1.md § INTEGRATED EXECUTION GRAPH + queue seed
# priority:     parallel-safe with EVERYTHING (pure Python lane; zero overlap with Bevy MIG /
#               cleanup / schedule plans). Natural partner: run alongside PLAN-BUILDING-QUALITY-v1
#               so QC surfaces (APSR-Q#) land as BQ gates come online.
# owner:        @coder-mcp implements · @designer charters IA/copy deltas (aps-design-ux rules) ·
#               @sim-steward sequences · lesser agents OK for APSR-D#/T# mechanical slices
# territory:    tools/mcp/art_pipeline_suite/* · tools/mcp/python/tests/test_aps_* ·
#               src/dev/design_aps_*.md (consolidation only, no deletions without designer sign-off)
# regression:   cd tools/mcp/python && python -m pytest tests -q -m "not slow" per slice ·
#               headless launch (APS_TEST_HEADLESS=1) green · domain-router IA contract test green
# rule:         behavior-preserving until APSR-S2 lands — refactor ≠ redesign; IA (Option D dual
#               notebook, pipeline spine, authority zones) is KEPT — audit rated Workflow/IA OK.

# ═════════════════════════════════════════════════════════════════════
# AUDIT VERDICTS (what makes it feel unprofessional — verified file:line)
# ═════════════════════════════════════════════════════════════════════
# KEEP (already good): Option D IA + domain_router contract (runtime-verified) · pipeline spine +
#   flow-verb gating · JobController single-flight async (thread-safe, never blocks UI) · headless
#   parity (single code path) · theme tokens + widget kit EXIST (aps_theme 415 LOC, aps_tk/paned/
#   scroll/collapsible) · tooltips dictionary (80+ keys, designer-reviewed).
#
# FIX:
#  V1 STATE MUTATION CHAOS — SuiteState (state.py, 27 fields) has 47 direct mutation sites across
#     10 files, no owner discipline. atlas_folder written by BOTH variants_panel.py:826 AND
#     atlas_panel.py:298/341/369 (last-writer-wins). assembly_snapshot_data has 4 writers + a
#     shadow copy (assembly_panel self._snapshot) = two sources of truth.
#  V2 STALE PANELS — no observer pattern; app.py must remember manual sync_from_state() calls
#     (only on_send_to_assembly does, app.py:837). Lane switch refreshes landscape panels
#     (app.py:327) but NEVER buildings panels → stale Assembly UI after lane round-trip.
#  V3 GOD PANEL — assembly_panel.py 1,386 LOC single class embedding footprint grid, material
#     browser, grammar suite (4 sub-components), 2 previews, validation, metadata.
#  V4 DUPLICATED PRESENTATION — assembly_preview vs atlas_preview each reimplement validity chips/
#     status logic around shared aps_preview_state helpers; material widget has two entry points
#     with different configs → drift, inconsistent status messages.
#  V5 UI-CALLS-BACKEND-INLINE — panels import rust_engine_mcp validators + call generate/validate/
#     write files directly from event handlers (assembly_panel.py:1032-1097) — no service seam,
#     untestable in isolation.
#  V6 LOOSE TOKEN GOVERNANCE — inline font=("Segoe UI", 9) and hand-rolled padding beside token
#     constants (assembly_panel.py:147+); tooltips bound manually with no coverage check.
#  V7 SPEC SPRAWL — 60+ design_aps_*_v1.md specs, no code↔spec traceability, no single IA source
#     of truth; tests cover layout/imports/tier gates but NOT state discipline or panel sync.

# ═════════════════════════════════════════════════════════════════════
# TARGET ARCHITECTURE (from audit; agent-executable in slices)
# ═════════════════════════════════════════════════════════════════════
#   SHELL app.py            — lane routing + flow-verb dispatch ONLY (no manual panel syncs)
#   SERVICES aps_services/  — AssemblyService · VariantsService · AtlasService · LandscapeService:
#                             sole writers of their SuiteState fields; wrap ALL rust_engine_mcp +
#                             file IO; publish events
#   STATE state.py          — SuiteState becomes read-view; mutations only via SuiteStateWriter
#                             (owner-checked) + EventBus (StateChanged{fields}, FlowVerbBlocked…)
#   PANELS                  — subscribe to events, render, call services; no validators, no file IO
#   Single-writer per field [K01 applied to the tool] · draft-vs-saved made explicit (dirty flag)

# ═════════════════════════════════════════════════════════════════════
# PHASE A0 — GUARDRAILS FIRST (freeze behavior before moving anything)
# ═════════════════════════════════════════════════════════════════════

APSR-T1 | Mutation inventory test (S, lesser-agent):
  Script + pytest that greps/asserts the EXACT current set of SuiteState mutation sites (the 47).
  Refactor slices must shrink this list monotonically; any NEW direct mutation fails CI.
APSR-T2 | Panel-sync characterization tests (S):
  Headless tests capturing today's sync behavior incl. the V2 stale-assembly bug (marked xfail) —
  flips to passing when APSR-S2 lands. Lane round-trip, send_to_assembly, bake_variants flows.
APSR-T3 | Spec consolidation (M, @designer sign-off):
  One src/dev/APS_IA_SPEC_v1.md as the living IA source of truth; the 60 design_aps_* files get a
  status header (ACTIVE/SUPERSEDED-BY/HISTORICAL) and an index table. Code gets spec-ID comments at
  panel top (e.g. "# spec: DES-APS-ASSEMBLY-WORKFLOW-001") — cheap traceability.

# ═════════════════════════════════════════════════════════════════════
# PHASE A1 — STATE OWNERSHIP (the core fix)
# ═════════════════════════════════════════════════════════════════════

APSR-S1 | EventBus + SuiteStateWriter (M):
  Tiny synchronous pub-sub (Tk-safe, dispatch via after(0)); SuiteStateWriter enforces field→owner
  map (assembly_* → AssemblyService, variant_* → VariantsService, atlas_folder+tile_batch_path →
  AtlasService EXCLUSIVELY, landscape_* → LandscapeService, lane/domain → shell). Direct dataclass
  writes removed field-family by field-family (each its own slice, mutation-inventory test shrinking).
APSR-S2 | Services extraction (M-L, one service per slice):
  Move generate/validate/file-IO out of panels into services (AssemblyService first — it absorbs
  assembly_panel.py:1032-1097 inline calls and the self._snapshot shadow state; draft-vs-saved gets
  an explicit dirty flag). Services publish StateChanged; panels re-render on events → V2 stale-panel
  bug dies structurally, not by another manual sync call.
APSR-S3 | Shell cleanup (S): app.py drops all remembered sync calls; lane switch just publishes
  LaneChanged; assert app.py < 700 LOC after.

# ═════════════════════════════════════════════════════════════════════
# PHASE A2 — PANEL DECOMPOSITION + PRESENTATION UNIFICATION
# ═════════════════════════════════════════════════════════════════════

APSR-P1 | Split assembly_panel.py (M, mechanical after S2):
  1,386 → assembly_panel (≤400, footprint+routing) + assembly_preview_section + assembly_grammar_
  section (wraps the 4 grammar sub-components) + assembly_validation_section + assembly_metadata_
  section (~150-200 each). Behavior-preserving; characterization tests green throughout.
APSR-P2 | Shared preview/status presentation (S-M):
  One preview_state_display module (validity chips, fidelity, thumbnail states, status colors);
  assembly_preview_panel + atlas_preview_panel + variants_preview_panel consume it. Kills V4 drift.
APSR-P3 | Material widget single entry (S): material_browser stays the ONE wrapper; both mount
  points use identical config; document in APS_IA_SPEC.

# ═════════════════════════════════════════════════════════════════════
# PHASE A3 — DESIGN-SYSTEM GOVERNANCE (lesser-agent friendly)
# ═════════════════════════════════════════════════════════════════════

APSR-D1 | Token lint (S): pytest scanning art_pipeline_suite/ for inline font tuples, hardcoded
  hex colors, non-token padding — allowlist current violations, ratchet to zero over slices.
APSR-D2 | Tooltip coverage assertion (XS): every interactive widget factory path binds a tooltip
  key or is explicitly exempted; extends existing tag_vocabulary_audit pattern (aps-design-ux rule 5).
APSR-D3 | Inline-feedback adoption sweep (S): all long-op results through apply_status_atom/
  set_inline_status — no bare label.config status writes.
APSR-D4 | Density/polish pass per design_aps_smoothness_charter + style_quality specs (@designer
  charter, after A2 so it lands on stable panels).

# ═════════════════════════════════════════════════════════════════════
# PHASE A4 — QUALITY SURFACES (partnership with PLAN-BUILDING-QUALITY)
# ═════════════════════════════════════════════════════════════════════

APSR-Q1 | Assembly QC strip: renders BQ-A2/BQ-F3 output — style-purity %, adjacency violations,
  missing-slot violations with cell locations; blocks "Approve snapshot" while red (extends
  generation_trace_strip + SuiteState.assembly_generation_approved).
APSR-Q2 | Kit-coverage panel: BQ-K2 style-pack slot-coverage audit surfaced in Catalog tab
  (per-pack completeness bar; missing families listed) — operators SEE holes before generating.
APSR-Q3 | Golden-seed review flow: BQ-Q3 golden set browsable in Assembly preview; approve/reject
  writes the operator rubric row (design_aps_operator_rubric_v2).

# ═════════════════════════════════════════════════════════════════════
# EXECUTION ORDER + QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# A0 (guardrails) → A1 S1→S2→S3 (state core) → A2 P1-P3 ∥ A3 D1-D4 → A4 Q1-Q3 (as BQ gates land)
#
# id                | issue    | owner      | effort | exit
# APSR-A0-T1-001    | APSR-T1  | coder_a    | S      | mutation-inventory test red-on-new-writes
# APSR-A0-T2-001    | APSR-T2  | coder_a    | S      | characterization tests + xfail stale-panel repro
# APSR-A0-T3-001    | APSR-T3  | designer   | M      | APS_IA_SPEC_v1.md + status headers + spec-ID comments
# APSR-A1-S1-001    | APSR-S1  | coder-mcp  | M      | EventBus + writer; atlas_folder single-owner first
# APSR-A1-S2-ASM-1  | APSR-S2  | coder-mcp  | M      | AssemblyService; shadow _snapshot removed; xfail flips
# APSR-A2-P1-001    | APSR-P1  | coder-mcp  | M      | assembly_panel ≤400 LOC; tests green
# APSR-A3-D1-001    | APSR-D1  | coder_b    | S      | token lint + ratchet allowlist
# APSR-A4-Q1-001    | APSR-Q1  | coder-mcp  | S-M    | QC strip live vs BQ witness (after BQ-F3/A2)
#
# Gate to declare "APS professional": mutation sites = services only · stale-panel tests pass ·
# token lint zero · every tab spec-ID-traced · QC surfaces live · pytest suite green headless.
