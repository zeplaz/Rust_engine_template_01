# PLAN-LEDGER-REFRESH-009 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-009** |
| **Scope** | Wave 4 planner picks reconcile (post wave 3 closure) |
| **Prior** | [`plan_ledger_refresh_008_checklist_v1.md`](plan_ledger_refresh_008_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v12.md`](planner_status_audit_v12.md) |
| **Fleet** | [`fleet_wave4_assignments_20260527_v1.md`](fleet_wave4_assignments_20260527_v1.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Load v11 OPEN tails + wave 4 `next_phase` picks | [`planner_status_audit_v11.md`](planner_status_audit_v11.md) |
| 2 | Deliver exec plans (no reopen wave 3 / wave6_archive exec) | PR4 exec · IND-E02 play exec |
| 3 | Spot-check wave 4 witness files | `wss_substrate_live.json`, `industrial_activation_live.json` |
| 4 | Complete [`planner_status_audit_v12.md`](planner_status_audit_v12.md) | **SIGNED** |
| 5 | Archive planner wave 4 rows in `planner_active_queue.json` | `wave6_archive` |
| 6 | Update workboard + fleet assignments | wave 4 planner ☑ |
| 7 | Point `next_phase.audit` at v12 | machine queue |

**Do not reopen:** wave 3 planner rows · `wave6_archive` exec plans (parametric, R4, M3, replay, hydro, PR-3, F7 stream, etc.).

---

## Wave 4 planner deliverable map (009)

| Queue ID | `plan_doc` |
|:---|:---|
| **PLAN-LEDGER-REFRESH-009** | `docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_009_checklist_v1.md` + audit v12 |
| **PLAN-WSS-PR4-EXEC-001** | `docs/archive/2026-06-src-dev/plans/plan_wss_pr4_exec_001_v1.md` |
| **PLAN-IND-E02-PLAY-EXEC-001** | `docs/archive/2026-06-src-dev/plans/plan_ind_e02_play_exec_001_v1.md` |
| **PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001** | optional — **deferred** (not in 009 scope) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Wave 4 picks reconcile |
