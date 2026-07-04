# CITY + BUILDING GRAMMAR UPGRADE PLAN v1 — from bevy_city patterns, not bevy_city code
# Generated 2026-07-03. Reference studied in full (1,226 LOC, 4 files):
#   https://github.com/bevyengine/bevy/tree/main/examples/large_scenes/bevy_city
#   (main.rs 410 · generate_city.rs 351 · assets.rs 271 · settings.rs 169 — Kenney GLB kits, 0.19 APIs)
# Companions: codebase_index_v1.md · plan_cleanup_v1.md (S11/S1c routed here)
# BSN (WorldAssetRoot): **product architecture — NOT migration.** MIG-A9 handoff **COMPLETE** (pilot + witness).
#   Ongoing BSN work lives here (CITY-C6 · § BSN ASSEMBLY CHARTER) + construction/procedural spawn plans.
# Deferrals: plan_deferral_registry_v1.md — DR-CITY-C6-VIS · DR-CITY-C6-BSN · DR-CITY-P1 · DR-CITY-P2
# Migration truth: debug_runs/mig_bevy_019/mig_v1_gate.json
# Issue codes: CITY-G# (grammar foundations) · CITY-C# (bevy_city-derived components) · CITY-P# (perf/polish)
#
# PROBLEM STATEMENT (user): building-generation grammar is weak; cities/towns are weaker still.
# Our stack today: CB-GRM building_grammar (965 LOC god file, stringly ids) → CB-PRC tile/module
# resolvers → parametric_commit → procedural_build_spawn. Town layer: STR-SET TownBook/DistrictBook/
# BlockBook + growth proposals + DevelopmentPressure + MarketSaturation — simulation books exist but
# nothing turns a district into a COHERENT VISUAL BLOCK; buildings commit one lot at a time with no
# block-level composition, no street-relative anchoring, no material-variation axis.

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-CITY-GRAMMAR-v1
# status:       CODER_CLOSED — G0–G3 + P1/P2 green 2026-07-03 · MIG-V1 gate_pass on master
# priority:     P3 — below PLAN-BUILDING-QUALITY-v1 Stream 2 (BQ-A1+) and PERF-VFX operator lane
# index:        development_plan_index.md + HANDOFF.md § PLAN-CITY-GRAMMAR-v1
# active_phase: CLOSED (coder) — only DR-MIG-TILEMAP remains blocked on bevy_ecs_tilemap 0.19
# branch:       **master** · Bevy **0.19** · all engine slices landed on master (not post-MIG branch)
# tandem:       n/a — coder drain complete; planner doc refresh CITY-DOC-002 done 2026-07-03
#               Territory lock: CITY = construction/procedural/strategic settlement · MIG = render graph only
# constraints:  mcp-production-rules apply UNCHANGED — deterministic seeded output, batch/atlas,
#               grid alignment, no AI final art. bevy_city's GLB-kit approach maps onto OUR module
#               kits (blender-geometry lane) + tile atlas variants, NOT onto downloading Kenney packs.
# version gate: CITY-G0..G3 + P1/P2 are **0.19-safe** on master (plain ECS + BSN product lane).
#               BSN (`WorldAssetRoot`) expansion is **product-owned** — § BSN ASSEMBLY CHARTER.
#               DR-MIG-TILEMAP: keep `bevy_tilemap_adapter` OFF until bevy_ecs_tilemap 0.19 ships.
#               CITY-P1/P2: **SHIPPED** — witnesses `city_p1_001_live.json` · `city_p2_001_live.json`.
# regression:   cargo test -p proc_A_dine01 --lib city_g0 city_g1 city_g3 city_p1 city_p2 city_c6
#               grammar determinism witness (same seed ⇒ byte-identical assembly snapshot)

# ═════════════════════════════════════════════════════════════════════
# PRIORITY + TANDEM MATRIX (vs migration / cleanup)
# ═════════════════════════════════════════════════════════════════════
# Global pick order (HANDOFF):
#   P0  PLAN-BEVY-019-MIG-v1 — **master only**
#   P1  PLAN-CLEANUP-v1 Phase 0 — zero-risk hygiene on master (parallel OK)
#   P2  PLAN-CITY-GRAMMAR-v1 G0 — this program (master, Bevy 0.19 — MIG-V1 green)
#   DEFER plan_cleanup Phase 2+ → DR-CLEANUP-P2 · plan_schedule_sync Wave 2+ → DR-SCHED-W2
#
# Tandem lanes (safe parallel — different files on master):
#   @coder_a   master: CITY-G0-S11-001 (building_grammar typed ids)
#              OR APSR-A0-T1/T2 · BQ-F when files don't overlap building_grammar
#   @coder_b   master: CITY-G0-S1C-001 (building_grammar split)  ← primary tandem pick
#              OR CITY-G2-C6 visual part · BQ when not touching building_grammar.rs
#   @coder     master: CITY-G0-WIT-001 after G0a/G0b land · RTT/VFX operator verify when primary
#   @coder-mcp parallel: CITY-C8 bake merge (pipeline) · APS-G4-COVERAGE-001 — not MIG render lane
#
# Ownership transfer from plan_cleanup (do NOT double-pick):
#   plan_cleanup S11 → CITY-G0a (CITY-G0-S11-001) — authoritative owner
#   plan_cleanup S1c building_grammar split → CITY-G0b (CITY-G0-S1C-001) — authoritative owner
#   plan_cleanup S11 → CITY-G0a (authoritative — do not pick S11 from cleanup queue)
#   plan_cleanup S1c building_grammar split → CITY-G0b (authoritative — do not pick S1c from cleanup queue)
#
# Conflict — do NOT parallelize without steward sign-off:
#   Same-file edits to building_grammar.rs by CITY-G0 and MIG slices in the same session
#   G1 block layer (STR-SET) while plan_schedule_sync SCH-E2 fire authority in flight
#   CITY-C5 palette charter — coordinate designer-mcp after G1 gate (APS-G4 pilot_hardcode green 2026-07-03)

# ═════════════════════════════════════════════════════════════════════
# WHAT bevy_city ACTUALLY DOES (verified against source — the extractable components)
# ═════════════════════════════════════════════════════════════════════
# B1 Density-field block typing: OpenSimplex noise → density scalar → block archetype by threshold
#    (forest <0.45 <low <0.6 <medium <0.7 <high). ONE scalar field drives the whole city character.
# B2 Anchor-tiled blocks: city = grid of fixed-size blocks (5.5×4.0) anchored at crossroads; every
#    asset placed RELATIVE to its block anchor. Trivially chunkable/streamable.
# B3 Deterministic seed chain: SmallRng::seed_from_u64(seed) at city root; noise seeded from rng;
#    per-spawn draws flow from one chain ⇒ same seed = same city (matches our production rules).
# B4 Kit × material-variation catalogs: Buildings { meshes: Vec<Handle<Mesh>>, materials: Vec<...> };
#    get_random_building = random mesh × random colormap variation. 5–12 meshes × 3–4 palettes ⇒
#    combinatorial variety from tiny asset counts.
# B5 Archetype spawn recipes: spawn_forest/low/medium/high_density are tiny hand-authored layout fns
#    (rows of houses + fences + trees; skyscraper grid; forest scatter with 1/3 empty jitter).
# B6 Road-as-stretched-asset: ONE road-straight mesh scaled along its axis instead of N segments;
#    vehicles are CHILDREN of a Road{start,end} entity moving by parametric distance_traveled.
# B7 Staged generation via Messages: load(untyped handle tracking + progress UI) → CityAssetsLoaded
#    → process (mesh merging) → CityAssetsReady → spawn → CitySpawned → post-pass (NoCpuCulling).
#    Each stage deliberately lands on the NEXT frame.
# B8 Post-load mesh merging: merge_all_mesh_3d collapses per-part GLBs (car wheels/doors) into one
#    mesh ⇒ fewer transforms to propagate + fewer indirect draw commands.
# B9 0.19 large-scene switches: StaticTransformOptimizations::Enabled, optional NoCpuCulling,
#    contact shadows, BSN for camera/UI scenes.
# NOT worth copying: hardcoded magic-number layouts (their own NOTE admits hand-tweaking), remote
# Kenney asset URLs, the toy car "simulation", feathers settings panel (we have egui HUD).

# ═════════════════════════════════════════════════════════════════════
# PHASE G0 — GRAMMAR FOUNDATIONS (0.19-safe plain ECS; prerequisite hardening)
# ═════════════════════════════════════════════════════════════════════

CITY-G0a | Typed grammar IDs — execute plan_cleanup S11 as the opening slice of THIS program:
  newtypes MassingId/SlotId/UsageId/StylePackId + enum CorridorType replacing contains("rail")
  dispatch in building_grammar.rs. Validation on deserialize. Blocks everything downstream (block
  grammar composes over these ids). Owner: coder_a. Effort: S-M.

CITY-G0b | Split building_grammar.rs (plan_cleanup S1c) — grammar_types / grammar_deserialize /
  grammar_evaluation, re-exports preserved. Mechanical. Owner: coder_b. Effort: M.

CITY-G0c | Grammar determinism witness — new witness: fixed seed + fixed spec ⇒ hash of
  AssemblySnapshot (CB-PRC assembly_snapshot.rs) is stable across runs. This is the regression net
  for every later phase [K02, B3]. Owner: coder. Effort: S.

# ═════════════════════════════════════════════════════════════════════
# PHASE G1 — BLOCK ARCHETYPE LAYER (adopts B1+B2, upgraded with OUR fields)
# The core structural fix: insert a BLOCK grammar tier between district sim and building grammar.
#   today:  GrowthProposal → single-lot building commit (incoherent streetscapes)
#   target: DistrictBook fields → BlockArchetype per block → block recipe → lots → building grammar per lot
# ═════════════════════════════════════════════════════════════════════

CITY-C1 | Field-driven block typing (B1, upgraded):
  bevy_city uses raw noise; WE ALREADY HAVE richer scalars — DevelopmentPressure (STR-SET pressure.rs),
  MarketSaturation (market.rs), district zoning class, transport access from IF-TRG graph distance.
  NEW: BlockArchetype enum (Forest/Park, LowDensityRes, MediumRes, HighDensity/Commercial, Industrial,
  Civic — extensible) + data-driven threshold table (RON asset, hot-tunable like tuning_io.rs) mapping
  a BlockScore { pressure, saturation, zoning, access, noise_jitter } → archetype.
  Noise stays ONLY as a tie-breaking jitter term (seeded, B3) so districts aren't uniform.
  Files: new src/strategic/settlement/block_archetype.rs + RON in assets/. Effort: M.

CITY-C2 | Anchor-tiled block frames (B2):
  BlockFrame { anchor: junction/tile coord, extent, orientation-to-street } derived from BlockBook +
  transport graph junctions (IF-TRG junction.rs) — bevy_city anchors at crossroads; we anchor at OUR
  graph junctions so blocks orient to actual streets, not a fixed grid. All lot/building placement
  becomes RELATIVE to BlockFrame (streamable, chunk-aligned per grid rules).
  Files: block_frame.rs + assign.rs integration. Effort: M.

CITY-C3 | Block recipes (B5, data not code):
  bevy_city hand-codes spawn_low_density() etc.; we make recipes DATA: BlockRecipe (RON) = list of
  lot rows/edge furniture/setbacks per archetype, evaluated deterministically from block_seed.
  Recipe vocabulary: lot_row(count, depth, facing=street), edge(fence|hedge|trees, spacing),
  scatter(tree_small|tree_large, density, jitter), park_fill, plaza. This is the town-scale analog
  of the building grammar — same seeded-evaluation discipline, charterable by @designer-mcp.
  Effort: M-L (vocabulary v1 small: 5 primitives above, extend later).

CITY-C4 | Seed chain formalization (B3):
  world_seed → town_seed(town_id) → block_seed(block_id) → lot_seed(lot_idx) → building grammar seed.
  One helper (idgen-adjacent), used by C1 jitter, C3 recipes, and existing building grammar. Replaces
  any ad-hoc seeding in parametric_commit. Witness from G0c proves stability. Effort: S.

# ═════════════════════════════════════════════════════════════════════
# PHASE G2 — VARIETY + STREETSCAPE (adopts B4+B6)
# ═════════════════════════════════════════════════════════════════════

CITY-C5 | Kit × variation axis (B4):
  Extend module_index/tile_atlas_index (CB-PRC) with an explicit PALETTE/VARIATION axis: a building
  variant = massing kit choice × palette variation, both seeded. For the tile lane this is atlas
  variant columns (tile-generation skill: variants as state machines); for the GLB module-kit lane
  it is material-slot palettes at bake (blender-geometry lane — batch, deterministic).
  bevy_city gets 36+ visually distinct buildings from 12 meshes × 3 palettes; we currently ship
  1 visual per variant id. Charter: @designer-mcp (palette specs are art-pipeline territory, G0-G5
  gates apply). Effort: M engine + M pipeline.

CITY-C6 | Street-relative furniture + stretched corridor visuals (B6):
  · Corridor visual = ONE module stretched/tiled along IF-TRG spline (spline.rs subdivide_edge
    already exists) instead of per-tile stamping where the profile allows.
  · Street furniture (fences, trees, path stones, lamps) as parametric-t placements along block
    edges/corridor splines — recipe-driven (C3 edge() primitive).
  · Visual-layer vehicles: children of corridor edge with parametric t (bevy_city Car pattern) as a
    CHEAP ambient layer, distinct from real logistics vehicles (EC-LOG) — flagged clearly as
    presentation-only so it never becomes a second traffic authority [K01].
  BSN PART (**DR-CITY-C6-BSN** — product, not migration): scene assembly via `WorldAssetRoot` after C6-visual
  + designer-mcp charter (§ BSN ASSEMBLY CHARTER). Pilot already shipped: `block_street_visual.rs` (settlement lane).
  Effort: M-L.

# ═════════════════════════════════════════════════════════════════════
# PHASE G3 — GENERATION FLOW + PERF (adopts B7+B8+B9)
# ═════════════════════════════════════════════════════════════════════

CITY-C7 | Staged block rollout via Messages (B7):
  Growth execute (STR-SET execute.rs) currently commits sites directly. Adopt the staged pattern:
  BlockPlanned → (assets/variants resolved) → BlockAssembled → (spawned over N frames within
  streaming budget IO-STR) → BlockCommitted, each stage a Message landing next frame — no
  spawn-storm frame spikes when a district upgrades. Reuses our existing Message discipline.
  Effort: M.

CITY-C8 | Bake-time mesh merging (B8, done OUR way):
  bevy_city merges at runtime; we have a headless Blender lane — merge module-kit parts per building
  variant AT BAKE (one GLB per variant, LOD-ready), keeping runtime cheap and deterministic.
  Runtime merge only as fallback for editor-authored composites. Charter: @coder-mcp (geometry op +
  GLB validator update). Effort: M pipeline.

CITY-P1 | **SHIPPED 2026-07-03** — block-scale `MigAStaticBulk` on street furniture · witness `city_p1_001_live.json`.
CITY-P2 | **SHIPPED 2026-07-03** — block LOD impostor (C8 pilot GLB fallback) vs street detail by `WorldLodBand` · witness `city_p2_001_live.json`.

# ═════════════════════════════════════════════════════════════════════
# EXECUTION ORDER + QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# G0 (now, 0.19-safe, parallel-ok):  CITY-G0a → G0b → G0c
# G1 (after G0):                     CITY-C4 → C1 → C2 → C3     ← the structural payoff
# G2 (after G1; pipeline+engine):    CITY-C5 · C6(visual part — pick now)
# G3 (C7 after G1; C8 anytime):      C7 · C8 · P1 (unblocked) · P2 (DR-CITY-P2) · C6(BSN — DR-CITY-C6-BSN)
#
# id                 | issue     | owner        | effort | exit
# CITY-G0-S11-001    | CITY-G0a  | coder_a      | S-M    | typed ids compile; deserialize validation test
# CITY-G0-S1C-001    | CITY-G0b  | coder_b      | M      | 3-way split, re-exports, no behavior change
# CITY-G0-WIT-001    | CITY-G0c  | coder        | S      | determinism witness green 3 consecutive runs
# CITY-G1-C4-001     | CITY-C4   | coder        | S      | seed-chain helper + call sites swapped; witness still green
# CITY-G1-C1-001     | CITY-C1   | coder        | M      | BlockArchetype + RON thresholds; unit test per band
# CITY-G1-C2-001     | CITY-C2   | coder        | M      | BlockFrame from junctions; overlay debug view
# CITY-G1-C3-001     | CITY-C3   | designer-mcp | M-L    | recipe vocab v1 charter + 3 archetype recipes authored
#
# Gate G1→G2: one full district renders as coherent street-anchored blocks from a fixed seed,
# determinism witness green, stage5 + construction tests green.

# ═════════════════════════════════════════════════════════════════════
# BSN ASSEMBLY CHARTER (product — NOT migration; MIG-A9 handoff complete 2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# Bevy Scene Next (BSN) = composable serialized scene roots via `WorldAssetRoot` / `WorldAsset`.
# Migration proved the API on one lane; **this program owns all further BSN adoption.**
#
# Migration pilot (done — do not re-pick as MIG-A9):
#   · block_street_visual.rs — presentation-only street furniture (settlement)
#   · Witness: debug_runs/mig_bevy_019/mig_a_a9_bsn_scene_handoff.json → adopted_settlement_lane
#   · Witness: debug_runs/city_c6_bsn_001_live.json
#
# Product question BSN answers: "Which composed visuals are a scene root vs per-entity ECS spawn?"
#
# OWNERSHIP (single authority per spawn path [K01]):
#   Lane                    | BSN when                         | ECS spawn when
#   ------------------------|----------------------------------|----------------------------------
#   Block street furniture  | C6-BSN charter: recipe edge()    | Until DR-CITY-C6-BSN unblocks
#   Building commit         | G3+ procedural assembly plan     | Default today (module entities)
#   Editor preview          | assembly_worker (existing)       | Transitional
#   Block LOD impostor      | block_lod_impostor (CITY-P2)     | After C8 merged GLB exists
#
# DETERMINISM (mcp-production-rules):
#   · Scene asset id + instance seed derived from block_seed / lot_seed chain (CITY-C4)
#   · Promoted scenes live under assets/models/modules/ — staging → validate → promote (MCP lane)
#   · No runtime procedural mesh; scene files come from deterministic bake only
#
# DESIGNER-MCP GATE (required before DR-CITY-C6-BSN closes):
#   · Which recipe primitives (edge/scatter) map to promotable BSN scene assets vs module GLB spawn
#   · AssetSpec per street-furniture archetype · atlas/module-kit alignment
#
# ENGINE SLICES (queue when gates clear):
#   CITY-C6-VIS-001  | C6 visual (ECS/recipe)     | coder_b  | DR-CITY-C6-VIS — pick now
#   CITY-C6-BSN-001  | BSN assembly expansion     | coder    | DR-CITY-C6-BSN — after charter
#   CITY-PROC-BSN-001| Building commit → scene    | coder    | After G1 + construction plan sign-off
#
# DO NOT: engine-wide BSN retrofit in one slice · duplicate spawn authority · pick "MIG-A9" from migration queue

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES (2026-07-03) — CLOSURE UPDATE
# ═════════════════════════════════════════════════════════════════════
# **Coder drain complete** — G0 witnesses green after grammar RON `footprint_mode` enum fix.
# G1–G3 + C6 BSN + P1/P2 witnesses green on master (Bevy 0.19, MIG-V1 gate_pass).
# DR-MIG-TILEMAP only remaining CITY deferral (bevy_ecs_tilemap 0.18.1 — adapter stays OFF).
# Next product lane: **PLAN-BUILDING-QUALITY-v1** (BQ-A1 adjacency landed; BQ-H/K/Q tail).
# BSN expansion beyond C6 pilot = designer-mcp charter when kit holes close (BQ-K1).
