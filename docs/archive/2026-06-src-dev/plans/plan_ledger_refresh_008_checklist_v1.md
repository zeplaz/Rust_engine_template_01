# PLAN-LEDGER-REFRESH-008 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-008** |
| **Scope** | Witness-first reconcile of `wave6_archive` + wave 3 secondary prep |
| **Prior** | [`plan_ledger_refresh_007_checklist_v1.md`](plan_ledger_refresh_007_checklist_v1.md) |
| **Script** | [`refresh_008_sync.py`](../../tools/orchestrator/scripts/refresh_008_sync.py) |
| **Audit** | [`planner_status_audit_v10.md`](planner_status_audit_v10.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Load `planner_active_queue.json` `wave6_archive` | machine |
| 2 | Spot-check `debug_runs/*_live.json` per archive row with `witness` | audit v10 § reconcile |
| 3 | Mark **CLOSED** only when acceptance keys are green on disk | audit v10 table |
| 4 | Mark planner-only rows **SIGNED** / **READY** without falsely CLOSED | audit v10 |
| 5 | Complete [`planner_status_audit_v10.md`](planner_status_audit_v10.md) | **SIGNED** |
| 6 | Deliver secondary plans (no reopen of archived exec) | § secondary map |
| 7 | Update [`planner_parallel_workboard_v1.md`](planner_parallel_workboard_v1.md) | wave 3 ☑ |
| 8 | Update [`fleet_wave3_assignments_20260527_v1.md`](fleet_wave3_assignments_20260527_v1.md) | ledger-008 done |
| 9 | Run `refresh_008_sync.py` | exit 0 |
| 10 | Archive **PLAN-LEDGER-REFRESH-008** + secondary rows in queue | `wave6_archive` |

**Do not reopen:** parametric, R4, M3, replay, hydro, PR-3 **exec** plan docs (witness may be green — status is CLOSED, not “re-plan”).

---

## Secondary plan map (008)

| Queue ID | `plan_doc` |
|:---|:---|
| **PLAN-ELEMENTAL-WAVE2-INDEX-001** | `docs/archive/2026-06-src-dev/plans/plan_elemental_wave2_index_001_v1.md` |
| **PLAN-WSS-HYBRID-RETIRE-PR4-001** | `docs/archive/2026-06-src-dev/plans/plan_wss_hybrid_retire_pr4_001_v1.md` |
| **PLAN-BQ128-APPLY-EXEC-001** | `docs/archive/2026-06-src-dev/plans/plan_bq128_apply_exec_001_v1.md` |
| **PLAN-LEDGER-REFRESH-008** | `docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_008_checklist_v1.md` |

---

## Sign-off

| Role | When |
|:---|:---|
| `@planner` | `refresh_008_sync.py` OK + audit v10 **SIGNED** |
