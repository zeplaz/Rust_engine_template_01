# Designer MCP onboarding — first session with `@designer`

| Step | Action |
|:---:|:---|
| 1 | **Restart Cursor** after MCP config update (`~/.cursor/mcp.json` has `rust-engine-art`) |
| 2 | Settings → MCP → confirm **rust-engine-art** is green |
| 3 | Terminal: `cd tools/mcp/python && pip install -r ../requirements.txt && pip install -e .` |
| 4 | `python -m rust_engine_mcp.cli locate-blender` → Steam Blender 5.1 path |
| 5 | New chat: **`@designer-mcp`** + run MCP **`ping`** (not `@designer` for asset jobs) |

## Designer-mcp workflow (G0–G5)

```text
G0  rules audit (order_critique YAML in chat)
G1  spec_validate / validate_report(mcp_spec|mcp_job)
G2  geometry_run_job → geometry_job_status
G3  validate_asset_report (preferred) or validate_glb_asset
G4  staging review → sign-off YAML → promote_staging_module
G5  library_search → write_witness(batch_id)
```

**Agent:** `@designer-mcp` for all `tools/mcp/` work. **`@designer`** = HUD/overlay only.

**Do in terminal instead of chat** (same code, zero tokens):

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m rust_engine_mcp.cli validate-spec ..\..\..\assets\staging\specs\wall_brick_1u.example.json
python -m rust_engine_mcp.cli run-geometry ..\schemas\examples\wall_job.example.json
python -m rust_engine_mcp.cli validate-glb ..\..\..\assets\staging\wall_brick_1u_example\model.glb
python -m rust_engine_mcp.cli promote wall_brick_1u_example
```

## Example `@designer` prompt

> Create an AssetSpec for `wall_concrete_2u` (4m×3m×0.3m, industrial west style), write it with **spec_write**, build a geometry job JSON, run **geometry_run_job**, validate, and report issues. Do not describe mesh steps in prose — use tools only.

## Tool list

See [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md).

## Blender

**Path:** `C:\Program Files (x86)\Steam\steamapps\common\Blender\blender.exe` (Steam 5.1)

Override: `tools/mcp/config.local.json` or env `BLENDER_EXE`.
