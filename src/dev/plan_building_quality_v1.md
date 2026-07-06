# BUILDING GENERATION QUALITY OVERHAUL v1 — from "incoherent jumbles" to real buildings
# Generated 2026-07-03 from 3-agent audit (grammar logic · geometry/bake lane · APS surface).
# **Integration hub** for dual-track execution: THIS plan (BQ-*) + companion
# [`plan_aps_refactor_v1.md`](plan_aps_refactor_v1.md) (APSR-*).
# Companions: codebase_index_v1.md (CB-GRM/CB-PRC) · plan_city_grammar_upgrade_v1.md (block/town tier —
#   CITY owns G0 typed-ids/split; THIS plan owns building-level visual quality) · plan_cleanup_v1.md
# Issue codes: BQ-F# (fast fixes) · BQ-C# (contracts+validation) · BQ-A# (adjacency/coherence)
#              BQ-H# (architectural hierarchy) · BQ-K# (kit/data enrichment) · BQ-Q# (visual QC gate)
# APSR codes (companion plan): APSR-T# guardrails · APSR-S# services · APSR-P# panels · APSR-D# design
#                             system · APSR-Q# quality surfaces (consume BQ witnesses)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA (integrated dual-track)
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-BUILDING-QUALITY-v1  (+ PLAN-APS-REFACTOR-v1 companion)
# status:       PLANNED — diagnosis signed 2026-07-03; execution not started
# priority:     P2 on master — parallel-safe with MIG (Bevy lane) and APS G4; BQ-F# + APSR-A0 safe now
# owner:        @sim-steward sequences · @coder engine (BQ-F2/A/H) · @coder-mcp bake+APS (BQ-F1/C, APSR)
#               @designer-mcp kit charters (BQ-K) · @designer IA spec (APSR-T3)
# companion:    plan_aps_refactor_v1.md · aps-design-ux skill · design_aps_operator_rubric_v2.md
# index:        development_plan_index.md + HANDOFF.md § PLAN-BUILDING-QUALITY-v1
# regression:   validate-report cargo (engine) · pytest tools/mcp/python (APS + validators) ·
#               CITY-G0c determinism witness · building_quality_live.json (BQ-Q1, once live)
# depends:      CITY-G0a/G0b preferred before BQ-H grammar splits; BQ-F# has no CITY dependency
# territory:    src/construction/procedural/* · src/render/extraction/procedural_build_extract.rs ·
#               tools/mcp/blender/scripts/ops/* · tools/mcp/python/rust_engine_mcp/validators/* ·
#               tools/mcp/art_pipeline_suite/* (APSR) · assets/configs/buildings/*
# rules:        mcp-production-rules UNCHANGED (deterministic, batch/atlas, grid, no AI final art)
# done_bar:     BQ-Q3 golden-seed set operator-approved · zero cross-style fallbacks · zero adjacency
#               violations · APSR mutation-inventory = services-only · APSR-Q1-Q3 live
#
# Thesis (user priority):
#   Track BQ — Phase F fast fixes FIRST (~80% visible win in days): bake defects, style-aware
#   selection, kill silent hide_slot → then contracts/validators so jank can't return → adjacency →
#   massing→facade → kit-hole filling (@designer-mcp charters) → golden-seed QC gate = "done".
#   Track APSR — guardrail tests FIRST (mutation inventory, stale-panel xfail) → services layer
#   (single-writer + event bus) → panel decomposition → design-system lint → APSR-Q# surfaces that
#   DISPLAY BQ gates in-tool so operators see violations before shipping janky output.

# ═════════════════════════════════════════════════════════════════════
# INTEGRATED EXECUTION GRAPH
# ═════════════════════════════════════════════════════════════════════
#
#   Week 0 (parallel, no blockers):
#     BQ-F1 bake fixes ──┐
#     BQ-F2 style filter ├──► ~80% visible improvement
#     BQ-F3 slot violations ──┘
#     APSR-A0 T1/T2 guardrails ──► freeze behavior before refactor
#
#   Week 1:
#     BQ-C1-C4 contracts/validators (lock in F wins)
#     APSR-A1 S1→S2→S3 services (core stale-panel fix)
#     BQ-K1 kit charters start (@designer-mcp, parallel)
#
#   Week 2+:
#     BQ-A1-A2 adjacency + quality gate ──► feeds APSR-Q1
#     APSR-A2 P1-P3 panel split + D1-D4 lint
#     BQ-H1-H3 hierarchy ∥ BQ-K2-K3 data
#     APSR-A4 Q1-Q3 as BQ witnesses land
#     BQ-Q1-Q3 golden-seed done bar (operator sign-off)
#
# Coupling (BQ → APSR):
#   BQ-F3 violations + BQ-A2 score ──► APSR-Q1 Assembly QC strip
#   BQ-K2 slot coverage audit        ──► APSR-Q2 Kit-coverage panel
#   BQ-Q3 golden seeds               ──► APSR-Q3 Golden-seed review flow
#
# Coupling (APSR → BQ):
#   APSR-S2 AssemblyService          ──► single writer for assembly_snapshot + generation_trace
#   APSR-Q1 blocks Approve           ──► operators can't ship red assemblies

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX
# ═════════════════════════════════════════════════════════════════════
# Lane                         | BQ/APSR items           | Rule
# -----------------------------|-------------------------|------------------------------------------
# PLAN-BEVY-019-MIG-v1 P0      | none direct             | BQ engine + APS Python parallel-safe
# PLAN-CITY-GRAMMAR G0         | BQ-H2 street-facing     | BlockFrame from CITY-C2 when ready; heuristic until then
# plan_cleanup S11/S1c         | BQ overlaps grammar split| CITY-G0b owns split; do not double-pick
# APS-G4-COVERAGE-001          | BQ-K prerequisite       | G4 content bar before kit ship claims
# PERF / Stage 5               | BQ engine only          | stage5 --lib after procedural extract edits

# ═════════════════════════════════════════════════════════════════════
# ACTIVE PHASE
# ═════════════════════════════════════════════════════════════════════
# current:   BQ-A2 gate shipped — `building_quality_live.json` witness · BQ-A1 adjacency stubbed at 0
# next_pick: BQ-A1-ADJ-001 · BQ-Q1-WITNESS-001 (wire APSR-Q1 after A2+F3 green)
# blocked:   APSR-A4-Q1-001 until BQ-F3 + BQ-A2 witnesses green · BQ-Q3 until BQ-A/K/H slices land

# ═════════════════════════════════════════════════════════════════════
# SLICE TEMPLATE
# ═════════════════════════════════════════════════════════════════════
# id:            BQ-F1-BAKE-001 | APSR-A0-T1-001
# issue:         BQ-F1 | APSR-T1
# owner:         coder | coder-mcp | coder_a | designer-mcp | designer
# exit_witness:  validate-report * · debug_runs/building_quality_live.json · pytest -k aps
# blocks:        slice ids
# parallel_ok:   see CONFLICT MATRIX

# ═════════════════════════════════════════════════════════════════════
# DIAGNOSIS (verified, file:line — why buildings are jumbles)
# ═════════════════════════════════════════════════════════════════════
# TWO INDEPENDENT JANK SOURCES — both must be fixed; fixing one alone leaves visible mess.
#
# SOURCE A — GENERATION LOGIC (src side):
#  A1 ZERO adjacency constraints: per-cell module picks are fully independent
#     (procedural_build_extract.rs:52-103); no edge/socket compatibility anywhere. CATASTROPHIC.
#  A2 Style enforcement broken at selection: prefer_stylepack_tier() picks by TIER ONLY and ignores
#     entry.style_pack/style_tags (module_index.rs:194-207, 266-276) — a steel/concrete "corner_L"
#     lands in a Victorian brick building whenever its tier sorts higher. HIGH, and a 1-line-class fix.
#  A3 No architectural hierarchy: massing strategy is picked but only affects footprint bounds —
#     never facade rhythm/openings/roof (grammar_evaluation.rs:200-203). Door HARDCODED at
#     (width/2, y=0) for every building (footprint_grid.rs:56). Window rows stacked identically per
#     floor. placement_tags + material_profiles are produced (grammar_evaluation.rs:267,293-337)
#     and NEVER consumed downstream.
#  A4 Validation gate absent: fallback_policy="hide_slot" silently culls missing modules
#     (load.rs:85, procedural_build_extract.rs:81-83); no post-assembly quality check — bad
#     buildings always ship.
#  A5 Data skeletal: grammars are 75-81 lines with ~zero constraint rules; style packs miss slots
#     (style_industrial_west has no window_1u → windows silently vanish); ~30% of module index
#     entries still smoke/greybox tier.
#
# SOURCE B — PHYSICAL ASSETS (tools/mcp side):
#  B1 Kit holes: NO brick/wood/concrete roofs, doors (brick), windows (wood/brick/concrete) exist in
#     the 100-module inventory — every non-steel building is FORCED into cross-style substitution.
#  B2 Bake defects: flat roof slab floats 0.1m above wall tops (module_roof.py:29 places at Y=t*0.5);
#     wall sill depth is d*1.05 (module_wall.py:30) → 1.5cm lip at every wall join.
#  B3 No geometric contract enforcement: validate_glb.py checks magic header + vertex count ONLY —
#     no pivot, no bounds-vs-grid_units (1u=4m is an UNWRITTEN assumption), no adjacent-edge/seam
#     checks, no height agreement (door 2.5m vs wall 3m = roofline steps).
#  B4 Scale chain opacity: no GRID_UNIT_M constant anywhere; ConstructionIsoDrawScale 1.5x multiplier
#     is visual-only and can shear visuals off collision footprints (iso_draw_scale.rs).

# ═════════════════════════════════════════════════════════════════════
# PHASE F — FAST FIXES (days, ~80% of visible improvement; do FIRST)
# ═════════════════════════════════════════════════════════════════════

BQ-F1 | @coder-mcp | Bake geometry defects (XS each, one slice):
  · module_roof.py:29 — place flat roof at Y=0 (flush on wall top), audit pitched/shed/sawtooth for
    the same t*0.5 pattern
  · module_wall.py:30 — remove the d*1.05 sill depth factor; all boxes at exact param dims
  · REBAKE affected module batches through the normal G-gate lane (deterministic re-run, same seeds)
  EXIT: rebaked GLBs promoted; before/after screenshots via bevy_preview_worker.

BQ-F2 | @coder | Style-aware module selection (S — the single highest-leverage engine fix):
  prefer_stylepack_tier()/stylepack_entry() (module_index.rs) gain a style_pack_id parameter;
  filter entry.style_pack == requested BEFORE tier comparison; fall back tier-only ONLY when zero
  same-style entries exist AND log the violation (no more silent cross-style).
  Propagate style_pack_id from assemble_procedural_build_instances() (procedural_build_extract.rs:44).
  EXIT: unit test — Victorian request can never return an industrial_west entry while a victorian
  entry exists; witness counts cross-style fallbacks (target 0 once BQ-K1 fills kits).

BQ-F3 | @coder | Kill silent hide_slot (XS):
  fallback_policy="hide_slot" → assembly records a MissingSlotViolation (slot, style_pack, cell)
  instead of vanishing the cell; violations surface in assembly_snapshot + APS QC panel
  (plan_aps_refactor APSR-Q1). Buildings with violations render with a debug tint in preview.
  EXIT: generating with style_industrial_west shows its missing window_1u as a violation, not a hole.

# ═════════════════════════════════════════════════════════════════════
# PHASE C — CONTRACTS + VALIDATORS (make jank impossible to re-introduce)
# ═════════════════════════════════════════════════════════════════════

BQ-C1 | @coder-mcp | Written module geometric contract (S):
  New tools/mcp/schemas/module_contract_v1.md + schema fields: GRID_UNIT_M=4.0 / FLOOR_HEIGHT_M=3.0
  as named constants (Python + mirrored Rust const in module_index.rs), pivot=bottom_center
  convention, per-family height table (door heights MUST equal wall heights per style), edge-socket
  naming (left/right/top edge profiles).

BQ-C2 | @coder-mcp | Bounds + pivot validator in G4 (S-M):
  Extend validate_glb/asset.py: parse GLB bounds → assert |bounds.w − grid_units.w×4m| < 1cm, same
  for depth/height; assert min.y ≈ 0 (bottom pivot). Run against ALL 100 promoted modules; produce a
  violation report — this both gates future bakes AND inventories existing defects for rebake.

BQ-C3 | @coder-mcp | Seam/pair validator (M):
  New G4b check: for each style pack, load its wall+corner+door+window GLB set and assert shared
  edge heights/depths agree within 1cm (the door-2.5m-vs-wall-3m class of defect).

BQ-C4 | @coder | Scale-chain audit (S):
  Document + assert the full chain (bake m → GLB → grid_units → local_translation floor×3 →
  iso_draw_scale 1.5x). Decide: fold 1.5x into placement so visual == footprint, or document why
  visual-only is safe. One authority for scale [K01].

# ═════════════════════════════════════════════════════════════════════
# PHASE A — ADJACENCY / COHERENCE (the structural fix)
# ═════════════════════════════════════════════════════════════════════

BQ-A1 | @coder | Edge-compatibility model (M):
  FootprintCell gains neighbor context; module entries gain edge profiles (from BQ-C1 sockets).
  Start with 5 hard rules (validated at assembly, violations recorded like BQ-F3):
   1 corner cells: wall/door neighbors on exactly the two adjacent sides
   2 door cells: wall neighbors on non-door sides; NEVER at a corner cell
   3 wall runs: left/right edges match (same family or declared-compatible)
   4 roof cells: continuous perimeter, no gaps; ridge direction consistent per massing
   5 windows floor>0 align vertically with floor-0 rhythm (see BQ-H2)
  NOT full WFC — constraint check + local repair (re-pick within style) is enough for v1.

BQ-A2 | @coder | Assembly quality gate (S):
  Post-assembly score: style purity % · adjacency violations · missing slots · silhouette continuity.
  Threshold from grammar data; failing assemblies are flagged (debug tint + witness), never silently
  shipped. Feeds APS QC panel + operator rubric.

# ═════════════════════════════════════════════════════════════════════
# PHASE H — ARCHITECTURAL HIERARCHY (buildings that read as designed)
# ═════════════════════════════════════════════════════════════════════

BQ-H1 | @coder | Massing → facade propagation (M):
  FacadeRule table per massing_id (data, in grammar RON): wall/window ratio, door count+placement
  policy, roof kind. long_hall → linear rhythm + sawtooth; yard_complex → perimeter doors only;
  l_shape → per-leg rhythm. Wire into generate() slot-override stage (grammar_evaluation.rs:225-238).

BQ-H2 | @coder | Semantic openings (M):
  Kill the hardcoded door at width/2 (footprint_grid.rs:56): door placement from street-facing edge
  (BlockFrame from CITY-C2 when available; footprint edge heuristic until then) + FacadeRule policy.
  Ground-floor rhythm ≠ upper-floor rhythm (shopfront vs window rows). CONSUME placement_tags and
  material_profiles that grammar already emits (grammar_evaluation.rs:267,293) — they are computed
  and dropped today.

BQ-H3 | @coder | v0 grammar retirement decision (S):
  arch_build_grammar_v0 reweighting can silently bias T1 toward strategies its presets never meant.
  Classify (cleanup-completion rules): migrate the useful presets into T1 grammar data, then retire
  v0 or freeze it behind a validation shim that rejects unknown massing ids.

# ═════════════════════════════════════════════════════════════════════
# PHASE K — KIT + DATA ENRICHMENT (@designer-mcp charter lane, parallel to A/H)
# ═════════════════════════════════════════════════════════════════════

BQ-K1 | Kit hole fill (charter → batch bakes, the B1 fix):
  Priority order (unblocks style purity): brick roof + brick door + brick window · wood roof + wood
  window · concrete roof + concrete window/door. ~10-14 new geometry jobs, standard G0-G5 lane,
  BQ-C2/C3 validators active so new modules are contract-clean by construction.
BQ-K2 | Style pack completion: every style pack covers 100% of standard slots (wall 1u/2u, door
  default/wide, window 1u/2u, corner inner/outer, roof default + massing overrides, prop) —
  no hide_slot fallbacks remain. Audit script counts coverage per pack (pytest).
BQ-K3 | Grammar data enrichment: +2-4 massing strategies per archetype (T/U/stepped), FacadeRule
  tables (BQ-H1), age/weathering progression bands with variant_tags mapped to APS mandate tags.
BQ-K4 | Palette/variation axis: per CITY-C5 (owned there) — BQ-K2's complete kits are its prerequisite.

# ═════════════════════════════════════════════════════════════════════
# PHASE Q — VISUAL QC GATE (close the loop; ties into APS refactor)
# ═════════════════════════════════════════════════════════════════════

BQ-Q1 | Style-purity + adjacency witness: per-assembly quality score (BQ-A2) written to
  debug_runs/building_quality_live.json; APS Assembly tab shows it (plan_aps_refactor APSR-Q1).
BQ-Q2 | Screenshot QC lane: bevy_preview_worker renders N seeded assemblies per style pack per
  batch; operator rubric row (design_aps_operator_rubric_v2) gets a "reads as a real building"
  criterion; failing seeds attached to the witness.
BQ-Q3 | Golden-seed regression set: ~12 seeds × archetype × style committed as approved snapshots;
  any grammar/kit change diffs assembly hashes against goldens (goldens updated only with operator
  approval).

# ═════════════════════════════════════════════════════════════════════
# EXECUTION ORDER + INTEGRATED QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
#
# ── WEEK 0 — parallel (start here) ──────────────────────────────────
#
# id                  | track | issue    | owner       | effort | exit
# BQ-F1-BAKE-001      | BQ    | BQ-F1    | coder-mcp   | S      | roof Y=0 flush; wall no 1.05 sill; rebaked GLBs promoted
# BQ-F2-STYLE-001     | BQ    | BQ-F2    | **coder_b** (CHAIN-B CB-BQ-001) | S      | style_pack filter in module_index; cross-style fallback logged
# BQ-F3-SLOT-001      | BQ    | BQ-F3    | **coder_b** (CHAIN-B CB-BQ-002) | XS     | MissingSlotViolation witness; debug tint; no silent hide
# APSR-A0-T1-001      | APSR  | APSR-T1  | coder_a     | S      | mutation-inventory pytest; new direct SuiteState write = fail
# APSR-A0-T2-001      | APSR  | APSR-T2  | coder_a     | S      | stale-assembly xfail repro; lane round-trip characterization
# APSR-A0-T3-001      | APSR  | APSR-T3  | designer    | M      | APS_IA_SPEC_v1.md + spec-ID headers on panels
#
# Week 0 gate: BQ-F1+F2+F3 done OR deferred with steward note · APSR-A0 T1+T2 red/green baseline recorded
#
# ── WEEK 1 — lock-in + services core ────────────────────────────────
#
# BQ-C1-CONTRACT-001    | BQ    | BQ-C1    | coder-mcp   | S      | module_contract_v1.md + GRID_UNIT_M both sides
# BQ-C2-BOUNDS-001      | BQ    | BQ-C2    | coder-mcp   | S-M    | G4 bounds/pivot on 100 modules; violation report
# BQ-C3-SEAM-001        | BQ    | BQ-C3    | coder-mcp   | M      | wall+door+window height agreement per style pack
# BQ-C4-SCALE-001       | BQ    | BQ-C4    | coder       | S      | scale chain doc + iso_draw_scale authority decision
# APSR-A1-S1-001        | APSR  | APSR-S1  | coder-mcp   | M      | EventBus + SuiteStateWriter; atlas_folder single-owner
# APSR-A1-S2-ASM-001    | APSR  | APSR-S2  | coder-mcp   | M      | AssemblyService; _snapshot shadow removed; xfail→pass
# APSR-A1-S3-001        | APSR  | APSR-S3  | coder-mcp   | S      | app.py <700 LOC; LaneChanged events only
# BQ-K1-KITFILL-001     | BQ    | BQ-K1    | designer-mcp| charter| brick/wood/concrete roof+door+window job specs
#
# ── WEEK 2+ — structure, hierarchy, surfaces, done bar ──────────────
#
# BQ-A1-ADJ-001         | BQ    | BQ-A1    | coder       | M      | 5 adjacency rules + violation witness
# BQ-A2-GATE-001        | BQ    | BQ-A2    | coder       | S      | style purity % + score → building_quality_live.json
# BQ-H1-FACADE-001      | BQ    | BQ-H1    | coder       | M      | FacadeRule per massing_id in grammar RON
# BQ-H2-OPENINGS-001    | BQ    | BQ-H2    | coder       | M      | kill width/2 door; consume placement_tags
# BQ-H3-V0-RETIRE-001   | BQ    | BQ-H3    | sim-steward | S      | classify arch_build_grammar_v0; migrate or freeze
# BQ-K2-COVERAGE-001    | BQ    | BQ-K2    | designer-mcp| M      | 100% slot coverage per style pack; pytest audit
# BQ-K3-GRAMMAR-001     | BQ    | BQ-K3    | designer-mcp| M      | +massing strategies + FacadeRule tables
# APSR-A2-P1-001        | APSR  | APSR-P1  | coder-mcp   | M      | assembly_panel ≤400 LOC; characterization green
# APSR-A2-P2-001        | APSR  | APSR-P2  | coder-mcp   | S-M    | shared preview_state_display module
# APSR-A2-P3-001        | APSR  | APSR-P3  | coder-mcp   | S      | material_browser single entry point
# APSR-A3-D1-001        | APSR  | APSR-D1  | coder_b     | S      | token lint ratchet (inline fonts/colors)
# APSR-A3-D2-001        | APSR  | APSR-D2  | coder_b     | XS     | tooltip coverage assertion
# APSR-A3-D3-001        | APSR  | APSR-D3  | coder_b     | S      | inline-feedback adoption sweep
# APSR-A4-Q1-001        | APSR  | APSR-Q1  | coder-mcp   | S-M    | QC strip ← BQ-F3/A2 (BLOCK until witnesses)
# APSR-A4-Q2-001        | APSR  | APSR-Q2  | coder-mcp   | S      | kit-coverage panel ← BQ-K2 audit
# APSR-A4-Q3-001        | APSR  | APSR-Q3  | operator    | M      | golden-seed browse/approve ← BQ-Q3
# BQ-Q1-WITNESS-001     | BQ    | BQ-Q1    | coder       | S      | building_quality_live.json + APSR-Q1 wire
# BQ-Q2-SCREEN-001      | BQ    | BQ-Q2    | coder-mcp   | M      | bevy_preview_worker N seeds/style; rubric row
# BQ-Q3-GOLDEN-001      | BQ    | BQ-Q3    | operator    | M      | ~12 seeds × archetype × style; hash regression
#
# ── DONE BAR (both tracks) ──────────────────────────────────────────
# PLAN-BUILDING-QUALITY-v1 CLOSED ⇔
#   BQ-Q3 golden set operator-approved (all style packs)
#   ∧ cross-style fallback count = 0 (BQ-F2 witness)
#   ∧ adjacency violation count = 0 on goldens (BQ-A1/A2)
#   ∧ G4 validators green on all promoted modules (BQ-C2/C3)
# PLAN-APS-REFACTOR-v1 CLOSED ⇔
#   mutation inventory = services-only (APSR-T1)
#   ∧ stale-panel characterization tests pass (APSR-T2)
#   ∧ APSR-Q1 blocks Approve on red QC · APSR-Q2/Q3 live
#   ∧ pytest -k aps green headless · token lint zero (APSR-D1)

# ═════════════════════════════════════════════════════════════════════
# APS REFACTOR COMPANION (detail in plan_aps_refactor_v1.md)
# ═════════════════════════════════════════════════════════════════════
# Full APSR issue catalog, audit file:line, and target architecture live in the companion plan.
# This hub owns sequencing + queue seeds + BQ↔APSR coupling only.
# Do NOT start APSR-A4-Q* until BQ-F3 (violations) and BQ-A2 (score) witnesses exist.

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES (2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# Audit spot-checks: module_roof.py:29 t*0.5 float · module_wall.py:30 d*1.05 sill ·
# prefer_stylepack_tier tier-only (module_index.rs:266) · assembly_panel shadow _snapshot (line 124) ·
# hide_slot silent cull (procedural_build_extract.rs). All confirmed.
# Integrated dual-track execution + queue seed landed; link HANDOFF + development_plan_index next.
