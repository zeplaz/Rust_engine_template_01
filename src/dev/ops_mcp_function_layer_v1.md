# OPS MCP function layer `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-MCP-FUNCTION-LAYER-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (L1781–L2006) |
| **Parent** | $ref:src/dev/ops_agent_compression_v1.md |
| **Implementer** | @coder-mcp (⟨MCP-OPS-REPORT-001⟩ family) |

Agents call **MCP tools** — never generate SQL. MCP calls Postgres function (later) or JSON rollup (now).

---

## Tool catalog

| MCP tool | Maps to | Backend (now) | Backend (gate) |
|:---|:---|:---|:---|
| `ops_get_project_brief` | `fn_project_brief` | `ops_project_brief_v1.json` | `fn_project_brief()` |
| `ops_get_agent_context` | `fn_agent_context` | `handoff_brief` + queue | SQL |
| `ops_get_quality_summary` | `fn_quality_summary` | `witness_brief` rollup | SQL |
| `ops_get_failure_patterns` | `fn_failure_patterns` | drift witness scan | SQL |
| `ops_get_token_budget` | `fn_token_budget` | `token_savings_guide` | SQL |
| `ops_get_cost_quality_ratio` | `fn_cost_quality_ratio` | metrics tier1 Q/$ | SQL |
| `ops_get_tool_recommendations` | `fn_tool_recommendations` | registry + history | SQL |
| `ops_get_recent_decisions` | `fn_recent_decisions` | queue done[] | SQL |
| `ops_get_authority_violations` | `fn_authority_violations` | debug-intelligence index | SQL |
| `ops_get_retry_guidance` | `fn_retry_guidance` | iteration + dQ/dT knee | SQL |
| `ops_get_quality_trends` | draft `mcp.get_quality_trends` | `ops_report_latest` series | SQL |
| `agent_run_append` | telemetry ingest | **shipped** JSON append | SQL insert |
| `ops_get_recent_failures` | `fn_recent_failures` | delta_wf + failures | SQL |
| `ops_get_ship_gate_status` | `fn_ship_gate_status` | tile index ship_allowed | SQL |
| `ops_get_project_health` | `fn_project_health` | ops_report + quality_signal | SQL |
| `ops_get_agent_efficiency` | `fn_agent_efficiency` | KE + Q/T rollup | SQL |
| `ops_get_unresolved_regressions` | `fn_unresolved_regressions` | queue reopened rows | SQL |

**Cycle 2 priority:** `ops_get_project_brief` only. Rows above = **gate** (S3).

### Queue / handoff routing (L2450+ — gate T2)

| MCP tool | Purpose | Backend (now) |
|:---|:---|:---|
| `ops_claim_task(agent)` | one task + path to read | `agent_queue_next` + phase3 queue |
| `ops_get_active_handoffs()` | open/blocked/review ids | `handoff_registry_v1.json` |
| `ops_stale_handoffs()` | freshness without Read | registry + file mtimes |
| `ops_next_review(agent)` | highest stale row | registry sort |
| `ops_changed_dependencies(path)` | tasks `review_needed` | defer until watcher |
| `ops_agent_health()` | KE + ARA + U rollup | `ops_report_latest` |

**Rule:** tools return **ids + paths** — agent Reads **one** HANDOFF/slice file. Truth stays on disk: $ref:src/dev/ops_truth_memory_split_v1.md

---

## Existing MCP parity (do not duplicate)

| Need | Already |
|:---|:---|
| Doc digest | `agent_doc_touch`, `snapshot_digest` |
| Queue pick | `agent_queue_next` |
| Witness compress | `witness_brief` |
| Preflight | `pipeline_preflight` |
| Handoff | `handoff_brief` |

**New tools** = thin wrappers that return **ops_project_brief_v1** shape — compose existing tools server-side.

---

## Implementation sketch (@coder-mcp)

```text
tools/mcp/python/rust_engine_mcp/ops_intelligence.py
  ops_get_project_brief() -> dict   # composes scan + HANDOFF + index
  ops_get_retry_guidance(task_id) -> dict
```

CLI: `python -m rust_engine_mcp.cli ops-get-project-brief`

Witness: `debug_runs/agent_ops/ops_mcp_function_layer_live.json`

---

## JSON schema — `ops_project_brief_v1`

Normative shape: $ref:src/dev/ops_agent_compression_v1.md§project-brief-shape

---

## Gate

- [ ] Phase 1 `agent_run_append` flowing
- [ ] `ops_intelligence_scan.ps1` writes brief JSON
- [ ] Then add MCP wrappers (no Postgres required for v1 tools)

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | MCP catalog from draft Postgres fn list |
| v1.1.0 | 2026-06-08 | L2450+ claim_task · handoff registry · routing tools |
