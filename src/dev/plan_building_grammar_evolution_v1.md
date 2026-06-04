# PLAN-BUILDING-GRAMMAR-001 — Hierarchical procedural building grammar `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-BUILDING-GRAMMAR-001** |
| **Source** | [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) (line 824+) |
| **Parent** | [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) · [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) |
| **Status** | **ACTIVE** |
| **Date** | 2026-06-02 |

---

## Strategic shift (planner)

**Material authority (L1315+):** Assign `material_profile` in **APS / assembly snapshot**; headless Blender **applies** snapshot at render — not manual DCC material editing. See [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md). **Pause** PILOT-GRAMMAR-001 G4 until material browser + preview + PG-MATERIAL-GENERATION are in place.

Evolve from **footprint → module placement** to **hierarchical architectural grammar** (Republic-style):

```text
District Style
  → Building Archetype
  → Massing Grammar
  → Facade Grammar
  → Detail Grammar
  → Damage/Age Grammar
```

**Generate API target:** `generate(archetype, district_style, seed)` — not only `generate(width, depth, floors)`.

**UX target:** Building Authoring Tool with previews + Grammar Inspector showing *why* a building was generated.

---

## File tiers (focus, not sprawl)

| Tier | Role | Paths |
|:---|:---|:---|
| **T1 Core generator** | Grammar + grid + snapshot + recipes | `src/construction/procedural/{footprint_grid,assembly_snapshot,types,load,module_index,variant_recipe}.rs` |
| **T2 Definitions** | What can be built | `building_definitions.rs`, `style_packs/*.ron`, `_module_index.ron` |
| **T3 Build compiler** | Scene assembly | `rust_engine_mcp/assembly.py`, `assembly_import.py`, `building_definition.py` |
| **Downstream** | Atlas, validators, witnesses, render extract — keep separate from grammar work | tile pipeline, promotion, `map_tile_atlas_stamp.rs`, etc. |

---

## Execution todos

| ID | Owner | Task | Depends | Status |
|:---|:---|:---|:---|:---:|
| **ARCH-PBG-MASSING-001** | @planner-mcp | PBG placement model: perimeter grid (now) vs mesh-face instancing (if scope grows) — **no codegen** until gate | — | **done** — [`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md) |
| **ARCH-BUILD-GRAMMAR-001** | @planner-mcp | Design `BuildingGrammar` RON/schema: `archetype`, `massing`, `roof`, `facade`, `detail`, `age`; wire `generate(archetype, district_style, seed)` contract | — | **done** — [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md) |
| **ARCH-BUILD-GRAMMAR-002** | @coder | Rust: `BuildingGrammar` types + evaluator in `src/construction/procedural/`; massing strategies (Long Hall, Double Hall, L, Yard…) for `IndustrialWarehouse` pilot | ARCH-BUILD-GRAMMAR-001 | **done** — `building_grammar.rs` |
| **ARCH-BUILD-GRAMMAR-003** | @coder-mcp | Python: mirror grammar in `assembly.py` — grammar picks modules/slots before footprint fill | ARCH-BUILD-GRAMMAR-001 | **done** — `building_grammar.py` + `generate_assembly_snapshot(..., archetype_id=, district_style=)` |
| **ARCH-ASSEMBLY-GRAPH-002** | @planner-mcp + @coder-mcp | Semantic nodes + snapshot `grammar_rule_chain` | ARCH-003 **done** | **done** — [`arch_assembly_graph_002_v1.md`](arch_assembly_graph_002_v1.md) |
| **APS-TAGS-001** | @planner-mcp | Tag taxonomy schema (4 categories) | — | **done** — [`aps_tags_001_v1.md`](aps_tags_001_v1.md) |
| **APS-TAGS-002** | @coder-mcp | APS + snapshot: categorized tags; grammar rules filter by category | APS-TAGS-001 | **ready** |
| **APS-UI-003b-EXPANDED** | @coder-mcp + @designer | **Building Authoring Tool**: clickable footprint grid, placement heatmap (W/D/C/R/roof/stack), archetype picker, grammar inspector panel | ARCH-BUILD-GRAMMAR-002, APS-TAGS-001 | **pending** |
| **APS-GRAMMAR-INSPECTOR-001** | @coder-mcp | Panel: show rule chain (archetype → massing → roof → facade → detail → seed) for loaded snapshot | ARCH-BUILD-GRAMMAR-002 | **pending** |
| **APS-PREVIEW-001** | @coder-mcp | Catalog: keep browser GLB preview (existing) | — | **partial** |
| **APS-PREVIEW-002** | @coder + @coder-mcp | Assembly: preview assembled building (Bevy subprocess thumb **or** three.js/model-viewer multi-GLB) | APS-UI-003b-EXPANDED | **pending** |
| **APS-PREVIEW-003** | @coder-mcp | Variants: preview variant state thumb; Atlas: atlas + UV grid overlay | APS-PREVIEW-002 | **pending** |
| **APS-PREVIEW-004** | @planner | Bevy preview worker architecture (doc only) | APS-PREVIEW-002 | **done** — [`aps_preview_004_bevy_worker_v1.md`](aps_preview_004_bevy_worker_v1.md) |
| **PG-MODULE-AUDIT-001** | @designer-mcp | Audit `_module_index` vs categories: walls, roofs, corners, windows, doors, stacks, vents, pipes, platforms, signs, lights, AC, cranes | — | **done** — [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md) |
| **PG-MODULE-AUDIT-002** | @coder-mcp | Gap report + production job_ids for missing categories (warehouse pilot first) | PG-MODULE-AUDIT-001 | **done** — witness `debug_runs/art_pipeline/pg_module_audit_002_live.json`; batch `kit_industrial_west_production_001` P0+P1 promoted |
| **PG-QUALITY-001** | @planner + @coder | Metrics: silhouette count, roof/facade/detail diversity per seed sweep; witness JSON | ARCH-BUILD-GRAMMAR-002, PG-MODULE-AUDIT-001 | **done** — `debug_runs/grammar_diversity_witness.json` |
| **ARCH-MATERIAL-AUTHORITY-001** | @planner | L1315+ contract: APS owns materials; snapshot authoritative; Blender worker only | — | **done** — doc |
| **APS-MATERIAL-BROWSER-001** | @coder-mcp | Thumbnail material library (not combobox-only) | ARCH-MATERIAL-AUTHORITY-001 | **done** |
| **PG-MATERIAL-GENERATION-001** | @coder + @coder-mcp | Generator emits `material_profile` per placement | ARCH-MATERIAL-AUTHORITY-001 | **ready** |
| **BUILD-WORKER-001** | @coder-mcp | Headless: snapshot-driven material apply + render | APS-MATERIAL-BROWSER-001, PG-MATERIAL-GENERATION-001 | **done** — witness `debug_runs/build_worker_001_live.json`; `--render-still` keyframe leg |
| **PILOT-GRAMMAR-E2E-001** | @coder-mcp | Grammar-only E2E witness (not placement-only) | grammar + verify + preview | **ready** — [`pilot_grammar_001_execution_v1.md`](pilot_grammar_001_execution_v1.md) Track A |
| **PILOT-GRAMMAR-001** | @designer-mcp + @coder-mcp | Full pilot: Track A + **ship** keyframe → G4 → register | Track B + Material Studio A | **blocked** (G4 rejection) |

---

## Planner orders 1–7 (reference)

| Order | Summary |
|:---|:---|
| **1** | Formal `BuildingGrammar` — ARCH-BUILD-GRAMMAR-* |
| **2** | Assembly graph semantic metadata — ARCH-ASSEMBLY-GRAPH-002 |
| **3** | Assembly Editor → Building Authoring Tool — APS-UI-003b-EXPANDED |
| **4** | Preview on every tab — APS-PREVIEW-* |
| **5** | Categorized tags — APS-TAGS-* |
| **6** | Grammar debug / inspector — APS-GRAMMAR-INSPECTOR-001 |
| **7** | Module library audit — PG-MODULE-AUDIT-* |

---

## Done (spine prerequisite)

| ID | Note |
|:---|:---|
| ARCH-001 | `assembly_graph_node_v1.schema.json` |
| ARCH-003 | `material_profile`, tags, `lod_policy` on placements |
| APS-UI-003b | Basic Assembly Editor (list + material/tags/LOD) |

---

## Agents

| Lane | Agent |
|:---|:---|
| Grammar architecture + schemas | @planner-mcp |
| Rust procedural core | @coder |
| MCP + APS UI + Blender | @coder-mcp |
| Module kit audit + G4 | @designer-mcp |
| Program order | @planner |

**Orchestrator paste:**

```text
Execute PLAN-BUILDING-GRAMMAR-001 in todo order: ARCH-BUILD-GRAMMAR-001 → grammar Rust/Python → ARCH-ASSEMBLY-GRAPH-002 → APS-TAGS → APS-UI-003b-EXPANDED + Grammar Inspector → APS-PREVIEW → PG-MODULE-AUDIT → PILOT-GRAMMAR-001.
Focus T1–T3 files only; tile/atlas work stays downstream.
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Todos from planner_fix_auto_build.md §824+ |
