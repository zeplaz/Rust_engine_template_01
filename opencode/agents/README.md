# Agents Reference

## Bootstrap Ritual

Every session starts with:

```text
SKILL-SYNC ⊳ node .cursor/skills/agent-lang/driver.mjs boot <agent> ⊳ work ⊳ WIT-HON ⊳ Q✓
```

## Handoff Chain

```
orchestrator → planner → coder/designer → verification
```

## Role Matrix

| Agent | Lane | Primary |
|-------|------|---------|
| orchestrator | Sequencing | plans, delegates |
| orchestrator-mcp | MCP art | spec→validate→promote |
| planner | Architecture | readonly, authority map |
| planner-mcp | MCP toolchain | schemas, rollout |
| coder | ECS/render | production bar, validation-first |
| coder-mcp | MCP tools | bpy ops, validators |
| designer | HUD/overlay | UX, ghosts |
| designer-mcp | MCP art | AssetSpec, quality gates |
| sim-steward | Simulation | bevy-grade + debug-intelligence |
| main-thread-orchestrator | Continuity | Task fail-cycle escalation |
| coparent-orchestrator | Secondary | parallel lanes, conflict matrix |
| operations-intelligence | OPS analyst | DSM, QCE, ΔWF routing |

## Playbook References

Each agent reads matching playbook in `tools/orchestrator/agents/`:
- viewport_cleanup_agent
- render_pipeline_agent
- stage5_readiness_agent
- ui_layout_agent
