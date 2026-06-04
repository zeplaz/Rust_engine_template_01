# Design — Tile bake spine convergence `v1` (authoritative)

| Field | Value |
|:---|:---|
| **ID** | **DESIGN-TILE-SPINE-001** |
| **Status** | **SIGNED** |
| **Date** | 2026-06-03 |
| **Owner** | @planner + @designer-mcp |
| **Programs** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) · [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) |

---

## Problem

`tile_batch_run` automated **headless `tile_ortho_bake`** on lod0 procedural assemblies. That path:

- Skipped **artist framing** in `utils/Light_keysshotsetup.blend`
- Skipped **keyframe_render.py** variant stills
- Produced **2-state slabs** instead of full per-state tiles
- Let agents treat bunker **lod0 pilot atlases** as production templates

**Root cause:** automation replaced the **wrong sub-step** — pack/register was correct (`tilemapgen`); **render quality bar** was dropped.

---

## Authoritative spine (all iso product art)

```text
MODEL IN REPO (GLB / assembly blend — ASSEMBLY collection only)
  → Append utils/Tile_iso_rig_v1.blend → TILE_ISO_RIG (camera + lights only)
  → keyframe_render.py (selected frames → {frame}_{name}_.png)
  → optional tilemapgen -pk (prefix normalize)
  → tile_atlas_pack / utils/tilemapgen (folder → tile_map_* or promoted *_atlas.png)
  → atlas_meta.json + _tile_atlas_index.ron
  → TileAtlasRegistry → map stamp (engine)
```

**Reference implementations in repo:**

| Asset family | Path | Notes |
|:---|:---|:---|
| Civilian truck | `assets/textures/vehicles/civ_truck_01/` | 8 facing views; empty/full; day/night emission |
| Power | `assets/textures/power/` | Hand-tuned iso rotation grids |
| Buildings (target) | `assets/staging/tiles/<batch_id>/` | One iso + many **state** rows; rotation only if gameplay needs facing |

---

## Bake modes (machine contract)

| `bake_source` | Who uses it | Renders PNGs via |
|:---|:---|:---|
| **`keyframe_pack`** | **Ship / production** (`ship: true`) | Artist + `keyframe_render` → folder; MCP only **packs + registers** |
| **`smoke_ortho_headless`** | CI / APS spine only | `tile_ortho_bake` bpy (dry-run or stub until parity) |

| Rule | Enforcement |
|:---|:---|
| `ship: true` **requires** `bake_source: keyframe_pack` | `validators/tile_batch.py` error |
| `ship: true` **forbids** `source_tier: lod0` | existing PT-2-003 |
| Program green for production tiles | Witness must record `bake_source: keyframe_pack` and `real_pngs_ok` |
| Agents must not promote lod0 pilot atlases | `lod0_atlas_ship_allowed: false` |

---

## Convergence by asset type

| Asset type | Primary bake | Atlas pack | Rotation grid |
|:---|:---|:---|:---|
| **Vehicles** | Blender + keyframes | tilemapgen (unchanged) | **8 views** (`tile_map_8_*`) |
| **Power / UI icons** | Hand or phase script | Multi-view sheet | As today |
| **Buildings (production)** | Same as truck: assembled scene, variant keyframes | tilemapgen → `*_atlas.png` | **`variant × facing × frame`** — [`atlas_meta_v2.schema.json`](../../tools/mcp/schemas/atlas_meta_v2.schema.json) + `visual_config` |
| **Terrain filler** | `smoke_ortho` or flat base | optional | N/A |

---

## What `tile_ortho_bake` is for (until parity)

| Allowed | Forbidden |
|:---|:---|
| `RUST_ENGINE_TILE_DRY_RUN=1` pytest | `ship: true` production witness |
| Assembly import smoke | Replacing keyframe still export |
| Future: headless port of **same rig** as Light_keysshotsetup | Thin ortho stub as “done” art |

**Parity gate:** headless bake may ship only when G4 rubric scores match civ_truck stills at 128px (T4 night read, T6 fire frames).

### Optional headless export (v1.1 — after manual G4 green)

| Env | Tool | Behavior |
|:---|:---|:---|
| `RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1` | `tile-keyframe-export <tile_batch.json>` | Blender headless: assembly blend + `Tile_iso_rig_v1.blend` (CI/schema — not ship art) |
| Same + `tile-batch-run` on `keyframe_pack` batch | Auto-fills missing `{variant_key}.png` in staging before pack |

Implementation: `tools/mcp/blender/scripts/ops/tile_keyframe_bake.py` · dispatch when `render.method == blender_keyframe_light_rig`.

**Do not** enable for TILE-PROD-001 program green until manual keyframe path + designer G4 pass on rowhouse minimum stills.

---

## Agent / skill policy (no half measures)

| Do | Don't |
|:---|:---|
| Pack existing PNG folders with `tile-atlas-pack [-pk]` | Run `tile-batch-run` on production batches expecting art |
| Author variants in APS → export via keyframe workflow | Treat 2-variant lod0 pilot as template |
| Expand matrix → bake stills in Blender → then MCP pack | Skip designer G4 on “automation green” |
| Use `variant_catalog` keys for states | Invent ad-hoc `variant_key` per batch |

**Orchestrator:** block TILE-PROD-001 pass unless `bake_source == keyframe_pack`.

---

## Practical workflow (buildings, today)

**Primary:** [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) — Art Pipeline Suite → MCP → headless Blender.

```text
python tools/mcp/art_pipeline_suite/run.py
  Catalog  → validate modules, material_profile in index
  Assembly → snapshot (+ future Assembly Editor: material/tags per slot)
  Variants → variant_set_v1 layers → Bake selected / tile_batch_run
  Atlas    → pack_atlas after G4 stills land in staging
```

CLI parity (agents): `assembly_snapshot_generate` → `assembly_build_job` → `variant_bake` / `tile-batch-run` → `tile-atlas-pack`.

**Debug/recovery only** (manual Blender): [`warehouse_tile_ship_workflow_v1.md`](warehouse_tile_ship_workflow_v1.md) — not ship until **PILOT-001**.

**APS Atlas tab:** **Pack atlas** on real PNG folders; `tile_batch_run` = production bake when RENDER-001 green, else CI/smoke until then.

---

## Related docs (updated)

| Doc | Change |
|:---|:---|
| [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) | Renamed conceptually to **proven spine** (files stay) |
| [`design_procedural_tile_production_bar_v1.md`](design_procedural_tile_production_bar_v1.md) | T-bake uses keyframe spine |
| [`mcp_fleet_tile_automation_orders_v1.md`](mcp_fleet_tile_automation_orders_v1.md) | AUTO-004 smoke-only banner |
| `.cursor/skills/tile-generation` | Primary workflow = keyframe → tilemapgen |
| `.cursor/agents/designer-mcp` | Forbidden: ortho stub as ship |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Convergence after bunker pilot postmortem; bake_source contract |
