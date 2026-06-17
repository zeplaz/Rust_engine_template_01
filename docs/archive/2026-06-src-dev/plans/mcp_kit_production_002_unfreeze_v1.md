# MCP-P2-KIT002-PLAN — `kit_production_002` unfreeze `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ MCP-P2-KIT002-PLAN
Date: 2026-06-14
Status: **SIGNED** (@planner-mcp 2026-06-14)
Parent: $ref:src/dev/plan_module_kit_production_tier_v1.md
Mirror: $ref:docs/archive/2026-06-src-dev/plans/mcp_fleet_production_pilot_rowhouse_v1.md
Queue: $ref:tools/orchestrator/queues/mcp_active_queue.json#MCP-P2-KIT002-PLAN
```

**Headline:** `kit_production_001` (rowhouse) is **closed**. `kit_production_002` unfreezes the **second production archetype** — **warehouse industrial west** — under the same tier contract. Sidecar batch `kit_industrial_west_production_001` is **absorbed**, not reopened.

---

## 1. Scope

| In | Out |
|:---|:---|
| `kit_production_002` manifest + G0–G5 gate table | `kit_greybox_004+` |
| `tile_batch_warehouse_industrial_west_production_v1` (paired) | shopfront / bunker batches (still frozen) |
| `variant_matrix_warehouse_v1` authoring | Postgres ops |
| Reuse promoted rows from `kit_industrial_west_production_001` | Re-run rowhouse bpy |

---

## 2. Unfreeze criteria (all required)

| # | Criterion | Witness |
|:---:|:---|:---|
| U1 | This plan **SIGNED** | This file |
| U2 | `MCP-PROD-SPRINT-ROWHOUSE-001` **CLOSED** | `debug_runs/art_pipeline/rowhouse_production_atlas_g0_g4_live.json` |
| U3 | `tile_promotion_honest_check` **SHIPPED** | `debug_runs/mcp_p2_honest_bake_001_live.json` |
| U4 | **ARCH-002** variant graph schema on disk | `tools/mcp/schemas/variant_graph_v1.schema.json` |
| U5 | Warehouse `variant_set` validates | `variant_set_warehouse_industrial_west_production_v1.json` |
| U6 | `@designer-mcp` **G0** rules YAML for warehouse production | `debug_runs/art_pipeline/warehouse_production_g0_rules.yaml` (next slice) |
| U7 | Manifest modules: `development_tier: production`, `pbr_status: shipped` at promote | `batch_kit_production_002.manifest.json` |
| U8 | **No** greybox ortho / dry-run as ship | `validate-report tile_promotion_honest` on batch |

**Hard rule:** bpy for net-new modules starts only after **U6** (designer G0). Promoted index rows from `kit_industrial_west_production_001` may be referenced without re-bpy.

---

## 3. Archetype #2 module set

Pilot: `warehouse_industrial` · `style_industrial_west` · grammar `industrial_warehouse_v1`

| module_id | Slot | material_profile | Notes |
|:---|:---|:---|:---|
| `wall_steel_1u` | W bays | `steel_panel_01` | P0 — may reference existing production GLB |
| `door_warehouse` | D ground | `steel_door_warehouse_01` | P0 |
| `win_industrial_3u` | window band | `steel_window_01` | P0 — canonical; not `window_industrial_1u` |
| `corner_L` | C corners | `steel_corner_01` | P1 — reuse `kit_industrial_west_production_001` row |
| `roof_industrial_shed_2u` | R plane | `metal_roof_01` | P1 |
| `stack_chimney_1u` | stack / vent | `metal_stack_01` | P2 — detail |

Manifest sketch: `$ref:tools/mcp/schemas/examples/batch_kit_production_002.manifest.json`

---

## 4. G0–G5 gate table

| Gate | Owner | Exit |
|:---|:---|:---|
| **G0** | @designer-mcp | Warehouse production rules YAML — silhouette + PBR ids + ¬greybox ship |
| **G1** | @designer-mcp | AssetSpec per module — `development_tier: production` |
| **G2** | @coder-mcp | bpy profiles match archetype (wall/door/window/roof/stack) |
| **G3** | @coder-mcp | `validate_asset_report` tier pass + honest bake |
| **G4** | @designer-mcp | Keyframe stills on warehouse variant matrix (readable art) |
| **G5** | @coder-mcp | Index + `tile_batch_warehouse_industrial_west_production_v1` register |

---

## 5. Frozen → unfrozen (queue)

After **U1–U5** (this sign + ARCH-002):

| Class | Item |
|:---|:---|
| **Unfrozen** | `kit_production_002` · `tile_batch_warehouse_industrial_west_production_v1` |
| **Still frozen** | `kit_production_003+` · shopfront/bunker batches · `variant_matrix_shopfront_v1` · `variant_matrix_bunker_v1` |

---

## 6. Agent routing

| Agent | Next |
|:---|:---|
| **@designer-mcp** | `MCP-P2-KIT002-G0` — warehouse G0 rules before bpy |
| **@coder-mcp** | Manifest-driven promote + tile batch (after G0) |
| **@planner-mcp** | **ARCH-002** (parallel — done same session) |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-14 | **SIGNED** — unfreeze criteria + manifest sketch authoritative |

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-14 | **SIGNED** — unblocks designer G0 + warehouse production lane |

```text
⟦/MCP-P2-KIT002-PLAN⟧  ΔWF→@designer-mcp ⟨MCP-P2-KIT002-G0⟩
```
