# @coder A — infra / stress lane (post-rebalance) `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-26 |
| **Queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) `coder_a.active` |
| **Regression** | `cargo test -p proc_A_dine01 --lib infrastructure_view_isolation stage6_live_proof phase_d_parity` |

**Pick-one rule:** one primary ID per session; priority order below.

---

## Priority 1 — TRIAGE-PHASE-D-PARITY-001

| | |
|:---|:---|
| **Status** | **DONE** (lib + witness) — do not reopen unless S4/S5 requested |
| **Plan** | [`overlay_parity_stress_plan_v1.md`](overlay_parity_stress_plan_v1.md) |
| **Code** | [`src/render/view_runtime/phase_d_parity_stress.rs`](../render/view_runtime/phase_d_parity_stress.rs) |
| **Witness** | `debug_runs/infrastructure_view_isolation_live.json` → `triage_phase_d_parity_001` + `stress.s1`–`s3` |

### Exit (closed)

- [x] **S1** — WorldMain ↔ SimulationMap distinct overlay masks (`stress_s1_*`)
- [x] **S2** — Multiview fire bounded; minimap heat-only default (`stress_s2_*`)
- [x] **S3** — Sim-enter overlay rebind clears stale minimap fire tint (`stress_s3_*`)
- [x] Witness `green` via `triage_phase_d_parity_001_green()` (not hand JSON)
- [x] `cargo test -p proc_A_dine01 --lib phase_d_parity`

### Optional follow-up (P2, not blocking)

- [ ] **S4** — Same-tick minimap heat-only + tactical fire rows (sim `--test visual` or harness)
- [ ] **S5** — Roll into existing `vm_09.triage_vm09_v2_green` regression only

---

## Priority 2 — INFRA-VM-DEEP-001

| | |
|:---|:---|
| **Status** | **DONE** — lib + sim writer paths (`source` / `sim_time_written` / extended `sim_trace`) |
| **Tier** | P2 · priority 2 |
| **Witness** | `debug_runs/infrastructure_view_isolation_live.json` → `infra_vm_deep_001` |

### Goal

Extend VM-08 / VM-10 / VM-11 **sim-written** fields beyond lib refresh (`refresh_infrastructure_view_isolation_live_witness` uses synthetic green).

### Todo

1. [x] **Wire sim writer proof** — assert `write_view_runtime_live_proof_system` sets `infra_vm_deep_001.sim_trace` from live `ViewIsolationDiagnostics` + `ViewFireIsolationWitness` (already in `build_proof_payload`; add `sim_time_written: true` + frame stamp when written from Simulation).
2. [x] **Deep trace fields** — extend `sim_trace` with:
   - `per_view_fire_instances` snapshot (from fire witness)
   - `trace_entry_count` / `trace_violation_count` (from `ViewRuntimeTrace`)
   - `active_surface` (from `ViewInputRoutingState`)
3. [x] **Lib test** — `infra_vm_deep_001_sim_trace_fields_present` in [`live_proof.rs`](../render/view_runtime/live_proof.rs) tests: refresh JSON includes new keys; `green` still requires `infrastructure_view_isolation_green`.
4. [x] **Do not** hand-set `sim_trace.*: true` in refresh helper — only structured defaults for lib-only path; mark `source: "lib_refresh"` vs `"sim_live"`.

### Files

| Path | Touch |
|:---|:---|
| `src/render/view_runtime/live_proof.rs` | `infra_vm_deep_001` payload + sim writer metadata |
| `src/render/view_runtime/plugin.rs` | Confirm sim proof system ordered after fire/isolation witness refresh |

### Verify

```powershell
cargo test -p proc_A_dine01 --lib infrastructure_view_isolation steward_vm09 -- --test-threads=1
# Operator: enter Simulation ≥90 frames → re-read infrastructure_view_isolation_live.json (_agent_meta.producer = view_runtime_live_proof)
```

### Exit

- `infra_vm_deep_001.green` true from **sim** path OR lib path with explicit `source` discriminator
- `sim_trace` populated with VM-08/10/11 live values when sim writer runs
- Queue → `done_2026_05_26`

---

## Priority 3 — STAGE6-OPS-WITNESS-001

| | |
|:---|:---|
| **Status** | **DONE** — `stage6_ops_witness_001` + sim helper + lib/wc_d04 path |
| **Tier** | P2 · priority 3 |
| **Plan** | [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) · [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) |
| **Witness** | `debug_runs/stage6_virtualization_live.json` |

### Goal

Operator **OPS-F03** can refresh stage6 JSON from a running sim without re-running lib-only `refresh_wc_d04_stage6_virtualization_live_witness`.

### Todo

1. [ ] Add **`stage6_ops_witness_001`** block to `build_stage6_proof_payload` (`gate`, `green`, `sim_time_written`, `producer`).
2. [x] Export **`refresh_stage6_ops_witness_001_from_sim_resources`** (thin wrapper around live `Stage6VirtualizationFrame` + witness + optional `FrameBudgetDiagnostics`) for diagnostics / CLI hook.
3. [x] Ensure `write_stage6_virtualization_live_proof_system` sets `stage6_ops_witness_001.green` when `gpu_upload_bytes_frame > 0` and readiness passes (WC-D04 C4).
4. [x] Lib test in [`stage6_live_proof.rs`](../render/stage6_live_proof.rs): `stage6_ops_witness_001_fields_present` after lib commit; bundle assert in `coder_a_wave3` or new `coder_a_infra_stress` test.
5. [x] Document operator step: enter Simulation → wait ≥90 frames → inspect `stage6_virtualization_live.json` (`wc_d04`, `stage6_ops_witness_001`).

### Files

| Path | Touch |
|:---|:---|
| `src/render/stage6_live_proof.rs` | OPS witness block + helper |
| `src/render/stage6_virtualization.rs` | Plugin ordering if helper needs extra systems |

### Verify

```powershell
cargo test -p proc_A_dine01 --lib stage6_live_proof wc_d04 -- --test-threads=1
```

### Exit

- `stage6_ops_witness_001.green` in live JSON
- Sim writer path documented; lib refresh remains fallback with `source: "lib_refresh"`
- Queue → `done_2026_05_26`

---

## INFRA-VM-FOLLOWON-001 — **DONE (qualified)** 2026-05-27

| | |
|:---|:---|
| **Queue** | Moved to `coder_a.done_2026_05_27` — **not** a separate implementation slice |
| **Rationale** | Phase C IN-C01..07 + stress bundle (parity / VM-deep / stage6 ops) already green on disk |
| **Witness** | `debug_runs/infrastructure_view_isolation_live.json` → rollup `infrastructure_view_isolation_green: true` |
| **Deferred** | Parity **S4/S5** (visual sim) — optional; `coder_active_queue.json` → `deferred_optional` |

**Do not reopen** unless VM-06..11 regression or new planner exec supersedes Phase C.

---

## Session order (recommended)

```
1. TRIAGE-PHASE-D-PARITY-001  → skip (done)
2. INFRA-VM-DEEP-001          → skip (done)
3. STAGE6-OPS-WITNESS-001     → skip (done)
4. INFRA-VM-FOLLOWON-001      → skip (done_qualified — duplicate tail)
```

## Copy-paste — @coder A pick INFRA-VM-DEEP

```
@coder A — INFRA-VM-DEEP-001 (priority 2)
Read: docs/archive/2026-06-src-dev/plans/coder_a_infra_stress_active_v1.md · src/render/view_runtime/live_proof.rs
First: infra_vm_deep_001 sim_trace + sim_time_written from write_view_runtime_live_proof_system
Do NOT: hand green sim_trace in lib refresh without source field
Verify: cargo test -p proc_A_dine01 --lib infrastructure_view_isolation
Exit: infrastructure_view_isolation_live.json sim_trace live after sim
```

## Copy-paste — @coder A pick STAGE6-OPS-WITNESS

```
@coder A — STAGE6-OPS-WITNESS-001 (priority 3)
Read: docs/archive/2026-06-src-dev/plans/coder_a_infra_stress_active_v1.md · src/render/stage6_live_proof.rs
First: stage6_ops_witness_001 block + sim refresh helper (OPS-F03)
Do NOT: replace wc_d04 lib path — add parallel ops gate
Verify: cargo test -p proc_A_dine01 --lib stage6_live_proof
Exit: stage6_virtualization_live.json stage6_ops_witness_001.green
```
