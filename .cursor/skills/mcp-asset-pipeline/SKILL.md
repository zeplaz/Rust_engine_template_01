---
name: mcp-asset-pipeline
description: Orchestrates deterministic MCP asset production for Rust_engine_template_01 — Agent spec JSON, rule enforcement, tool selection, staging validation, and Bevy promotion. Use when wiring MCP tools, authoring AssetSpec/geometry jobs, batch asset pipelines, or routing art work to @designer / @coder — not for freeform mesh or texture generation.
disable-model-invocation: true
---

# MCP Asset Pipeline Orchestrator

Deterministic **STATE → TOOL → BATCHED ART** production. The LLM writes **structured specs** and selects **MCP/CLI tools** — it does not generate final meshes or textures.

## When to use

- Authoring or reviewing MCP requests / `AssetSpec` / geometry job JSON
- Chaining spec → Blender → validate → promote → Bevy registry
- Deciding which MCP server or micro-CLI to invoke
- Planning tile, prop, or animation pipeline extensions (see drafts)

## Primary rule

> Agents are policy + skill routing + tool selection — not asset generators.

## Quick workflow

1. Read [`prompts/llm_agent_brief.md`](prompts/llm_agent_brief.md) (token contract).
2. Read shipped toolchain: [`tools/mcp/README.md`](tools/mcp/README.md), [`MICRO_TOOLS_REGISTRY_v1.md`](tools/mcp/MICRO_TOOLS_REGISTRY_v1.md).
3. Read full pipeline: [reference.md](reference.md) + source drafts [`prompts/MCP/mcp_drafts.md`](prompts/MCP/mcp_drafts.md), [`prompts/MCP/rules_skills_draft.md`](prompts/MCP/rules_skills_draft.md).
4. Attach **`mcp-production-rules`** — enforce before any tool call.
5. Classify request: **geometry** · **tile** · **prop** · **material** · **reference-only**.
6. Emit MCP request JSON (never bpy/chat mesh instructions).
7. Execute via MCP tool or `python -m rust_engine_mcp.cli` (same code path).
8. Validate → promote → register in asset library.

## Execution model

```text
Agent (@designer / @coder)
  ↓ Load skills + apply rules
  ↓ Select tool (MCP or micro-CLI)
  ↓ MCP Request JSON
  ↓ Python toolchain (tools/mcp/python/)
  ↓ External binary (Blender / future tile batch)
  ↓ assets/staging/<job_id>/
  ↓ validate_glb_asset → promote_staging_module
  ↓ Bevy (BuildingDefinition / StylePack / RepresentationResult)
```

## Shipped vs planned (repo truth)

| Lane | Status | Skill |
|------|--------|-------|
| Geometry (Blender modules) | **Shipped** — `tools/mcp/` | [blender-geometry](blender-geometry/SKILL.md) |
| AssetSpec + validate + promote | **Shipped** | this skill |
| Tile keyframe pack + atlas | **Shipped** (pack/register); ortho bake CI-only | [tile-generation](tile-generation/SKILL.md) |
| Prop / smoke / light MCP | **Planned** (draft spec) | [blender-geometry](blender-geometry/SKILL.md) |
| Skill runtime engine (Rust) | **Future** (draft only) | reference.md |

## MCP request template

```yaml
tool: geometry_run_job | tile.generate | prop.generate  # use shipped names when available
input:
  spec_path: tools/mcp/schemas/examples/wall_job.example.json
  # or inline AssetSpec / geometry_job_v1 fields
rules_applied: [no_ai_generated_images, deterministic_output, batch_processing, grid_alignment]
expected_outputs: [model.glb, metadata, status.json]
promotion_target: assets/models/modules/
```

## Route conflicts

| Situation | Delegate |
|-----------|----------|
| UX / visual state readability | `@designer-mcp` |
| MCP server / CLI / bpy ops | `@coder-mcp` |
| Multi-lane pipeline architecture | `@planner-mcp` |
| ECS registry / Bevy load contract | `@coder` + bevy-simulation-grade |
| Rule violation or cleanup of draft shims | `@sim-steward` |

## Token discipline

Compress to YAML + `path` + tool name. No log dumps, no full glb/base64 in chat.

## Additional resources

- Exec plan: [`src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md`](src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md)
- Module kit: [`src/dev/design_procedural_module_kit_v1.md`](src/dev/design_procedural_module_kit_v1.md)
- Full architecture: [reference.md](reference.md)
