# PBR pilot — rowhouse production (`MCP-PROD-PBR-PILOT`)

| Field | Value |
|:---|:---|
| **Sprint** | [`mcp_fleet_production_sprint_rowhouse_v1.md`](mcp_fleet_production_sprint_rowhouse_v1.md) |
| **Batch** | `kit_production_001` |
| **Enforcement** | `tier.py` TIER-004 + `promote_module` via `validate_asset_glb` |

## Material Maker path (preferred)

1. Install [Material Maker](https://github.com/RodZill4/material-maker) (desktop) or use the engine’s pinned export folder under `assets/textures/tileable/`.
2. Author **tileable** sets at 512×512 (or 1024 for hero brick), seamless on U/V.
3. Export maps as `{set_id}_albedo.png`, `{set_id}_normal.png`, `{set_id}_orm.png` into `assets/textures/tileable/{set_id}/`.
4. Reference the set in module specs as `material_profile` (alias of `tileable_set_id` in validators).

## Pilot allowlist (`PRODUCTION_TILEABLE_SET_IDS`)

| `material_profile` | Module slot |
|:---|:---|
| `brick_red_01` | `wall_brick_1u`, `corner_L`, `prop_chimney` |
| `wood_plank_01` | `door_residential` |
| `roof_tile_01` | `roof_pitched_gable` |
| `concrete_grey_01`, `steel_panel_01`, `stucco_cream_01`, `glass_panel_01`, `roof_metal_01` | reserved for wave 002+ |

## Spec contract (production promote)

```json
{
  "development_tier": "production",
  "pbr_status": "shipped",
  "material_profile": "brick_red_01"
}
```

Promote **fails** when `pbr_status != shipped`, `material_profile` is missing, or the id is not in the pilot allowlist.

## Waiver (temporary)

If Material Maker is not installed on a machine, designers may still **bake geometry** with `pbr_status: shipped` only after:

- A written waiver row in witness JSON (`pbr_waiver: documented_tileable_set_only`), and
- Tileable PNGs already present under `assets/textures/tileable/{set_id}/` from a prior export.

Do not promote production modules with `pbr_status: deferred` or `none`.

## Witness

Refresh after validator or allowlist changes:

```powershell
python tools/mcp/scripts/write_pbr_pilot_rowhouse_witness.py
```

Output: `debug_runs/art_pipeline/pbr_pilot_rowhouse_live.json`
