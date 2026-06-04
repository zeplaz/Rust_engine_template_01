# Doc drift review `2026-05-27` (v2)

**Snapshot:** [`fleet_snapshot_20260527_v1.md`](fleet_snapshot_20260527_v1.md)  
**Coder backlogs:** [A](coder_a_backlog_wave6_20260527_v1.md) · [B](coder_b_backlog_wave6_20260527_v1.md)

---

## Witness truth (authoritative)

| File | Green / open |
|:---|:---|
| `construction_stage_live.json` | parametric + R4 **CLOSED** |
| `wss_substrate_live.json` | slab spine **CLOSED**; `hybrid_ecs_smoke_authoritative: true` → smoke prod **OPEN** |
| `stage7_behavioral_live.json` | M4 play **CLOSED**; `s7b_m3_green` / `s7b_steward_green` **false** |
| `minimap_compositor_live.json` | M3 units + tray **CLOSED** |
| `replay_editor_parity_live.json` | replay ring **CLOSED** |
| `industrial_activation_live.json` | IND-E02 **CLOSED** |
| `stage5_full_app_live.json` | readiness + F2 **CLOSED** |

---

## Queue reconciliation (interrupted session)

**Fixed / current:**

- `coder_active_queue.json` v4.9 — Coder A/B wave-3..5 closures in `done_2026_05_27`; wave 6 `active[]` = PR5 smoke + H-A2 + infra (A); S7B M3 + LOG-E01 (B)
- `planner_active_queue.json` / `designer_active_queue.json` — `active: []`
- `designer_active_queue.json` → `routed_to_coder` UI-P3-M2-TRAY-OPT **status: done**
- `HANDOFF.md` + [`fleet_wave6_coder_dispatch_v1.md`](fleet_wave6_coder_dispatch_v1.md)

**Do not re-queue:** parametric 002..006, R4, M3, replay, tray, hydro boundary, BQ-128, WSS atmos/hydro (fixture path), S7B M4.

---

## Historical docs (archive tone — not execution truth)

Unless refreshed, treat as history:

- `coder_dual_queue_v3.md`
- `coder_fleet_multistage_matrix_v1.md` (B-C1..C6 sequence obsolete)
- `coder_unblock_dispatch_v1.md` (superseded by wave6 dispatch + maturity routing)
- `planner_parallel_workboard_v1.md` legacy “while B-C1 runs” wording

**Use for sessions:** `coder_active_queue.json` + `stage_*_workboard_v1.md` + this snapshot + A/B backlogs.

---

## Remaining drift risk

| Doc | Issue |
|:---|:---|
| `plan_wss_pr5_smoke_prod_001_v1.md` | Says smoke OPEN — matches witness |
| `coder_dual_queue_multistage_v1.md` | Checkbox rows for A-W2/B-H1 may still show ☐ |
| `development_plan_index.md` | Links to old dispatch only — snapshot linked in v2 |

---

## Changelog

| v | Date | Notes |
|:---|:---|:---|
| v1 | 2026-05-27 | Initial parametric drift pass |
| v2 | 2026-05-27 | Full witness scan + A/B backlogs + fleet snapshot |
