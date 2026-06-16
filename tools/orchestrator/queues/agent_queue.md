# Agent queue

## Project Cursor agents (`.cursor/agents/`)

- `@orchestrator` — `.cursor/agents/orchestrator.md` (primary critical path)
- `@orchestrator-mcp` — `.cursor/agents/orchestrator-mcp.md` (MCP art lane — `mcp_lane_order_v1.md`)
- `@coparent-orchestrator` — `.cursor/agents/coparent-orchestrator.md` (secondary parallel pathways)
- `@planner` — `.cursor/agents/planner.md`
- `@coder` — `.cursor/agents/coder.md`
- `@designer` — `.cursor/agents/designer.md`

## Lane playbooks (`tools/orchestrator/agents/`)

- `stage5_readiness_agent`
- `viewport_cleanup_agent`
- `render_pipeline_agent`
- `ui_layout_agent`
- `dead_code_agent`
- `migration_tracker_agent`
- `runbook_sync_agent`
- `thread_health_agent`
- `warning_classifier_agent`

## ACTIVE MIGRATIONS

```text
⚡P0 MASTER DRAIN — $ref:tools/orchestrator/queues/coder_master_drain_queue.json
Tracks: REWIRE → FIRE → BUILD → VEG (seq 1–24)
$ref:src/dev/coder_drain_order_master_v1.md
```

**Lang:** `$ref:.cursor/skills/agent-lang/SKILL.md` · `$ref:src/dev/agent_lang_v1.md`
