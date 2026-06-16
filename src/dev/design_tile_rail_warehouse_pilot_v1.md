# Rail warehouse iso tile pilot `v1` (BUILD-READ-VISUAL-002)

| Field | Value |
|:---|:---|
| **Program** | **BUILD-READ-VISUAL-002** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer-mcp` (spec) · `@coder-mcp` bake · `@coder` registry |
| **Verdict** | **PASS** (spec on disk) |
| **Parent** | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) |
| **Pilot** | [`logistics_rail_warehouse_pilot_v1.json`](../assets/configs/buildings/pilots/logistics_rail_warehouse_pilot_v1.json) |
| **Staging spec** | [`assets/staging/specs/tile_rail_warehouse_pilot_v1.json`](../assets/staging/specs/tile_rail_warehouse_pilot_v1.json) |
| **Witness** | [`debug_runs/design_tile_rail_warehouse_pilot_live.json`](../debug_runs/design_tile_rail_warehouse_pilot_live.json) |

**MCP lane:** spec + staging JSON only — bake via keyframe spine; no chat bpy.

---

## Tile identity

| Field | Value |
|:---|:---|
| **tile_id** | `tile_rail_warehouse_pilot_v1` |
| **style_pack** | `style_industrial_west` |
| **archetype** | `industrial_warehouse_l` |
| **footprint** | L-shape 6×5 (11 occupied) — matches mock shape |
| **bake_source** | `keyframe_pack` (`Light_keysshotsetup.blend` + `keyframe_render.py`) |
| **ship** | false (pilot — CI/smoke until G4) |

---

## Variant state machine (v1 pilot)

Minimum rows for BUILD-READ-VISUAL-001 acceptance:

| State id | Required | Notes |
|:---|:---:|:---|
| `clean_day` | Yes | Default operational read |
| `clean_night_off` | Yes | Lights off |
| `clean_night_on` | Yes | Yard/rail edge read |
| `damaged_day` | Defer v1.1 | After clean pass |
| `under_construction_01` | Yes | Ties CON-P2 stage read |
| `burning_00` … `burning_03` | Defer | Fire ecology lane |

**Pilot batch:** 4 frames → atlas pack → `assets/textures/buildings_iso/staging/tile_rail_warehouse_pilot_v1_atlas.png`

---

## Visual read targets

| Metric | Target |
|:---|:---|
| Primary screen height @ α0.42 | 40–90 px (with WORLD-002 multiplier) |
| Yard void readable | Side yard darker pad in iso still |
| Rail edge | North row rail glyph in still + site overlay |
| LOD | Iso tile stamp until PG-2 lod0 wired |

---

## MCP job chain (@coder-mcp)

```text
1. Validate staging spec (mcp_spec)
2. keyframe_render.py — batch stills → assets/staging/tiles/tile_rail_warehouse_pilot_v1/
3. tile-atlas-pack → atlas_meta.json
4. validate_asset_report (atlas PNG)
5. promote → assets/textures/buildings_iso/… + registry row
```

Seed: `440013` (deterministic pilot — do not change without witness bump).

---

## Bevy registry row (@coder)

| Field | Value |
|:---|:---|
| `catalog_id` | `pilot:logistics_rail_warehouse_v0` |
| `tile_atlas_id` | `tile_rail_warehouse_pilot_v1` |
| `default_state` | `clean_day` |
| `mesh_authority` | tile until PG-2 lod0 for pilot module kit |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (spec) | 2026-06-13 |
| `@coder-mcp` | pending bake | — |
| `@coder` | pending VISUAL-001 wire | — |

```text
BUILD-READ-VISUAL-002 spec complete
Unblocks: coder-mcp keyframe batch · BUILD-READ-VISUAL-001
```
