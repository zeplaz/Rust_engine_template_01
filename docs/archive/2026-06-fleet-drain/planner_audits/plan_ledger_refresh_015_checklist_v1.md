# PLAN-LEDGER-REFRESH-015 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-015** |
| **Scope** | Reconcile v14 OPEN rows vs 2026-05-28 disk + horizon exec plans |
| **Prior** | [`plan_ledger_refresh_010_checklist_v1.md`](plan_ledger_refresh_010_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v15.md`](planner_status_audit_v15.md) |
| **Fleet** | [`fleet_snapshot_20260528_v1.md`](fleet_snapshot_20260528_v1.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Reconcile v14 vs disk (smoke, M3, M4, post-spine, H-A2) | audit v15 § v14 corrections |
| 2 | Sign **PLAN-VISUAL-PERF-EXEC-001** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) |
| 3 | Sign **PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001** | [`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md) |
| 4 | Sign **PLAN-STAGE7-M4-PLAY-001** (if B needs wiring spec) | [`plan_stage7_m4_play_001_v1.md`](plan_stage7_m4_play_001_v1.md) |
| 5 | Publish audit v15 **SIGNED** | this checklist + v15 |
| 6 | Point `development_plan_index.md` + fleet at v15 | index row |
| 7 | Close stale queue rows (M3/steward done; M4 play P1) | `coder_active_queue.json` hygiene — @coder B |

**Do not reopen:** wave 6 closed exec plans unless witness regression.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | Horizon exec + disk drift reconcile |
