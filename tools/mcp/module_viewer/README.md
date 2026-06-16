# Art Pipeline Suite (Module Catalog workspace)

**Evolving from “Module Kit Viewer”** into a four-stage desktop suite: **Catalog → Assembly → Variants → Atlas**.

Design: [`docs/archive/2026-06-src-dev/plans/design_art_pipeline_suite_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_art_pipeline_suite_v1.md) · Exec: [`docs/archive/2026-06-src-dev/plans/plan_art_pipeline_suite_exec_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_art_pipeline_suite_exec_v1.md)

**Today:** **Art Pipeline Suite** — tabs **Catalog | Assembly | Variants | Atlas** + flow buttons (Send to Assembly, Bake variants, Pack atlas). All actions call `rust_engine_mcp` CLI/MCP.

**Launch (preferred):**

```powershell
python C:\dev\github\Rust_engine_template_01\tools\mcp\art_pipeline_suite\run.py
```

**Legacy shim (same app):**

```powershell
python C:\dev\github\Rust_engine_template_01\tools\mcp\module_viewer\run.py
```

From repo root (use **`.cmd`** if PowerShell blocks scripts — `ExecutionPolicy`):

```cmd
tools\mcp\scripts\open_module_viewer.cmd
```

PowerShell (if scripts allowed, or bypass once):

```powershell
.\tools\mcp\scripts\open_module_viewer.ps1
# Or: powershell -ExecutionPolicy Bypass -File .\tools\mcp\scripts\open_module_viewer.ps1
```

**After `pip install -e tools/mcp/python`:**

```powershell
module-viewer
```

**Manual `-m` module (PYTHONPATH must be parent `tools/mcp`, not `module_viewer` itself):**

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
$env:PYTHONPATH = ".."
$env:RUST_ENGINE_REPO = "C:\dev\github\Rust_engine_template_01"
C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe -m module_viewer
```

## What it does

| Action | Description |
|--------|-------------|
| **Browse** | Lists modules from `assets/configs/buildings/_module_index.json` |
| **Filter** | By batch (`kit_greybox_001`, `kit_greybox_002`, …) and category |
| **Validate GLB** | Same checks as MCP `validate_glb_asset` |
| **Edit metadata** | JSON editor for `*.module.json` AssetSpec sidecar → **Save** |
| **Reindex** | Rebuilds `_module_index.ron` after metadata edits |
| **Preview in browser** | **Recommended** — local HTTP + Google model-viewer (materials, orbit/zoom) |
| **Open in Blender** | Imports GLB via glTF import script (not raw `.glb` argv) |
| **3D preview (trimesh)** | Optional — must install for **Python 3.13** (see below) |
| **Open GLB (OS app)** | Legacy Windows 3D Viewer — often broken/outdated; avoid |

## Requirements

- **Python 3.13** with `rust_engine_mcp` + `jsonschema` (parent MCP deps — not only `module_viewer/requirements.txt` trimesh extras):

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe -m pip install -r ..\requirements.txt
C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe -m pip install -e .
```

Or one-shot: `.\tools\mcp\install_designer_mcp.ps1` (same env as Cursor MCP).

**`ModuleNotFoundError: jsonschema`** — you launched with a bare `python` (often 3.14) that never got MCP requirements. Use 3.13 + commands above, or `tools\mcp\scripts\open_module_viewer.cmd` (installs deps then opens viewer).
- tkinter (included with Windows Python)
- Optional **3D preview (trimesh)** — **pyglet 1.x only** (trimesh rejects pyglet 2.x on Windows):
  ```powershell
  C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe -m pip install -r tools\mcp\module_viewer\requirements.txt
  ```
  Installs latest `pyglet` 1.5.x + `trimesh`. Do **not** `pip install pyglet` without `<2` — you get 2.x and the viewer errors.

## Greybox materials

Older exports used `export_materials=NONE` (flat untextured shapes). After the export fix, **re-run geometry** to get tinted PBR greybox materials:

```powershell
.\tools\mcp\scripts\reexport_greybox_001.ps1
```

Then re-promote modules if staging changed.

## Pipeline tab (tiles + procedural batch)

| Action | What it runs |
|--------|----------------|
| **Pack tile atlas** | Legacy [`utils/tilemapgen`](../../utils/tilemapgen) — PNG folder → `tile_map_*.png`; optional **-pk** keyframe rename |
| **Run batch** | [`kit_lod0_batch_runner.py`](../scripts/kit_lod0_batch_runner.py) phases `g0g1` / `geometry` / `promote` / `full` |
| **Open light setup** | `utils/Light_keysshotsetup.blend` — iso/Keyshot-style rig for tile stills |
| **Keyframe render addon** | `utils/keyframe_render.py` — render selected animation frames to PNG |

Legacy inventory: [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md).  
Architecture: [`docs/archive/2026-06-src-dev/plans/plan_art_preview_hub_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_art_preview_hub_v1.md).

**Agents / CI — same operations as MCP:**

```powershell
python -m rust_engine_mcp.cli tile-atlas-pack C:\path\to\png_folder -pk
python -m rust_engine_mcp.cli lod0-batch-run --batch kit_lod0_003 --phase geometry
python -m rust_engine_mcp.cli validate-report tile_batch tools\mcp\schemas\examples\tile_batch_factory_floor_v1.json
```

MCP tools: `tile_atlas_pack_tool`, `lod0_batch_run_tool`, `tile_batch_validate`, `tile_batch_run` (stub).

**Workflow (manual):** assemble building in Blender (or future assembly import) → keyframe variants → PNG folder → pack atlas → engine atlas index (T2).

## Not in scope (use other tools)

- Full game engine / Bevy viewport
- Mesh editing (use Blender)
- Automated `tile_batch_run` MCP (T2 — schema in progress)

## Related

- MCP pipeline: `tools/mcp/README.md`
- Full asset editor (vehicles/buildings JSON): `src/utils/asset_tools/run.py`
- Module kit spec: `docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md`
