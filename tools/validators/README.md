# Validators — structured reports for agents

Agents consume **ValidationReport JSON**, not raw logs.

## CLI

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli validate-report cargo --compress 3
python -m rust_engine_mcp.cli validate-report cargo --cached
python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01
python -m rust_engine_mcp.cli validate-report mcp_job ..\schemas\examples\wall_job.example.json
python -m rust_engine_mcp.cli validate-report asset_glb ..\..\..\assets\models\modules\wall_concrete_2u_run001\model.glb
```

### Studio DCC — ParamFrame · massing · grammar (PLAN v3 S5)

Agents validate Studio payloads with **ValidationReport JSON** only (IO-09 · ECS-15 · PCG-06 / legacy `asset_pipeline` · `shape_grammars`).

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli validate-report param_frame tools/mcp/schemas/examples/param_frame_rail_warehouse_v1.json --compress 3
python -m rust_engine_mcp.cli validate-report massing_preview tools/mcp/schemas/examples/massing_preview_pilot_rail_warehouse_v1.json --compress 3
python -m rust_engine_mcp.cli validate-report building_grammar tools/mcp/schemas/examples/building_grammar_industrial_warehouse_v1.json --compress 3
```

MCP: `validate_report(validator="param_frame"|"massing_preview"|"massing"|"building_grammar", target=<path>)`.

Aliases: `massing` → same validator as `massing_preview` (voxel presentation path in scope).

## MCP tools

`validate_cargo_report`, `validate_bevy_report`, `validate_asset_report`, `validate_report`

## Schema + knowledge

- `schemas/validation_report_v1.schema.json`
- `knowledge/error_signatures.json`

## Plan

`docs/archive/2026-06-src-dev/plans/plan_validation_runtime_v1.md`

## Book-upgrade static checks (PLAN-REF-BOOKS-UPGRADE-v2)

```powershell
python tools/validators/sole_writer_check.py      # E1 — ResMut outside owner prefix (report-only)
python tools/validators/determinism_lint.py       # E4 — rng / wall-clock / HashMap iter in sim dirs
```

Owners: `tools/validators/knowledge/sole_writers.tsv`.
