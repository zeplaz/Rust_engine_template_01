# PLAN-LEDGER-REFRESH-010 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-010** |
| **Scope** | Full coder-return reconcile + wave 6 planner exec |
| **Prior** | [`plan_ledger_refresh_009_checklist_v1.md`](plan_ledger_refresh_009_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v14.md`](planner_status_audit_v14.md) |
| **Routing** | [`fleet_maturity_signoff_routing_20260527_v1.md`](fleet_maturity_signoff_routing_20260527_v1.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Reconcile v13 wave 4 coder returns | audit v14 § v13 corrections |
| 2 | Deliver **PLAN-WSS-PR5-SMOKE-PROD-001** | `plan_wss_pr5_smoke_prod_001_v1.md` |
| 3 | Deliver **PLAN-HANABI-H-A2-EXEC-001** | `plan_hanabi_h_a2_exec_001_v1.md` |
| 4 | Publish audit v14 **SIGNED** | this checklist + v14 |
| 5 | Update elemental index PR-5 / BQ-128 rows | `plan_elemental_wave2_index_001_v1.md` v1.1 |
| 6 | Archive planner rows in `planner_active_queue.json` | wave 6 exec done |
| 7 | Point `_meta.audit` at v14 | machine queue |

**Do not reopen:** wave6_archive exec plans · wave 3 closure rows.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Coder-return + exec plans |
