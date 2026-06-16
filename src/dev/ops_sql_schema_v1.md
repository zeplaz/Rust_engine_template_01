# AOI-SQL schema outline `v1` (Part A — **DEFERRED**)

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **AOI-SQL-001** |
| **Status** | **DEFERRED** — gate: $ref:src/dev/plan_ops_metalogic_split_v1.md§Part-A |
| **Parent** | $ref:src/dev/plan_agent_operations_intelligence_v1.md §Phase 4 |

**Do not implement** until JSON event volume proves need. This doc exists so Part B events have a **stable target shape**.

**Architecture:** $ref:src/dev/ops_sql_workstation_arch_v1.md · **Metrics guard:** $ref:src/dev/ops_metrics_goodhart_guard_v1.md

---

## Gate

- [ ] `agent_run_event_v1` appending in `debug_runs/agent_ops/`
- [ ] >500 events OR 30 days stable `ops_report_latest.json`
- [ ] Complexity budget ≥ 1.0

---

## Tables (from draft L1444–L1478)

### `agent_run`

| Column | Type | Notes |
|:---|:---|:---|
| `id` | uuid | PK |
| `parent_run_id` | uuid | nullable — iteration lineage |
| `agent` | text | `@coder`, `coder-mcp`, … |
| `model` | text | |
| `task_id` | text | `⟨APS-PREVIEW-001⟩` |
| `prompt_version` | text | hash or semver |
| `tokens_in` | int | |
| `tokens_out` | int | |
| `cost_usd` | numeric | |
| `duration_ms` | int | |
| `quality_signal` | text | derived — see goodhart guard |
| `validator_passed` | bool | |
| `designer_approved` | bool | |
| `runtime_pass` | bool | |
| `reopened` | bool | |
| `rework_count` | int | |
| `files_modified` | int | |
| `result` | text | success / fail / partial |
| `status` | text | success / fail / partial |
| `changed_dimension` | text | slutsky tag: prompt / model / tool / context / review |
| `created_at` | timestamptz | |

### `tool_usage`

| Column | Type |
|:---|:---|
| `run_id` | uuid FK |
| `tool` | text |
| `calls` | int |
| `success` | int |
| `failure` | int |
| `latency_ms` | int |

### `file_reads`

| Column | Type |
|:---|:---|
| `run_id` | uuid FK |
| `path` | text |
| `reason` | text |
| `bytes` | int |
| `time_ms` | int |

### `feedback`

| Column | Type |
|:---|:---|
| `run_id` | uuid FK |
| `reviewer` | text |
| `event_type` | text | observable only — no self score |
| `issue_type` | text |

### `handoff_registry` (Tier 2 — index only)

| Column | Type | Notes |
|:---|:---|:---|
| `id` | serial | |
| `path` | text | FS path — **content not in DB** |
| `owner` | text | |
| `status` | text | open / blocked / review / stale / done |
| `priority` | int | |
| `review_count` | int | |
| `last_agent` | text | |
| `blocked_by` | text | |
| `last_read_at` | timestamptz | |
| `last_review_at` | timestamptz | |
| `source_files_changed` | int | since last_review |
| `created_at` / `updated_at` | timestamptz | |

### `task_dependency`

| Column | Type |
|:---|:---|
| `task_id` | text |
| `depends_on_task_id` | text |

**Interim JSON:** `debug_runs/agent_ops/handoff_registry_v1.json`

---

## Views (v1)

| View | Purpose |
|:---|:---|
| `v_quality_per_dollar` | `AVG(quality_score / NULLIF(cost_usd,0))` by `prompt_version` |
| `v_slutsky_attribution` | child vs parent on tagged dimension |
| `v_jacobian_proxy` | finite-diff from paired runs |
| `v_utility_rank` | `U = Q − λ·Ct − μ·Cm − ν·Dp` per agent/task |

---

## Functions (draft L1788–L1801)

**Agents never write SQL.** MCP tools call these functions.

| Function | Returns | MCP tool |
|:---|:---|:---|
| `fn_agent_context(run_id)` | agent + task + parent chain | `ops_get_agent_context` |
| `fn_project_brief(project)` | Q, failures, focus, picks | `ops_get_project_brief` |
| `fn_quality_summary(project)` | rollup by lane | `ops_get_quality_summary` |
| `fn_failure_patterns(agent)` | top failure classes | `ops_get_failure_patterns` |
| `fn_token_budget(agent)` | remaining budget hint | `ops_get_token_budget` |
| `fn_cost_quality_ratio(run_id)` | Q/$ Q/T | `ops_get_cost_quality_ratio` |
| `fn_tool_recommendations(agent)` | ranked tools | `ops_get_tool_recommendations` |
| `fn_recent_decisions(project)` | last N queue closes | `ops_get_recent_decisions` |
| `fn_authority_violations(run_id)` | drift flags | `ops_get_authority_violations` |
| `fn_novelty_score(response_hash)` | defer low ROI | optional |
| `fn_retry_guidance(task_id)` | loop count + knee | `ops_get_retry_guidance` |
| `fn_recent_failures(project)` | failure patterns | `ops_get_recent_failures` |
| `fn_ship_gate_status(batch_id)` | ship_allowed rollup | `ops_get_ship_gate_status` |
| `fn_project_health(project)` | quality_signal + trends | `ops_get_project_health` |
| `fn_agent_efficiency(agent)` | KE + Q/T | `ops_get_agent_efficiency` |
| `fn_unresolved_regressions()` | reopened queue rows | `ops_get_unresolved_regressions` |

Catalog: $ref:src/dev/ops_mcp_function_layer_v1.md

### Schema namespaces (gate)

`telemetry` · `decisions` · `witnesses` · `quality` · `failures` · `trends` — $ref:src/dev/ops_sql_workstation_arch_v1.md§Schema-namespaces

**JSON-first:** same tool names return rollups from `debug_runs/agent_ops/` until gate opens.

---

## Ingest

```text
tools/ops/scripts/ingest_agent_events.py  ← debug_runs/agent_ops/*.json
```

MCP: `⟨MCP-OPS-REPORT-001⟩` — wrapper only after schema frozen.

---

## AGENT-LANG

- Events use `⟨ID⟩` for `task_id`
- Ingest logs `$ref:debug_runs/agent_ops/ops_report_latest.json` on completion
- **Separate** `domain` column: `agent` | `sim` when emotional tables added

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Outline only — Part A deferred |
| v1.1.0 | 2026-06-08 | Postgres fn catalog from draft L1788+ |
| v1.2.0 | 2026-06-08 | L2104+ namespaces, observable fields, Goodhart denylist |
| v1.3.0 | 2026-06-08 | L2450+ handoff_registry · task_dependency · witness index-only |
