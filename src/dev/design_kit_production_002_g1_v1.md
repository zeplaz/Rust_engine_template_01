# kit_production_002 — G1 production AssetSpec per module `v1`

| Field | Value |
|:---|:---|
| **Gate** | **MCP-P2-KIT002-G1** |
| **Batch** | `kit_production_002` |
| **Owner** | `@designer-mcp` |
| **Verdict** | **PASS** |
| **Plan** | [`mcp_kit_production_002_unfreeze_v1.md`](../docs/archive/2026-06-src-dev/plans/mcp_kit_production_002_unfreeze_v1.md) |
| **G0** | [`warehouse_production_g0_rules.yaml`](../debug_runs/art_pipeline/warehouse_production_g0_rules.yaml) |
| **Index** | [`kit_production_002_assetspec_index.json`](../assets/staging/specs/kit_production_002_assetspec_index.json) |
| **Witness** | [`kit_production_002_g1_live.json`](../debug_runs/art_pipeline/kit_production_002_g1_live.json) |

**MCP lane:** AssetSpec JSON on disk only — bpy already run for `roof_industrial_shed_2u` (G2 ★).

---

## G1 exit criteria

| # | Criterion | Pass |
|:---:|:---|:---:|
| 1 | One `AssetSpec` per manifest module (6/6) | ✓ |
| 2 | `development_tier: production` + `pbr_status: shipped` | ✓ |
| 3 | `batch_id: kit_production_002` on all specs | ✓ |
| 4 | `material_profile` matches manifest PBR ids | ✓ |
| 5 | `validate-report mcp_spec` green per file | ✓ |
| 6 | `ref:gate:MCP-P2-KIT002-G1` on each spec | ✓ |

---

## Module table

| module_id | Spec | material_profile | job_id | Reuse |
|:---|:---|:---|:---|:---:|
| `wall_steel_1u` | `wall_steel_1u_production.json` | `steel_panel_01` | `wall_steel_1u_production_run001` | — |
| `door_warehouse` | `door_warehouse_production.json` | `steel_door_warehouse_01` | `door_warehouse_production_run001` | kit001 |
| `win_industrial_3u` | `win_industrial_3u_production.json` | `steel_window_01` | `win_industrial_3u_production_run001` | kit001 |
| `corner_L` | `corner_L_industrial_west_production.json` | `steel_corner_01` | `corner_L_industrial_west_production_run001` | kit001 |
| `roof_industrial_shed_2u` | `roof_industrial_shed_2u_production.json` | `metal_roof_01` | `roof_industrial_shed_2u_production_run001` | G2 bpy |
| `stack_chimney_1u` | `stack_chimney_1u_production.json` | `metal_stack_01` | `stack_chimney_1u_production_run001` | kit001 |

**G1 fix:** `win_industrial_3u` material `glass_panel_01` → `steel_window_01` (manifest authority).  
**G1 fix:** `stack_chimney_1u` material `steel_panel_01` → `metal_stack_01`.

---

## Production rules (G0 carry-forward)

```yaml
rules_check:
  passed: true
  blocked_by: []
  no_ai_generated_images: true
  deterministic_output: true
  tier_production_pbr_shipped: true
  proceed_tile_ship: no  # TILE-FIX-001 + G4 keyframes
```

---

## Handoff

| Slice | Owner | Do |
|:---|:---|:---|
| **MCP-P2-KIT002-G3** | @coder-mcp | `validate_asset_report` tier pass all 6 GLBs |
| **MCP-P2-KIT002-G4** | @designer-mcp | Keyframe G4 on warehouse variant matrix |
| **MCP-P2-KIT002-G5** | @coder-mcp | Index + tile batch register |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-14 |
| `@coder-mcp` | pending G3 | — |

```text
MCP-P2-KIT002-G1 complete
6/6 AssetSpecs on disk · mcp_spec green · unblocks G3
```
