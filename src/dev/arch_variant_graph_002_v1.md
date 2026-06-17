# ARCH-002 — Variant graph schema `v1`

| Field | Value |
|:---|:---|
| **ID** | ARCH-002 |
| **Status** | **SIGNED** (@planner-mcp 2026-06-14) |
| **Schema** | [`tools/mcp/schemas/variant_graph_v1.schema.json`](../../tools/mcp/schemas/variant_graph_v1.schema.json) |
| **Example** | [`tools/mcp/schemas/examples/variant_graph_warehouse_industrial_west_v1.json`](../../tools/mcp/schemas/examples/variant_graph_warehouse_industrial_west_v1.json) |
| **Pairs with** | [`variant_set_v1.schema.json`](../../tools/mcp/schemas/variant_set_v1.schema.json) · [`assembly_graph_node_v1.schema.json`](../../tools/mcp/schemas/assembly_graph_node_v1.schema.json) |

---

## Authority split

| Artifact | Granularity | When |
|:---|:---|:---|
| `variant_set_v1` | Variant-level **layers** (lighting, damage, fill) | APS Variants tab · tile bake matrix |
| `variant_graph_v1` | **Per-node** overrides on assembly graph | Headless compile · role/node targeted patches |

```text
assembly_snapshot_v1
      ▼
variant_set_v1 (variant_key + layers)
      ▼
variant_graph_v1 (node patches per variant_key)
      ▼
variant_bake / tile_batch_run
```

---

## VariantNode patch (schema)

Each `variant_node_patch` may set any subset of:

| Layer | Fields |
|:---|:---|
| `material_overrides` | `material_profile`, `weathering`, `roughness_multiplier`, `metallic_multiplier`, `color_tint` |
| `visibility_overrides` | `visible`, `hide_children`, `opacity` |
| `emission_overrides` | `emissive_strength`, `emissive_color`, `night_lights`, `color_temperature_k` |
| `decal_overrides[]` | `decal_id`, `layer`, `strength`, `uv_scale` |

**Target resolution:** `node` (by `node_id`) · `role` (all placements with role) · `semantic_tag` (flattened tag match).

**Inheritance:** `inherits` variant_key — patches apply on top of inherited variant (compile expands chain).

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-14 | **SIGNED** — schema + warehouse example validate |

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-14 | **SIGNED** — unblocks BUILD-001 / variant-aware bakes |
