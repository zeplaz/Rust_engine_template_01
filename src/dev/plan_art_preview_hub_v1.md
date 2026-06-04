# Plan — Art preview hub (modules + assembly + tiles) `v1`

| Field | Value |
|:---|:---|
| **ID** | **PLAN-ART-PREVIEW-HUB-001** |
| **Owner** | `@planner-mcp` + `@coder-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **SUPERSEDED for primary path** by [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) |
| **Policy** | Manual UI = **debug only** after AUTO-010 |

---

## Problem

- **Module viewer** covers single GLBs only.
- **Legacy utils** (`tilemapgen`, `keyframe_render.py`, `Light_keysshotsetup.blend`) hold the real tile workflow but are disconnected from MCP.
- Artists need **one preview surface** to run **manual or automated**:
  1. Procedural module generation (batch)
  2. Assembly preview (multi-module scene — future)
  3. Keyframe stills → tile atlas pack

---

## Pipeline (coupled)

```text
[Automated] kit_lod0_batch_runner / geometry_run_job
[Manual]    Blender + Light_keysshotsetup.blend + keyframe_render.py
            ↓ PNG folder
            tilemapgen (utils) or future atlas_packer
            ↓
            Preview atlas in hub + register _tile_atlas_index.ron (T2)
```

**Building tiles:** bake from **assembled** scene ([`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md)), not single modules.

---

## Preview hub phases

| Phase | Deliverable | Mode |
|:---|:---|:---|
| **PV-0** | Pipeline tab in `module_viewer` | **SHIPPED** — pack tiles, run batch, open blend |
| **PV-1** | Assembly preview: import N GLBs into Blender collection from StylePack + footprint JSON | manual + script |
| **PV-2** | `tile_ortho_bake` bpy op using `Light_keysshotsetup.blend` camera/lights | automated MCP |
| **PV-3** | Browser preview for atlas PNG + UV grid overlay | automated |
| **PV-4** | PG-2 engine preview (Bevy) — separate from hub | engine |

---

## Tri-mode (required)

Every shipped step must exist on **three surfaces** with **one implementation**:

| Surface | Who uses it |
|:---|:---|
| **Manual** | Artists — `module_viewer` Pipeline tab, Blender GUI |
| **CLI** | Scripts / CI — `python -m rust_engine_mcp.cli …` |
| **MCP** | Cursor agents — `rust-engine-art` FastMCP tools |

Shared module: `tools/mcp/python/rust_engine_mcp/tile_pipeline.py`.

## Manual vs automated

| Step | Manual | CLI / MCP (agent) |
|:---|:---|:---|
| Module GLB | Blender review | `geometry_run_job` / `lod0_batch_run_tool` |
| Assembly layout | Blender append modules | `assembly_import_blender` (**PLANNED**) |
| Variant stills | `keyframe_render.py` UI | `tile_batch_run` (**PLANNED** T2) |
| Atlas pack | Hub → **Pack tile folder** | `tile_atlas_pack_tool` / `tile-atlas-pack` |
| Validate | Hub + `validate_report` | `tile_batch_validate` |

---

## Env vars

| Var | Default |
|:---|:---|
| `RUST_ENGINE_REPO` | repo root |
| `RUST_ENGINE_TILE_LIGHT_BLEND` | `utils/Light_keysshotsetup.blend` |
| `RUST_ENGINE_TILEMAPGEN` | `utils/tilemapgen` (cargo run) |

---

## Acceptance (PV-0)

1. User selects folder of PNGs → hub runs tilemapgen → opens `tile_map_*.png`.
2. User selects `kit_lod0_003` → hub runs batch runner (validate or geometry) → log in UI.
3. User clicks **Open light setup blend** → Blender opens rig file.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Legacy scan + module viewer Pipeline tab |
