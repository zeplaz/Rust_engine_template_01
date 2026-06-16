# PLAN-LEDGER-REFRESH-019 checklist `v1`

| Field | Value |
|:---|:---|
| **Checklist ID** | **PLAN-LEDGER-REFRESH-019** |
| **Audit** | [`planner_status_audit_v19.md`](planner_status_audit_v19.md) |
| **Prior** | [`plan_ledger_refresh_018_checklist_v1.md`](plan_ledger_refresh_018_checklist_v1.md) |
| **Trigger** | CON-P3-WIT green · infra E0–E3 green · organic B drain green · construction lib 144/144 |
| **Status** | **SIGNED** — 2026-06-03 |

---

## Checklist

| # | Task | Artifact | Done |
|:---:|:---|:---|:---:|
| 1 | Publish audit v19 with Operator + DSM columns | `planner_status_audit_v19.md` | ✓ |
| 2 | Close v18 stale open rows (CON-P2, CONTAIN-MINIMAP, infra E0 in active[]) | v19 § closed | ✓ |
| 3 | Score new witnesses: transport, build_worker, grammar_iter, aps atlas | v19 matrix | ✓ |
| 4 | Sync `coder_active_queue.json` `_meta` + `infrastructure_program.coder_*_next` | queue JSON v5.6.0 | ✓ |
| 5 | Sync HANDOFF + `stage_planner_workboard_v1.md` | handoff / workboard | ✓ |
| 6 | Move PLAN-AUDIT-019 + PLAN-QUEUE-SYNC-002 → planner `done` | `planner_active_queue.json` | ✓ |
| 7 | Sign checklist | this file | ✓ |

---

## Exit

- Orchestrator can paste **≤10 open slices** from v19 without cross-checking three queue files.
- **G-PLAY-01** row explicitly **OPEN** until operator checklist signed (**PLAN-G-PLAY-CLOSE-001** next).
- **Wrong picks prevented:** no CON-P2, no infra B `coder_b_next`, no ECON-OG-SAVE in active[].
