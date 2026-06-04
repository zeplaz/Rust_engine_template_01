# Textures — lane map

## Folder layout (target)

```text
textures/
  terrain/              # Lane A — ground, floor, road surface (factory_floor, dirt, asphalt…)
  buildings_iso/        # Lane B — whole-building iso atlases
    production/         # Ship — keyframe + tilemapgen only
    archive/            # Retired lod0 pilots (see assets/archive/)
  vehicles/             # Lane B — 8-view rotation + cargo/light states (civ_truck_01…)
  power/                # Lane B — UI / build-rail iso grids
  _legacy_tiles/        # Optional symlink-era flat `tiles/` until reorganize script runs
```

**Proven bake spine:** [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md)  
Model in repo → `Light_keysshotsetup.blend` → `keyframe_render.py` → `tilemapgen` → atlas PNG.

---

## Lane A — Terrain / surface (keep & sort here)

| Pattern | Example | Use |
|:---|:---|:---|
| `tile_batch` terrain mode | `factory_floor_greybox_001` | Flat iso cells — damage/power/fill variants |
| Tiled tilesets | `assets/tiled/` | Editor / world layers |
| Terrain configs | `assets/config/terrain/` | Material rules — not PNGs |

These are **not** buildings. They tile the ground.

---

## Lane B — Vehicles & power (legacy gold — do not reorganize blindly)

| Path | Contract |
|:---|:---|
| `vehicles/<id>/tile_map_8_*` | 8 facings · states in filename (`empty`/`full`, `miday`/`night`) |
| `power/tile_map_*` | Hand-tuned iso rotation sheets for UI |

Referenced by [`assets/configs/vehicles/vehicle_configs.json`](../configs/vehicles/vehicle_configs.json) and Phase 4 icon atlas scripts.

---

## Lane B — Building iso atlases

| Tier | Path (after organize script) | Runtime stamp? |
|:---|:---|:---:|
| **production** | `buildings_iso/production/<atlas_id>_atlas.png` | Yes (when indexed) |
| **lod0 pilot** | `archive/lod0_tile_pilots_2026-06/` | **No** — smoke / witness only |

Pilot atlases under old flat `tiles/*_pilot_v1_*` are **quarantined**, not deleted. Index: [`_tile_atlas_index_archive.ron`](../configs/buildings/_tile_atlas_index_archive.ron).

**Not** the same as `power/` rotation grids: building atlases are **state rows** (day/night/damage/fire), one canonical iso view per variant unless gameplay adds a facing axis.

---

## What not to put here

- Single module GLBs → [`../models/modules/`](../models/modules/)
- Staging PNG sequences → [`../staging/tiles/`](../staging/tiles/) until pack + promote
