# Assets organization `v1`

| Field | Value |
|:---|:---|
| **ID** | **ASSETS-ORG-001** |
| **Date** | 2026-06-03 |
| **Status** | **ACTIVE** |
| **Owner** | `@designer-mcp` + `@coder` |

---

## Problem

`assets/textures/tiles/` mixed **terrain filler**, **lod0 building pilots**, and confused agents into thinking building art = power-style rotation sheets. Module GLBs live in flat `models/modules/<job_id>/` folders with no lane docs.

---

## Principles

1. **No blind deletes** — quarantine under `assets/archive/` + `MOVED_LOG.json`.
2. **Indexes are law** — moving PNGs requires index path updates (script or MCP register).
3. **Three lanes** — see [`assets/README.md`](../../assets/README.md).
4. **Runtime** loads `_tile_atlas_index.ron` only; **production** tier stamps on map.

---

## Target texture tree

```text
assets/textures/
  terrain/                 # Lane A
  buildings_iso/
    production/            # Lane B ship
  vehicles/                # Lane B (unchanged)
  power/                   # Lane B (unchanged)
assets/archive/
  lod0_tile_pilots_2026-06/
assets/models/modules/<job_id>/   # Lane C — path stable until category migration script
```

---

## Reorganization script

```powershell
.\tools\mcp\scripts\organize_texture_assets.ps1 -WhatIf   # preview
.\tools\mcp\scripts\organize_texture_assets.ps1           # move + log
```

Moves (when source exists):

| From | To |
|:---|:---|
| `textures/tiles/*_pilot_v1_atlas.png` | `archive/lod0_tile_pilots_2026-06/` |
| `textures/tiles/factory_floor_*` | `textures/terrain/` |

Does **not** move `vehicles/`, `power/`, or `models/modules/`.

---

## Building vs procedural (clarity)

| Layer | Artifact | Count per building |
|:---|:---|:---|
| 3D assembly | `module_placements[]` + GLBs | Many (W×D footprint) |
| Map stamp | One `variant_key` UV rect | One iso image scaled to footprint |
| Fire sim | `burning_00`…`07` | 8 **full** atlas cells |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Lane READMEs, archive manifest, empty active tile index, organize script |
