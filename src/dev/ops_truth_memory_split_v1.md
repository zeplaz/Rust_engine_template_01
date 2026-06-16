# OPS truth vs memory split `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-TRUTH-MEMORY-SPLIT-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (L2450–L2795) |
| **Parent** | $ref:src/dev/plan_ops_metalogic_split_v1.md |
| **Lang** | $ref:src/dev/agent_lang_v1.md |

**Rule:** **Git + filesystem = project truth.** **Dev Postgres = workstation operational intelligence only** — not in the game binary. **In-game embedded DB is allowed when gameplay or performance warrants it** (gated) — $ref:src/dev/guide_sim_effect_spine_v1.md §GAME-STORE-GATE◈.

---

## STORAGE-THREE-WORLDS◈

```text
① DEV POSTGRES     workstation · agent/dev progress · never in proc_A_dine01
② GAME HOT PATH    ECS + RAM rings · SimEffect tick authority
③ GAME PERSIST     FS/RON saves (Wave S) · optional embedded DB when gated
```

```text
proc_A_dine01  →  ② hot ECS  +  ③ FS saves (embedded DB if EV/Cx gate)
Dev workstation →  ① optional Postgres (dev analytics index)
```

SimEffect spine P0–P1 lives in **②**. Dev Postgres indexes **copies** of JSONL for cross-session analytics — $ref:src/dev/guide_sim_effect_spine_v1.md §DEV-OPS-POSTGRES-ROLE◈. Player event log / replay may later use **③ embedded DB** — not ①.

---

## Why filesystem-first

Moving project truth into PostgreSQL creates:

| Risk | Sym |
|:---|:---|
| Hidden state | 🔴 |
| Merge conflicts | 🔴 |
| Backup complexity | 🔴 |
| Harder git review / diffs | 🔴 |
| Harder agent inspection | 🔴 |

```text
Git + Filesystem  =  Project Truth 🏛
Postgres          =  Memory 🧠
MCP               =  Nervous System ⚡
Agents            =  Workers 🔧
```

---

## Canonical paths (truth — stay on disk)

```text
assets/
src/
tools/
prompts/
debug_runs/     ← witness JSON (git-tracked proofs)
schemas/
```

**Never ingest as authoritative:** building definitions · assemblies · materials · assets · source · RON/JSON spec bodies.

Postgres may **index** paths and freshness — not **own** content.

---

## Three tiers of work products

| Tier | Class | Storage | Examples |
|:---:|:---|:---|:---|
| **1** | Permanent | **Git only** | plans, specs, designs, `src/dev/*_v1.md` |
| **2** | Working memory | **Filesystem + DB index** | HANDOFF, queue tasks, reviews |
| **3** | Telemetry | **DB only** (JSON until gate) | runs, metrics, failures, costs |

| Tier | BLANG |
|:---:|:---|
| 1 | `$ref:` + git review |
| 2 | `BLANG:HO` + registry row |
| 3 | `BLANG:RUN` + `agent_run_append` |

---

## Handoff split (content vs state)

### HANDOFF **file** (human-readable — Tier 2 FS)

```text
Problem
Context
Decision
Required Work
```

Repo today: single $ref:tools/orchestrator/queues/HANDOFF.md — **evolution target:** numbered snapshots `HANDOFF_NNN.md` for archive; live file stays canonical until migration.

### **Registry** (operational state — Tier 2 DB / JSON until gate)

| Field | Purpose |
|:---|:---|
| `id` | handoff row id |
| `path` | `tools/orchestrator/queues/HANDOFF.md` |
| `owner` | `@agent` |
| `status` | open \| blocked \| review \| done |
| `priority` | P0–P3 |
| `created_at` / `updated_at` | |
| `review_count` | |
| `last_agent` | |
| `blocked_by` | `⟨ID⟩` or gate |
| `read_count` / `last_read` | freshness |
| `review_state` | |

**JSON interim:** `debug_runs/agent_ops/handoff_registry_v1.json` until Postgres `handoff_registry` table.

---

## Queue / task routing (where PG pays off)

**Anti-pattern:**

```text
Agent → scan queue folder → scan handoffs → scan logs → find work
```

**Target:**

```text
Agent → mcp.claim_task() → Task ⟨ID⟩ → read ONE $ref:HANDOFF or slice doc
```

| MCP tool | Returns |
|:---|:---|
| `ops_claim_task(agent)` | `{task_id, handoff_path, witness}` |
| `ops_get_active_handoffs()` | `{open: [], blocked: [], review: []}` |
| `ops_stale_handoffs()` | rows where review stale |
| `ops_next_review(agent)` | highest-priority stale row |
| `ops_changed_dependencies(path)` | tasks marked `review_needed` |

Catalog: $ref:src/dev/ops_mcp_function_layer_v1.md

**Repo now:** `agent_queue_next` + `post_drain_phase3_queue.json` — compose into `ops_claim_task` wrapper.

---

## Review freshness (stale without human read)

Example:

| Signal | Value |
|:---|:---|
| `created` | 10 days ago |
| `last_review` | 8 days ago |
| `source_files_changed` | 47 since last_review |

→ auto `status: STALE` — no agent must Read handoff to discover.

**Watcher path:**

```text
FILESYSTEM change event
  → Watcher 👁
  → Postgres / handoff_registry update
  → dependency graph traverse
  → affected tasks: review_needed=true
```

Example: `assembly_panel.py` changed → mark `APS-PREVIEW-001`, `APS-MAT-009`, `APS-UI-004`.

**Gate implementer:** file watcher + `task_dependency` edges in DB — not markdown graph.

---

## Task dependency graph (DB only)

```text
Task 201 → Task 180, 172, 165
```

Store in `decisions.task_dependency` — **not** inside markdown. Graph traversal = SQL/DB job.

Repo bridge: `depends_on[]` in `post_drain_phase3_queue.json` → ingest edges on gate.

---

## Postgres belongs / does not belong

**Scope reminder:** everything below is **dev workstation** — not gameplay, not shipped in the sim loop.

### ✅ Dev operational intelligence (Tier 2 index + Tier 3)

- Agent runs · telemetry
- Queue state · task routing
- Review history · handoff metadata
- Performance metrics · failure analytics
- Prompt evolution · workflow statistics
- Dependency graph · stale flags

### ❌ Project truth (Tier 1 — filesystem)

- Building definitions · assemblies · materials
- Assets · source code · RON/JSON authority bodies

---

## Agent Read Amplification (ARA)

```text
ARA = files_read / files_needed
```

| Agent | read | needed | ARA | Verdict |
|:---|:---:|:---:|:---:|:---|
| A | 120 | 4 | 30 | bad |
| B | 8 | 4 | 2 | good |

Complements KE ($ref:src/dev/ops_metrics_goodhart_guard_v1.md). Emit: `metrics_tier1.ara`.

`files_needed` = from `ops_claim_task` + `$ref:` list in exec doc — not agent guess.

---

## Dense topology

```text
FILESYSTEM◉
  SRC╋ASSETS╋PLANS╋HANDOFFS╋QUEUES
      │
      ▼
  WATCHERS👁
      │
      ▼
POSTGRES◉  (or JSON registry until gate)
  TASKS╋STATE╋DEPGRAPH╋REVIEWS╋TELEMETRY╋COSTS
      │
      ├─► ops_claim_task()
      ├─► ops_next_review()
      ├─► ops_stale_handoffs()
      ├─► ops_changed_dependencies()
      └─► ops_agent_health()
      │
      ▼
  AGENTS🤖  READ↓ WRITE↓ REVIEW↓
      │
      ▼
FILESYSTEM◉   (commits = truth)
```

---

## Repo alignment (honest today)

| Draft target | Current repo | Migration |
|:---|:---|:---|
| `HANDOFF_NNN.md` series | single `HANDOFF.md` | optional archive snapshots |
| `handoff_registry` PG | none | `handoff_registry_v1.json` interim |
| `mcp.claim_task()` | `agent_queue_next` | `ops_claim_task` wrapper |
| File watcher → stale | manual planner sync | Phase S2+ |
| Queue in PG | `*_queue.json` on disk | index only in PG |

**Invariant:** queue JSON files remain **authoritative for content** until explicit migration; DB tracks **state** only.

---

## Implementation phases

| Phase | Deliverable |
|:---:|:---|
| T0 | This doc + ARA in metrics tier |
| T1 | `handoff_registry_v1.json` from scan |
| T2 | `ops_claim_task` / `ops_get_active_handoffs` MCP |
| T3 | Watcher + dependency edges (gate) |
| T4 | Postgres schemas per $ref:src/dev/ops_sql_workstation_arch_v1.md |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | L2450+ filesystem-first · truth/memory split |
