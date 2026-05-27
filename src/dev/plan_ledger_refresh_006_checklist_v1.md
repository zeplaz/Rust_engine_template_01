# PLAN-LEDGER-REFRESH-006 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-006** |
| **Scope** | Reconcile human boards + fleet truth after qualified visual / wave 3 closure |
| **Prior** | [`plan_ledger_refresh_005_checklist_v1.md`](plan_ledger_refresh_005_checklist_v1.md) |
| **Script** | [`refresh_006_sync.py`](../../tools/orchestrator/scripts/refresh_006_sync.py) |
| **Audit** | [`planner_status_audit_v8.md`](planner_status_audit_v8.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Read [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) | authority |
| 2 | Read [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) v3.0 | `active: []` |
| 3 | Spot-check `debug_runs/*_live.json` | audit v8 § witness |
| 4 | Complete [`planner_status_audit_v8.md`](planner_status_audit_v8.md) | **SIGNED** |
| 5 | Update [`stage_open_todos_v1.md`](stage_open_todos_v1.md) | fleet closed / tails |
| 6 | Update [`active_coder_queue_v1.md`](active_coder_queue_v1.md) | fleet closed |
| 7 | Update [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) | fleet closed / tails |
| 8 | Mark stale markdown in audit v8 | § Stale markdown |
| 9 | Run `refresh_006_sync.py` | exit 0 |
| 10 | Mark **PLAN-LEDGER-REFRESH-006** done in planner queue | machine |

**Do not:** Re-plan F7 / M3 / MV wave 4 specs. **Do not** open wave 6 unless product names Round 4 or P2 depth.

---

## Sign-off

| Role | When |
|:---|:---|
| Orchestrator | `refresh_006_sync.py` OK + audit v8 **SIGNED** |
