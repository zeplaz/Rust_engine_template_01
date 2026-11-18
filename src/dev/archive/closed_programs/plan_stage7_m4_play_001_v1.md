# PLAN-STAGE7-M4-PLAY-001 — S7B M4 live sim witness wiring `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-STAGE7-M4-PLAY-001** |
| **Coder row** | **S7B-M4-PLAY-REMEDY-001** |
| **Prior specs** | [`s7b_m4_sim_playtest_spec_v1.md`](s7b_m4_sim_playtest_spec_v1.md) · [`s7b_m4_live_sim_playtest_plan_v1.md`](s7b_m4_live_sim_playtest_plan_v1.md) |
| **Steward (closed)** | [`plan_stage7_m3_steward_001_v1.md`](plan_stage7_m3_steward_001_v1.md) — M3/steward **green on disk** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@planner` → **`@coder B`** |
| **Status** | **SIGNED** — lib path **CLOSED**; live sim writer **OPEN** |

**Planner sign-off:** PASS (2026-05-28). Lib `refresh_s7b_m4_play_001_live_witness()` is green; disk `play_enqueue_wired: false` is a **schedule / last-writer** defect, not missing enqueue API.

---

## 1. Problem statement

| Observation (disk 2026-05-28) | Value |
|:---|:---|
| `s7b_m3_green` / `s7b_steward_green` | **true** |
| `pending_dispatch_count` | **1** |
| `play_enqueue_wired` | **false** |
| `s7b_m4_play_green` | **false** |

**Root cause class:** `Stage7BehavioralWitnessState.s7b_m4_play_enqueue_wired` is set in `seed_stage7_m4_playtest_enqueue` (called from `seed_stage7_behavioral_sim_session` on `OnEnter(Simulation)`), but `write_stage7_behavioral_live_proof_system` reads witness state **before** sim seed runs, or from a lib-only refresh path that never sets the flag. M3 overlay refresh (`ensure_stage7_behavioral_m3_witness_fields`) does **not** clear M4 — the flag is never set on the live writer path.

**Forbidden fixes:** hand-edit JSON; merge `stage7_play_live.json` into behavioral JSON; zero M3 fields in M4-only paths.

---

## 2. PASS gate (unchanged)

```text
s7b_m4_play_green :=
  behavioral.s7b_m4_play_enqueue_wired
  AND queue.pending_count() >= 1
```

| Key | Target |
|:---|:---|
| `/s7b_m4_play_001/play_enqueue_wired` | `true` |
| `/s7b_m4_play_001/pending_dispatch_count` | `>= 1` |
| `/s7b_m4_play_001/green` | `true` |
| `/s7b_m4_play_green` | `true` |
| `/s7b_m3_green`, `/s7b_steward_green` | **preserve true** |

---

## 3. Authority map

| Resource | Writer | Reader |
|:---|:---|:---|
| `StrategicCommandQueue` | `seed_stage7_behavioral_sim_session`, player UI, scenario | witness payload |
| `Stage7BehavioralWitnessState.s7b_m4_play_enqueue_wired` | `seed_stage7_m4_playtest_enqueue` **or** derived sync | `build_stage7_behavioral_live_proof_payload` |
| `debug_runs/stage7_behavioral_live.json` | `write_stage7_behavioral_live_proof_system` / `commit_stage7_behavioral_live_proof` | planner audit, fleet snapshot |

---

## 4. Execution slices

| Slice | Files | Change | Exit |
|:---:|:---|:---|:---|
| **M4-1** | `src/strategic/stage7_behavioral.rs` | Add `sync_stage7_m4_play_witness_from_queue` — set `s7b_m4_play_enqueue_wired = true` when pending Move/Secure corridor missions present (idempotent) | unit test: queue seeded → flag true |
| **M4-2** | `src/strategic/stage7_behavioral.rs` | Call sync from `tick_strategic_command_queue_system` **or** chain after `seed_stage7_behavioral_sim_session` before first witness write | sim tick path sets flag |
| **M4-3** | `src/dev/stage7_behavioral_live_proof.rs` | Ensure `write_stage7_behavioral_live_proof_system` runs **after** `seed_stage7_behavioral_sim_session` + overlay publish in Simulation (schedule audit in `Stage7BehavioralPlugin` / bridge in `economy/activation/bridge.rs`) | ordering doc in impl plan |
| **M4-4** | `src/dev/s7b_m4_play_sim_witness_tests.rs` (new) or `stage7_behavioral.rs` `#[cfg(test)]` | Integration: sim enter → one witness write → `play_enqueue_wired` true | lib test green |
| **M4-5** | — | `cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play coder_a_wave3_closure` + refresh disk JSON | witness green |

**Preferred minimal fix (M4-1 + M4-2):** derive flag from queue state each tick so lib-only and live sim paths converge:

```rust
// Pseudocode — implement in stage7_behavioral.rs
fn sync_m4_play_enqueue_wired(queue: &StrategicCommandQueue, witness: &mut Stage7BehavioralWitnessState) {
    if queue.pending_count() >= 1 {
        witness.s7b_m4_play_enqueue_wired = true;
    }
}
```

Call from `tick_strategic_command_queue_system` after dispatch tick (Simulation only).

---

## 5. Schedule order (target)

```text
OnEnter(Simulation):
  seed_stage7_behavioral_sim_session
  → seed_stage7_behavioral_overlay_resources_on_simulation_enter

Update (Simulation):
  … publish logistics/ecology snapshots …
  → publish_stage7_behavioral_overlay_samples
  → sync_m4_play_enqueue_wired (NEW — in tick_strategic_command_queue_system)
  → write_stage7_behavioral_live_proof_system (interval)
```

Witness write must **not** run on `OnEnter` before sim session seed unless payload uses queue-only derivation.

---

## 6. Verification

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play comms_contract
cargo test -p proc_A_dine01 --lib coder_a_wave3_closure
cargo test -p proc_A_dine01 --lib stage7
```

### Witness check

| File | Keys |
|:---|:---|
| `debug_runs/stage7_behavioral_live.json` | `s7b_m4_play_001.green`, `play_enqueue_wired`, `s7b_m4_play_green` |
| `debug_runs/stage7_play_live.json` | `s7p_play_witness_ok` (unchanged) |

---

## 7. Risks and rollback

| Risk | Mitigation | Rollback |
|:---|:---|:---|
| M4 sync clears M3 overlay counts | Only touch `s7b_m4_play_enqueue_wired` | Revert M4-1 |
| False green with empty queue | Require `pending_count >= 1` for rollup | Tighten predicate |
| Schedule regression | `stage7` lib tests + steward keys unchanged | Revert M4-3 ordering |

---

## 8. Definition of done

- [ ] Disk `stage7_behavioral_live.json`: `play_enqueue_wired: true`, `s7b_m4_play_green: true`.
- [ ] M3/steward keys remain **true**.
- [ ] `coder_a_wave3_closure` + `stage7_behavioral` lib tests green.
- [ ] Queue row **S7B-M4-PLAY-REMEDY-001** closed in `coder_active_queue.json`.

---

## 9. Start Here (@coder B)

1. Implement **M4-1** (`sync_m4_play_enqueue_wired` in `tick_strategic_command_queue_system`).
2. Run lib tests; refresh behavioral witness via existing test harness.
3. Confirm disk JSON; close queue row.
4. **Do not** reopen M3 steward work — already green.

**Estimated scope:** 1 PR, ≤3 production files + test.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | PLAN-STAGE7-M4-PLAY-001 signed for B wiring |
