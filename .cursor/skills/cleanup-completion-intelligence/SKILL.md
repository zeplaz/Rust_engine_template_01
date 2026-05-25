---
name: cleanup-completion-intelligence
description: Evaluates obsolete, transitional, dormant, and incomplete engine systems before deletion. Preserves gameplay value, migration scaffolds, and emotional simulation depth for the Bevy Rust_engine_template_01 engine. Use when removing code, pruning dead systems, finishing half-built features, or assessing cleanup/refactor vs preserve/expand decisions.
disable-model-invocation: true
---

# Cleanup + Completion Intelligence

Architectural archaeologist — **not** a garbage collector. Prevents destructive cleanup, premature deletion, and feature loss in a large Bevy simulation engine.

## When to use

- Before deleting, renaming, or consolidating modules/systems/resources
- When code looks unused, broken, or "temporary"
- When migrations leave shims, mirrors, or compat bridges
- When assessing whether to **remove**, **refactor**, **preserve**, or **expand**

## Primary rule

Nothing is deleted until: intent understood · dependencies mapped · gameplay value evaluated · emotional simulation impact analyzed · migration status verified · replacement paths confirmed.

## Quick workflow

1. Read [`prompts/llm_agent_brief.md`](prompts/llm_agent_brief.md) (token contract, layer stack, verify ✅ in `src/`).
2. Read full rules: [reference.md](reference.md).
3. Classify each candidate: **A** obsolete · **B** transitional · **C** dormant gameplay · **D** incomplete infrastructure.
4. Build `dependency_graph` (readers, writers, migration targets, gameplay deps).
5. Score `feature_value` (gameplay, atmosphere, ECS reuse, future modularity).
6. Check **other agent activity** — do not delete mid-migration (viewport, extraction, GPU parity).
7. Output compressed decision (see reference) — never dump large code blocks.
8. Prefer **completion_plan** over deletion when value is high.

## Decision outputs (YAML snippets)

```yaml
remove: { safe: true, migration_dependency: false, gameplay_value: none }
refactor: { authority_conflict: true, gameplay_value: high }
preserve: { future_system_anchor: true, emotional_depth: high }
expand: { simulation_synergy: high, ecs_scalability: high }
completion_plan: { target: "...", missing: [...], dependencies: [...] }
```

## ECS never-remove (without successor)

Authority boundaries · isolation scaffolds · extraction contracts · cleanup systems · scheduling guards · sync witnesses.

## Route conflicts

| Situation | Delegate |
|-----------|----------|
| Authority / schedule ambiguity | `@planner` |
| Multi-phase cleanup + implementation | `@orchestrator` |
| Shim removal + replacement code | `@coder` |
| Atmospheric/overlay gameplay value | `@designer` |
| All three skills + Task-blocked sequential work | **`@sim-steward`** — [`.cursor/agents/sim-steward.md`](../../agents/sim-steward.md) |

## Token discipline

Compress findings to structured YAML/bullets. Follow `llm_agent_brief.md`: cite `path` + `Symbol`, ≤~10 lines, no log dumps.

## Additional resources

- Full philosophy, categories, emotional tags, matrices: [reference.md](reference.md)
- Source drafts: `prompts/rough_agents/cleanup_a1.skill.md`, `draft_agent_cleanup_intel_info.md`
