# Project skills index (Cursor base)

**Source of truth for authoring:** `.claude/skills/`  
**Cursor discovery path:** `.cursor/skills/` (this tree)

## Every session (all agents)

1. **Sync** if any `SKILL.md` is missing or empty:
   ```powershell
   powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1
   ```
2. **Attach** [agent-lang/SKILL.md](agent-lang/SKILL.md) — BLANG, queue, witnesses, validators
3. **Read** domain skills for your role (see `AGENTS.md` § Skills)

After MCP/Python toolchain changes: **reload Cursor MCP** server `rust-engine-art`.

## Skill catalog

| Skill | Role |
|:---|:---|
| [agent-lang](agent-lang/SKILL.md) | **Base** — all agents, every session |
| [sync-claude-skills](sync-claude-skills/SKILL.md) | Mirror `.claude/skills` → `.cursor/skills` |
| [validation-first](validation-first/SKILL.md) | validate-report after builds |
| [bevy-simulation-grade](bevy-simulation-grade/SKILL.md) | ECS / viewport authority |
| [debug-intelligence](debug-intelligence/SKILL.md) | Witness drift, viewport/ECS |
| [cleanup-completion-intelligence](cleanup-completion-intelligence/SKILL.md) | Before deletes |
| [operations-intelligence](operations-intelligence/SKILL.md) | OPS witness spine |
| [mcp-asset-pipeline](mcp-asset-pipeline/SKILL.md) | MCP art lane |
| [mcp-production-rules](mcp-production-rules/SKILL.md) | Ship gates, bake source |
| [tile-generation](tile-generation/SKILL.md) | Iso tile / atlas |
| [blender-geometry](blender-geometry/SKILL.md) | bpy modules |

## Agent bootstrap

Normative ritual: [`.cursor/agents/_fragments/session_bootstrap_v1.md`](../agents/_fragments/session_bootstrap_v1.md)

```text
SKILL-SYNC ⊳ node .claude/skills/agent-lang/driver.mjs boot <agent> ⊳ work ⊳ WIT-HON ⊳ Q✓
```
