---
name: mcp-production-rules
description: Enforces non-negotiable MCP asset production constraints for Rust_engine_template_01 — no AI final art, deterministic seeded output, batch/atlas processing, and grid alignment. Use before any MCP tool call, asset generation request, or when evaluating whether a workflow violates production rules.
disable-model-invocation: true
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# MCP Production Rules

Hard constraints — **not** suggestions. Apply **pre_execution** before any toolchain call.

## When to use

- Before invoking MCP tools or micro-CLI (`rust_engine_mcp.cli`)
- When a request mentions diffusion, AI textures, chat-only bpy, or single one-off assets
- When validating agent/tool configs from draft JSON schemas
- Pair with [mcp-asset-pipeline](mcp-asset-pipeline/SKILL.md) on every art lane

## Primary rule

> Same input + same seed → same output. No LLM freeform generation leakage into final artifacts.

## Rule checklist (enforce all that apply)

| Rule ID | Enforcement | Pass when |
|---------|-------------|-----------|
| `no_ai_generated_images` | pre_execution | No diffusion/image-gen for final albedo/mesh; references/metadata only |
| `deterministic_output` | pre_execution | All variation is seed-based; no unseeded random |
| `batch_processing` | pre_execution | Process asset **groups** / atlases — not ad-hoc singles |
| `grid_alignment` | pre_execution | Fixed tile unit; no free rotation in isometric system |

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

**Pre-execution verdict** — one line:

```yaml
rules_check: { passed: 🟢|🔴, blocked_by: [no_ai_generated_images, ...], seed: "<required>" }
```

Blocked → `ΔWF→@designer-mcp` with `$ref:` reroute spec — not chat mesh.

## Quick workflow

1. `BLANG:REF` on [reference.md](reference.md) for full conditions.
2. Inspect proposed `tool` + `input` JSON.
3. Block and reroute if any **hard_rule** fails.
4. Emit compressed verdict:

```yaml
rules_check:
  passed: true | false
  blocked_by: []  # rule ids
  reroute: "spec JSON + geometry_run_job" | "tile_batch spec" | "reference-only"
  seed: "<required if variation>"
```

## Blocked patterns

| Don't | Do instead |
|-------|------------|
| GenerateImage / diffusion for final assets | Blender orthographic bake, Material Maker CLI |
| Describe mesh in chat | `geometry_run_job` + `geometry_job_v1` JSON |
| Paste base64 textures | Staging path + manifest |
| Single tile without batch context | `tile_batch` spec or atlas pack plan |
| Free-rotated props in iso grid | Snap to grid; fixed unit from module kit |
| `tile_batch_run` ortho for production buildings | `keyframe_render` PNGs → `tile-atlas-pack`; `bake_source: keyframe_pack` |
| Promote lod0 ortho pilot atlases | Production keyframe stills + G4 sign-off |

## ECS / engine alignment

Rules apply to **toolchain outputs** entering Bevy — not simulation ECS authority. Promotion paths must match [`design_procedural_module_kit_v1.md`](docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md) pivot and naming.

## Route violations

| Violation type | Owner |
|----------------|-------|
| Architectural (new tool bypass) | `@planner-mcp` |
| Implement fix / MCP wiring | `@coder-mcp` |
| UX spec without AssetSpec | `@designer-mcp` |

## Additional resources

- Source: [`docs/archive/2026-06-fleet-drain/prompts_drafts/rules_skills_draft.md`](docs/archive/2026-06-fleet-drain/prompts_drafts/rules_skills_draft.md) §3
- Designer agent art pipeline: [`.cursor/agents/designer.md`](../../agents/designer.md)
- Full rule definitions: [reference.md](reference.md)
