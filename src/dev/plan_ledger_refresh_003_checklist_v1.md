# PLAN-LEDGER-REFRESH-003 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-003** |
| **Scope** | Apply planner output to **machine state** — **no new specs** |
| **Audit output** | [`planner_status_audit_v5.md`](planner_status_audit_v5.md) |
| **Do not** | Re-run planner specs **2–11** |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Read [`planner_status_audit_v5.md`](planner_status_audit_v5.md) | Witness ↔ spec matrix |
| 2 | Verify witnesses vs specs (wave_p, ui_shell, stage5, industrial) | §2 in audit v5 |
| 3 | Apply [`industrial_activation_board_reconcile_v1.md`](industrial_activation_board_reconcile_v1.md) | Ledger + [`post_stage6_active_todos.md`](post_stage6_active_todos.md) |
| 4 | Update [`coder_triage_list_v1.md`](coder_triage_list_v1.md) | Open / STALE / done |
| 5 | Update [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) | `plan_doc` paths |
| 6 | Update [`continuation_queue.json`](../tools/orchestrator/queues/continuation_queue.json) | Hygiene + `plan_doc` |
| 7 | Wire `plan_doc` for IND-E03, PROJ2, witness slices | See § plan_doc map |
| 8 | Mark **PLAN-LEDGER-REFRESH-003** **done** in [`planner_active_queue.json`](../tools/orchestrator/queues/planner_active_queue.json) | Machine queue |

---

## plan_doc map (003)

| Queue / slice ID | `plan_doc` |
|:---|:---|
| **IND-E03-CODER-A** | `src/dev/industrial_grid_overload_impl_plan_v1.md` |
| **INFRA-PROJ2-001** / **INFRA-PROJ2-CODER-B** | `src/dev/infra_proj2_sole_writer_plan_v1.md` |
| **UI-WP-LAYOUT-002** / **COD-B-WP-WITNESS-001** | `src/dev/wave_p_witness_spec_v1.md` |
| **UI-P2B-CODER-B** / **UI-SHELL-REFRESH-001** | `src/dev/ui_shell_witness_spec_v1.md` |
| **LOG-E01** / visual logistics | `src/dev/logistics_projection_impl_plan_v1.md` |
| **S7P-IND-001** | `src/dev/industrial_activation_board_reconcile_v1.md` |

---

## Sign-off

| Role | When |
|:---|:---|
| Orchestrator | All rows ☑ + `planner_active_queue` **PLAN-LEDGER-REFRESH-003** = `done` |
