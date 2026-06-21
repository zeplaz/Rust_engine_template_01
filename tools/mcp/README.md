# Rust Engine — Designer art MCP toolchain

**Onboarding:** [`docs/archive/2026-06-src-dev/plans/designer_mcp_onboarding_v1.md`](../../docs/archive/2026-06-src-dev/plans/designer_mcp_onboarding_v1.md)  
**Micro tools list:** [`MICRO_TOOLS_REGISTRY_v1.md`](MICRO_TOOLS_REGISTRY_v1.md)  
**Exec plan:** [`docs/archive/2026-06-src-dev/plans/plan_designer_mcp_art_toolchain_exec_001_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_designer_mcp_art_toolchain_exec_001_v1.md)  
**Inbound alignment:** [`docs/archive/2026-06-src-dev/plans/plan_art_design_inbound_alignment_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_art_design_inbound_alignment_v1.md) · [`docs/reference/user/art_design_inbound.md`](../../docs/reference/user/art_design_inbound.md)

## Quick start (Windows)

```powershell
.\install_designer_mcp.ps1
```

That script: pip install → writes **`~/.cursor/mcp.json`** + **`~/.cursor/rust_engine_art_mcp.env`** → smoke geometry → verify.

| Setting | Value |
|:---|:---|
| **Blender** | `C:\Program Files (x86)\Steam\steamapps\common\Blender\blender.exe` (Steam 5.1) |
| **Python (MCP)** | `C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe` — **not** default `python` (often 3.14 without deps) |
| **Repo** | `C:\dev\github\Rust_engine_template_01` |
| **User config** | `~/.cursor/mcp.json` · `~/.cursor/rust_engine_art_mcp.env` |
| **Verify** | `.\scripts\verify_mcp_setup.ps1` |

## Philosophy

**Micro CLI first** — same code as MCP tools; LLM only orchestrates.

```powershell
cd tools\mcp\python
python -m rust_engine_mcp.cli run-geometry ..\schemas\examples\wall_job.example.json
python -m rust_engine_mcp.cli validate-glb ..\..\..\assets\staging\wall_brick_1u_example\model.glb
python -m rust_engine_mcp.cli promote wall_brick_1u_example
```

## MCP tools (Cursor — use `@designer-mcp`)

Full list: [`MICRO_TOOLS_REGISTRY_v1.md`](MICRO_TOOLS_REGISTRY_v1.md). Agents must use **validation-first** reports (`validate_report`, `validate_asset_report`) — not raw CLI logs.

| Tool | Purpose |
|:---|:---|
| `ping` / `locate_blender` | Health + Blender path |
| `spec_write` / `spec_validate` | AssetSpec JSON |
| `geometry_operations` | List bpy op ids (wall/roof/door/window/prop) |
| `geometry_run_job` / `geometry_job_status` | Blender headless + status file |
| `validate_glb_asset` / `validate_asset_report` | GLB checks (structured report preferred) |
| `validate_report` | `mcp_spec` / `mcp_job` schema gates |
| `list_staging` | Staging folders before G4 sign-off |
| `promote_staging_module` | → `assets/models/modules/` (auto `library_register`) |
| `library_register` / `library_search` | G5 `_module_index.ron` + JSON mirror |
| `write_witness` | `debug_runs/art_pipeline/<batch>_live.json` |
| `micro_tool_help` | CLI command list (same code path) |

## Schema form — the signature book (token lever)

Tool **schemas injected per request are ~96% of the always-on token budget**; publishing them as a
one-line **signature book** cuts that **−92% at 100% callability** (CB-notation research report,
`§13 / W1`). Form + the **SACRED exact-name rule** (never strip the tool name → 12% callable) live in
[`MICRO_TOOLS_REGISTRY_v1.md` §Schema form](MICRO_TOOLS_REGISTRY_v1.md). Encode tool **results** as
`●◐○` vectors (−73% vs JSON) and deep diagnoses as `HYP/EV/INFER + ρ`
(`$ref:prompts/SYMBOLIC_LANGUAGE.meta.md §3.12`). The −92% is realised **server-side** (the MCP
server emits the book); gate on **callability ≥95%**, reported separately from token-Δ.

## Layout

See repo `tools/mcp/` — schemas, blender scripts, python package, job status under `tools/mcp/jobs/`.

## Status

| Component | Status |
|:---|:---|
| MCP server + CLI | **SHIPPED** |
| Blender wall/roof/door/window/prop ops | **SHIPPED** (`scripts/smoke_geometry.ps1`) |
| Promotion + auto `library_register` | **SHIPPED** |
| `_module_index.ron` + JSON mirror (G5) | **SHIPPED** |
| `write-witness` / batch live JSON | **SHIPPED** |
| Module Kit Viewer | **SHIPPED** — `tools/mcp/scripts/open_module_viewer.cmd` (or `.ps1` if execution policy allows) |
| `tile.generate` execution | **PLANNED** (draft specs only) |
| `art_validator` Rust grid CLI | **PLANNED** |
| Material Maker / gltf-transform | **PLANNED** (Tier 3) |
