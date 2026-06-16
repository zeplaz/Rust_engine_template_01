# @planner-mcp deliverables — 2026-06-03 session

| Priority | ID | Status | Output |
|:---:|:---|:---:|:---|
| 1 | **GRAMMAR-ITER-SNAPSHOT-001** | **done** | [`assembly_snapshot_v1.schema.json`](../../tools/mcp/schemas/assembly_snapshot_v1.schema.json) — `grammar_lineage`, `grammar_overrides` |
| 2 | **GRAMMAR-ITER-RESULT-001** | **done** | [`grammar_iterate_result_v1.schema.json`](../../tools/mcp/schemas/grammar_iterate_result_v1.schema.json) |
| 3 | **APS-VALIDATOR-PLAIN-001** | **done** | [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md) |
| 4 | **APS-MAT-CATEGORY-SCHEMA-001** | **done** | [`aps_mat_category_schema_001_v1.md`](aps_mat_category_schema_001_v1.md) + [`material_category_tree_v1.schema.json`](../../tools/mcp/schemas/material_category_tree_v1.schema.json) + [`material_category_tree_v1.json`](../../assets/materials/profiles/material_category_tree_v1.json) |
| 5 | **GRAMMAR-002-SLICE-001** | **done** | [`grammar_002_slice_001_v1.md`](grammar_002_slice_001_v1.md) |

---

## Next assign (@orchestrator)

| ID | Agent | Task |
|:---|:---|:---|
| GRAMMAR-ITER-001-API | @coder-mcp | Implement `iterate_grammar` against schemas |
| APS-VALIDATOR-PLAIN-002 | @coder-mcp | Wire plain language into P0 gate UI |
| APS-MAT-003 | @coder-mcp | Load `material_category_tree_v1.json` in Materials tab |
| GRAMMAR-002-SLICE G2S-2 | @coder | Rust roof/facade layers — after ITER massing |

---

## Index links

- [`grammar_iter_001_spec_v1.md`](grammar_iter_001_spec_v1.md)
- [`development_plan_index.md`](development_plan_index.md)
