# BUILDING GENERATION QUALITY OVERHAUL v1 — from "incoherent jumbles" to real buildings
# Generated 2026-07-03 from 3-agent audit (grammar logic · geometry/bake lane · APS surface).
# Companions: codebase_index_v1.md (CB-GRM/CB-PRC) · plan_city_grammar_upgrade_v1.md (block/town tier —
#   CITY owns G0 typed-ids/split; THIS plan owns building-level visual quality) · plan_aps_refactor_v1.md
#   (QC surfaces land there) · plan_cleanup_v1.md
# Issue codes: BQ-F# (fast fixes) · BQ-C# (contracts+validation) · BQ-A# (adjacency/coherence)
#              BQ-H# (architectural hierarchy) · BQ-K# (kit/data enrichment) · BQ-Q# (visual QC gate)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-BUILDING-QUALITY-v1
# status:       PLANNED — diagnosis signed 2026-07-03 (three independent audits agree)
# priority:     recommend P1 on master alongside MIG Phase 0/1 — engine slices are in
#               construction/procedural + tools/mcp (no overlap with MIG render lane);
#               BQ-F# fast fixes are safe immediately
# owner:        @coder engine selection/constraint slices · @coder-mcp bake ops + validators ·
#               @designer-mcp kit charters, style packs, grammar data · @sim-steward sequences
# depends:      CITY-G0a (typed ids) + CITY-G0b (grammar split) land FIRST — BQ engine slices build
#               on the split files. CITY-G0c determinism witness is BQ's regression net too.
# territory:    src/construction/procedural/* · src/render/extraction/procedural_build_extract.rs ·
#               tools/mcp/blender/scripts/ops/* · tools/mcp/python/rust_engine_mcp/validators/* ·
#               assets/configs/buildings/* · assets/models/modules/* (via MCP lane only)
# regression:   validate-report cargo per engine slice · pytest tools/mcp/python per pipeline slice ·
#               CITY-G0c determinism witness · NEW BQ-Q1 style-purity + adjacency witness once it exists
# rules:        mcp-production-rules UNCHANGED (deterministic, batch/atlas, grid, no AI final art)

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
# EXECUTION ORDER + QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# Order: F1+F2+F3 (days, big visible win) → C1-C4 (lock it in) → A1-A2 ∥ K1-K2 → H1-H3 ∥ K3 → Q1-Q3
#
# id               | issue | owner        | effort | exit
# BQ-F1-BAKE-001   | BQ-F1 | coder-mcp    | S      | roof flush + sill exact; rebaked batch promoted
# BQ-F2-STYLE-001  | BQ-F2 | coder        | S      | style-filtered selection + fallback log + unit test
# BQ-F3-SLOT-001   | BQ-F3 | coder        | XS     | violations recorded, debug tint in preview
# BQ-C1-CONTRACT-1 | BQ-C1 | coder-mcp    | S      | contract doc + GRID_UNIT_M constants both sides
# BQ-C2-BOUNDS-001 | BQ-C2 | coder-mcp    | S-M    | G4 bounds/pivot check + 100-module violation report
# BQ-A1-ADJ-001    | BQ-A1 | coder        | M      | 5 rules enforced; violation witness
# BQ-K1-KITFILL-1  | BQ-K1 | designer-mcp | charter| brick/wood/concrete roof+door+window jobs queued
#
# Gate to declare "buildings fixed": BQ-Q3 golden set approved by operator across all style packs
# with zero cross-style fallbacks and zero adjacency violations.
