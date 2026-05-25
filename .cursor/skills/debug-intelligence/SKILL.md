---
name: debug-intelligence
description: Reads diagnostics, compresses engine debug state into ECS-aware intelligence, traces authority drift, and routes findings to specialist agents for Rust_engine_template_01. Use when interpreting witness JSON, viewport/render drift, multi-writer resources, migration VM debt, or noisy logs — does not fix systems directly.
disable-model-invocation: true
---

# Debug Intelligence Orchestrator

Converts noisy engine state into **compressed, ECS-aware operational intelligence**. Does **not** fix systems directly — extracts meaning, preserves context, routes work.

## When to use

- Witness JSON / debug overlays / trace interpretation
- Authority drift, dual writers, viewport or render contract mismatch
- Migration tracking (VM-*), shim permanence, orphaned diagnostics
- Before `@coder` or `@planner` works on viewport/render/ECS bugs

## Quick workflow

1. Read [`prompts/llm_agent_brief.md`](prompts/llm_agent_brief.md) and [reference.md](reference.md).
2. **Never dump full logs** — summarize, compress, semantic deltas only.
3. Run evidence pipeline: `raw logs → extraction → authority analysis → compression → ECS classification → routing package`.
4. Emit one of: authority drift · render contract mismatch · ECS authority graph · delegation block.
5. Route to `@planner` (architecture), `@coder` (fix), `@designer` (overlay UX), `@orchestrator` (multi-domain), or **`@sim-steward`** (all three skills + sequential shifts when Task blocked — [`.cursor/agents/sim-steward.md`](../../agents/sim-steward.md)).
6. Update `persistent_engine_knowledge` mental model (Tier 1–3, see reference).

## Primary debug targets (repo)

| Area | Paths | Watch for |
|------|-------|-----------|
| View authority | `src/gui/view_authority.rs`, `src/gui/view_projection_authority.rs` | dual writes, lockstep, stale mirrors, hidden globals |
| Viewport pipeline | `src/render/viewport_pipeline.rs`, `src/gui/authoritative_viewport.rs` | semantic/render mismatch, drift, rescue-floor, stale propagation |
| Map view | `src/gui/map_view/` | presentation leaks, texture mismatch, preview/minimap bleed |
| Projection graph | `src/render/extraction/render_projection_graph.rs`, `src/render/fire_view_extract.rs` | tactical-only assumptions, ViewId bypasses, shared overlay hazards |

## Output template

```yaml
issue:
  id: VM-XX-DRIFT-001
  severity: HIGH | MED | LOW
root_cause: [...]
affected: [...]
evidence: [compressed bullets]
recommendation: [...]
owner: planner | coder | designer | orchestrator
confidence: 0.0-1.0
delegation:
  target_agent: coder
  reason: [...]
  files: [...]
```

## Token tiers

- **Tier 1** — permanent architectural truths (long-term)
- **Tier 2** — transitional migration state (VM-*)
- **Tier 3** — volatile frame diagnostics (discard after routed)

## Task quota fallback

If Cursor **Task** subagents hit quota, route via **@coder** / **@planner** in main chat per `prompts/guides/subagent_continuity_playbook_v1.md` — do not retry Task.

## Additional resources

- Full routing, parallel analysis model, ECS rules: [reference.md](reference.md)
- Source drafts: `prompts/rough_agents/debug_intel_a1.skill.md`, `draft_agent_debug_intelother_info.md`
