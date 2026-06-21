# Opencode Shared Entry Point

Cross-model reference for skills and agents. All paths resolve to `.cursor/` directory.

## Quick Reference

| Type | Entry | Description |
|------|-------|-------------|
| Skills | [skills/README.md](skills/README.md) | Skill catalog and sync instructions |
| Agents | [agents/README.md](agents/README.md) | Agent roles and handoff chain |
| Bootstrap | [agents/_fragments/session_bootstrap_v1.md](agents/_fragments/session_bootstrap_v1.md) | Session ritual |

## Skills Catalog

| Skill | Purpose |
|-------|---------|
| [agent-lang](skills/agent-lang/SKILL.md) | Base layer — all agents, every session |
| [sync-claude-skills](skills/sync-claude-skills/SKILL.md) | Mirror `.claude/skills` → `.cursor/skills` |
| [bevy-simulation-grade](skills/bevy-simulation-grade/SKILL.md) | ECS / viewport authority |
| [debug-intelligence](skills/debug-intelligence/SKILL.md) | Witness drift, viewport/ECS |
| [validation-first](skills/validation-first/SKILL.md) | Build validation reports |
| [operations-intelligence](skills/operations-intelligence/SKILL.md) | OPS witness spine, DSM/QCE |
| [cleanup-completion-intelligence](skills/cleanup-completion-intelligence/SKILL.md) | Before deletes |
| [mcp-asset-pipeline](skills/mcp-asset-pipeline/SKILL.md) | MCP art lane |
| [mcp-production-rules](skills/mcp-production-rules/SKILL.md) | Ship gates, bake source |
| [tile-generation](skills/tile-generation/SKILL.md) | Iso tile / atlas |
| [blender-geometry](skills/blender-geometry/SKILL.md) | bpy modules |

## Agents Catalog

| Agent | Primary Role |
|-------|--------------|
| [orchestrator](agents/orchestrator.md) | Sequencing |
| [orchestrator-mcp](agents/orchestrator-mcp.md) | MCP art pipeline |
| [planner](agents/planner.md) | Architecture plans |
| [planner-mcp](agents/planner-mcp.md) | MCP toolchain |
| [coder](agents/coder.md) | ECS, render, diagnostics |
| [coder-mcp](agents/coder-mcp.md) | MCP toolchain |
| [designer](agents/designer.md) | HUD, overlays, UX |
| [designer-mcp](agents/designer-mcp.md) | MCP art pipeline |
| [sim-steward](agents/sim-steward.md) | Simulation steward |
| [operations-intelligence](agents/operations-intelligence.md) | Pipeline/ops analyst |

## Session Bootstrap

Always sync skills first if any `SKILL.md` is missing:

```powershell
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1
```

Then attach [agent-lang/SKILL.md](skills/agent-lang/SKILL.md) for every session.

## Model Compatibility

This entry point works across all models. Paths are relative to repo root.