# PLAN-ARTIST-STUDIO-EXPANSION-v1 — APS shell + Blender addon + HTML dash + voxel tab

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ARTIST-STUDIO-EXPANSION-v1** |
| **Status** | **ACTIVE** |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) · [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) · [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) |
| **Voxels** | **In scope** (Studio massing/param frames + engine presentation). [`DR-VOXEL-OCCUPATION`](plan_deferral_registry_v1.md) gates **sim occupation authority replace** only — not a ban on voxel work. Upgrade program: [`plan_ref_books_upgrade_v3.md`](plan_ref_books_upgrade_v3.md). |

## Summary

Treat Artist Studio as the **creative DCC**: parametric frames, grammars, modifiers, and connection logic first; share one **StudioTask** JSON bus so Blender (addon + headless) and other full DCCs are optional upstream sinks. HTML report dashboards (sim_review pattern) and a **presentation-only** Voxel/Massing subsection stay in-kit. Mission / world-builder tools reuse the same HTML kit.

## Creative DCC stance (locked)

**Artist Studio is the DCC** for day-to-day authoring — not a thin remote for Blender.

| Principle | Meaning |
|:---|:---|
| **Parametric frames** | Operators work a **parametric interface** (seeded frames / params / grammars / modifiers / connection graphs), not ad-hoc bake clicks. |
| **Maximize the parametric stage** | Prefer engine-API-reachable param surfaces (StudioTask bus, grammar chains, massing preview, assembly snapshots) so creative iteration stays in-studio and in-engine. |
| **Guide, don't trap** | Studio guides a pipeline process (spec → validate → stage → promote) while unlocking creative potential — not blocking on a full DCC session for every move. |
| **Upstream when needed** | Heavy mesh / audio / material work **may** escalate to a full DCC (Blender, audio DAW, etc.) via the same job schemas — optional, not the default loop. |
| **Open generative tooling** | Studio-adjacent tools expose **generative grammars, modifiers, and connection logic** so progress does not require always tying backend bpy/CLI into the pipeline by live call or by mirroring every backend knob as a parameterized view. |

**Anti-pattern:** every creative step = `geometry_run_job` / Blender round-trip.  
**Preferred loop:** edit param frame → grammar/modifier/connection preview in Studio → engine presentation read → escalate to Blender only when the frame cannot express the intent.

**Agent continuity:** Studio work rides [`plan_ref_books_upgrade_v3.md`](plan_ref_books_upgrade_v3.md) §0.1 — tripwire on response-stop / lazy / confused exits; no user ping on §0 locks (“Studio = DCC”, voxels in scope); cut the next OPEN tools-suite or v3 slice when blocked.

## Locked decisions

- Keep **Tk APS** as primary shell — no Electron rewrite.
- **Studio = DCC**; Blender/audio are **upstream sinks**, same schemas when used.
- Blender **addon + headless** call the same job schemas / ops (parity when escalated).
- Dashboards = **static HTML from JSON** — not Grafana.
- Voxel tab = **massing presentation** only; no sim occupation writes.
- Prefer **parametric frames + grammars/modifiers** over mandatory backend tool calls for iteration.

## Authority map

| Surface | Sole writer | Consumers |
|:---|:---|:---|
| Parametric frames / grammar / modifiers (studio) | APS / StudioTask authors | Engine preview, MCP when escalated |
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
| **ParamFrame** (S0) | `tools/mcp/schemas/param_frame_v1.schema.json` · `examples/param_frame_rail_warehouse_v1.json` — StudioTask `kind: param_frame`; seed + grammar chain + modifiers + connection graph ids + massing intent; **no bpy** required to edit; `sim_occupation: false` always |
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
