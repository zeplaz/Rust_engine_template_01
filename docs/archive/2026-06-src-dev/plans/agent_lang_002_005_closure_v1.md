# AGENT-LANG-002→005 — program closure `v1`

| Field | Value |
|:---|:---|
| **⟨Program⟩** | ⟨PLAN-MCP-AGENT-LANG-001⟩ |
| **Spec** | $ref:src/dev/agent_lang_v1.md |
| **Board** | $ref:src/dev/master_chain_board_4d_v1.md — Chain **C** φ2 🟢 |
| **Planner-mcp** | **SIGNED** |
| **Date** | 2026-06-03 |

---

## Slice closure

| ⟨ID⟩ | φ | Deliverable |
|:---|:---:|:---|
| ⟨AGENT-LANG-001-SPEC⟩ | 🟢 | $ref:src/dev/agent_lang_v1.md |
| ⟨AGENT-LANG-002-REF⟩ | 🟢 | $ref:docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_coder_dispatch_20260603_v1.md · grammar_iter · bevy_hud orders |
| ⟨AGENT-LANG-003-BLANG⟩ | 🟢 | `token_savings_guide().blang` · $ref:tools/mcp/MICRO_TOOLS_REGISTRY_v1.md§Tier-1a-BLANG |
| ⟨AGENT-LANG-004-RITUAL⟩ | 🟢 | $ref:.cursor/agents/orchestrator.md · coder-mcp · coder · planner-mcp |
| ⟨AGENT-LANG-005-HANDOFF⟩ | 🟢 | $ref:tools/orchestrator/queues/HANDOFF.md v2.0.0 |
| ⟨AGENT-LANG-006-INFRA-REF⟩ | 🟢 | $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md · $ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md |
| ⟨MCP-PRODUCTIVITY-P1-PLAN⟩ | 🟢 | $ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md |

**Policy:** Delta `$ref` / `$sym:` only — **do not** rewrite coder-mcp queue rows.

---

## BLANG ritual (enforced)

```text
BLANG:PRE → BLANG:Q+ → L4 work → L5 tools → L6 WIT → BLANG:Q✓
Orient: BLANG:HO · BLANG:DOC — not full HANDOFF/witness Read
```

---

## Next REF targets (maintenance — not blocking)

| Priority | Doc | Pass |
|:---:|:---|:---|
| 1 | $ref:src/dev/development_plan_index.md | hub `$ref` trim |
| 2 | $ref:AGENTS.md | BLANG index line |
| 3 | $ref:docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_collective_dispatch_20260607_v1.md | Chain rows only |

**Idle block:** $ref:docs/archive/2026-06-src-dev/plans/planner_mcp_maintenance_idle_v1.md — paste when `BLANG:Q+("planner-mcp")` returns idle.

**Stop:** Chain C `maintain_only` — planner-mcp picks MCP-PRODUCTIVITY-P1-PLAN only on explicit orchestrator order.

---

## Changelog

| Ver | Date |
|:---|:---|
| v1.0.0 | 2026-06-03 |
