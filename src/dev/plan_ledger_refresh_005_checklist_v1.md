# PLAN-LEDGER-REFRESH-005 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-005** |
| **Scope** | Machine-state sync after **planner wave 4** (items 1–12) — **no spec re-author** |
| **Prior cycle** | [`plan_ledger_refresh_004_checklist_v1.md`](plan_ledger_refresh_004_checklist_v1.md) |
| **Board** | [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) |
| **Script** | [`refresh_005_sync.py`](../../tools/orchestrator/scripts/refresh_005_sync.py) |
| **Audit** | [`planner_status_audit_v7.md`](planner_status_audit_v7.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Verify wave 4 plan docs 1–12 on disk | § plan_doc map |
| 2 | Run spot lib tests (F7, S7B, replay, construction) | green |
| 3 | Update [`planner_active_queue.json`](../../tools/orchestrator/queues/planner_active_queue.json) | wave 4 → **done** |
| 4 | Update [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) | all ☑ |
| 5 | Update [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) | wave 4 **CLOSED** |
| 6 | Bump [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) | v1.3.0 wave 4 planner |
| 7 | Mark **PLAN-LEDGER-REFRESH-005** **done** | audit v7 |

---

## plan_doc map (005 — wave 4)

| Queue ID | `plan_doc` |
|:---|:---|
| **PLAN-F7-A-EXIT-001** | `src/dev/fire7_f7_a_exit_acceptance_v1.md` |
| **PLAN-F7-B-STREAM-001** | `src/dev/fire7_f7_b_streaming_impl_plan_v1.md` |
| **PLAN-F7-C-LOD-001** | `src/dev/fire7_f7_c_lod_impl_plan_v1.md` |
| **PLAN-CONSTRUCTION-MV-001** | `src/dev/construction_multiview_sim_spec_v1.md` |
| **PLAN-IND-E02-PLAY-001** | `src/dev/ind_e02_default_play_spec_v1.md` |
| **PLAN-LOG-E01-VISUAL-001** | `src/dev/log_e01_visual_acceptance_v1.md` |
| **PLAN-VISUAL-RUN-GATE-001** | `src/dev/visual_run_acceptance_matrix_v1.md` |
| **PLAN-M3-MINMAP-001** | `src/dev/minimap_m3_units_replay_impl_plan_v1.md` |
| **PLAN-REPLAY-PARITY-001** | `src/dev/replay_editor_parity_impl_plan_v1.md` |
| **PLAN-S7B-M4-SIM-001** | `src/dev/s7b_m4_sim_playtest_spec_v1.md` |
| **PLAN-PHASE-D-PARITY-001** | `src/dev/overlay_parity_stress_plan_v1.md` |
| **PLAN-LEDGER-REFRESH-005** | `src/dev/plan_ledger_refresh_005_checklist_v1.md` |

---

## Sign-off

| Role | When |
|:---|:---|
| Orchestrator | `python tools/orchestrator/scripts/refresh_005_sync.py` exits 0 + audit v7 written |
