# APS-TAGS-001 — Categorized semantic tags `v1`

| Field | Value |
|:---|:---|
| **ID** | APS-TAGS-001 |
| **Status** | **done** |
| **Schema** | [`tools/mcp/schemas/aps_tag_taxonomy_v1.schema.json`](../../tools/mcp/schemas/aps_tag_taxonomy_v1.schema.json) |
| **Example** | [`tools/mcp/schemas/examples/aps_tag_taxonomy_v1.json`](../../tools/mcp/schemas/examples/aps_tag_taxonomy_v1.json) |
| **Parent** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |

---

## Categories (planner §824+)

| Category | Purpose | Example ids |
|:---|:---|:---|
| **location** | Footprint / street relation | `street_facing`, `corner`, `rear`, `interior` |
| **architectural** | Style family | `industrial`, `commercial`, `residential` |
| **detail** | Equipment / ornament | `stack`, `ventilation`, `loading_dock`, `pipework` |
| **condition** | Age / damage → variant graph | `clean`, `weathered`, `damaged`, `abandoned` |

Grammar rules **filter and assign** by category — not a single flat string list.

---

## On assembly nodes (ARCH-ASSEMBLY-GRAPH-002)

Preferred field:

```json
"semantic_tags": {
  "location": ["street_facing"],
  "architectural": ["industrial"],
  "detail": ["stack"],
  "condition": ["weathered"]
}
```

**Legacy:** `placement_tags` remains a **flat** deduped list for validators and MCP until APS-TAGS-002 ships. Writers should populate both: flat = flatten(`semantic_tags`).

---

## Map from APS-UI flat checkboxes (today)

Current [`assembly_panel.py`](../../tools/mcp/art_pipeline_suite/assembly_panel.py) uses `assembly.COMMON_PLACEMENT_TAGS`:

| Flat checkbox | → category | → tag_id |
|:---|:---|:---|
| `exterior` | location | `street_facing` |
| `interior` | location | `interior` |
| `wall` | architectural | `industrial` |
| `door` | detail | `door_rollup` |
| `corner` | location | `corner` |
| `roof` | architectural | `industrial` |
| `industrial` | architectural | `industrial` |
| `weathered` | condition | `weathered` |
| `clean` | condition | `clean` |
| `damaged` | condition | `damaged` |
| `night` | condition | `clean` *(lighting lives in variant_set)* |

**Variant tags** (`clean`, `damaged`, `night`, `construction`, `fire`) → map to **condition** + variant_set layers in APS-TAGS-002.

---

## Grammar use

| Grammar layer | Reads |
|:---|:---|
| `facade` | `location.street_facing`, `architectural.*` |
| `detail` | `detail.*` density |
| `age` | `condition.*` → `variant_tags` |

See [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md) `placement_tags` / `rule_chain`.

---

## Next

| ID | Owner |
|:---|:---|
| APS-TAGS-002 | @coder-mcp — APS categorized UI + `semantic_tags` read/write |
| APS-GRAMMAR-INSPECTOR-001 | @coder-mcp — show tags applied per rule step |
