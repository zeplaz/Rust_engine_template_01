# PLAN-APS-EXEC-001 — Art Pipeline Suite implementation `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-EXEC-001** |
| **Design** | [`design_art_pipeline_suite_v1.md`](design_art_pipeline_suite_v1.md) |
| **Owner** | `@coder-mcp` |
| **Depends on** | [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) AUTO-007+ |
| **Status** | **READY** |

---

## Coder-mcp phases

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **APS-UI-001** | Package rename shim: `art_pipeline_suite/run.py` → calls same app; keep `module_viewer/run.py` | both entrypoints work |
| **APS-UI-002** | Tab shell: Catalog \| Assembly \| Variants \| Atlas | empty tabs OK |
| **APS-UI-003** | Move current viewer into **Catalog** tab | parity with today |
| **APS-VAR-001** | `variant_set_validate` MCP + CLI | jsonschema on drafts |
| **APS-VAR-002** | `variant_set_patch` MCP + CLI | RFC6902-style patch, deterministic |
| **APS-UI-004** | Variants tab: edit layers (lighting, damage, material), tags | saves `.ron` / `.json` |
| **APS-UI-005** | **Request agent** → writes `variant_agent_request.json` to `debug_runs/art_pipeline/` | human pastes to Cursor |
| **APS-UI-003b** | **Assembly Editor** — placements list; per-slot **material_profile** + tag/variant/LOD; load/save/validate | **done** — `art_pipeline_suite/assembly_panel.py` |
| **APS-UI-006** | Flow buttons: Send to Assembly, Bake variants, Pack atlas | wired to tile_pipeline |
| **APS-VAR-003** | `variant_bake(variant_key)` single-variant job | PNG + bake.status in variant_set |
| **APS-AGENT-001** | `variant_agent_request` MCP stub → returns suggested patch JSON | no LLM inside repo |

---

## Paste — @coder-mcp (after AUTO-007)

> Implement **APS-UI-001→006** and **APS-VAR-001→003** from `plan_art_pipeline_suite_exec_v1.md`. Follow `design_art_pipeline_suite_v1.md`. Suite must call same APIs as MCP agents. Variants = `variant_set_v1` layers (lights, materials, damage) — not duplicate blend files per variant.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Suite exec plan |
