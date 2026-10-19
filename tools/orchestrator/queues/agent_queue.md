# Agent queue

**Lean entry (AGN-MIN-HARNESS-001):** default chat = Auto / `@coder` · home queues below · full guide `$ref:prompts/guides/agent_orchestration_minimal_harness_v1.md`.

## Default lean queues

| Use | Path |
|:---|:---|
| Coder home | `coder_active_queue.json` (or HANDOFF ACTIVE pick) |
| Defer blockers | `defer_registry.json` |
| Session continuity | `HANDOFF.md` |

`*_drain_*` / `parallel_wave_*` / `post_drain_*` = **fleet / historical batch** — valid, not default chat entry.

## Project Cursor agents (`.cursor/agents/`)

**Primary:** `@coder` · `@planner` · `@designer` (+ `*-mcp` in territory) · `@sim-steward`  
**Escalation:** `@orchestrator` · `@orchestrator-mcp` · `@plan-orchestrator` · `@main-thread-orchestrator` · `@coparent-orchestrator` · `@flow-controller` · `@operations-intelligence`

- `@orchestrator` — `.cursor/agents/orchestrator.md` (escalation)
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

_None — infrastructure viewport migration complete; use @coder for new work._

