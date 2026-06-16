# Stage 7 behavioral — implementation plan `v1` (S7B-PLAN-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **S7B-PLAN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — unblocks **S7B-PREFLIGHT-001** → **S7B-M1-001** |
| **Design gate** | [`stage7_behavioral_d_signoff_v1.md`](stage7_behavioral_d_signoff_v1.md) **SIGNED** |
| **Full plan** | [`stage7_behavioral_full_plan_v1.md`](stage7_behavioral_full_plan_v1.md) |
| **Witness spec** | [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md) |
| **Play witness (maintain)** | `debug_runs/stage7_play_live.json` |

**No gameplay AI in this plan.** Contracts + delay loop + overlay readers only.

---

## Signed product locks (worksheet)

| ID | Pick | Implementation consequence |
|:---|:---|:---|
| **D-S7-01** | StrategicCommand only | v1 queue writer: `CommunicationPlane::StrategicCommand` only |
| **D-S7-02** | Recon + logistics stress | `StrategicOverlayType::Recon` + `LogisticsStress` on minimap path (M3) |
| **D-S7-03** | Move + secure corridor | `MissionKind::MoveCorridor` + `MissionKind::SecureCorridor` |
| **D-S7-04** | Fixed ticks | `dispatch_delay_ticks` constant in M2; no distance solver |
| **D-S7-05** | Tray + map tint | HUD tray badge + `MapViewInstanceId::Stage7IntelMap` tint reader |
| **D-S7-06** | F3 + context tray | `ExplainabilityViewerState` + context tray intel tab (existing shell hooks) |

**EW:** Minimap channel only — **UI-P3-M4-001**; not behavioral M3 overlay v1.

---

## Prerequisite matrix (must stay green)

| Gate | Witness | Role |
|:---|:---|:---|
| S7-PLAY closed | `stage7_play_live.json` → `s7p_steward_green` | Maintain only — no re-seed |
| Wave P | `wave_p_live.json` | Preview contract |
| UI 2B | `ui_shell_migration_live.json` → `phase2b_closed` | Sim egui gate |
| VM-09 slice 2 | `infrastructure_view_isolation_live.json` | Planning unblocked |
| **TRIAGE-VM-09-v2** | — | **Deferred** — required before **M2+ comm authority** in sim (see § VM-09) |

```powershell
cargo test -p proc_A_dine01 --lib stage5 construction industrial_activation stage7_play
```

---

## Authority map (hard)

| Domain | Sole writer | Readers |
|:---|:---|:---|
| Comms DTOs / enums | `src/strategic/comms_contract.rs` | HUD, save, witness |
| Dispatch queue (M2) | New `src/strategic/strategic_command_queue.rs` (proposed) | `stage7_ui_shell` read-only |
| Overlay samples (M3) | Logistics + recon resources → snapshot | `minimap_compositor` extract |
| View pose | `ViewManager` / `view_runtime` | all views |
| World Preview | Wave P panel | **No** mission execute |
| Industrial / construction | existing activation paths | **No** second execute funnel |

**Forbidden:** Second logistics extract; `MapCameraDesired` from behavioral stubs; egui mission authority in sim shell (2B gate).

---

## Existing code (M1 partial — do not duplicate)

| Path | Status |
|:---|:---|
| `src/strategic/comms_contract.rs` | **LANDED** — `CommunicationPlane`, `DispatchMessage`, `DispatchEnvelope`, `BeliefRecord`, `IntelConfidence`, `UtilityChannel`, `StrategicOverlayType`, `MissionIntent` |
| `src/gui/hud/stage7_ui_shell.rs` | **LANDED** — mock DTO viewers (editor/dev) |
| `src/gui/map_view/presentation/mod.rs` | `Stage7IntelMap` presentation slot |
| `src/strategic/comms_contract.rs` tests | RON roundtrip |

**S7B-M1-001** completes: mission enum, queue resource stub, live proof writer, envelope index entry.

---

## Phase M1 — **S7B-M1-001** (contracts + witness scaffold)

| Slice | Files (≤4) | Deliverable |
|:---|:---|:---|
| **M1-A** | `src/strategic/mission_kind.rs` (or extend `comms_contract.rs`) | `MissionKind::{MoveCorridor, SecureCorridor}` + RON tests |
| **M1-B** | `src/strategic/strategic_command_queue.rs` | `StrategicCommandQueue` resource: pending `DispatchMessage` vec, read-only in sim |
| **M1-C** | `src/dev/stage7_behavioral_live_proof.rs` | `build_stage7_behavioral_live_proof_payload` + lib test writes JSON |
| **M1-D** | `src/dev/debug_run_envelope.rs`, `src/dev/mod.rs`, `economy/activation/bridge.rs` or `CorePlugin` | Register writer; add `stage7_behavioral_live.json` to `KNOWN_LIVE_PROOF_PATHS` |

**M1 exit (`behavioral_contract_ok`):**

- All `comms_contract` + `mission_kind` lib tests pass
- `stage7_behavioral_live.json` written with `behavioral_contract_ok: true`, `s7b_m1_green: true`
- `s7p_steward_green` still true in `stage7_play_live.json` (regression)

**M1 forbidden:** Tick dispatch solver; coalition AI; EW overlay publish; gameplay mutation from preview.

---

## Phase M2 — **S7B-M2-001** (fixed-tick dispatch delay) — **DONE**


| Slice | Scope |
|:---|:---|
| **M2-A** | `deliver_after = issued_at + dispatch_delay_ticks` on enqueue (constant e.g. **8** sim ticks — tune in plan only) |
| **M2-B** | Orders-pending surface: ops strip / context tray DTO from queue depth |
| **M2-C** | Stale intel: `IntelConfidence` decay + tray + `Stage7IntelMap` tint |

**M2 exit:**

- `dispatch_delay_ticks` > 0 in live JSON
- `stale_intel_surface: true` when queue non-empty and confidence below threshold
- `s7b_m2_green: true`
- Lib test: message not visible to consumer before `deliver_after`

**Hard gate:** **TRIAGE-VM-09-v2** **GO** before sim-session comm authority (invert bridge M2+).

---

## Phase M3 — **S7B-M3-001** (recon + logistics overlays) — **DONE**


| Slice | Scope |
|:---|:---|
| **M3-A** | Read `LogisticsVisualSnapshot` / corridor book → `StrategicOverlayType::LogisticsStress` samples |
| **M3-B** | Recon belief grid → `StrategicOverlayType::Recon` samples |
| **M3-C** | Publish into minimap compositor path (same spine as UI-P3-M3 — no parallel extract) |

**M3 exit:**

- `recon_overlay_enabled: true` and `logistics_stress_overlay_enabled: true`
- `minimap_compositor_live.json` shows overlay rows > 0 for behavioral channels
- `s7b_m3_green: true` → `s7b_steward_green: true`

---

## Schedule placement (Bevy)

```text
CoreSystemSet::Sim
  → logistics / industrial (existing)
  → strategic_command_queue_tick (M2, after sim tick)
  → overlay_sample_publish (M3, before render extract)

Render / HUD
  → stage7_ui_shell (read-only DTO)
  → write_stage7_behavioral_live_proof_system (Simulation, interval)
```

Do **not** insert before `ViewportAuthority` resolve or parallel to `fill_logistics_snapshot` writer.

---

## VM-09 v2 (deferred — not blocking this plan)

| ID | Status | Note |
|:---|:---|:---|
| **TRIAGE-VM-09-v2** | **DONE** | Invert bridge — [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md) |

**S7B-PLAN-001** sign-off does **not** require v2 landed. Coder may start **M1** without v2; **M2+** is blocked until v2 witness green.

---

## Witness bundle

| File | Phase | Rollup field |
|:---|:---|:---|
| [`debug_runs/stage7_behavioral_live.json`](../debug_runs/stage7_behavioral_live.json) | M1+ | `s7b_steward_green` |
| `stage7_play_live.json` | maintain | `s7p_steward_green` |
| `minimap_compositor_live.json` | M3 | overlay row counts |

Schema: [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md).

---

## Copy-paste — S7B-PREFLIGHT-001 (@sim-steward)

```
Track: S7-BEHAV — S7B-PREFLIGHT-001
Read: docs/archive/2026-06-src-dev/plans/stage7_behavioral_implementation_plan_v1.md
      docs/archive/2026-06-src-dev/plans/stage7_behavioral_live_witness_spec_v1.md
Verify: cargo test -p proc_A_dine01 --lib stage7_play comms_contract
Witness: stage7_play_live.json s7p_steward_green
Deliver: GO/NO-GO in steward note — M1 file budget ≤4
```

---

## Copy-paste — S7B-M1-001 (@coder)

```
Track: S7-BEHAV — S7B-M1-001
Read: docs/archive/2026-06-src-dev/plans/stage7_behavioral_implementation_plan_v1.md
      src/strategic/comms_contract.rs (extend, do not fork)
Prereq: S7B-PREFLIGHT GO
Deliver: mission_kind + StrategicCommandQueue stub + stage7_behavioral_live_proof.rs
Verify: cargo test -p proc_A_dine01 --lib comms_contract stage7_behavioral
Do NOT: dispatch solver, MapCameraDesired writers, preview mutation
```

---

## Acceptance — S7B-PLAN-001

| # | Criterion | Met |
|:---:|:---|:---:|
| 1 | Worksheet picks reflected in phases | ☑ |
| 2 | M1/M2/M3 slices + file budget | ☑ |
| 3 | Witness schema document + stub JSON | ☑ |
| 4 | Authority map + VM-09 deferral explicit | ☑ |
| 5 | Existing `comms_contract` acknowledged | ☑ |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — S7B-PLAN-001 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial implementation plan + witness schema |
