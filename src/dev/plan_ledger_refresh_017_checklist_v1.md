# PLAN-LEDGER-REFRESH-017 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-017** |
| **Scope** | PHASE-STABLE pivot — playability + de-hack |
| **Prior** | [`plan_ledger_refresh_016_checklist_v1.md`](plan_ledger_refresh_016_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v17.md`](planner_status_audit_v17.md) |
| **Phase plan** | [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Sign **PLAN-FLEET-STABILITY-INTEGRITY-001** | `plan_fleet_stability_integrity_001_v1.md` |
| 2 | Publish coder exec slices | `plan_fleet_stability_integrity_exec_001_v1.md` |
| 3 | Publish audit v17 **SIGNED** with playability column | this checklist + v17 |
| 4 | Mark PHASE-NEXT **SUPERSEDED** for open work only | `plan_fleet_phase_next_001_v1.md` banner |
| 5 | Repopulate `coder_active_queue.json` from plan §9 | `next_phase` → PHASE-STABLE-2026-06 |
| 6 | Update `development_plan_index.md` fleet pointer | § Fleet snapshot |
| 7 | Publish coder dispatch | `fleet_stability_coder_dispatch_v1.md` |

**Do not reopen:** PHASE-NEXT closed rows (containment 002–007, perf tails, UI 2B/P3–P5).

**Stop:** treating lib fixture green as ship sign-off.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | PHASE-STABLE ledger v17 |
