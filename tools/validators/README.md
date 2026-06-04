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

## MCP tools

`validate_cargo_report`, `validate_bevy_report`, `validate_asset_report`, `validate_report`

## Schema + knowledge

- `schemas/validation_report_v1.schema.json`
- `knowledge/error_signatures.json`

## Plan

`src/dev/plan_validation_runtime_v1.md`
