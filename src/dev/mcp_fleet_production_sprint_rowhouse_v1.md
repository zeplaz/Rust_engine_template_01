# MCP production sprint — Victorian rowhouse `v1`

| Field | Value |
|:---|:---|
| **Sprint ID** | **MCP-PROD-SPRINT-ROWHOUSE-001** |
| **Parent** | [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md) |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-03 |
| **Archetype** | **rowhouse** · `style_victorian` · 4×3×2 |
| **Queue** | [`mcp_active_queue.json`](../../tools/orchestrator/queues/mcp_active_queue.json) |
| **HANDOFF** | [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) § Production sprint |

**One-sentence priority:** Unfreeze `kit_production_001` + coder-mcp real bpy/PBR for one rowhouse slice → designer-mcp promotes production modules and ships one keyframe-packed atlas → coder wires ENG-PT-4 so the map uses it.

---

## Plans index (read once)

| Doc | Role |
|:---|:---|
| [`development_plan_index.md`](development_plan_index.md) | Hub |
| [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md) | Module tiers · Phase B–D · `kit_production_001` |
| [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) | Tile + sim binding · PT-2–6 |
| [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) | **Keyframe = ship** (not smoke ortho) |
| [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md) | G4 rubric |
| [`assets_organization_v1.md`](assets_organization_v1.md) | Where files go |
| [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) | Designer onboarding |
| [`assets/textures/README.md`](../../assets/textures/README.md) | Iso output paths |

**Matrix (only):** [`variant_matrix_rowhouse_v1.yaml`](../../debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml)

---

## Existing shell (continue here — no new runners)

When staging already has variant PNGs (e.g. after **`pt2_production_ortho_seed.py`** in a background terminal, or a prior `tile-batch-run`):

```powershell
cd C:\dev\github\Rust_engine_template_01
$env:RUST_ENGINE_TILE_DRY_RUN = '0'
.\tools\mcp\scripts\run_rowhouse_production_existing.ps1
```

| Script | When |
|:---|:---|
| [`run_rowhouse_production_existing.ps1`](../../tools/mcp/scripts/run_rowhouse_production_existing.ps1) | Pack + register from **existing** `assets/staging/tiles/tile_rowhouse_victorian_production_v1/` |
| [`pt2_production_ortho_seed.py`](../../tools/mcp/scripts/pt2_production_ortho_seed.py) | Dev bridge only — seeds staging before the script above |
| [`mcp_prod_rowhouse_assembly_keyframe_g4.py`](../../tools/mcp/scripts/mcp_prod_rowhouse_assembly_keyframe_g4.py) | Full tail: production snapshot + assembly + **headless** keyframe + G4 |
| [`mcp_prod_tile_index_finalize.py`](../../tools/mcp/scripts/mcp_prod_tile_index_finalize.py) | Called by `run_rowhouse_production_existing.ps1` |

Set `pre_baked_folder` in the batch JSON to the folder that actually holds `{variant_key}.png` (staging or `keyframe_stills/`). Do **not** start `kit_production_001_batch_runner.py` if production modules are already in `_module_index.ron`.

**Next after pack:** `@coder` **ENG-PT-4-001** (map stamp). Designer ship sign-off still uses keyframe rubric when replacing ortho seeds with artist stills.

---

## Parallel lanes (do not confuse)

| Lane | Designer work | Ships in game? |
|:---|:---|:---:|
| **Production pilot (this sprint)** | Modules + keyframe atlas | **Yes** when ENG-PT-4 lands |
| lod0 `kit_lod0_003–010` | Roadmap — PG-2 assembly | 3D yes; **not** production map tiles |
| lod0 tile pilots | **Stop** — archived | No |
| Terrain `factory_floor` | Lane A under `textures/terrain/` | Surface only |
| Vehicles / power | UI briefs only | Icons — separate contract |

---

## Week plan

### Week 1 — unblock (coder-mcp before designer modules)

| Owner | ID | Deliverable |
|:---|:---|:---|
| @orchestrator-mcp | — | Unfreeze `kit_production_001` + `tile_rowhouse_victorian_production_v1` only ✓ |
| @coder-mcp | **MCP-PROD-B2** | Tier-aware `validate_asset_report` — cubes cannot pass pitched/arched |
| @coder-mcp | **MCP-PROD-C-PILOT** | bpy profiles: `module_wall`, `module_door`, `module_window`, `module_roof` (rowhouse slots) |
| @designer-mcp or @planner-mcp | **MCP-PROD-PBR-PILOT** | Material Maker install doc **or** waiver doc + tileable set ids for first 5 modules |

**Blocker rule:** Without **C-pilot + PBR-pilot**, designer-mcp cannot honestly label `development_tier: production` — only lod0 meshes.

### Week 2 — designer-mcp (one building)

| Track | Gates | Outcome |
|:---|:---|:---|
| **1 Modules** | G0→G5 on `kit_production_001` | 5–10 production rows in `_module_index.ron` |
| **2 Atlas** | Assembly + variants + pack + G4 | `rowhouse_victorian_production_signoff.yaml` `proceed_ship: yes` |

### Week 3 — coder (gameplay)

| ID | Outcome |
|:---|:---|
| **ENG-PT-4-001** | Map shows rowhouse production atlas by sim phase |
| (later) | Repeat pattern for warehouse / shopfront / bunker — **frozen** until sprint closes |

---

## Track 1 — Production modules (`kit_production_001`)

**Plan:** [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md) Phase **D** · manifest [`batch_kit_production_001.manifest.json`](../../tools/mcp/schemas/examples/batch_kit_production_001.manifest.json)

| Step | Gate | Designer-mcp | Output |
|:---|:---:|:---|:---|
| 0 | prereq | Wait for **MCP-PROD-C-PILOT** + **MCP-PROD-PBR-PILOT** | — |
| G0 | rules | `rules_audit` + `reference_tags` for Victorian rowhouse | `debug_runs/art_pipeline/rowhouse_production_module_g0_rules.yaml` |
| G1 | spec | AssetSpecs: `development_tier: production`, `pbr_status: shipped`, canonical `module_id` | `assets/staging/specs/` |
| G2 | geom | `geometry_run_job` (uses coder-mcp profiles) | `tools/mcp/jobs/*.json` |
| G3 | validate | `validate_asset_report` | validation-first only |
| G4 | sign | Silhouette + sim-read per module | notes in witness / signoff YAML |
| G5 | ship | `promote` + `library_register` | `_module_index.ron` tier `production` |

**Rowhouse slots (start with 5, may extend to 10):** `wall_brick_1u`, `corner_L`, `door_residential`, `roof_pitched_gable`, `prop_chimney`.

---

## Track 2 — Production assembly + tile atlas

**Plan:** [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) · matrix [`variant_matrix_rowhouse_v1.yaml`](../../debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml)

**Starts after Track 1 G5** (production GLBs in index).

| Step | Designer-mcp | Tooling |
|:---|:---|:---|
| Assembly | Production `module_placements` only; `reference_tags` on snapshot | Blender import or `assembly_build` |
| Variants | Bake all **required** keys (≥6 + `burning_00`…`07` per matrix) | `keyframe_render.py` + `utils/Light_keysshotsetup.blend` |
| Pack | PNG folder → atlas | `python -m rust_engine_mcp.cli tile-atlas-pack <folder> --keyframe-rename` |
| Register | `tile-batch-run` with `bake_source: keyframe_pack`, `ship: true`, **`pre_baked_folder`** set | Not `smoke_ortho_headless` |
| G4 | Atlas sign-off | `debug_runs/art_pipeline/rowhouse_victorian_production_signoff.yaml` → `proceed_ship: yes` |

**Batch JSON:** [`tile_batch_rowhouse_victorian_production_v1.json`](../../tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json)  
**Atlas target:** `assets/textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png`

**Designer onboarding paths:** [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) · [`assets/textures/README.md`](../../assets/textures/README.md)

---

## Coder-mcp (tooling + CI)

| ID | Task | Acceptance |
|:---|:---|:---|
| **MCP-PROD-B2** | Phase B2 tier validator | 24-vert cube fails pitched/sawtooth/arched profiles |
| **MCP-PROD-C-PILOT** | Phase C rowhouse profiles (4 ops) | Tactical-zoom readable silhouettes |
| **MCP-PROD-PBR-PILOT** | PBR path doc or waiver | `pbr_status: shipped` enforceable on promote |
| **MCP-PROD-TILE-VAL** | `tile_batch_validate` + `variant_matrix_expand` | Reject `ship: true` without `keyframe_pack` / `pre_baked_folder` |
| **MCP-PROD-INDEX** | Register production atlas | `_tile_atlas_index.ron`: `development_tier: production`, `ship_allowed: true` |
| **MCP-PROD-WIT** | Witness rollup | `procedural_tiles_production_bake_live.json` (rowhouse section) |
| **AUTO-010** (label) | APS Pipeline tab copy | **Production = keyframe folder → Pack**; `tile_batch_run` = CI/register only |

**Do not** use `bake_source: smoke_ortho_headless` for designer ship work.

---

## Engine (`@coder` — after atlas G4)

| ID | Code | Designer needs for |
|:---|:---|:---|
| **ENG-PT-4-001** | `TileVariantResolver` + `map_tile_atlas_stamp` + hide PG-2 mesh when production atlas | **Seeing buildings on map** |
| **ENG-PT-5-001** | Fire frame tick + dirty stamp | Fire variants from matrix |

Designer can finish atlases before PT-4; runtime catches up.

---

# Paste prompts (copy as-is)

## @coder-mcp — Week 1 unblock (run first)

```
Sprint MCP-PROD-SPRINT-ROWHOUSE-001 — Week 1 from src/dev/mcp_fleet_production_sprint_rowhouse_v1.md.

Scope: Victorian rowhouse only. validation-first on all validators.

1) MCP-PROD-B2 — plan_module_kit_production_tier_v1.md Phase B2:
   validate_asset_report tier rules; 24-vert cube must fail pitched/sawtooth/arched; greybox:* → smoke only.

2) MCP-PROD-C-PILOT — Phase C pilot (4 bpy ops for rowhouse manifest slots):
   module_wall, module_door, module_window, module_roof — NOT one scaled cube.
   Manifest: tools/mcp/schemas/examples/batch_kit_production_001.manifest.json

3) MCP-PROD-PBR-PILOT — document Material Maker install path OR waiver + tileable set ids
   (brick_red_01) so promote can enforce pbr_status: shipped.

Exit: designer-mcp unblocked for kit_production_001 G1–G5.
pytest tools/mcp/python/tests/ -q green.
```

---

## @designer-mcp — Track 1: production modules (after Week 1 coder-mcp)

```
Sprint MCP-PROD-SPRINT-ROWHOUSE-001 — Track 1 from src/dev/mcp_fleet_production_sprint_rowhouse_v1.md.

BLOCKED until MCP-PROD-C-PILOT + MCP-PROD-PBR-PILOT green.

Scope: kit_production_001 — Victorian rowhouse ONLY (5 modules, style_victorian).
Plan: plan_module_kit_production_tier_v1.md Phase D.
Do NOT open warehouse/shopfront/bunker matrices or other kit_production_* batches.

G0  rules_audit + reference_tags → debug_runs/art_pipeline/rowhouse_production_module_g0_rules.yaml
G1  AssetSpecs under assets/staging/specs/ — development_tier: production, pbr_status: shipped, canonical module_id
G2  geometry_run_job (coder-mcp profiles must exist)
G3  validate_asset_report — validation-first only
G4  Per-module silhouette + sim-read notes
G5  promote + library_register → _module_index.ron rows tier production

Modules: wall_brick_1u, corner_L, door_residential, roof_pitched_gable, prop_chimney (+ up to 5 more rowhouse slots if needed).

No lod0 relabel as production. No Blender GUI unless RUST_ENGINE_ART_DEBUG_GUI=1.
```

---

## @designer-mcp — Track 2: assembly + keyframe atlas (after Track 1 G5)

```
Sprint MCP-PROD-SPRINT-ROWHOUSE-001 — Track 2 from src/dev/mcp_fleet_production_sprint_rowhouse_v1.md.

Prereq: kit_production_001 modules promoted (production tier in _module_index.ron).

Plan: plan_procedural_building_tiles_production_v1.md
Matrix: debug_runs/art_pipeline/variant_matrix_rowhouse_v1.yaml ONLY
Spine: design_tile_bake_spine_convergence_v1.md — keyframe = ship

1) Production assembly snapshot — production module_placements + reference_tags
   (Blender import or assembly_build from production GLBs)

2) Variant PNGs — all required matrix keys (≥6 + burning_00…07)
   keyframe_render.py + utils/Light_keysshotsetup.blend

3) Pack: python -m rust_engine_mcp.cli tile-atlas-pack <export_folder> --keyframe-rename

4) Register batch tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json:
   bake_source: keyframe_pack, ship: true, pre_baked_folder: <your export folder>
   Then tile-batch-run for index/witness (CI path — not primary artist path)

5) G4: debug_runs/art_pipeline/rowhouse_victorian_production_signoff.yaml — proceed_ship: yes
   Rubric: design_procedural_tile_production_bar_v1.md

FORBIDDEN: bake_source smoke_ortho_headless for ship work.
Read: utils/LEGACY_ART_PIPELINE_README.md + assets/textures/README.md
```

---

## @designer — on-call player read (after Track 2 G4)

```
Sprint MCP-PROD-SPRINT-ROWHOUSE-001 — designer on-call from src/dev/mcp_fleet_production_sprint_rowhouse_v1.md.

After rowhouse_victorian_production_signoff.yaml proceed_ship: yes:
- Tactical review: production atlas reads as brick Victorian rowhouse vs lod0 pilot
- Co-sign rowhouse production player read (not DESIGN-PROC-ART-ACCEPTANCE-001 full 50 modules)
- No Rust

Witness note in procedural_assembly_pg2_signoff.yaml or new rowhouse_production_ux_signoff.yaml
```

---

## @coder — ENG-PT-4/5 (Week 3, after atlas exists)

```
Sprint MCP-PROD-SPRINT-ROWHOUSE-001 — engine from src/dev/mcp_fleet_production_sprint_rowhouse_v1.md.

Prereq: procedural_tiles_production_bake_live.json rowhouse green + _tile_atlas_index.ron production row.

ENG-PT-4-001 (≤6 files):
- Load assets/configs/buildings/_variant_catalog.ron (create if missing per plan)
- TileVariantResolver — sim phase → variant_key → UV from production atlas meta
- map_tile_atlas_stamp on map view
- Suppress PG-2 procedural GLB meshes when production atlas present for site

ENG-PT-5-001 (tail): fire frame tick + subregion dirty on fire band change.

Witness: debug_runs/art_pipeline/procedural_tiles_runtime_live.json
Do not rework PG-2 spine (procedural_assembly_live.json already green).
```

---

## Minimum designer-mcp checklist

- [ ] Week 1: coder-mcp B2 + C-pilot + PBR-pilot witnesses exist
- [ ] Track 1: 5+ modules at `production` in `_module_index.ron`
- [ ] Track 2: production assembly + reference_tags
- [ ] Track 2: keyframe PNGs → `tile-atlas-pack` → atlas at `assets/textures/buildings_iso/production/`
- [ ] G4 `proceed_ship: yes` on rowhouse production signoff
- [ ] **Not done:** warehouse / shopfront / bunker / 50-module acceptance

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Sprint doc + paste prompts + queue alignment |
