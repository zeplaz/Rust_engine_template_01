# Project skills (`Rust_engine_template_01`)

Cursor discovers skills in `.cursor/skills/<skill-name>/SKILL.md` when this repo is the workspace root.

## Skills

| Skill | Invoke | Use when |
|-------|--------|----------|
| [debug-intelligence](debug-intelligence/SKILL.md) | Attach skill or ask with “debug intelligence” / witness JSON / VM drift | Compress diagnostics, trace authority drift, route to `@planner` / `@coder` / `@designer` — does not fix code directly |
| [cleanup-completion-intelligence](cleanup-completion-intelligence/SKILL.md) | Attach skill or ask before large deletes | Classify obsolete vs transitional vs dormant vs incomplete; preserve gameplay/migration value |
| [mcp-asset-pipeline](mcp-asset-pipeline/SKILL.md) | Attach for MCP art / AssetSpec / staging → Bevy | Orchestrate deterministic spec → tool → validate → promote; route tile vs geometry lanes |
| [mcp-production-rules](mcp-production-rules/SKILL.md) | Attach before any MCP/CLI art call | Enforce no AI final art, deterministic seed, batch/atlas, grid alignment |
| [tile-generation](tile-generation/SKILL.md) | Attach for isometric tile variants / atlas | Tile state machines, orthographic bake specs, batch tile MCP (planned toolchain) |
| [blender-geometry](blender-geometry/SKILL.md) | Attach for geometry jobs / bpy ops | AssetSpec + `geometry_run_job`, module kit GLB export via `tools/mcp/` |
| [validation-first](validation-first/SKILL.md) | Attach before cargo/test/MCP verify | Structured ValidationReport — never paste raw compiler logs |

All project skills set `disable-model-invocation: true` — attach explicitly (or name the skill) rather than expecting ambient auto-load.

## Layout

```
.cursor/skills/
├── debug-intelligence/
│   ├── SKILL.md
│   └── reference.md
├── cleanup-completion-intelligence/
│   ├── SKILL.md
│   └── reference.md
├── mcp-asset-pipeline/
│   ├── SKILL.md
│   └── reference.md
├── mcp-production-rules/
│   ├── SKILL.md
│   └── reference.md
├── tile-generation/
│   ├── SKILL.md
│   └── reference.md
└── blender-geometry/
    ├── SKILL.md
    └── reference.md
```

## Source drafts

| Skill family | Draft source |
|--------------|--------------|
| debug / cleanup | `prompts/rough_agents/` |
| MCP / art pipeline | [`prompts/MCP/mcp_drafts.md`](../../prompts/MCP/mcp_drafts.md), [`prompts/MCP/rules_skills_draft.md`](../../prompts/MCP/rules_skills_draft.md) |

## Composite agents

### Engine lane

| Agent | Invoke | Bundles |
|-------|--------|---------|
| [**sim-steward**](../agents/sim-steward.md) | `@sim-steward` | bevy-simulation-grade + debug-intelligence + cleanup-completion-intelligence; **Shift A→B→C** in main chat when Task subagents are blocked |
| [**main-thread-orchestrator**](../agents/main-thread-orchestrator.md) | `@main-thread-orchestrator` | Task fail-cycle + foreground slice queue; pairs with sim-steward shifts when Multitask/Task quota fails |
| [**coparent-orchestrator**](../agents/coparent-orchestrator.md) | `@coparent-orchestrator` | Secondary parallel pathways (operator, VFX, designer tails); debug + cleanup + sim-grade conflict matrix vs primary `@orchestrator` P1 |

### MCP art pipeline lane

| Agent | Invoke | Role |
|-------|--------|------|
| [**orchestrator-mcp**](../agents/orchestrator-mcp.md) | `@orchestrator-mcp` | Phase graph + gates G0–G5; blocks skips |
| [**planner-mcp**](../agents/planner-mcp.md) | `@planner-mcp` | Schemas, tool architecture, SHIPPED/PLANNED honesty |
| [**designer-mcp**](../agents/designer-mcp.md) | `@designer-mcp` | Critique, AssetSpec, staging sign-off |
| [**coder-mcp**](../agents/coder-mcp.md) | `@coder-mcp` | `tools/mcp/` implementation, CLI/MCP parity |

**Handoff:** `orchestrator-mcp` → `planner-mcp` (if needed) → `designer-mcp` → `coder-mcp` → validate → promote.

| Agent | Invoke | Role |
|-------|--------|------|
| [**designer**](../agents/designer.md) | `@designer` | HUD/UX, overlays, multiview — **charters** art; **`@designer-mcp`** runs jobs |

## Consumer vs builder (required)

| Role | Invoke MCP tools? | Build `tools/mcp/`? |
|:---|:---:|:---:|
| coder, designer, planner, orchestrator, sim-steward | **Use** (validate-report / charter) | **No** |
| *-mcp agents | **Yes** | **Yes** (coder-mcp / designer-mcp) |

Fleet guide: [`src/dev/agent_mcp_consumer_guide_v1.md`](../../src/dev/agent_mcp_consumer_guide_v1.md)

## Related

- Personal: **bevy-simulation-grade** (`~/.cursor/skills/`) — ECS/view/render implementation patterns
- Agents: [`.cursor/agents/`](../agents/) — includes **sim-steward**, **designer**
- MCP toolchain: [`tools/mcp/README.md`](../../tools/mcp/README.md)
- Token contract: [`prompts/llm_agent_brief.md`](../../prompts/llm_agent_brief.md)
- Continuity: [`prompts/guides/subagent_continuity_playbook_v1.md`](../../prompts/guides/subagent_continuity_playbook_v1.md)
