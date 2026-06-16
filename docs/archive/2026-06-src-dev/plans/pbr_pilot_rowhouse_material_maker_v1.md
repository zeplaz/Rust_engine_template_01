# PBR pilot — rowhouse production (`MCP-PROD-PBR-PILOT`)

| Field | Value |
|:---|:---|
| **Sprint** | [`mcp_fleet_production_sprint_rowhouse_v1.md`](mcp_fleet_production_sprint_rowhouse_v1.md) |
| **Batch** | `kit_production_001` |
| **Enforcement** | `tier.py` TIER-004 + `promote_module` via `validate_asset_glb` |
| **Status** | **CLOSED** (2026-06-06) — witness green · unblocks `MCP-PROD-MOD-G0-G5` (still waits `MCP-PROD-C-PILOT`) |

## Authoritative texture path (repo)

Tileable PBR sets live under:

`assets/materials/textures/{material_profile}/`

Expected maps (512×512 pilot):

| File | Role |
|:---|:---|
| `albedo.png` | Base color |
| `normal.png` | Normal |
| `roughness.png` | Roughness (ORM split for pilot) |

Each folder may include `manifest.json` (seed + generator metadata).

## Material Maker path (preferred for hero sets)

1. Install [Material Maker](https://github.com/RodZill4/material-maker) (desktop).
2. Author **tileable** sets at 512×512 (or 1024 for hero brick), seamless on U/V.
3. Export maps into `assets/materials/textures/{set_id}/` using the filenames above.
4. Reference the set in module specs as `material_profile` (alias of `tileable_set_id` in validators).

## MCP procedural path (pilot waiver — rowhouse Week 1)

When Material Maker is not installed, generate deterministic pilot maps via CLI:

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli generate-material-textures --profile brick_red_01
python -m rust_engine_mcp.cli generate-material-textures --profile wood_plank_01
python -m rust_engine_mcp.cli generate-material-textures --profile roof_tile_01
```

Record waiver in witness: `pbr_waiver: procedural_mcp_generator_v1`. **Not** a substitute for hero Material Maker sets in wave 002+.

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

Production promote with `pbr_status: shipped` is allowed when **either**:

- Material Maker exports exist under `assets/materials/textures/{set_id}/`, **or**
- MCP procedural maps were generated (see above) and witness records `pbr_waiver: procedural_mcp_generator_v1`.

Do not promote production modules with `pbr_status: deferred` or `none`.

## Witness

Refresh after validator or allowlist changes:

```powershell
python tools/mcp/scripts/write_pbr_pilot_rowhouse_witness.py
```

Outputs:

- `debug_runs/art_pipeline/pbr_pilot_rowhouse_live.json`
- `debug_runs/art_pipeline/pbr_pilot_rowhouse_witness.yaml`
