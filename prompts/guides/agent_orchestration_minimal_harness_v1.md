# Agent orchestration — minimal harness `v1`

| Field | Value |
|:---|:---|
| **ID** | **AGN-MIN-HARNESS-001** |
| **Thesis** | [arXiv:2609.40303](https://arxiv.org/abs/2609.40303) — strong coding agents + clean env beat elaborate outer harnesses |
| **Scope** | Agent/orchestration defaults only — **not** Bevy authority, MCP art G0–G5, or witness honesty |
| **Status** | **ACTIVE** (2026-10-19) |
| **Rule** | `.cursor/rules/agent-orchestration-lean-path.mdc` |
| **Cleanup packet** | ⟨CLEANUP-CLOSE⟩ AGN-MIN-HARNESS-001 |

---

## Verdict (repo terms)

Frontier single-agent loops with **read / write / exec** already do adaptive search and error correction. Extra multi-agent orchestration, rigid search loops, and mandatory ceremony **do not** improve slice quality here when the backbone is strong — they add coordination tax.

**Keep invariants. Thin defaults.**

---

## Default lean path (use this)

```text
goal → Auto / @coder | @designer | @planner (one owner)
     → edit → validate-report (on build/test)
     → witness-brief + WIT-HON only if closing a product queue row
```

| Do | Don't |
|:---|:---|
| Flat modules, direct exec loops | Wrapper “workflow managers,” speculative scaffolding |
| One chat thread for one slice | Spawn Task trees / parallel orchestrators by default |
| `validate-report` / `witness-brief` | Raw cargo walls / full witness JSON in chat |
| Escalate when blocked | Micro-manage with rigid outer phase graphs |

---

## Harness tiers

| Tier | When | Ritual |
|:---|:---|:---|
| **L0 lean** | Bounded edit, single owner, no queue lease | Skill parity check if `agent-lang` empty; domain skill as needed; `validate-report` after build/test |
| **L1 program** | Multi-file lane, HANDOFF lease, or multi-domain | `driver.mjs boot <agent>` (PRE + brief orient + HO); `agent_flow_route` before Task spawn |
| **L2 product close** | Queue row with `exit_predicate` / live witness | Full: work → **WIT-HON** → WIT → **Q✓** (WIT-HON never optional here) |

---

## Escalation personas (opt-in — not default entry)

| Persona | Invoke when |
|:---|:---|
| `@orchestrator` | Multi-phase engine graph across owners / authority boundaries |
| `@orchestrator-mcp` | MCP art program sequencing (spec→validate→promote) |
| `@plan-orchestrator` | Which `plan_*.md` slice / conflict matrix |
| `@main-thread-orchestrator` | Task quota fail-cycle → foreground continuity |
| `@coparent-orchestrator` | Explicit secondary parallel lanes vs P1 |
| `@flow-controller` | Multi-step Task dispatch needing L0→L1→L2 packets |
| `@operations-intelligence` | Lane close, pre-architecture commit, or supervisor review — **not** every cargo check |
| `@sim-steward` | Authority drift / cleanup classification / sequential A→B→C repairs |

Primary exec quartet: **Auto / `@coder` / `@planner` / `@designer`** (+ `*-mcp` when in that territory).

---

## Default queues (lean entry)

| Use | Path |
|:---|:---|
| **Coder home** | `tools/orchestrator/queues/coder_active_queue.json` (or current HANDOFF ACTIVE pick) |
| **Defer blockers** | `tools/orchestrator/queues/defer_registry.json` |
| **Session continuity** | `tools/orchestrator/queues/HANDOFF.md` |

`*_drain_*` / `parallel_wave_*` / `post_drain_*` queues remain **valid for fleet/historical batch** — they are **not** the default chat entry.

---

## Must keep (do not “simplify” away)

- **validation-first** — structured reports over raw logs
- **Witness spine** — **HOT only** (`debug_runs/HOT.md`); `lib green ≠ done`; do not glob `debug_runs/**`
- **WIT-HON before Q✓** on product / `exit_predicate` rows
- **Consumer vs `*-mcp` builder** split
- **bevy-simulation-grade** authority for `src/`
- **`cargo orchestrate`** as diagnostic classifier (not agent brain)

Bulk preview/validator dumps live in `_local_archive/debug_runs/` (gitignored) — not agent-facing.

---

## Collaboration contract (every coding / architecture assist)

1. **Simplicity over scaffolding** — no unnecessary wrappers, multi-agent sims, or rigid layers unless requested or required by system constraints.
2. **Lean execution** — flat, modular, testable; every line serves immediate utility.
3. **Trust the backbone** — clear ownership + direct loops beat heavy-handed orchestration rules.
4. **Zero slop** — no speculative complexity.

---

## Success criteria

- Fewer Task/orchestrator hops per closed slice
- Same or better: schema-green / witness-honest / authority-clean
- No new `src/` abstraction without a failing test or live witness that demands it
