# PLAN-STAGE7-M3-STEWARD-001 — Stage 7 M3 overlay + steward rollup `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-STAGE7-M3-STEWARD-001** |
| **Coder lane** | **S7B-M3-STEWARD-REMEDY-001** (@coder B **P1**) |
| **Witness spec** | [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md) § M3 |
| **Impl reference** | [`stage7_behavioral_live_proof.rs`](stage7_behavioral_live_proof.rs) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **Disk regression (2026-05-27):** `s7b_m3_green: false`, `s7b_steward_green: false` while M1/M2/M4 play green.

---

## Summary

Restore **Simulation-backed** Stage 7 M3 overlay witnesses (Recon + LogisticsStress) and the **steward rollup** (`s7b_steward_green`) on `debug_runs/stage7_behavioral_live.json`. Lib-only seed refresh is insufficient — the **live sim writer** must publish overlay sample counts during **Simulation**.

---

## Problem statement

| Symptom | Evidence |
|:---|:---|
| M3 red on disk | `recon_overlay_enabled: false`, `logistics_stress_overlay_enabled: false`, sample counts 0 |
| Steward rollup red | `s7b_steward_green: false` (requires M1 ∧ M2 ∧ M3) |
| Lib refresh may pass | `refresh_s7b_m3_steward_remedy_001_live_witness()` seeds witness in test — **not** authoritative for fleet close |

**Root cause class:** partial writers (M2/M4-only paths) or sim systems not updating `Stage7BehavioralWitnessState` before `write_stage7_behavioral_live_proof_system`.

---

## Authority map

| Resource | Single writer | Readers |
|:---|:---|:---|
| `Stage7BehavioralWitnessState` overlay flags | Stage 7 overlay publish systems (sim) | `stage7_behavioral_live_proof` rollup |
| `debug_runs/stage7_behavioral_live.json` | `write_stage7_behavioral_live_proof_system` / `commit_stage7_behavioral_live_proof` | audit, steward, HANDOFF |
| `StrategicCommandQueue` | strategic command plane | M4 play enqueue witness |
| Minimap cross-check (optional) | `minimap_compositor_live.json` | `ui_w3_m3_001.stage7_operational_green` telemetry only |

**Forbidden:** hand-editing JSON; merging M4-only refresh that clears M3 fields.

---

## Green predicates (authoritative)

From witness spec + `s7b_m3_green()`:

```text
s7b_m3_green :=
  recon_overlay_enabled == true
  AND logistics_stress_overlay_enabled == true
  AND recon_overlay_sample_count >= 1
  AND logistics_stress_overlay_count >= 1

s7b_steward_green :=
  s7b_m1_green AND s7b_m2_green AND s7b_m3_green
```

Cross-check: `stage7_play_live.json` → `s7p_steward_green: true` (maintain, do not merge files).

---

## Coder task list (≤3 files per PR)

### S7-M3-1 — Wire overlay readers in Simulation

1. Ensure recon + logistics stress overlay channels publish during **Simulation** ticks (not editor-only).
2. Update `Stage7BehavioralWitnessState` from real overlay readers — match `overlay_channels_v1`: `Recon`, `LogisticsStress`.
3. Do **not** zero overlay fields in M2/M4-only code paths.

**Files (candidate — adjust to actual wiring):**
- `src/dev/stage7_behavioral_live_proof.rs` (rollup only if needed)
- Stage 7 overlay / strategic HUD systems that own witness state
- `src/gui/hud/` or strategic overlay module per impl plan

### S7-M3-2 — Live proof writer ordering

1. `write_stage7_behavioral_live_proof_system` must run **after** overlay publish systems each frame/tick in Simulation.
2. If multiple refresh helpers exist, add **steward remedy** call path that preserves M3 fields when M4 play refresh runs.

**Files (≤3):**
- `src/dev/stage7_behavioral_live_proof.rs`
- plugin schedule registration for Stage 7 witness write

### S7-M3-3 — Lib test + disk exit

1. Keep / extend `refresh_s7b_m3_steward_remedy_001_live_witness` test.
2. Add regression: after sim harness run, read disk JSON — assert M3 + steward green.

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play comms_contract
cargo test -p proc_A_dine01 --lib stage7_behavioral_live_proof::refresh_s7b_m3_steward_remedy_001_live_witness -- --nocapture
```

---

## Witness schema

**File:** `debug_runs/stage7_behavioral_live.json`

| Path | Required value |
|:---|:---|
| `/s7b_m3_green` | `true` |
| `/s7b_steward_green` | `true` |
| `/recon_overlay_enabled` | `true` |
| `/logistics_stress_overlay_enabled` | `true` |
| `/recon_overlay_sample_count` | `>= 1` |
| `/logistics_stress_sample_count` | `>= 1` |
| `/s7b_m1_green` | `true` (preserve) |
| `/s7b_m2_green` | `true` (preserve) |
| `/s7b_m4_play_001/green` | `true` (preserve) |

Optional cross-check: `minimap_compositor_live.json` → `ui_w3_m3_001.stage7_operational_green` (informational; not sole gate).

---

## Designer (optional)

If overlay readability unclear after wiring: **DESIGN-S7B-M3-READ-001** (tray + map tint for Recon / LogisticsStress).

---

## Anti-patterns

- Lib seed-only green without sim overlay publish
- M4 play refresh clearing M3 overlay fields
- Reopening parametric / R4 / M3 minimap / replay archived exec plans
- Merging `stage7_play_live.json` into behavioral JSON

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | **S7B-M3-STEWARD-REMEDY-001** fleet close |
| **Witness** | `debug_runs/stage7_behavioral_live.json` |
| **Acceptance** | `s7b_m3_green=true` AND `s7b_steward_green=true` on disk after Simulation proof write |
| **Mutex** | Finish before **LOG-E01-FULLAPP-UPGRADE-001** (coder B P2) |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib stage7
```

After fix, confirm disk:

```powershell
# PowerShell: spot-check keys
Get-Content debug_runs/stage7_behavioral_live.json | Select-String "s7b_m3_green|s7b_steward_green|recon_overlay|logistics_stress"
```
