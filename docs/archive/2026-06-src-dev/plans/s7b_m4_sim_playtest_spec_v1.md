# PLAN-S7B-M4-SIM-001 — S7B M4 sim playtest spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-S7B-M4-SIM-001** |
| **Coder lane** | **S7B-M4-SIM-001** (Coder A #8) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — lib witness **CLOSED** · live sim **PARTIAL** — [`s7b_m4_live_sim_playtest_plan_v1.md`](s7b_m4_live_sim_playtest_plan_v1.md) |
| **Closure rollup** | [`s7b_closure_plan_v1.md`](s7b_closure_plan_v1.md) |
| **Witness** | `debug_runs/stage7_behavioral_live.json` |
| **Code** | [`src/dev/stage7_behavioral_live_proof.rs`](../dev/stage7_behavioral_live_proof.rs) |

**No Rust in this deliverable.**

---

## Naming

| ID | Meaning |
|:---|:---|
| **S7B-M4-SIM-001** | Sim enqueue + `pending_dispatch_count` (coder wave 3) |
| **S7B-M4-PLAY-001** | Same witness block `s7b_m4_play_001` |
| **S7B-M3** | Recon + logistics overlay samples — **not** M4 |

---

## PASS gate (M4 sim)

```text
s7b_m4_play_green :=
  s7b_m4_play_enqueue_wired
  AND pending_dispatch_count >= 1
```

| # | Criterion | Evidence (2026-05-26 disk) |
|:---:|:---|:---|
| M4-1 | Enqueue wired | `play_enqueue_wired: true` |
| M4-2 | Pending orders | `pending_dispatch_count: 2` |
| M4-3 | Block green | `s7b_m4_play_001.green: true` |
| M4-4 | Lib refresh | `refresh_s7b_m4_play_001_live_witness()` green |

**Note:** `s7b_m3_green: false` on disk does **not** fail M4 — steward rollup is M1∧M2∧M3; M4 is additive play slice.

---

## Witness paths

| Path | Meaning |
|:---|:---|
| `/s7b_m4_play_001/gate` | `"S7B-M4-PLAY-001"` |
| `/s7b_m4_play_001/green` | M4 rollup |
| `/pending_dispatch_count` | Queue depth |
| `/s7b_m4_play_green` | Alias rollup |

---

## Operator playtest

| # | Step | Pass when |
|:---:|:---|:---|
| 1 | Enter **Simulation** | PLAY-01 HUD |
| 2 | Enqueue Move/Secure corridor | `pending_dispatch_count` > 0 |
| 3 | Advance ticks | delivery after `dispatch_delay_ticks` (8) |
| 4 | Lib regression | `cargo test -p proc_A_dine01 --lib stage7_behavioral` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play comms_contract
```

---

## STALE policy

| Observation | Verdict | Action |
|:---|:---|:---|
| `s7b_m4_play_green: false` after M3-only lib writer | **STALE** | `cargo test -p proc_A_dine01 --lib coder_a_wave3_closure` |
| `play_enqueue_wired: false` in live sim | **OPEN** live tail | **PLAN-S7B-M4-LIVE-001** |
| Disk green after `refresh_s7b_m4_play_001_live_witness` | **CURRENT** | maintain |

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Reopen M1–M3 without lib contradiction | Closure signed |
| Merge `stage7_play_live.json` into behavioral JSON | Separate witnesses |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-S7B-M4-SIM-001** signed |
