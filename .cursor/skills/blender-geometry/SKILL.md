---
name: blender-geometry
description: Authors Blender headless geometry jobs, procedural module specs, building visual layers, and prop MCP requests for Rust_engine_template_01. Use when creating AssetSpec/geometry_job JSON, extending bpy ops in tools/mcp/blender, or planning GLB/LOD/collision exports — never chat-only mesh generation.
disable-model-invocation: true
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Blender Geometry Skill

Procedural **modules** and geometry jobs via structured JSON → headless Blender — aligned with shipped `tools/mcp/`.

## When to use

- Writing `AssetSpec` or `geometry_job_v1` JSON
- Running `geometry_run_job` / `python -m rust_engine_mcp.cli run-geometry`
- Adding bpy ops under `tools/mcp/blender/scripts/ops/`
- Planning building/prop/smoke/light layered visual states

## Primary rule

> Footprint + style + state in JSON → bpy op graph → GLB + validation → promote.

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

**DSM node WRK** — geometry jobs only via JSON:

```text
$ref:tools/mcp/schemas/geometry_job_v1.schema.json → geometry_run_job → BLANG:validate_asset → 🟢|🔴
```

**Pivot rule:** `$ref:docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md§grid` — bottom-center, 1u grid.

## Quick workflow

1. Attach **`mcp-production-rules`** + [mcp-asset-pipeline](mcp-asset-pipeline/SKILL.md).
2. Read shipped ops: [`MICRO_TOOLS_REGISTRY_v1.md`](tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) Tier 2.
3. Author job JSON against [`geometry_job_v1.schema.json`](tools/mcp/schemas/geometry_job_v1.schema.json).
4. Execute: `geometry_run_job` or CLI `run-geometry`.
5. `validate_glb_asset` → `promote_staging_module`.
6. For multi-module buildings: compose from module kit — not monolithic mesh in chat.

## Shipped bpy operations

| Op | Params | Output |
|----|--------|--------|
| `module_wall` | width_m, height_m, depth_m | box mesh, bottom-center pivot |
| `module_roof` | width_m, depth_m, thickness_m | flat slab |
| `module_door` | width_m, height_m, depth_m | frame box |

Add ops: `tools/mcp/blender/scripts/ops/` + register in `run_job.py`.

## Geometry job template

```json
{
  "job_id": "wall_brick_1u_example",
  "operation": "module_wall",
  "params": {
    "width_m": 1.0,
    "height_m": 3.0,
    "depth_m": 0.2
  }
}
```

## Building visual layers (planned / Republic-style)

Layer stack for state-driven districts — see [reference.md](reference.md):

Base structure → damage → lights → smoke → cargo → power emission

## Prop MCP (planned)

Small objects (crates, pipes, stacks) via `prop.generate` — spec in drafts, not shipped.

## Outputs

| Artifact | Path pattern |
|----------|----------------|
| GLB | `assets/staging/<job_id>/model.glb` |
| Status | `tools/mcp/jobs/<job_id>.status.json` |
| Promoted | `assets/models/modules/` + RON sidecar |

## Grid alignment

- Modules align to **1u grid** from [`design_procedural_module_kit_v1.md`](docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md)
- Bottom-center pivot on walls; no free rotation in iso presentation

## BUILD-GRAMMAR◈ → MODULE-RUNS◈ (v0)

Building grammar (`building_grammar_v1` + optional `arch_build_grammar_v0`) selects massing and slot keys; **this skill** produces the module GLBs.

```text
ARCH-DNA◈ → β◈ → SHAPE-GRAMMAR◈ → facade/roof/detail slots → geometry_job_v1 → GLB
```

| Grammar slot | bpy op |
|:---|:---|
| `wall_slot` | `module_wall` |
| `door_slot` | `module_door` |
| `window_slot` | `module_window` |
| `roof_*` | `module_roof` |
| `prop_slot` | `module_prop` |

- `βmod` (modularity pressure) drives module-run **density** — not new ops in v0.
- Refs: [`arch_build_grammar_v0_baseline_v1.md`](src/dev/arch_build_grammar_v0_baseline_v1.md) · [`industrial_warehouse_v1.ron`](assets/configs/buildings/grammars/industrial_warehouse_v1.ron)

## Route conflicts

| Situation | Delegate |
|-----------|----------|
| New bpy op implementation | `@coder` |
| Module kit / style pack UX | `@designer` |
| Bevy load / representation | `@coder` + bevy-simulation-grade |

## Additional resources

- Onboarding: [`docs/archive/2026-06-src-dev/plans/designer_mcp_onboarding_v1.md`](docs/archive/2026-06-src-dev/plans/designer_mcp_onboarding_v1.md)
- Source drafts: [`docs/archive/2026-06-fleet-drain/prompts_drafts/mcp_drafts.md`](docs/archive/2026-06-fleet-drain/prompts_drafts/mcp_drafts.md) §2.1, §3, §5
- Layer specs + future building MCP: [reference.md](reference.md)
