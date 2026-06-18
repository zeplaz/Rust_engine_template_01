# DMCP spec — substation yard `v1`

| Field | Value |
|:---|:---|
| **ID** | **DMCP-SPEC-SUBSTATION-YARD-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer-mcp` |
| **Depends** | **DES-ART-UTILITY-STYLE-001** PASS |
| **Catalog** | [`grid_substation.json`](../../assets/configs/buildings/grid_substation.json) |
| **Site pilot** | [`power_substation_yard_site_v0.json`](../../assets/configs/buildings/pilots/power_substation_yard_site_v0.json) |
| **Staging spec** | [`kit_substation_yard_production_001.json`](../../assets/staging/specs/kit_substation_yard_production_001.json) |
| **Verdict** | **PASS** (spec ready — bpy batch pending) |

```text
DMCP-SPEC-SUBSTATION-YARD-001 Q✓
4×3 yard kit — grid_substation.json authority
```

---

## 1. AssetSpec summary

| Field | Value |
|:---|:---|
| `asset_id` | `kit_substation_yard_production_001` |
| `archetype` | `module_kit` |
| `style_pack` | `style_industrial_west` |
| `development_tier` | `production` |
| `grid_units` | **[4, 3]** |
| `snap` | `floor_edge` |
| `pivot` | `bottom_center` |
| `batch_id` | `kit_utility_power_production_001` |

---

## 2. Module composition

| Slot | Module | Grid | Count |
|:---|:---|:---|:---:|
| yard_primary | `bus_bay_simplified` | 2×1 | 2 |
| yard_primary | `breaker_block` | 1×1 | 2 |
| yard_utility | `gravel_pad_1u` | 1×1 | fill |
| yard_utility | `fence_chainlink_1u` | 1×1 | perimeter |
| yard_service | `control_shack_1u` | 1×1 | 1 |
| optional | `warning_sign_1u` | 1×1 | 2 |

**ARCH-DNA:** `yard_complex` weight 28% · zones per site pilot.

---

## 3. Materials

Per [`design_utility_industrial_style_v1.md`](../../src/dev/design_utility_industrial_style_v1.md) §6:

- `galvanized_steel_01` — bus, breaker housings
- `concrete_pad_01` — equipment bases
- `gravel_yard_01` — yard fill
- `warning_paint_yellow_01` — signs

---

## 4. MCP pipeline

| Step | Tool |
|:---|:---|
| Validate | `validate_asset_spec` |
| Generate | `blender_batch_modules` |
| Staging | `assets/staging/meshes/kit_substation_yard_production_001/` |
| Promote | registry row `grid_substation` |

**Witness:** `debug_runs/art_pipeline/dmcp_substation_yard_spec_live.json`

---

## 5. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (spec) | 2026-06-18 |
