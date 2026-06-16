# PLAN-F7-STREAM-DEEP-001 — F7-B streaming depth `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-F7-STREAM-DEEP-001** |
| **Prior** | [`fire7_f7_b_streaming_impl_plan_v1.md`](fire7_f7_b_streaming_impl_plan_v1.md) — **CLOSED** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **P2 optional** |

**No Rust.**

---

## Forward work

| ID | Task | Exit |
|:---|:---|:---|
| B-DEEP-1 | Fixed-seed neighbor wake lib test | deterministic `wake_transitions > 0` |
| B-DEEP-2 | Tie `FIRE_STREAMING_SLEEP_RADIUS` to residency window | stage6 fields move with focus |
| B-DEEP-3 | Witness extension `neighbor_wake_observed: true` | `fire_streaming_live.json` |

**Maintain:** `fire_streaming_b_green` baseline — do not regress F7-A exit.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | P2 depth after F7-B signoff |
