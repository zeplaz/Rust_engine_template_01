# Art pipeline extensions `v1` (index)

| Field | Value |
|:---|:---|
| **Extends** | [`art_design.md`](art_design.md) |
| **Exec (coders + designer ops)** | [`src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md`](../../src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md) |
| **Tooling root** | [`tools/mcp/README.md`](../../tools/mcp/README.md) · onboarding [`designer_mcp_onboarding_v1.md`](designer_mcp_onboarding_v1.md) |
| **Module kit (50 greybox targets)** | [`src/dev/design_procedural_module_kit_v1.md`](../../src/dev/design_procedural_module_kit_v1.md) |

## What this file is for

`art_design.md` defines **architecture** (MCP roles, structured specs, no AI-final assets).  
This file tracks **extensions** and **implementation status** — not duplicate the design essay.

## Extension backlog

| ID | Extension | Phase in exec plan |
|:---|:---|:---|
| EXT-01 | Blender headless + Geometry MCP | ART-GEO-* |
| EXT-02 | Validation MCP + Rust `art_validator` | ART-VAL-* |
| EXT-03 | Material Maker / PBR CLI adapter | ART-MAT-* |
| EXT-04 | Reference MCP (OSM, Natural Earth metadata) | ART-REF-* |
| EXT-05 | Houdini / Substance adapters (optional) | Phase 4+ |
| EXT-06 | District rule JSON → procedural assembly (no mesh in agent) | construction Phase 6+ |

## Designer Cursor checklist

1. Install Blender 4.2+; set `BLENDER_EXE`.
2. Enable MCP: copy `tools/mcp/cursor-mcp.example.json` per exec plan §2.2.
3. Use `@designer-mcp` + tools: `spec_write` → `geometry_run_job` → `validate_glb_asset` → `promote_staging_module` → `library_register`.
4. Review outputs in `assets/staging/` before promotion.

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Index populated; was empty |
