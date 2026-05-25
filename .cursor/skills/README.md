# Project skills (`Rust_engine_template_01`)

Cursor discovers skills in `.cursor/skills/<skill-name>/SKILL.md` when this repo is the workspace root.

## Skills

| Skill | Invoke | Use when |
|-------|--------|----------|
| [debug-intelligence](debug-intelligence/SKILL.md) | Attach skill or ask with “debug intelligence” / witness JSON / VM drift | Compress diagnostics, trace authority drift, route to `@planner` / `@coder` / `@designer` — does not fix code directly |
| [cleanup-completion-intelligence](cleanup-completion-intelligence/SKILL.md) | Attach skill or ask before large deletes | Classify obsolete vs transitional vs dormant vs incomplete; preserve gameplay/migration value |

Both skills set `disable-model-invocation: true` — attach explicitly (or name the skill) rather than expecting ambient auto-load.

## Layout

```
.cursor/skills/
├── debug-intelligence/
│   ├── SKILL.md          # workflow + output templates
│   └── reference.md      # full routing / ECS rules
└── cleanup-completion-intelligence/
    ├── SKILL.md
    └── reference.md
```

## Source drafts

Merged from `prompts/rough_agents/` (`debug_intel_a1.skill.md`, `cleanup_a1.skill.md`, and companion `draft_agent_*` files).

## Composite agents

| Agent | Invoke | Bundles |
|-------|--------|---------|
| [**sim-steward**](../agents/sim-steward.md) | `@sim-steward` | bevy-simulation-grade + debug-intelligence + cleanup-completion-intelligence; **Shift A→B→C** in main chat when Task subagents are blocked |
| [**main-thread-orchestrator**](../agents/main-thread-orchestrator.md) | `@main-thread-orchestrator` | Task fail-cycle + foreground slice queue; pairs with sim-steward shifts when Multitask/Task quota fails |

## Related

- Personal: **bevy-simulation-grade** (`~/.cursor/skills/`) — ECS/view/render implementation patterns
- Agents: [`.cursor/agents/`](../agents/) — includes **sim-steward**
- Token contract: [`prompts/llm_agent_brief.md`](../../prompts/llm_agent_brief.md)
- Continuity: [`prompts/guides/subagent_continuity_playbook_v1.md`](../../prompts/guides/subagent_continuity_playbook_v1.md)
