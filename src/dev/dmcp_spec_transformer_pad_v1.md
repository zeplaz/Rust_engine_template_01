# DMCP spec — transformer pad `v1`

| Field | Value |
|:---|:---|
| **ID** | **DMCP-SPEC-TRANSFORMER-PAD-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer-mcp` |
| **Depends** | **DES-ART-UTILITY-STYLE-001** PASS |
| **Catalog** | [`grid_distribution_transformer.json`](../../assets/configs/buildings/grid_distribution_transformer.json) |
| **Staging spec** | [`prop_transformer_production_run001.json`](../../assets/staging/specs/prop_transformer_production_run001.json) |
| **Verdict** | **PASS** (spec ready — bpy batch pending) |

```text
DMCP-SPEC-TRANSFORMER-PAD-001 Q✓
2×2 pad prop — grid_distribution_transformer.json authority
```

---

## 1. AssetSpec summary

| Field | Value |
|:---|:---|
| `asset_id` | `prop_transformer_production_run001` |
| `archetype` | `module_prop` |
| `style_pack` | `style_industrial_west` |
| `development_tier` | `production` |
| `grid_units` | **[2, 2]** |
| `snap` | `floor_center` |
| `pivot` | `bottom_center` |
| `batch_id` | `kit_utility_power_production_001` |

**Supersedes:** lod0 stub [`prop_transformer.json`](../../assets/staging/specs/prop_transformer.json).

---

## 2. Geometry brief

| Element | Spec |
|:---|:---|
| Tank | Horizontal cylinder, W×H×D ≈ 1.8 × 1.2 × 1.2 m within 2×2 tile |
| Bushings | **3** ceramic caps on top — white read @ 32px |
| Pad | Concrete slab + 0.5 tile gravel margin |
| Berm | Optional 0.1 m lip — oil containment hint |

**Silhouette:** cylinder + three dots — matches `node_transformer` glyph.

---

## 3. Materials

- `galvanized_steel_01` — tank body
- `ceramic_insulator_01` — bushings
- `concrete_pad_01` — footing
- `gravel_yard_01` — margin

---

## 4. MCP pipeline

| Step | Tool |
|:---|:---|
| Validate | `validate_asset_spec` |
| Generate | `blender_batch_modules` |
| Staging | `assets/staging/meshes/prop_transformer_production_run001/` |
| Promote | registry row `grid_distribution_transformer` |

**Witness:** `debug_runs/art_pipeline/dmcp_transformer_pad_spec_live.json`

---

## 5. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (spec) | 2026-06-18 |
