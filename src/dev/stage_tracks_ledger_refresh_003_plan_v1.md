# Ledger refresh cycle 003 `v1` (PLAN-LEDGER-REFRESH-003)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-003** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Status** | **CLOSED** |
| **Scope** | **Machine-state sync only** — no new planner specs |
| **Checklist** | [`plan_ledger_refresh_003_checklist_v1.md`](plan_ledger_refresh_003_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v5.md`](planner_status_audit_v5.md) |
| **Script** | [`tools/orchestrator/scripts/refresh_003_sync.py`](../../tools/orchestrator/scripts/refresh_003_sync.py) |

---

## What this cycle does

Applies planner output to machine state:

1. Verify witnesses vs specs (`wave_p`, `ui_shell`, `stage5`, `industrial`)
2. Apply [`industrial_activation_board_reconcile_v1.md`](industrial_activation_board_reconcile_v1.md) to ledger + board
3. Update [`coder_triage_list_v1.md`](coder_triage_list_v1.md), [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json), [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json)
4. Wire `plan_doc` for IND-E03, PROJ2, witness slices
5. Mark **PLAN-LEDGER-REFRESH-003** **done** in [`planner_active_queue.json`](../../tools/orchestrator/queues/planner_active_queue.json)

**Do not** re-run planner specs **2–11**.

---

## Sign-off

| Role | Status |
|:---|:---|
| Orchestrator | **CLOSED** — run `python tools/orchestrator/scripts/refresh_003_sync.py` |
