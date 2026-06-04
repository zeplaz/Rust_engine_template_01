# Proven iso tile bake spine (utils — keep files in place)

**Policy:** [`src/dev/design_tile_bake_spine_convergence_v1.md`](../src/dev/design_tile_bake_spine_convergence_v1.md)

**Do not delete or move** these paths. MCP **wraps** them for pack/register; **does not replace** keyframe still export for production ship art.

---

## Authoritative workflow (vehicles, buildings, power)

```text
Model in assets (GLB / blend)
  → Blender + Tile_iso_rig_v1.blend (camera + lights, no meshes)
  → keyframe_render.py (export selected frames → PNGs)
  → tilemapgen [-pk] (folder → tile_map_*.png / atlas)
  → _tile_atlas_index.ron + engine TileAtlasRegistry
```

**Reference:** `assets/textures/vehicles/civ_truck_01/` — 8 facings, empty/full, day/night emission (`vehicle_configs.json`).

---

## Tile atlas packing (Rust)

| Path | Role |
|:---|:---|
| [`tilemapgen/`](tilemapgen/) | Primary: folder of PNG stills → single `tile_map_<parent>_<folder>.png` grid |
| [`tileMape_gen_basic/`](tileMape_gen_basic/) | Same algorithm (duplicate crate name `tilemapgen`) |

**Usage:**

```text
tilemapgen <folder_of_pngs>           # pack all PNGs into one atlas image
tilemapgen <folder> -pk               # rename keyframe PNG prefixes first (40→00, 1→01, …)
```

**MCP:** `python -m rust_engine_mcp.cli tile-atlas-pack <folder> [-pk]` — same binary.

---

## Blender render / lighting

| File | Role |
|:---|:---|
| `Tile_iso_rig_v1.blend` | **Authoritative** iso camera + lights only (`TILE_ISO_RIG` collection) |
| `Light_keysshotsetup.blend` | **Deprecated** legacy source (97MB, civ truck scene) — extract-only via `build-iso-rig` |

Assembly staging blends (`assets/staging/assemblies/*.blend`) are **ASSEMBLY GLBs only** — never embed the rig.  
Append/link `Tile_iso_rig_v1.blend` at bake or manual keyframe time. Env: `RUST_ENGINE_TILE_LIGHT_BLEND`.

Rebuild rig: `python -m rust_engine_mcp.cli build-iso-rig`  
Clean + rebuild assemblies: `python tools/mcp/scripts/cleanup_assembly_blends.py`

---

## Keyframe → PNG sequence (Blender script)

| Path | Role |
|:---|:---|
| [`keyframe_render.py`](keyframe_render.py) | Blender addon: UI panel → render selected frames to `{frame}_{basename}_.png` |

**Production buildings:**

1. Import assembly (or open staging blend).
2. Set variant layers (day/night/damage/fire).
3. **Render Keyframes to Images**.
4. `tile-atlas-pack` with `-pk` if needed.
5. Register atlas (`tile-atlas-register`).

**Not production:** headless `tile_ortho_bake` alone (lod0 bunker pilot slabs) — CI/smoke only per convergence doc.

---

## MCP `bake_source` contract

| Mode | Use |
|:---|:---|
| `keyframe_pack` | **Ship** — pre-baked PNG folder; MCP packs + registers only |
| `smoke_ortho_headless` | CI / pytest dry-run — `tile_ortho_bake` bpy stub |

---

## Preview / APS entry points

```powershell
python tools/mcp/art_pipeline_suite/run.py
# Atlas tab: Pack atlas (tilemapgen) | Keyframe addon | tile_batch_run = CI/smoke unless keyframe_pack batch
```

See [`src/dev/plan_art_preview_hub_v1.md`](../src/dev/plan_art_preview_hub_v1.md).

---

## Not tile-related (reference only)

| File | Role |
|:---|:---|
| `world_generator_tool.py` | Terrain/worldgen helper |
| `road_*.py`, `vechicle_config_builder.py` | Transport/vehicle JSON |
| `asset_tools/` | Qt full asset editor (buildings JSON) |
