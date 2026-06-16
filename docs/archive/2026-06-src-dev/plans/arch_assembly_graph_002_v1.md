# ARCH-ASSEMBLY-GRAPH-002 — Semantic assembly nodes `v1`

| Field | Value |
|:---|:---|
| **ID** | ARCH-ASSEMBLY-GRAPH-002 |
| **Status** | **done** |
| **Schema** | [`tools/mcp/schemas/assembly_graph_node_v1.schema.json`](../../tools/mcp/schemas/assembly_graph_node_v1.schema.json) |
| **Snapshot** | [`tools/mcp/schemas/assembly_snapshot_v1.schema.json`](../../tools/mcp/schemas/assembly_snapshot_v1.schema.json) |
| **Tags** | [`aps_tags_001_v1.md`](aps_tags_001_v1.md) |

---

## Node fields (beyond ARCH-003 geometry)

| Field | Type | Required ship | Meaning |
|:---|:---|:---|:---|
| `node_id` | string | recommended | Stable `{module_id}_{gx}_{gy}_f{floor}` |
| `role` | enum | recommended | Semantic slot: `primary_wall`, `corner`, `door`, `roof`, `window_band`, `detail_prop`, … |
| `module_id` / `job_id` / `glb_path` | — | yes | Geometry |
| `material_profile` | string | **yes** (ship) | PBR profile id |
| `style` | string | optional | Style pack slice / facade id (e.g. `industrial_west`) |
| `weathering` | enum | optional | `clean` \| `light` \| `medium` \| `heavy` |
| `semantic_tags` | object | recommended | APS-TAGS categories — see taxonomy schema |
| `placement_tags` | string[] | legacy flat | Deduped flatten of `semantic_tags` until APS-TAGS-002 |
| `variant_tags` | string[] | optional | Variant graph hints |
| `lod_policy` | enum | optional | `lod0` \| `production` \| `hero` |

### `role` enum (v1)

`primary_wall`, `secondary_wall`, `corner`, `door`, `window`, `roof`, `parapet`, `stack`, `vent`, `platform`, `sign`, `detail_prop`

Token → default role: `W` → `primary_wall`, `C` → `corner`, `D` → `door`, `R` → `roof`.

---

## Snapshot-level grammar metadata

Optional on `assembly_snapshot_v1`:

| Field | Purpose |
|:---|:---|
| `archetype_id` | e.g. `industrial_warehouse` |
| `district_style` | e.g. `industrial_west` |
| `grammar_rule_chain` | Inspector + PG-QUALITY — same shape as `GrammarGenerateResult.rule_chain` |

---

## Example placement

```json
{
  "node_id": "wall_steel_1u_1_0_f0",
  "role": "primary_wall",
  "module_id": "wall_steel_1u",
  "material_profile": "steel_panel_01",
  "style": "industrial_west",
  "weathering": "medium",
  "semantic_tags": {
    "location": ["street_facing"],
    "architectural": ["industrial"],
    "detail": [],
    "condition": ["weathered"]
  },
  "placement_tags": ["street_facing", "industrial", "weathered"],
  "slot_key": "wall_1u",
  "token": "W",
  "grid_x": 1,
  "grid_y": 0,
  "floor": 0
}
```

---

## Next

| ID | Owner |
|:---|:---|
| APS-TAGS-002 | @coder-mcp — populate `semantic_tags` from APS + grammar |
| CODER-SNAPSHOT-GRAMMAR-WIRE | @coder — persist `grammar_rule_chain` on snapshot |
