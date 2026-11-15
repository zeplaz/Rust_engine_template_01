# PLAN-ARTIST-STUDIO-EXPANSION-v1 — APS shell + Blender addon + HTML dash + voxel tab

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ARTIST-STUDIO-EXPANSION-v1** |
| **Status** | **ACTIVE** |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) · [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) · [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) |
| **Deferral** | **DR-VOXEL-OCCUPATION** — sim voxels out of scope ([`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md)) |

## Summary

Expand the Artist Studio (Tk APS) into a multi-surface shell that shares one **StudioTask** JSON bus with headless MCP Blender and an installable Blender addon. Add HTML report dashboards (sim_review pattern) and a **presentation-only** Voxel/Massing subsection. Mission / world-builder tools reuse the same HTML kit.

## Locked decisions

- Keep **Tk APS** as primary shell — no Electron rewrite.
- Blender **addon + headless** call the same job schemas / ops.
- Dashboards = **static HTML from JSON** — not Grafana.
- Voxel tab = **massing presentation** only; no sim occupation writes.

## Authority map

| Surface | Sole writer | Consumers |
|:---|:---|:---|
| AssetSpec / assembly snapshot / RON | MCP / APS generator | Blender applies only |
| StudioTask + job status | enqueue CLI / APS / addon | pollers |
| `massing_columns` presentation | Voxel tab / Bevy paint | ghost only |
| HTML reports | `tools/studio_dash/render_report.py` | APS Dashboard, mission adapters |

## Republic grammar

Dashboards and Blender status must surface `grammar_rule_chain`, seed, and material strategy — not bake status alone.

## Phases

| Phase | Deliverable | Status |
|:---|:---|:---|
| **P0** | Charter + `StudioTask` schema/CLI on `jobs_root` | **SHIPPED** |
| **P1** | Shared Blender addon + APS send/poll | **SHIPPED** |
| **P2** | HTML kit + APS Dashboard panel | **SHIPPED** |
| **P3** | Voxel/Massing APS tab (presentation) | **SHIPPED** |
| **P4** | Mission/world-builder dashboard adapters | **SHIPPED** |

## Code map

| Piece | Path |
|:---|:---|
| Task bus | `tools/mcp/python/rust_engine_mcp/studio_task.py` |
| Schema | `tools/mcp/schemas/studio_task_v1.schema.json` |
| Blender addon | `tools/mcp/blender/addon/rust_engine_studio/` |
| HTML kit | `tools/studio_dash/render_report.py` |
| APS Dashboard | `tools/mcp/art_pipeline_suite/dashboard_panel.py` |
| APS Voxel | `tools/mcp/art_pipeline_suite/voxel_massing_panel.py` |
| Mission adapter | `tools/studio_dash/mission_adapter.py` |

## Book lanes — Voxel/Massing presentation (parallel)

Parent contract: [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) §8.

| Book | Why it maps here |
|:---|:---|
| **PCG-06** [ADOPTED] | Shape/massing grammar — height intent + APS grammar chain; code already lists `voxel_massing.rs` |
| **IO-09** | Asset conditioning pipeline — APS/MCP own preview JSON + StudioTask, not runtime ECS |
| **ECS-15** | Tool-side vs runtime — APS Massing tab never writes sim occupation |
| **RND-01 / RND-09 / RND-18** | Paint/extract/instancing/viewport — consume `massing_columns` only |

**Parallel next cuts (independent PRs):**

1. **APS** — tighten `massing_preview.json` schema (floors clamp, seed required); dashboard opens massing QC HTML.
2. **Engine** — optional load of studio massing override into ghost derive (still sole writer of `massing_columns`); keep DR-VOXEL-OCCUPATION.
3. **Render** — keep pixel_world / world_map_frame consumers RO; later RND-09 instanced column batches if count grows.

Do **not** merge these into one owner or invent a second `FootprintMatrix`.
