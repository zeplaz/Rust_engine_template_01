# TILE-FIX-001 — Greybox production tile freeze `v1`

| Field | Value |
|:---|:---|
| **Status** | **FROZEN** (2026-06-02) |
| **Program** | PLAN-TILE-FIX-AUTO-BUILD-001 |
| **Source** | [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) |

---

## What was wrong

| Issue | Detail |
|:---|:---|
| Wrong spine | PG assembly → greybox GLB → **one ortho per variant_key** → pack |
| Missing axes | No **facing** grid; fire keys are separate states, not `variant × facing × frame` |
| False green | Witnesses passed on **PNG exists** + byte size, not art or lookup completeness |
| Active index | `buildings_iso/production/*` registered as `ship_allowed: true` |

---

## What we did

1. **Emptied** [`_tile_atlas_index.ron`](../assets/configs/buildings/_tile_atlas_index.ron) — runtime registry loads **zero** building iso atlases until v2 ships.
2. **Archived** all production v1 + pilot rows in [`_tile_atlas_index_archive.ron`](../assets/configs/buildings/_tile_atlas_index_archive.ron) with `ship_allowed: false` and `development_tier: greybox_frozen_v1` (production) or `lod0` (pilots).
3. **Moved** art + staging under `assets/archive/greybox_tile_production_v1_frozen_2026-06/` (script: `tools/mcp/scripts/freeze_greybox_production_v1.py`).
4. **Set** all `tile_batch_*_production_v1.json` examples to `ship: false`, `frozen: true`.
5. **Schema v2** — [`atlas_meta_v2.schema.json`](../../tools/mcp/schemas/atlas_meta_v2.schema.json) + [`visual_config_v1.schema.json`](../../tools/mcp/schemas/visual_config_v1.schema.json).
6. **Validators** — ship batches require `atlas_schema_version: 2` + `visual_config_ref`; G3 witness requires v2 lookup completeness (not PNG-only).

---

## v2 headless atlas (2026-06-03) — also NOT ship art

`warehouse_industrial_west_v2_atlas.png` under **`buildings_iso/production/`** was produced by **`tile_keyframe_bake` headless** (same spine as v1 grey slabs). TILE-FIX-010 **schema green** ≠ civ-truck quality.

- **Moved** to `assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/` (debug only).
- **`tile_fix_10_warehouse_industrial_live.json`** set `green: false`, `art_quality: rejected_headless_procedural`.
- **Active index** stays empty until **manual `keyframe_render.py`** + designer G4.

## Do not

- Re-register v1 or v2 headless atlases in the active index.
- Treat `assets/textures/buildings_iso/production/*` as the ship target path.
- Mark `proceed_ship: yes` on greybox signoffs.
- Run `mcp_export_pilot_keyframes_g4.py` or `tile_compile_minimum_bake` alone as finished art.

---

## Next (TILE-FIX-03+)

Build v2 atlas from procedural assembly → keyframe_render → pack; promote only when `atlas_meta.schema_version == 2` and all `render_contract.facings` cells exist per state.
