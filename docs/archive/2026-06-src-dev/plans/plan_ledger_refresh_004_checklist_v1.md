# PLAN-LEDGER-REFRESH-004 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-004** |
| **Scope** | Machine-state sync after **UI batch v2** (items 1–7) — **no spec re-author** |
| **Prior cycle** | [`plan_ledger_refresh_003_checklist_v1.md`](plan_ledger_refresh_003_checklist_v1.md) |
| **Batch** | [`planner_queue_ui_batch_v2.md`](planner_queue_ui_batch_v2.md) |
| **Script** | [`refresh_004_sync.py`](../../tools/orchestrator/scripts/refresh_004_sync.py) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Verify UI batch plans 1–7 on disk | § plan_doc map below |
| 2 | Run steward + minimap + P4/P5 lib tests | green |
| 3 | Update [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) | `plan_doc` + `done` for batch IDs |
| 4 | Update [`planner_active_queue.json`](../../tools/orchestrator/queues/planner_active_queue.json) | batch **done** |
| 5 | Update [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) | P5-PAUSE → **done** if lib green |
| 6 | Update [`coder_triage_list_v1.md`](coder_triage_list_v1.md) | STALE / done rows |
| 7 | Bump [`planner_queue_ui_batch_v2.md`](planner_queue_ui_batch_v2.md) | **CLOSED** |
| 8 | Mark **PLAN-LEDGER-REFRESH-004** **done** | machine queue |

---

## plan_doc map (004 — UI batch v2)

| Queue ID | `plan_doc` |
|:---|:---|
| **PLAN-UI-OH-CLOSURE-004** | `docs/archive/2026-06-src-dev/plans/ui_overhaul_phase23_closure_plan_v1.md` |
| **PLAN-UI-P5-PAUSE-001** | `docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md` |
| **UI-OH-P5-001** | `docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md` |
| **PLAN-UI-P4-ATLAS-001** | `docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md` |
| **UI-OH-P4-001** | `docs/archive/2026-06-src-dev/plans/ui_oh_p4_001_plan_v1.md` |
| **PLAN-UI-P3-M3-001** | `docs/archive/2026-06-src-dev/plans/plan_ui_p3_m3_operational_stage7_plan_v1.md` |
| **UI-OH-M3-001** | `docs/archive/2026-06-src-dev/plans/ui_oh_m3_001_plan_v1.md` |
| **PLAN-UI-PHASE6-001** | `docs/archive/2026-06-src-dev/plans/ui_phase6_shell_perf_multiview_plan_v1.md` |
| **PLAN-UI-2C-001** | `docs/archive/2026-06-src-dev/plans/ui_phase2c_left_command_rail_plan_v1.md` |
| **PLAN-UI-THEME-MERGE-001** | `docs/archive/2026-06-src-dev/plans/ui_theme_merge_impl_spec_v1.md` |
| **PLAN-LEDGER-REFRESH-004** | `docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_004_checklist_v1.md` |

---

## Sign-off

| Role | When |
|:---|:---|
| Orchestrator | `python tools/orchestrator/scripts/refresh_004_sync.py` exits 0 + batch v2 **CLOSED** |
