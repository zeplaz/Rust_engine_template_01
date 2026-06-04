# PLAN-LEDGER-REFRESH-018 checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH-018** |
| **Scope** | PHASE-STABLE P2 sign-off — per-witness playability / production / proof grade |
| **Prior** | [`plan_ledger_refresh_017_checklist_v1.md`](plan_ledger_refresh_017_checklist_v1.md) |
| **Audit** | [`planner_status_audit_v18.md`](planner_status_audit_v18.md) |
| **P2 dispatch** | [`fleet_stability_phase2_dispatch_v1.md`](fleet_stability_phase2_dispatch_v1.md) |

---

## Checklist

| # | Step | Artifact |
|:---:|:---|:---|
| 1 | Publish audit v18 **SIGNED** with per-witness matrix | `planner_status_audit_v18.md` |
| 2 | Co-sign G-PLAY-01 runbook (planner row) | `play_scenario_acceptance_runbook_v1.md` |
| 3 | **PLAN-STABLE-P2-SIGN** — sign P2 dispatch | `fleet_stability_phase2_dispatch_v1.md` § Sign-off |
| 4 | Align `coder_active_queue.json` `coder_a` / `coder_b` `active[]` with P2 + dual track | queue v5.3 |
| 5 | Move PLAN-AUDIT-018 / PLAN-G-PLAY-001 to planner `done` | `planner_active_queue.json` |
| 6 | Update `development_plan_index.md` audit pointer | § Fleet snapshot |

**Do not reopen:** P1 DEHACK rows (ENG/RENDER/LOG), wave 7 PERF-VIS, DEV-CONTAIN-002–006 unless regression.

**Stop:** treating any single witness `green: true` as G-PLAY-01 close without operator runbook sign-off.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | P2 ledger v18 + dispatch sign |
