# PILOT-GRAMMAR-001 — G4 checklist (manual keyframe path) `v1`

| Field | Value |
|:---|:---|
| **Todo ID** | **PILOT-GRAMMAR-001** (prep doc — bake still blocked on APS-UI) |
| **Program** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |
| **Archetype** | `IndustrialWarehouse` |
| **District style** | `style_industrial_west` |
| **Owner** | `@designer` (+ `@designer-mcp` for G4 execution) |
| **Date** | 2026-06-02 |
| **Prerequisite audit** | [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md) |
| **Massing placement model** | [`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md) — pilot uses **perimeter grid (A)**; mesh-face (B) deferred |
| **Ship policy** | [`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](mcp_orchestrator_tile_fix_warehouse_slice_v2.md) — **reject headless minimum bake** |

---

## Bottom line

**Ship art = snapshot-authoritative materials in APS → headless worker `keyframe_render` + designer G4 on real 128px PNGs.**

> **Planner L1315+ (required):** Material assignment is **not** a Blender UI step. Assign `material_profile` in **Art Pipeline Suite** / assembly snapshot; blend build and headless worker **inherit** snapshot. **Pause** this pilot’s Blender-centric Phase 2 until [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md) slices are green. Manual keyframe = **render export** only, not “open blend and paint materials.”

| Path | Ship? |
|:---|:---:|
| Grammar snapshot → assembly blend → `Tile_iso_rig_v1` → **keyframe_render** → G4 | **yes** |
| `tile_compile_minimum_bake.py` (headless) | **no** |
| Headless `tile_keyframe_bake` / procedural grid in `tile_warehouse_industrial_v2_minimum_g4/` | **no** |
| `validate_tile_promotion` green on procedural PNGs alone | **no** (`proceed_ship` stays **no**) |

Reference rig: [`utils/TILE_ISO_RIG_README.md`](../../utils/TILE_ISO_RIG_README.md).

---

## Phase 0 — Preconditions

- [ ] **PG-MODULE-AUDIT-001** complete — [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md)
- [ ] Production shell GLBs present: `wall_steel_1u_production_run001`, `roof_sawtooth_production_run001`
- [ ] Assembly blend clean — **`ASSEMBLY` collection only** (no embedded rig/truck)
- [ ] Staging atlas **de-indexed** — `_tile_atlas_index.ron` entries `[]` until this checklist passes

**Cleanup if polluted:**

```powershell
cd C:\dev\github\Rust_engine_template_01
python tools/mcp/scripts/cleanup_assembly_blends.py
```

---

## Phase 1 — Grammar snapshot → assembly

**Goal:** `generate(archetype=IndustrialWarehouse, district_style=style_industrial_west, seed=…)` → snapshot JSON → MCP assembly build.

| Step | Action | Artifact / command |
|:---:|:---|:---|
| 1.1 | Confirm grammar rule chain in snapshot | `grammar_rule_chain` in assembly snapshot JSON |
| 1.2 | Generate or load warehouse pilot snapshot | `tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json` (or grammar-generated equivalent) |
| 1.3 | Build / refresh assembly blend (**modules only**) | `assets/staging/assemblies/industrial_west_4x2_s43_a879.blend` |
| 1.4 | Verify building definition matches snapshot modules | [`building_definition_warehouse_industrial_west_production_v1.json`](../../tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json) |

**Gate:** Open blend in Blender — single collection **`ASSEMBLY`**; no `TILE_ISO_RIG` saved into file.

---

## Phase 2 — Materials (APS / snapshot authority) — **not Blender UI**

| Step | Action | Notes |
|:---:|:---|:---|
| 2.1 | Ensure every placement has `material_profile` in snapshot | Generator + `enrich_placement` / **PG-MATERIAL-GENERATION-001** |
| 2.2 | APS Assembly Editor: material library → apply to selected slots | [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md) — **no** manual Blender material paint |
| 2.3 | APS preview sanity (Bevy worker or material thumbs) | **APS-PREVIEW-002** — steel/roof profiles readable before bake |
| 2.4 | Document lod0 fallbacks still in assembly | corner/door lod0 acceptable for prep; flag for PG-MODULE-AUDIT-002 |

**Gate:** Snapshot + APS preview green — **not** “open blend and assign in viewport.”

~~Deprecated (DCC drift):~~ Blender Phase 2 “Assign PBR materials in viewport” — superseded by planner §1315+.

---

## Phase 3 — Append iso rig (not saved into assembly)

Per [`TILE_ISO_RIG_README.md`](../../utils/TILE_ISO_RIG_README.md):

| Step | Action |
|:---:|:---|
| 3.1 | Rebuild rig if missing: `python -m rust_engine_mcp.cli build-iso-rig` |
| 3.2 | Open assembly blend |
| 3.3 | File → Append → `utils/Tile_iso_rig_v1.blend` → collection **`TILE_ISO_RIG`** |
| 3.4 | Confirm camera + lights drive keyframe targets — **no building meshes in rig file** |

**Gate:** Rig appended at bake time only; assembly file on disk remains ASSEMBLY-only after save discipline.

---

## Phase 4 — Manual keyframe_render (ship PNGs)

**Matrix (minimum G4 ship):** **3 states × 8 facings = 24 cells**

| State | In minimum set |
|:---|:---:|
| `clean_day` | yes |
| `clean_night_on` | yes |
| `damaged_night_on` | yes |

Facings: `0…7` (quarter-turn iso). Config: [`visual_config_warehouse_industrial_west_v2.json`](../assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json) · matrix YAML: [`warehouse_state_facing_matrix_v1.yaml`](../../debug_runs/art_pipeline/warehouse_state_facing_matrix_v1.yaml).

| Step | Action |
|:---:|:---|
| 4.1 | Run Blender addon **`keyframe_render.py`** against appended rig |
| 4.2 | Export **128×128 PNGs** per cell — real stills, not procedural slabs |
| 4.3 | Write export marker / folder path used by designer witness (e.g. `keyframe_manual.export` metadata) |
| 4.4 | **Do not** use `tile_compile_minimum_bake.py` as ship path |

**Forbidden:** `python tools/mcp/scripts/tile_compile_minimum_bake.py` without `--plan-only` for ship art.

---

## Phase 5 — Pack staging atlas (post-stills, pre-G4)

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli tile-atlas-pack path\to\manual_keyframe_png_folder
```

| Step | Action |
|:---:|:---|
| 5.1 | Pack PNG folder → staging atlas + `atlas_meta.json` |
| 5.2 | Validate schema: `python -m rust_engine_mcp.cli validate-report atlas_meta_v2 assets/staging/tiles/.../atlas_meta.json` |
| 5.3 | Validate visual config: `validate-report visual_config assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json` |

**Gate:** `minimum_g4_ship: true` on meta **and** PNGs sourced from manual keyframe folder — not headless bake output.

---

## Phase 6 — Designer G4 (@designer-mcp)

**Rubric:** [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md) · [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md)

| Step | Action | Witness |
|:---:|:---|:---|
| 6.1 | Review all **24** stills at 128px (truck / light spine parity) | signoff YAML |
| 6.2 | Set `proceed_ship: yes` **only** if art passes — not on schema green alone | `tile_fix_09_warehouse_g4_signoff.yaml` |
| 6.3 | Run designer witness CLI | `write-tile-fix-designer-g4-witness` |
| 6.4 | Confirm `art_quality: keyframe_manual` | `tile_fix_10_warehouse_industrial_live.json` → `green: true` |

**CLI chain (parity with @designer-mcp):**

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli validate-report visual_config assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json
python -m rust_engine_mcp.cli validate-report atlas_meta_v2 assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/atlas_meta.json
python -m rust_engine_mcp.cli write-tile-fix-10-witness --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json
python -m rust_engine_mcp.cli validate-report tile_promotion tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json
python -m rust_engine_mcp.cli write-tile-fix-designer-g4-witness --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json
```

**Exit G4:** `proceed_ship: yes` **and** `art_quality: keyframe_manual` **and** `minimum_g4_ship: true`.

---

## Phase 7 — Downstream (not @designer stop point)

Blocked until Phase 6 passes:

| Owner | Action |
|:---|:---|
| @coder-mcp | `--register` one `_tile_atlas_index.ron` row |
| @coder | Map stamp smoke (`map_tile_atlas_stamp`) |

---

## Current status (2026-06-03)

**Execution plan:** [`pilot_grammar_001_execution_v1.md`](pilot_grammar_001_execution_v1.md) — **Track A** (grammar E2E) vs **Track B** (ship/G4).

| Track / Phase | Status |
|:---|:---|
| **A Grammar E2E** (generate → verify → preview → assembly-build) | **ready** — code path done; close with `pilot_grammar_001_grammar_e2e_live.json` |
| 0 Preconditions | partial — 2 production modules; index de-indexed after rejection |
| 1 Grammar snapshot | **done** — `grammar_rule_chain`, APS grammar generate, PG-MATERIAL |
| 2 Materials (APS) | **partial** — material browser + Bevy preview; Material Studio tab pending |
| 3–4 Manual keyframe | **not done** — operator rejected headless stills |
| 5–7 Pack / G4 / register | **blocked** on Track B |

| Witness | `green` |
|:---|:---:|
| [`mcp_pilot_grammar_001_live.json`](../../debug_runs/art_pipeline/mcp_pilot_grammar_001_live.json) | **no** |
| [`mcp_pilot_grammar_001_rejected_live.json`](../../debug_runs/art_pipeline/mcp_pilot_grammar_001_rejected_live.json) | documents why |

**PILOT-GRAMMAR-001** = Track A + Track B. **Do not** mark program done on grammar code alone; **do not** ship on placement-only snapshots.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Prep checklist — manual keyframe path; rejects headless minimum bake |
