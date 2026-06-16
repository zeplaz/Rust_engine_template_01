# PLAN-OPS-METALOGIC-SPLIT-001 — Two-part architecture from economy draft `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-OPS-METALOGIC-SPLIT-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (from L1390 — governance / SQL / utility) |
| **Parent** | $ref:src/dev/plan_agent_operations_intelligence_v1.md |
| **Lang** | $ref:src/dev/agent_lang_v1.md |
| **Date** | 2026-06-08 |
| **Planner** | **SIGNED** |

**Read scope:** L1390–end — governance, SQL observation, utility, **compression layer**, Postgres `fn_*` via MCP, metric tiers 1–3, routing matrix, ROI ranking, sim emotional graph (L1713+).

---

## Executive split

The tail of the draft is **not one system**. It is two programs that share telemetry but differ in **consumer** and **schema depth**:

| Part | Name | Question it answers | When to build |
|:---|:---|:---|:---|
| **B** | **OPS-METALOGIC** — symbolic governance + evolution loop | *What ΔWF should @orchestrator issue next?* | **Now** — JSON witnesses + BLANG |
| **B′** | **OPS-COMPRESSION** — MCP function layer + project brief | *20 tokens not 20k files?* | **Now** — compose existing MCP tools |
| **A** | **AOI-SQL** — Postgres `fn_*` behind same MCP names | *Which prompt/tool/model improved Q per $C?* | Phase 4 gate (>500 events) |

```text
User → Supervisor → MCP/BLANG → [JSON brief | Postgres fn] → Agent
              │
              └── RUN → TRACE → STORE → KPI → PROMPTΔ ↺
```

**Do not** let agents generate SQL. **Do not** build Postgres before JSON brief proves value. **Do not** chain LLM→LLM→LLM re-explaining reality (draft L1734).

---

## Part A — AOI-SQL (PostgreSQL observation system)

### What the draft claims (L1438–L1496)

PostgreSQL is not “storage” — it is a **mathematical observation system** over:

| Table | Purpose |
|:---|:---|
| `agent_run` | tokens, cost, duration, quality_score, bug_score, review_score |
| `tool_usage` | per-run tool calls, success/failure, latency |
| `file_reads` | path, reason, bytes, time |
| `feedback` | reviewer, score, issue_type |

Example queries: quality per dollar by `prompt_id`; files correlated with bug fixes.

### Repo alignment (honest)

| Draft | Already on disk | Gap |
|:---|:---|:---|
| `agent_run` | `debug_runs/agent_ops/agent_run_event_v1.json` (planned) | No append hook yet |
| `tool_usage` | `_agent_meta.agent_commands` in witness JSON | Not normalized |
| `file_reads` | `agent_doc_touch` / BLANG digest logs | No run_id join |
| `quality_score` | witness `green` + slice keys | Not scalar Q yet |

### Part A deliverables (when gated)

| Artifact | Path | Owner |
|:---|:---|:---|
| Schema spec | `src/dev/ops_sql_schema_v1.md` | @planner |
| Migrations | `tools/ops/db/migrations/` | @coder-mcp (optional) |
| Ingest CLI | `tools/ops/scripts/ingest_agent_events.py` | @coder-mcp |
| Materialized views | `ops_slutsky_decomposition_v1.sql` | @operations-intelligence review |
| MCP wrapper | MCP-OPS-REPORT-001 (deferred) | @coder-mcp |

### Core SQL views (from draft)

**Slutsky-style attribution** — store run lineage so:

```text
ΔQ = ΔPrompt + ΔModel + ΔTool + ΔContext + ΔReview
```

View: `v_quality_attribution` — compare child run to `parent_run_id` holding other dimensions fixed.

**Jacobian proxies** (L1550–L1590) — do not compute real partial derivatives day one; use **finite differences** between tagged A/B runs:

| Symbol | Stored column | Meaning |
|:---|:---|:---|
| `∂Q/∂Prompt` | `delta_q_prompt` | Q change when only prompt version changes |
| `∂Q/∂Tool` | `delta_q_tool` | Q change when tool policy changes |
| `∂C/∂Tool` | `delta_c_tool` | Cost change for same |

**Utility gate** (L1713–L1730): $ref:src/dev/ops_utility_function_v1.md

**Postgres function layer** (L1788–L1801): $ref:src/dev/ops_sql_schema_v1.md§functions · exposed via $ref:src/dev/ops_mcp_function_layer_v1.md

**Metric tiers** (L1864–L1968): $ref:src/dev/ops_metrics_tiers_v1.md — Q/T, FTR, dQ/dT, review-loop knee

### Part A — do not start until

- [ ] Phase 1 JSON events flowing (`plan_agent_operations_intelligence_v1.md` §Phase 1)
- [ ] `ops_report_latest.json` stable 30 days OR >500 events
- [ ] Complexity budget review ≥ 1.0

---

## Part B — OPS-METALOGIC (governance without Postgres)

### What the draft claims (L1391–L1437, L1674–L1711)

Replace linear **Agent → Prompt → Answer** with:

```text
RUN → TEL → KPI → SUP → PROMPTΔ ↺
         │
    Governance Core (meta supervisor)
         │
    PLAN · COD · DES · ANA · VAL · WIT
```

**Goal:** not better single answers — **better evolution of answer production**.

### Mapped to this repo (concrete)

| Draft node | Repo anchor | Symbol |
|:---|:---|:---|
| Governance Core | `@orchestrator` + `@operations-intelligence` | `OPS★` |
| EVENTS / TELEMETRY | `debug_runs/` + `_agent_meta` | `TEL` |
| Meta Analytics | `ops_intelligence_scan.ps1` | `KPI` |
| Optimization Engine | HANDOFF + queue `BLANG:Q✓` | `ΔWF` |
| Prompt Evolution | `session_bootstrap_agent_prompts_v1.md` | `PROMPTΔ` |
| Emotional layer (sim) | Track C / fire ecology / faction (future) | `E★` |
| Narrative layer (sim) | `play_scenario`, grammar graph | `GRAPH★` |

### Part B artifact tree (markdown + tools)

**Normative docs** (read before implement):

| File | Purpose |
|:---|:---|
| `src/dev/ops_metalogic_lexicon_v1.md` | Dense map: AUTH, LOOP, J(Q), U(agent), E-vector |
| `src/dev/ops_utility_function_v1.md` | **U(agent)** + λ defaults + anti-pattern |
| `src/dev/ops_metrics_tiers_v1.md` | Tier 0–2 metrics + review-loop knee |
| `src/dev/ops_agent_compression_v1.md` | **project brief** shape + JSON fn parity |
| `src/dev/ops_mcp_function_layer_v1.md` | MCP tool catalog → Postgres fn |
| `src/dev/ops_metrics_goodhart_guard_v1.md` | Observable facts · denylist · quality_signal |
| `src/dev/ops_truth_memory_split_v1.md` | **Filesystem-first** · Tier 1/2/3 · claim_task · ARA |
| `src/dev/ops_sql_workstation_arch_v1.md` | Local PG · schema namespaces · repo metric map |
| `src/dev/ops_slutsky_attribution_v1.md` | How to tag runs for ΔQ decomposition *(pending)* |
| `src/dev/ops_dsm_supervisor_loop_v1.md` | RUN→TRACE→STORE ritual *(pending)* |

**Tools** (JSON-first, no SQL):

| Tool | Path | Emits |
|:---|:---|:---|
| `ops_intelligence_scan` | `tools/orchestrator/scripts/ops_intelligence_scan.ps1` | `ops_report_latest.json` |
| `ops_witness_rollup` | `tools/mcp/python/rust_engine_mcp/ops_witness_rollup.py` | DSM block for HANDOFF |
| `agent_run_append` | MCP registry (exists) | `agent_ops/*.json` |
| `handoff_ops_footer` | `invoke_handoff.ps1 -OpsEvent` | run event on session close |

**Witness targets:**

| Witness | Keys |
|:---|:---|
| `debug_runs/agent_ops/ops_report_latest.json` | `utility_score`, `metrics_tier1`, `routes[]` |
| `debug_runs/agent_ops/ops_project_brief_v1.json` | compressed agent context packet |
| `debug_runs/agent_ops/ops_mcp_function_layer_live.json` | MCP wrapper smoke |

---

## AGENT-LANG integration (improvements over raw draft)

The draft uses prose and ASCII art. **This repo already has a compression layer** — extend it instead of parallel vocab.

### 1. AUTH spine = pipeline state (not duplicate)

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★
LOOP: RUN⇢TEL⇢KPI⇢OPS★⇢ΔWF↺
```

Store in HANDOFF header + `ops_report_latest.json` — one line, not paragraphs.

### 2. `$ref` for all OPS docs

```text
$ref:src/dev/ops_utility_function_v1.md§lambda-defaults
$ref:debug_runs/agent_ops/ops_report_latest.json
```

Agents resolve via `agent_doc_touch(intent=ref)` — never Read full draft.

### 3. `$sym` for measurable fields

| $sym | Path / witness |
|:---|:---|
| `$sym:AgentRunEvent@debug_runs/agent_ops/agent_run_event_v1.json` | event schema |
| `$sym:UtilityScore@ops_report_latest.json` | U(agent) rollup |
| `$sym:SlutskyDeltaQ@ops_slutsky_attribution_v1.md` | attribution rules |
| `$sym:VisibleFireChunkSet@src/render/fire_view_extract.rs` | sim extract (not OPS — example boundary) |

### 4. `⟨ID⟩` for OPS programs

| ⟨ID⟩ | Part |
|:---|:---|
| `⟨PLAN-OPS-INTELLIGENCE-001⟩` | Parent program |
| `⟨AOI-SQL-001⟩` | Part A (deferred) |
| `⟨OPS-METALOGIC-001⟩` | Part B (active) |
| `⟨MCP-OPS-REPORT-001⟩` | SQL ingest wrapper (defer) |

### 5. BLANG tokens for OPS ritual

| Token | OPS meaning |
|:---|:---|
| `BLANG:OPS` | `ops_intelligence_scan` |
| `BLANG:WIT` | witness brief + append `_agent_meta` |
| `BLANG:Q✓` | close queue row + emit `agent_run_event` |
| `BLANG:MARK` | `agent_marker_append` with `⟨BP:SHARE⟩` |

### 6. Status emoji on OPS surfaces

| Sym | Use |
|:---|:---|
| 🟢 | witness green + U ≥ threshold |
| 🟡 | qualified (Q up, Ct high — Jacobian bad trade) |
| 🔴 | authority drift / GRAPH⛔ |
| 🧊 | AOI-SQL deferred |

### 7. Emotional grammar — **sim domain only**

Draft L1592–L1648 (`E=[trust,fear,…]`, event→ΔE) applies to **Track C narrative / NPC sim**, not agent routing.

| Layer | Symbol | Repo |
|:---|:---|:---|
| Agent OPS emotion | `E★ confusion_risk` | APS grey slab, wrong keyframe |
| Sim NPC emotion | `E(t)` vector | deferred — `stage7_behavioral_*` |

**Do not** merge these in one SQL table without `domain: agent | sim` column.

---

## Implementation order (Part B first)

| Cycle | Deliverable | Agent |
|:---:|:---|:---|
| 1 | Lexicon + utility + metrics + compression + MCP layer docs | @planner **done** |
| 2 | `ops_intelligence_scan` emits `ops_project_brief_v1.json` | @coder-mcp |
| 3 | MCP `ops_get_project_brief` composes existing tools | @coder-mcp |
| 4 | `agent_run_append` on every `BLANG:Q✓` | @coder-mcp |
| 5 | `handoff_registry_v1.json` + `ops_claim_task` MCP | @coder-mcp |
| 6 | Gate → Postgres index-only + watcher | after >500 events |

---

## What to skip / rename

| Item | Action |
|:---|:---|
| `docs/reference/outside/effwecny_mpc_draft.md` | Keep as **archive draft**; normative work moves to `src/dev/ops_*` |
| Hidden subagent strata (L1232–L1298) | **Conceptual only** — map to existing `@coder` / `@sim-steward`; do not spawn Σψ agents |
| PostgreSQL day one | **Defer** per existing ops plan |
| EGUI-QC / warehouse in OPS | Out of scope — use territory matrix |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Split Part A SQL vs Part B metalogic; AGENT-LANG overlay |
| v1.1.0 | 2026-06-08 | L1713+ — compression layer, MCP fn catalog, metric tiers, routing ROI |
| v1.2.0 | 2026-06-08 | L2104+ — Goodhart guard, workstation SQL arch, KE/CHL, quality_signal |
| v1.3.0 | 2026-06-08 | L2450+ — truth vs memory · handoff registry · claim_task · watchers |
