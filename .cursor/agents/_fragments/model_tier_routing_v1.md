# Model tier routing `v1`

**Control plane:** MCP `agent_flow_route` / CLI `agent-flow-route` · agent `@flow-controller`  
**Policy surface:** `agent_flow_policy()` · also listed in `token_savings_guide()` under `briefs.agent_flow`

## Law

```text
L0 CHEAP  (composer-2.5-fast / explore / ops briefs)
   └─ research_packet ─⬡[hard_gate?]▶
L1 HARD   (claude-opus-5-thinking-high / planner|debug|ops|designer-mcp)
   └─ decision_packet ─▶
L2 EXEC   (inherit / @coder|@coder-mcp|@designer|@sim-steward)
   └─ WIT-HON → Q✓
```

## Hard-gate triggers (any ⇒ L1)

`authority` · `dual writer` · `architecture` · `migration` · `conflict` · `EV/Cx` · `contested` · `root cause` · `tradeoff` · `design fork` · `schedule` · `SystemSet` · `viewport drift` · `render contract`

Override: `--force-hard` / `--skip-hard`.

## Domain → owners

| Domain | L0 | L1 | L2 |
|:---|:---|:---|:---|
| fire | explore | debug-intelligence | coder / sim-steward |
| ui | explore | planner | coder / designer |
| render | explore | debug-intelligence | coder / sim-steward |
| art | explore | designer-mcp | coder-mcp / designer-mcp |
| ops | operations-intelligence | operations-intelligence | orchestrator |

## Parent rules

1. Call `agent_flow_route` **before** Task spawn.
2. Never run L1/L2 models on `file_digest` / queue / ops briefs / **deterministic lints**.
3. Prefer Python/MCP refactors (`terrain-honesty-lint`, `pilot_hardcode_lint`) over LLM search-replace for honesty/rename sweeps.
4. Task usage error → same packet on `@chat_agent` in foreground (no Task retry).
5. Parallel Tasks only if ¬file∩ ∧ ¬authority∩.

**CB-notation:** `$ref:prompts/guides/cb_notation_playbook_v1.md` (amortize concepts; sacred tool names; blend ≠ pure glyph).
