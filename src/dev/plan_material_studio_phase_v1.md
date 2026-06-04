# PLAN-MATERIAL-STUDIO-001 — Material track (post L1701+) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-MATERIAL-STUDIO-001** |
| **Source** | [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) **L1701–L2097** |
| **Parent** | [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md) · [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |
| **Status** | **ACTIVE** |
| **Date** | 2026-06-03 |

---

## Planner intent (L1701+)

Artists live in **Art Pipeline Suite** — not Blender. Materials are **first-class assets** (maps + recipe), not dropdown strings. **Material Studio** = dedicated APS tab with library, previews (sphere / wall / building section), and layer recipes later. Blender = **render / bake / convert worker** only ([`ARCH-BLENDER-001`](#arch-blender-001)).

```text
Artist → APS (Catalog | Assembly | Materials | Variants | Atlas | Grammar | Validation)
      → Build pipeline → Blender worker (invisible)
```

---

## What is already landed (do not redo)

| Planner ID | Repo slice | State |
|:---|:---|:---:|
| Material authority L1315+ | [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md) | **done** |
| APS-MATERIAL-BROWSER-001 | Assembly tab — thumbnail grid, click → `update_placement` | **done** |
| APS-PREVIEW-002 / 004 | `bevy_preview_worker` + assembly preview panel | **done** |
| CODER-PG-MATERIAL-GENERATION-001 | `district_styles[].material_profiles` → placements (Rust + Python) | **done** (queue) |
| Profile registry v1 | [`material_profiles_v1.json`](../../assets/materials/profiles/material_profiles_v1.json) + [`material_textures.py`](../../tools/mcp/python/rust_engine_mcp/material_textures.py) | **partial** — albedo/normal/roughness only; procedural tile gen |
| MCP-PILOT-GRAMMAR-001 | G4 ship | **blocked** — art rejection; not a material-code blocker |

**Gap vs L1701+:** no **Materials tab**, no categorized library, no material-only previews (sphere/wall), no layer recipes, no pipeline status board, no Blender worker material **apply** from snapshot.

---

## Material track map (planner L2055–L2078 → repo IDs)

| Planner | Repo ID | Phase |
|:---|:---|:---:|
| APS-MAT-001 Material Studio | **APS-MAT-001** | **A** |
| APS-MAT-002 Material Preview System | **APS-MAT-002** | **A** |
| APS-MAT-006 Material Library Browser | **APS-MAT-006** (extend browser) | **A** |
| APS-MAT-007 Building Material Assignment UI | **APS-MAT-007** (Assembly integration) | **A** (partial) |
| APS-MAT-008 Material Validation Gates | **APS-MAT-008** | **B** |
| ARCH-BLENDER-001 Blender Worker Contract | **BUILD-WORKER-001** | **B** |
| APS-MAT-003 Layer-Based Material Recipes | **APS-MAT-003** | **C** |
| APS-MAT-004 Procedural Texture Generation | **APS-MAT-004** | **C** |
| APS-MAT-009 Material Variant Integration | **APS-MAT-009** | **C** |
| APS-MAT-005 Reference Image Extraction | **APS-MAT-005** | **D** (defer) |

---

## Phase A — Material Studio MVP (next 2–3 slices)

**Goal:** Artist can browse, preview, and assign **real profile assets** without opening Blender.

### A1 — `APS-MAT-001` Materials tab shell

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | New **Materials** notebook tab in [`art_pipeline_suite/app.py`](../../tools/mcp/art_pipeline_suite/app.py) |
| @coder-mcp | Split layout: library (left) · preview stack (right) · properties (bottom) |

**Not in A1:** layer stacks, reference import.

### A2 — `APS-MAT-006` Categorized material library

| Owner | Deliverable |
|:---|:---|
| @planner-mcp | Extend [`material_profiles_v1.json`](../../assets/materials/profiles/material_profiles_v1.json) or `material_library_v1.json` with **`category`** (`industrial/steel`, `residential/brick`, …) per L1991–L2018 |
| @coder-mcp | Refactor [`material_browser.py`](../../tools/mcp/art_pipeline_suite/material_browser.py) → filter by category; search by id/label |
| @coder-mcp | Promote shared widget used by **Materials** tab + **Assembly** slot picker (one implementation) |

### A3 — `APS-MAT-002` Material preview modes

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | **Sphere** — PBR thumb from profile maps (Pillow / three.js sphere; same maps as today) |
| @coder-mcp | **Wall strip** — UV-friendly quad with tiling |
| @coder-mcp | **Building section** — reuse `bevy_preview_worker` on a **fixed warehouse slice** snapshot OR degraded three.js wall+roof GLBs |

**Witness:** `debug_runs/aps_material_studio_live.json` — selected profile, three preview modes green.

### A4 — `APS-MAT-007` Assignment polish (Assembly)

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | Assembly: show **applied profile** + category; “open in Materials tab” sync |
| @coder | Ensure grammar defaults + APS override both write `material_profile` on save |

**Phase A exit:** Warehouse pilot Phase 2 checklist = APS Materials + Assembly assign + Bevy assembly preview — **not** Blender viewport materials.

---

## Phase B — Worker + validation (unblock ship pipeline)

### B1 — `BUILD-WORKER-001` / ARCH-BLENDER-001

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | bpy/headless: read snapshot `material_profile` per placement → apply node groups / image textures from `assets/materials/textures/{profile_id}/` |
| @planner-mcp | Doc: Blender roles = **render | bake | convert** only — [`arch_blender_worker_contract_v1.md`](arch_blender_worker_contract_v1.md) (thin) |

### B2 — `APS-MAT-008` Material validation gates

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | Validator: every placement has `material_profile`; maps exist; optional AO/height warnings |
| @coder-mcp | CLI + MCP: `validate-material-profiles` / assembly ship gate hook |

**Witness:** `debug_runs/material_validation_live.json`.

### B3 — Pipeline status board (L2019–L2053)

| Owner | Deliverable |
|:---|:---|
| @coder-mcp | APS **Validation** or Assembly header: Grammar / Assembly / Materials / Variants / Preview / Atlas / Validation checklist for `assembly_id` |

---

## Phase C — Material grammar (later; do not block pilot)

| ID | Summary | Depends |
|:---|:---|:---:|
| **APS-MAT-003** | `MaterialRecipe` schema: base + layers (dirt, rust, snow, burn, moss) + params | A2 |
| **APS-MAT-004** | Bake maps from recipe (extend `material_textures` or Material Maker CLI) | A3 |
| **APS-MAT-009** | Variant sets map `clean_day` / `damaged` → profile ids or recipes | grammar variants + A3 |

**Rule:** Generated profiles like `steel_panel_01 + weathering:medium + rust:heavy` are **deterministic** (seed-driven), not LLM art.

---

## Phase D — Deferred

| ID | Why defer |
|:---|:---|
| **APS-MAT-005** Reference image → recipe | Needs vision/analysis contract; not pilot-critical |
| Full Material Maker tier-3 | External toolchain; keep procedural_tile_v1 until C2 |

---

## ARCH-BLENDER-001

Blender is **not** missing — it is **mis-roled**. Contract:

| Role | Allowed |
|:---|:---|
| Render worker | keyframe_render, tile ortho bake |
| Bake worker | atlas pack |
| Convert worker | GLB import/export repair |
| **Forbidden** | Primary material authoring, daily viewport assign |

Document only in Phase B1; enforce via checklists + worker APIs.

---

## Recommended execution order (next agent work)

```text
1. APS-MAT-001  Materials tab shell
2. APS-MAT-006  Categories + shared library widget
3. APS-MAT-002  Sphere + wall preview (building section can reuse Bevy)
4. APS-MAT-007  Assembly ↔ Materials tab sync
5. BUILD-WORKER-001  Snapshot-driven material apply in Blender worker
6. APS-MAT-008  Validation gates
7. Re-run MCP-PILOT-GRAMMAR-001  (human G4 on worker PNGs only)
```

**Parallel:** grammar / mesh-face massing ([`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md)) — **orthogonal** to Material Studio.

---

## Agent routing

| Slice | Agent |
|:---|:---|
| APS-MAT-001/002/006/007, BUILD-WORKER-001 | @coder-mcp |
| Schemas, categories, ARCH-BLENDER-001 | @planner-mcp |
| PG-MATERIAL defaults (if gaps) | @coder |
| G4 stills after B1 | @designer-mcp |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Phases A–D from planner_fix_auto_build L1701+ |
