# PLAN-S7B-M4-LIVE-001 — S7B M4 live sim playtest `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-S7B-M4-LIVE-001** |
| **Prior** | [`s7b_m4_sim_playtest_spec_v1.md`](s7b_m4_sim_playtest_spec_v1.md) — lib seed **CLOSED** |
| **Coder lane** | **S7B-M4-LIVE-001** (tail) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |

**No Rust.**

---

## Problem

`refresh_s7b_m4_play_001_live_witness()` seeds enqueue in lib — **CLOSED** for infrastructure.

**Live Simulation** may write `play_enqueue_wired: false` when another writer runs last → disk **STALE** for M4.

---

## PASS gate (live sim)

| # | Criterion | Evidence |
|:---:|:---|:---|
| L1 | Enter **Simulation** | `BaseState::Simulation` |
| L2 | User or scenario enqueues Move/Secure | UI or script |
| L3 | `play_enqueue_wired: true` | `stage7_behavioral_live.json` |
| L4 | `pending_dispatch_count >= 1` | same |
| L5 | `s7b_m4_play_001.green: true` | same |

**Lib regression (maintain):**

```powershell
cargo test -p proc_A_dine01 --lib coder_a_wave3_closure
```

---

## STALE policy

| Observation | Verdict |
|:---|:---|
| Lib bundle green, disk `s7b_m4_play_green: false` | **STALE** — run M4 refresh test or live playtest |
| Disk green after live session | **CURRENT** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Live sim tail beyond lib seed |
