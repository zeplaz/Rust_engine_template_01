# PLAN-LEDGER-REFRESH-007 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-007** |
| **Scope** | Fleet + planner truth after **P1 prep drain** (hydro coupling, PR-3, archived R4/M3/replay) |
| **Prior** | [`plan_ledger_refresh_006_checklist_v1.md`](plan_ledger_refresh_006_checklist_v1.md) |
| **Script** | [`refresh_007_sync.py`](../../tools/orchestrator/scripts/refresh_007_sync.py) |
| **Audit** | [`planner_status_audit_v9.md`](planner_status_audit_v9.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Confirm P1 plan docs on disk | § plan_doc map |
| 2 | Spot-check witnesses (substrate, construction, minimap, replay) | audit v9 § witness |
| 3 | Confirm `planner_active_queue.json` `active: []` + P1 rows in `wave6_archive` | machine |
| 4 | Complete [`planner_status_audit_v9.md`](planner_status_audit_v9.md) | **SIGNED** |
| 5 | Update [`planner_parallel_workboard_v1.md`](planner_parallel_workboard_v1.md) | P1 ☑ |
| 6 | Update [`planner_wave7_parallel_todos_v1.md`](planner_wave7_parallel_todos_v1.md) | hydro + PR-3 ☑ |
| 7 | Update [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) | P1 archived refs |
| 8 | Bump [`planner_delivery_signoff_matrix_v1.md`](planner_delivery_signoff_matrix_v1.md) | v1.1.0 wave 6/7 tails |
| 9 | Update [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) | @planner P1 done; coder B hydro/PR-3 when queued |
| 10 | Run `refresh_007_sync.py` | exit 0 |
| 11 | Mark **PLAN-LEDGER-REFRESH-007** done in planner queue | `wave6_archive` |

**Do not:** Reopen archived `PLAN-CONSTRUCTION-R4-EXEC-001`, `PLAN-M3-DEPTH-EXEC-001`, `PLAN-REPLAY-RING-EXEC-001`, or parametric closure rows.

---

## plan_doc map (007 — P1 prep + ledger)

| Queue ID | `plan_doc` |
|:---|:---|
| **PLAN-CONSTRUCTION-HYDRO-COUPLING-001** | `src/dev/plan_construction_hydro_coupling_001_v1.md` |
| **PLAN-WSS-SLAB-PR-3-EXEC-001** | `src/dev/plan_wss_slab_pr3_exec_001_v1.md` |
| **PLAN-LEDGER-REFRESH-007** | `src/dev/plan_ledger_refresh_007_checklist_v1.md` |

---

## Sign-off

| Role | When |
|:---|:---|
| `@planner` | `refresh_007_sync.py` OK + audit v9 **SIGNED** |
