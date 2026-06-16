# Planner queue — UI batch `v2` (2026-05-25)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` / `@orchestrator` |
| **Status** | **CLOSED** — items 1–8 **DONE** |
| **Ledger cycle** | **PLAN-LEDGER-REFRESH-004** |
| **Prior batch** | [`planner_queue_todos_v1.md`](planner_queue_todos_v1.md) |

**Rule:** `[x]` = planner deliverable **SIGNED** on disk. Coder slices may remain **OPEN** under each plan.

---

## Todo list

| # | Queue ID | Deliverable | Status |
|:---:|:---|:---|:---:|
| 1 | **PLAN-UI-OH-CLOSURE-004** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) + steward gates | **DONE** |
| 2 | **PLAN-UI-P5-PAUSE-001** | [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) · OH [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) | **DONE** |
| 3 | **PLAN-UI-P4-ATLAS-001** | [`ui_phase4_icon_atlas_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md) · OH [`ui_oh_p4_001_plan_v1.md`](ui_oh_p4_001_plan_v1.md) | **DONE** |
| 4 | **PLAN-UI-P3-M3-001** | [`plan_ui_p3_m3_operational_stage7_plan_v1.md`](plan_ui_p3_m3_operational_stage7_plan_v1.md) | **DONE** |
| 5 | **PLAN-UI-PHASE6-001** | [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) | **DONE** |
| 6 | **PLAN-UI-2C-001** | [`ui_phase2c_left_command_rail_plan_v1.md`](ui_phase2c_left_command_rail_plan_v1.md) | **DONE** |
| 7 | **PLAN-UI-THEME-MERGE-001** | [`ui_theme_merge_impl_spec_v1.md`](ui_theme_merge_impl_spec_v1.md) | **DONE** |
| 8 | **PLAN-LEDGER-REFRESH-004** | [`plan_ledger_refresh_004_checklist_v1.md`](plan_ledger_refresh_004_checklist_v1.md) | **DONE** |

---

## Verification (batch close)

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle ui_p3_m3 ui_p5_pause icon_atlas minimap_compositor stage5
python tools/orchestrator/scripts/refresh_004_sync.py
cargo orchestrate --skip-cargo
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Eight-item UI planner batch + **PLAN-LEDGER-REFRESH-004** |
