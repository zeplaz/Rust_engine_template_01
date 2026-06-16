# Planner queue todos `v1` (batch 2026-05-25)

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` / `@orchestrator` |
| **Status** | **CLOSED** — **PLAN-LEDGER-REFRESH-003** executed |
| **Audit** | [`planner_status_audit_v5.md`](planner_status_audit_v5.md) |
| **Machine queue** | [`planner_active_queue.json`](../tools/orchestrator/queues/planner_active_queue.json) |
| **Machine queue** | [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Workboard** | [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) |

**Rule:** `[x]` = planner deliverable **SIGNED** (doc on disk). **Open** rows may still unblock **@coder** slices.

---

## Todo list

| # | Queue ID | Deliverable | Status | Agent |
|:---:|:---|:---|:---:|:---:|
| 1 | **PLAN-LEDGER-REFRESH-003** | [`stage_tracks_ledger_refresh_003_plan_v1.md`](stage_tracks_ledger_refresh_003_plan_v1.md) | **DONE** | orchestrator |
| 2 | **PLAN-WAVE-P-WITNESS-SPEC-001** | [`wave_p_witness_spec_v1.md`](wave_p_witness_spec_v1.md) | **DONE** | planner |
| 3 | **PLAN-UI-SHELL-WITNESS-SPEC-001** | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) | **DONE** | planner |
| 4 | **PLAN-IND-E03-001** | [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) | **DONE** | planner |
| 5 | **PLAN-UI-P3-COMPOSITOR-001** | [`ui_phase3_minimap_compositor_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_compositor_full_plan_v1.md) | **DONE** | planner |
| 6 | **PLAN-INFRA-PROJ2-001** | [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) | **DONE** | planner |
| 7 | **PLAN-FIRE-VFX-CLOSURE-001** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) | **DONE** | planner |
| 8 | **PLAN-UX-BQ128-001** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) | **DONE** | planner |
| 9 | **PLAN-UI-P4-ATLAS-001** | [`ui_phase4_icon_atlas_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md) | **DONE** | planner |
| 10 | **PLAN-LOGISTICS-PROJECTION-001** | [`logistics_projection_impl_plan_v1.md`](logistics_projection_impl_plan_v1.md) | **DONE** | planner |
| 11 | **PLAN-IND-BOARD-RECONCILE-001** | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) | **DONE** | planner |
| 12 | **PLAN-UI-OH-CLOSURE-004** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) | **DONE** |
| **PLAN-UI-P5-PAUSE-001** | [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) | **DONE** (P5-PAUSE-001 **CLOSED**) | planner |

---

## Post-batch — active planner/coder

| ID | Owner | Notes |
|:---|:---|:---|
| **S7B-PLAN-001** | @planner | **DONE** — [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) |
| **S7B-PREFLIGHT-001** | @sim-steward | Next |
| **S7B-M1-001** | @coder | After preflight |
| **TRIAGE-VM-09-v2** | coder | **DONE** — invert bridge; witness `vm_09.triage_vm09_v2_green` |
| **PLAN-LEDGER-REFRESH** | orchestrator | Recurring after coder cycles |
| **UI-P5-PAUSE-001** | @coder (P2) | Bevy pause |
| **LOG-E01** | @coder / operator | STALE `log_rows` on disk |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | **PLAN-LEDGER-REFRESH-003** closed — audit v5, ledger v1.2.5 |
| v1.0.0 | 2026-05-25 | Twelve-item planner batch registered |
