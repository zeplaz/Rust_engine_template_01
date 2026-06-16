# APS-MAT-CATEGORY-SCHEMA-001 — Material category tree (300+ profiles) `v1`

| Field | Value |
|:---|:---|
| **ID** | **APS-MAT-CATEGORY-SCHEMA-001** |
| **Owner** | @planner-mcp |
| **Implement** | @coder-mcp (`material_library_widget.py` studio tree) |
| **Status** | **done** (schema + seed tree) |
| **Date** | 2026-06-03 |

---

## Problem

Flat `industrial/steel` slash paths and combobox filters **do not scale** to 300 material profiles. Artists need:

```text
Industrial
  Steel
  Corrugated
  Concrete
Residential
  Brick
  Plaster
```

---

## Artifacts

| File | Role |
|:---|:---|
| [`material_category_tree_v1.schema.json`](../../tools/mcp/schemas/material_category_tree_v1.schema.json) | JSON Schema |
| [`material_category_tree_v1.json`](../../assets/materials/profiles/material_category_tree_v1.json) | Seed tree + infer rules + pilot bindings |

---

## Resolution algorithm (coder-mcp)

For each `profile_id` in catalog:

1. **Explicit binding** in `profile_bindings[]` — wins  
2. **`infer_rules[]`** — highest `priority` match on profile_id substring (case-insensitive)  
3. **Fallback** — existing `infer_category()` in `material_profiles.py`  
4. **Default** — `other`

**Category path** = slash-separated ids from root, e.g. `industrial/steel`.

---

## APS tree UI contract

| UI | Data |
|:---|:---|
| Left tree | `roots[]` nested `children` — 2–3 levels max |
| Profile list | Filtered by selected node path (prefix match) |
| Count badges | `Industrial (42)` — count profiles under subtree |
| Unsorted bucket | `other` root catches unmatched profiles |

**Scale target:** 300 profiles, list virtualized or paginated; tree stays &lt;30 nodes.

---

## Registry integration (future)

`material_profiles_v1.json` may add optional field per row:

```json
{ "profile_id": "steel_panel_01", "category_path": "industrial/steel" }
```

Promotion from APS **Add profile** writes binding into tree JSON or registry — Phase 2 (`APS-MAT-003`).

---

## Acceptance

| Criterion | Proof |
|:---|:---|
| Schema validates seed tree | `validate_json` in tests |
| All PILOT_PROFILES resolve to a path | coder-mcp unit test |
| Tree depth ≤ `max_depth` | schema constraint |

---

## Queue

| ID | Owner | Task |
|:---|:---|:---|
| APS-MAT-CATEGORY-SCHEMA-001 | @planner-mcp | **done** — this doc + schema + seed |
| APS-MAT-003 | @coder-mcp | Wire tree loader into Materials tab |
