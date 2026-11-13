# Rust Engine — art toolchain (MCP server, CLI, Art Pipeline Suite)

Three front ends over one code path in `python/rust_engine_mcp/`:

| Tool | Launch | Purpose |
|:---|:---|:---|
| Art Pipeline Suite (APS) | `python tools\mcp\art_pipeline_suite\run.py` (or `scripts\open_module_viewer.cmd`) | Tk desktop studio: Catalog / Materials / Effects / Assembly / Variants / Atlas (buildings), Presets / Grammar / States / Atlas (landscape) |
| CLI | `cd tools\mcp\python; python -m rust_engine_mcp.cli commands` | Deterministic steps: spec, Blender geometry, GLB validation, promote, library, tile/atlas batches, variants, effects |
| MCP server | `scripts\run_rust_engine_mcp.cmd` (stdio; wired by `install_designer_mcp.ps1` / `write_cursor_mcp_config.ps1`) | Same operations as MCP tools for agents (`rust-engine-art`) |

`module_viewer/` is the older name of APS (its `run.py` launches the same app; its `model_store.py` etc. are still imported by APS).
`blender/` holds the headless bpy scripts the CLI invokes (`blender/headless_run.ps1`).

## Setup (Windows)

```powershell
python -m pip install -r tools\mcp\requirements.txt
python -m pip install -e tools\mcp\python
.\tools\mcp\scripts\verify_mcp_setup.ps1     # checks python, blender, mcp.json
.\tools\mcp\scripts\smoke_geometry.ps1       # real Blender run: wall job -> GLB
```

Blender path comes from `config.local.json` (copy `config.local.json.example`), `BLENDER_EXE`, or the Steam default in `config.defaults.json`.

## Typical flow

```powershell
cd tools\mcp\python
python -m rust_engine_mcp.cli run-geometry ..\schemas\examples\wall_job.example.json
python -m rust_engine_mcp.cli validate-glb ..\..\..\assets\staging\wall_brick_1u_example\model.glb
python -m rust_engine_mcp.cli promote wall_brick_1u_example
```

`python -m rust_engine_mcp.cli commands --grep tile` lists commands; `help <command>` shows arguments.
Job inputs/status are written to `tools/mcp/jobs/` (gitignored). Tests: `cd tools\mcp\python; python -m pytest -q`.
Review of the running game goes through `tools/sim_review/`, not this toolchain.
