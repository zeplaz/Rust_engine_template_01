# Assets layout (authoritative map)

**Policy:** [`src/dev/assets_organization_v1.md`](../src/dev/assets_organization_v1.md) · tile bake spine: [`src/dev/design_tile_bake_spine_convergence_v1.md`](../src/dev/design_tile_bake_spine_convergence_v1.md)

Nothing under `assets/` is deleted by automation — only **moved to `archive/`** with a manifest log.

---

## Three lanes (do not mix)

| Lane | Question it answers | Primary paths |
|:---|:---|:---|
| **A — Surface / terrain** | What does the **ground** look like? | [`textures/terrain/`](textures/terrain/) · [`tiled/`](tiled/) · terrain configs |
| **B — Iso stamps** | What does a **whole building / vehicle / icon** look like on the map or UI? | [`textures/vehicles/`](textures/vehicles/) · [`textures/power/`](textures/power/) · [`textures/buildings_iso/`](textures/buildings_iso/) |
| **C — 3D modules** | What **pieces** snap together for procedural buildings? | [`models/modules/`](models/modules/) · [`configs/buildings/style_packs/`](configs/buildings/style_packs/) |

**Buildings:** procedural gen places **many module GLBs** (lane C). The **map** shows **one iso image per state** over the footprint (lane B), not one brick tile per grid cell.

---

## Top-level folders

| Path | Role |
|:---|:---|
| [`textures/`](textures/README.md) | All raster iso / surface art |
| [`models/`](models/README.md) | GLB module kit (one folder per promoted job) |
| [`staging/`](staging/) | WIP only — Blender jobs, tile PNGs, assemblies (safe to prune after promote) |
| [`archive/`](archive/README.md) | Quarantined pilots / superseded art — **kept for history** |
| [`configs/`](configs/) | JSON/RON gameplay data (`_module_index.ron`, `_tile_atlas_index.ron`, buildings, vehicles) |
| [`inventory/`](inventory/lane_map.json) | Machine-readable lane classification |
| [`tiled/`](tiled/) | Tiled TMX/TSX (editor / legacy iso workflows) |
| [`data/`](data/) | Legacy `.dat` tables (trucks, terrains) — reference only |

---

## Indexes (engine)

| Index | Points at |
|:---|:---|
| `_module_index.ron` | Lane C — `module_id` → `model.glb` |
| `_tile_atlas_index.ron` | Lane B — **production** building iso atlases only |
| `_tile_atlas_index_archive.ron` | Retired lod0 pilots (not loaded at runtime) |
| `vehicle_configs.json` | Lane B — vehicle `tile_map_8_*` paths |

---

## Safe reorganization

```powershell
# Preview moves (no changes):
.\tools\mcp\scripts\organize_texture_assets.ps1 -WhatIf

# Execute moves + update archive manifest:
.\tools\mcp\scripts\organize_texture_assets.ps1
```

See [`archive/README.md`](archive/README.md) before moving anything by hand.
