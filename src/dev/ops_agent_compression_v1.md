# OPS agent compression layer `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-AGENT-COMPRESSION-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (L1750–L1832, L2084–L2100) |
| **Parent** | $ref:src/dev/plan_ops_metalogic_split_v1.md |

**ROI thesis:** Biggest token savings = **compressed intelligence queries**, not better prompts. Target **~20 tokens vs ~20,000** for session orientation.

---

## Stack (target)

```text
User → Supervisor → MCP/BLANG → [Postgres fn | JSON rollup] → Compressed State → Agent
```

**Phase 0–2 (now):** JSON rollups + existing MCP tools.  
**Phase 4 (gate):** Postgres functions behind same MCP names.

---

## §project-brief-shape (agent context packet)

What the agent should see **instead of** read 80 files:

```json
{
  "schema": "ops_project_brief_v1",
  "project": "Rust_engine_template_01",
  "quality_score": 74,
  "utility_score": 61.2,
  "auth_spine": "MAT★⇢APS★⇢SNAP★⇢WRK★⇢ATL★⇢RT★",
  "known_failures": [
    "material_authority",
    "preview_gap"
  ],
  "top_failures_ranked": [
    {"id": "material_authority", "severity": "P1"},
    {"id": "preview_missing", "severity": "P1"},
    {"id": "stale_snapshots", "severity": "P2"}
  ],
  "recent_improvements": [
    "APS-MAT-008",
    "BUILD-WORKER-004"
  ],
  "suggested_focus": "Preview authority validation",
  "active_picks": {
    "@coder": "TRIAGE-FIRE-EXTRACT-FINAL-001",
    "@designer": "SIM-HUD-PRODUCT-CLOSE-001",
    "Operator": "G-PLAY-01"
  },
  "last_20_runs_summary": "…",
  "metrics_tier1": {
    "q_per_token": 0.004,
    "ftr": 0.72,
    "rtr": 0.18
  }
}
```

**Witness:** `debug_runs/agent_ops/ops_project_brief_v1.json` (emit from scan script).

---

## JSON-first equivalents (before Postgres)

| Draft `fn_*` | Now (Part B) | Later (Part A) |
|:---|:---|:---|
| `fn_project_brief(project)` | `ops_report_latest.json` + HANDOFF header | SQL function |
| `fn_agent_context(id)` | `handoff_brief()` + `agent_queue_next` | SQL function |
| `fn_quality_summary(project)` | `witness_brief` + unified index | SQL function |
| `fn_failure_patterns(agent)` | `@debug-intelligence` compress | SQL function |
| `fn_token_budget(agent)` | `token_savings_guide()` | SQL function |
| `fn_cost_quality_ratio(run)` | `ops_metrics_tiers` Q/$ | SQL function |
| `fn_tool_recommendations(agent)` | MCP registry + past `tool_usage` | SQL function |
| `fn_recent_decisions(project)` | queue `done[]` last N | SQL function |
| `fn_authority_violations(run)` | viewport/render drift witnesses | SQL function |
| `fn_novelty_score(response)` | defer — low ROI | optional |
| `fn_retry_guidance(task)` | `BLANG:Q+` + iteration knee | SQL function |

**Rule:** MCP tool names **stable** — swap JSON backend for Postgres without agent prompt changes.

---

## Feedback topology

```text
RUN → TRACE → STORE → AGGREGATE → SUPERVISOR → PROMPTΔ → NEXT RUN
```

| Stage | Repo |
|:---|:---|
| RUN | Cursor session / Task |
| TRACE | `agent_run_append` |
| STORE | `debug_runs/agent_ops/` |
| AGGREGATE | `ops_intelligence_scan.ps1` |
| SUPERVISOR | `@operations-intelligence` |
| PROMPTΔ | `session_bootstrap_agent_prompts_v1.md` |

TRACE fields: tokens · tools · files · duration · failures · outcome.

---

## Agent routing engine (draft L2007–L2034)

**Classifier → historical success matrix → best agent.**

| Task class | Route when success high |
|:---|:---|
| Architecture | `@planner` |
| Implementation | `@coder` |
| Review / witness | `@sim-steward` |
| Presentation | `@designer` |
| MCP batch | `@coder-mcp` |

Store matrix in `ops_report_latest.json` → `routing.success_rate_by_agent_task`.

**Do not** auto-route without witness-backed history — cold start uses HANDOFF picks.

---

## ROI ranking (draft L2084–L2098)

| Layer | ROI | Build order |
|:---|:---:|:---|
| Postgres functions (via MCP) | ████████████ | Phase 4 |
| Telemetry DB | ██████████ | Phase 1–2 JSON |
| Agent routing | █████████ | Phase 2 matrix |
| Prompt evolution | ███████ | PROMPTΔ |
| Full agent swarms | ███ | **avoid** |
| Always-on LLM NPCs | █ | sim only on trigger |

---

## BLANG entry ritual

```text
BLANG:PRE → BLANG:OPS → BLANG:HO → work
```

| Token | Emits |
|:---|:---|
| `BLANG:OPS` | refresh `ops_project_brief_v1.json` |
| `BLANG:HO` | `handoff_brief()` digest |
| `BLANG:WIT` | witness keys only |

**Forbidden:** `Read` full draft · `Read` 80 files for orientation.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Compression layer from draft L1750+ |
