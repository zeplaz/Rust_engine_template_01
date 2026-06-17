# APS file-wipe recovery — 2026-06-15

| Field | Value |
|:---|:---|
| **Incident** | Art Pipeline Suite dead — 14 `.py` files at 0 bytes on `HEAD` (`5a340510`) |
| **Verdict** | Accidental regression in bulk `waves` commit — **not** intentional deprecation |
| **Recovery** | Group A from `a982ff20`; Group B reconstructed from call sites + witness contracts |

## Root cause

Commit **`5a340510` ("waves")** is a ~20k-file monorepo churn with message only `waves`. In the same commit:

1. **`app.py` was upgraded** to import `aps_tooltips`, `job_controller`, job strip, pipeline status — new UX shell.
2. **Implementation files were truncated to 0 bytes** (`scrollable`, `variants_panel`, `grammar_inspector`, `assembly_preview_panel`, `pg_module_audit_002`, …).
3. **Never-before-committed modules were added as empty stubs** (`aps_tooltips`, `job_controller`, `aps_catalog_preview`, `aps_slot_preview`, `aps_mat_002`, `material_brief`, `material_studio_preview`).

Commit **`a982ff20` ("tools and wild refactor")** still had working bodies for Group A + `variant_set` / `assembly_build_worker`, but a simpler `app.py` that did not depend on tooltips/job controller.

**Why witnesses stayed green:** pytest/witness drivers assert JSON on disk or load functions in isolation — they did not gate on `import art_pipeline_suite.app`.

## Recovery inventory

| File | Method | Status |
|:---|:---|:---|
| `scrollable.py` | `git checkout a982ff20` | restored |
| `grammar_inspector.py` | `git checkout a982ff20` | restored |
| `assembly_preview_panel.py` | `git checkout a982ff20` | restored |
| `variants_panel.py` | `git checkout a982ff20` | restored |
| `pg_module_audit_002.py` | `git checkout a982ff20` | restored |
| `variant_set.py` | `git checkout a982ff20` | restored |
| `assembly_build_worker.py` | `git checkout a982ff20` | restored |
| `aps_tooltips.py` | reconstructed (call sites + UX review §7) | restored |
| `job_controller.py` | reconstructed (`test_aps_ux_async_001` contract) | restored |
| `aps_catalog_preview.py` | reconstructed | restored |
| `aps_slot_preview.py` | reconstructed | restored |
| `material_studio_preview.py` | reconstructed | restored |
| `material_brief.py` | reconstructed | restored |
| `aps_mat_002.py` | reconstructed | restored |

## Guards added

- `tools/mcp/python/tests/test_aps_imports.py` — import smoke; run in CI via existing APS pytest lanes.

## Honest witness refresh

After recovery, re-run:

```text
cd tools/mcp/python
python -m pytest tests/test_aps_imports.py tests/test_aps_ux_async_001.py tests/test_material_brief.py tests/test_material_studio.py -q
python -c "from rust_engine_mcp.aps_witness_refresh import refresh_aps_witnesses; print(refresh_aps_witnesses())"
```

Do **not** treat pre-wipe `green: true` JSON as authoritative until import + pytest pass.

## Policy

- HANDOFF still lists **Tk APS | maintain** — no deprecation was signed.
- Future bulk commits touching `tools/mcp/art_pipeline_suite/` must keep `test_aps_imports` green.
