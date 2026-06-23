# PLAN-OPS-INTELLIGENCE-001 — Operations Intelligence Agent `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-OPS-INTELLIGENCE-001** |
| **Sources** | [`docs/reference/outside/dsm_ops_subagent_tooling.ini`](../../docs/reference/outside/dsm_ops_subagent_tooling.ini) · [`docs/reference/outside/effwecny_mpc_draft.md`](../../docs/reference/outside/effwecny_mpc_draft.md) (archive) · **split:** [`plan_ops_metalogic_split_v1.md`](plan_ops_metalogic_split_v1.md) |
| **Agent** | `@operations-intelligence` — [`.cursor/agents/operations-intelligence.md`](../../.cursor/agents/operations-intelligence.md) |
| **Status** | **ACTIVE** — agent + Phase 1 telemetry **SHIPPED** (`ops_telemetry.py`, `ops_dashboard_live.json`) |
| **Date** | 2026-06-03 |

---

## What problem this solves

Two drafts describe the **same closed loop** in different vocabularies:

| Draft | Loop | Question |
|:---|:---|:---|
| **DSM OPS** | EDITOR → GRAPH → TRIGGER → RUNTIME → CAPTURE → ANALYTICS → EDITOR | Is the **simulation/narrative field** observable, triggerable, measurable? |
| **Economy MCP** | RUN → TEL → KPI → SUP → ΔWF → APS → … | Why did this run **cost** what it cost, and which workflow should change? |

This repo already has **half** of the instrumentation:

- Pipeline truth: `debug_runs/*.json`, `_agent_meta`, [`debug_run_envelope.rs`](debug_run_envelope.rs)
- Lane queues: `tools/orchestrator/queues/`, HANDOFF
- Authority docs: ARCH-MAT-001, three-track plan

Missing: a **readonly agent** that compresses witnesses + run history into **DSM surfaces** and **routing packets** — without building PostgreSQL on day one.

**Filesystem-first (L2450+):** Git owns truth; Postgres indexes operational state only — $ref:src/dev/ops_truth_memory_split_v1.md. Do **not** move assemblies, RON, or source into the database.

---

## Proposed agent: `@operations-intelligence`

**Role:** Pipeline + agent **operations analyst** (not implementer, not cheerleader).

**Does:**

1. Read witness JSON, HANDOFF, agent queue, `_agent_meta` index
2. Emit **DSM dense maps** (authority, flow, risk, cost, feedback) per [`effwecny_mpc_deaft.md`](../../docs/reference/outside/effwecny_mpc_draft.md) § DSM lexicon
3. Map pipeline nodes to repo paths (below)
4. Score proposals with **Complexity Budget** (value/complexity ratio)
5. Route **ΔWF** (workflow deltas) to `@orchestrator`, `@planner`, `@sim-steward`, `@debug-intelligence`

**Does not:**

- Write production code or MCP tools
- Mandate full AOI / PostgreSQL before JSON witnesses prove value
- Replace `@debug-intelligence` (ECS/viewport drift) or `@orchestrator` (sequencing)

**Model policy:** `model: auto` for routine scans; expensive model only for **periodic supervisor reviews** (weekly/monthly), not per file edit.

---

## Repo mapping (DSM OPS → this project)

### Pipeline spine (economy draft AUTH/FLOW rows)

**Live glyphs:** $ref:tools/orchestrator/queues/master_chain_tensor_v1.json — all MAT→RT nodes ★ as of DSM-SIGNOFF-001.

```text
AUTH: MAT★⇢APS★⇢SNAP★⇢WRK★⇢ATL★⇢RT★
FLOW: ART◇⇢APS⇢SNAP⇢WRK⇢PNG⇢ATL⇢RT
LOOP: RUN⇢TEL⇢KPI⇢OPS★⇢ΔWF↺⇢APS★
```

| Node | Authority | Repo anchor |
|:---|:---|:---|
| **MAT★** | Material profile catalog | `assets/materials/profiles/`, ARCH-MAT-001 |
| **APS★** | Artist authoring + grammar | `tools/mcp/art_pipeline_suite/` |
| **SNAP★** | Assembly snapshot truth | `assets/staging/assemblies/*.json`, `assembly_snapshot` schema |
| **WRK★** | Blender / Bevy workers | `tools/mcp/blender/`, `bevy_preview_worker`, `assembly-build-run` · $ref:debug_runs/dsm_signoff_001_live.json |
| **ATL★** | Tile atlas / PNG compression | `tile-atlas-pack`, `assets/staging/tiles/` · $ref:debug_runs/atl_sign_001_live.json |
| **RT★** | Runtime lookup / map stamp | Bevy registry, atlas stamp systems · $ref:debug_runs/rt_registry_001_live.json |
| **TEL** | Telemetry ingest | `debug_runs/`, `agent_debug_index.json` |
| **KPI** | Compressed metrics | OPS report JSON (Phase 1) |
| **OPS★** | This agent | `@operations-intelligence` |
| **ΔWF** | Workflow change proposals | HANDOFF + queue row updates |

### DSM OPS narrative layer (future / parallel)

Maps to **Track C** grammar + economy sim — not blocking warehouse spine:

| DSM OPS node | Repo hook (when mature) |
|:---|:---|
| GRAPH★ | Quest/faction graph — deferred; grammar `grammar_rule_chain` is **proto-graph** today |
| TRIGGER★ | Scenario scripts, sim events — `play_scenario`, fire ecology witnesses |
| CAPTURE★ | `debug_runs/*_live.json`, preview PNGs, agent transcripts |
| ANALYTICS★ | OPS intelligence reports |
| EDITOR★ | APS + map editor — human sculptor surface |

---

## Quality / cost / emotion fields (translated)

| Field | Pipeline meaning | Agent meaning |
|:---|:---|:---|
| **Q★ coherence** | SNAP valid + grammar chain complete | Witness `green: true`, validators passed |
| **Q★ stability** | No authority drift, no dual writers | `@debug-intelligence` Tier 1 clear |
| **C★ compute** | WRK bake/preview time | Blender/Bevy job duration in witness |
| **C★ tokens** | — | Agent run estimates (Phase 1 manual → Phase 2 hooks) |
| **E★ clarity** | APS preview visible | APS-PREVIEW-001, material browser scale |
| **E★ confusion_risk** | Grey slabs, fake keyframe labels | `mcp_pilot_grammar_001_rejected_live.json` class |

---

## Failure modes (observable checklist)

| Signal | DSM | Detection |
|:---|:---|:---|
| GRAPH desync | GRAPH⛔ | Placements without `material_profile`; grammar vs footprint mismatch |
| EMOTION flatline | ATL+RT collapse | Atlas green but runtime wrong; artist can't preview |
| TRIGGER chaos | T★ overload | Parallel lanes conflicting (coparent vs primary P1) |
| QUEST loop lock | Narrative stagnation | Same warehouse task re-queued without ΔWF |
| COST escalation | WRK⛔ | Headless minimum bake labeled as ship; Ct runaway |

---

## Implementation phases (20% before 80%)

### Phase 0 — Agent + lexicon (**now**)

- [x] Agent definition: `.cursor/agents/operations-intelligence.md`
- [x] This plan doc
- [x] Register in `AGENTS.md` agent routing table
- [x] Skill: `.cursor/skills/operations-intelligence/SKILL.md`
- [x] Track D: `ops_witness_index.py`, `ops_intelligence_scan.ps1`, `OPS_WITNESS_SPINE.md`, all-agent contract

### Phase 1 — JSON telemetry (no database)

**Schema:** `debug_runs/agent_ops/agent_run_event_v1.json` (array append or per-run files)

```json
{
  "schema": "agent_run_event_v1",
  "run_id": "ac603ba6-…",
  "agent": "coder-mcp",
  "lane": "track_a_aps",
  "task_id": "APS-PREVIEW-001",
  "model": "auto",
  "status": "success",
  "duration_ms": null,
  "witness_paths": ["debug_runs/aps_preview_001_slot_live.json"],
  "files_written": ["tools/mcp/art_pipeline_suite/slot_preview_panel.py"],
  "iteration": 1,
  "parent_run_id": null,
  "notes": "optional"
}
```

**Capture:** manual HANDOFF footer template + optional `invoke_handoff.ps1 -OpsEvent` flag.

**Output:** `debug_runs/agent_ops/ops_report_latest.json` — KPI rollup for OPS agent reads.

### Phase 2 — Witness envelope extension

Extend `_agent_meta` in [`debug_run_envelope.rs`](debug_run_envelope.rs):

- `lane` (track_a | track_b | track_c)
- `task_id` (APS-PREVIEW-001, PILOT-GRAMMAR-001, …)
- `agent_role` (coder-mcp, planner, …)

No breaking change to existing consumers.

### Phase 3 — Orchestrator hook + compression brief

`tools/orchestrator/scripts/ops_intelligence_scan.ps1`:

- Scan `debug_runs/agent_debug_index.json` + queue JSON + HANDOFF
- Write `ops_report_latest.json` + **`ops_project_brief_v1.json`** ($ref:src/dev/ops_agent_compression_v1.md)
- Compute `utility_score` + `metrics_tier1` ($ref:src/dev/ops_utility_function_v1.md · $ref:src/dev/ops_metrics_tiers_v1.md)
- DSM text block for HANDOFF
- `@operations-intelligence` invoked after major lane closes or weekly

### Phase 3b — MCP function layer (JSON backend)

Thin wrappers per $ref:src/dev/ops_mcp_function_layer_v1.md — compose `handoff_brief`, `witness_brief`, `agent_queue_next`, `token_savings_guide`. **No Postgres required.**

Witness: `debug_runs/agent_ops/ops_mcp_function_layer_live.json`

### Phase 4 — Deferred (complexity gate)

| Item | Gate |
|:---|:---|
| PostgreSQL event store | >500 structured events OR cross-project analytics |
| Supervisor on Opus | Phase 3 reports show stable KPIs for 30 days |
| CAPTURE video clips | VFX coparent lane only |
| Full GRAPH/QUEST editor | Track C grammar maturity |

**Complexity budget (reviewer rule):** Phase 4 total must score **value/complexity ≥ 1.0** on pilot data or stays deferred.

---

## Routing matrix

| OPS finding | Route to |
|:---|:---|
| SNAP authority drift | `@sim-steward` + `@debug-intelligence` |
| WRK cost / dishonest validators | `@orchestrator-mcp` + `@designer-mcp` |
| APS clarity / preview gaps | `@coder-mcp` + `@designer` |
| Agent waste / wrong model lane | `@orchestrator` + HANDOFF |
| Grammar/content risk | `@planner` + Track C docs |
| Proposal stress-test | Self (AGENT-REVIEW-CRITICAL phases) |

---

## Near-term priorities (planner-aligned)

1. **P0** — OPS reads existing witnesses; produce DSM map for three-track plan (no new infra)
2. **P1** — Phase 1 JSON events on HANDOFF close
3. **P2** — First `ops_report_latest.json` after warehouse B2 attempt
4. **P3** — Complexity-budget review before any PostgreSQL work

---

## Success criteria

| Criterion | Proof |
|:---|:---|
| Lead can read 15-line DSM and recover authority + risk + cost | OPS report artifact |
| Warehouse rejection classed as WRK⛔ not "pause warehouse" | Routed ΔWF in HANDOFF |
| Iteration diminishing returns visible | `iteration` field on events |
| No duplicate ownership with debug-intelligence | Routing matrix respected |

---

## References

- Three-track execution: [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md)
- Debug witness skill: [`.cursor/skills/debug-intelligence/SKILL.md`](../../.cursor/skills/debug-intelligence/SKILL.md)
- Subagent continuity: [`prompts/guides/subagent_continuity_playbook_v1.md`](../../prompts/guides/subagent_continuity_playbook_v1.md)
