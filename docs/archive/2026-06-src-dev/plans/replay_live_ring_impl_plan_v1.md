# PLAN-REPLAY-LIVE-RING-001 — live replay ring `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-REPLAY-LIVE-RING-001** |
| **Prior** | [`replay_editor_parity_impl_plan_v1.md`](replay_editor_parity_impl_plan_v1.md) — lib **CLOSED** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SUPERSEDED** — use [`plan_replay_ring_exec_001_v1.md`](plan_replay_ring_exec_001_v1.md) for coder handoff |

**No Rust.**

---

## Scope

Wire **Simulation** ticks to grow `CommittedSimReplayRing` (not lib seed len 4 only).

| # | Criterion | Evidence |
|:---:|:---|:---|
| R1 | `replay_ring_len >= 2` after N sim ticks | runtime witness |
| R2 | Minimap scrub visible when ring live | `replay_scrub_enabled` |
| R3 | **REPLAY-PARITY-001** stays green | `replay_editor_parity_live.json` |

**Orthogonal:** editor scenario panel parity (already wired).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Live ring product depth |
