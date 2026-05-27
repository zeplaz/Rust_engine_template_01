# PLAN-F7-B-STREAM-001 — F7-B streaming signoff `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-F7-B-STREAM-001** |
| **Coder lane** | **FIRE7-F7-B-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — impl **CLOSED** @coder wave 3 |
| **Prereq** | **F7-A-EXIT** — [`fire7_f7_a_exit_acceptance_v1.md`](fire7_f7_a_exit_acceptance_v1.md) |
| **Architecture** | [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) § F7-B |
| **Code** | [`src/render/fire_streaming.rs`](../render/fire_streaming.rs) |
| **Witness** | `debug_runs/fire_streaming_live.json` |

**No Rust in this deliverable.** Post-impl acceptance for landed streaming; neighbor-wake depth is **optional P2**.

---

## Executive summary

| Verdict | Meaning |
|:---|:---|
| **CLOSED** | Sleep/wake mutates `FireChunkRuntime`, witness from **runtime or lib refresh**, not hand JSON |
| **P2 optional** | Extended neighbor-wake stress fixtures; residency window depth |

---

## Exit criteria (B1–B4)

| # | Criterion | Pass when | Evidence |
|:---:|:---|:---|:---|
| **B1** | Sleep/wake mutates runtime | `sleep_transitions > 0` **or** `wake_transitions > 0` after harness | `fire_streaming.rs` `apply_fire_streaming_sleep_wake_system` |
| **B2** | Active set non-empty | `active_chunk_count > 0` | `ActiveFireChunkSet` |
| **B3** | Witness from writer | `runtime_writer: true` or lib `refresh_fire_streaming_live_witness` | `fire_streaming_live.json` `_agent_meta.source_system` |
| **B4** | No second global extract | F7-A exit still green | `fire7_f7_a_exit_001.green: true` |

**Rollup:** `fire_streaming_b_green(witness, active)` in code.

---

## Witness JSON

| Path | Green when |
|:---|:---|
| `/gate` | `"FIRE7-F7-B-001"` |
| `/green` | `true` |
| `/sleep_transitions` | `> 0` **or** `/wake_transitions` `> 0` |
| `/runtime_writer` | `true` (sim path) |
| `/active_chunk_count` | `> 0` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib fire_streaming
```

Lib refresh: `refresh_fire_streaming_live_witness()` (see `coder_a_wave3_closure_v1.rs`).

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Hand-edit `green: true` without transitions | Not F7-B |
| Streaming JSON before F7-A exit | Violates gate chain |
| New global fire extract for greens | F7-A violation |

---

## Forward (P2)

| ID | Task | Owner |
|:---|:---|:---|
| F7-B-DEEP-001 | Fixed-seed neighbor wake lib test | @coder |
| F7-B-RES-001 | Tie sleep radius to `PerViewResidencyConsumerWindow` | @coder |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Post-impl signoff **PLAN-F7-B-STREAM-001** |
