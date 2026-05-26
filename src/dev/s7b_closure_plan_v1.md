# S7B-CLOSURE-PLAN-001 — Stage 7 behavioral post-M3 rollup `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **S7B-CLOSURE-PLAN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — M1/M2/M3 **CLOSED** on disk; tune-only forward |
| **Impl plan** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) |
| **Witness spec** | [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md) |
| **Track plan (superseded rollup)** | [`stage7_behavioral_track_plan_v1.md`](stage7_behavioral_track_plan_v1.md) → use **this doc** for exit |
| **Witness** | [`debug_runs/stage7_behavioral_live.json`](../../debug_runs/stage7_behavioral_live.json) |

**No Rust in this deliverable.**

---

## Executive summary

| Phase | Verdict | Rollup field |
|:---|:---|:---|
| **Preflight** | **GO** | `s7b_preflight_green: true` |
| **M1** contracts + queue | **CLOSED** | `s7b_m1_green`, `behavioral_contract_ok` |
| **M2** fixed-tick dispatch | **CLOSED** | `s7b_m2_green`, `dispatch_delay_ticks: 8` |
| **M3** recon + logistics overlays | **CLOSED** | `s7b_m3_green`, sample counts > 0 |
| **Steward rollup** | **PASS** | `s7b_steward_green: true` |

**Maintain regression only** — do not reopen M1–M3 without contradicting lib tests.

---

## Witness fields (authoritative)

| Path | Value (2026-05-25 disk) | Gate |
|:---|:---|:---|
| `behavioral_contract_ok` | `true` | M1 |
| `communication_plane_v1` | `StrategicCommand` | D-S7-01 |
| `mission_kinds_supported` | `MoveCorridor`, `SecureCorridor` | D-S7-03 |
| `dispatch_delay_ticks` | `8` | M2 |
| `dispatch_delay_model` | `fixed_ticks` | D-S7-04 |
| `stale_intel_surface` | `true` | M2 |
| `orders_pending_ui_hook` | `true` | M2 |
| `recon_overlay_enabled` | `true` | M3 |
| `logistics_stress_overlay_enabled` | `true` | M3 |
| `recon_overlay_sample_count` | `100` | M3 |
| `logistics_stress_sample_count` | `18` | M3 |
| `s7b_steward_green` | `true` | rollup |
| `s7p_play_witness_ok` | `true` | cross-check play JSON |

Full schema: [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md).

---

## Done vs tune (forward)

| Item | Status | Owner |
|:---|:---|:---|
| DTOs + `StrategicCommandQueue` | **DONE** | maintain |
| Fixed-tick delay (no distance solver) | **DONE** | tune constant only via plan amendment |
| Tray + map tint stale intel | **DONE** | designer polish P2 |
| F3 + context tray explainability | **DONE** | copy pass optional |
| Coalition AI / full dispatch solver | **OUT OF SCOPE** v1 | future lane |
| VM-09-v2 invert bridge | **DONE** (per fleet snapshot) | maintain witness |
| Playtest scenario script | **OPEN** optional | designer — not blocking rollup |

---

## Playtest checklist (operator)

| # | Step | Pass when |
|:---:|:---|:---|
| 1 | Enter **Simulation** | PLAY-01 HUD defaults; `phase2b_closed` |
| 2 | Open context tray / intel | Stage 7 DTO surfaces visible (dev/mock OK) |
| 3 | Enqueue strategic command | `pending_dispatch_count` > 0 in witness after action |
| 4 | Advance sim ticks | delivery after `dispatch_delay_ticks` (8) |
| 5 | Observe stale intel | tray/tint when confidence low |
| 6 | Minimap | logistics stress + recon channels on (`minimap_compositor` crosscheck optional) |
| 7 | Regression lib | `cargo test -p proc_A_dine01 --lib stage7_behavioral comms_contract stage7_play` |

---

## Regression (maintain)

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play comms_contract steward_s7b_preflight_001_lib_bundle
```

**Cross-check:** `stage7_play_live.json` → `s7p_steward_green: true` (do not merge JSON files).

---

## Board hygiene

| Doc | Action |
|:---|:---|
| [`stage7_behavioral_track_plan_v1.md`](stage7_behavioral_track_plan_v1.md) | Mark **S7-BEHAV** implementation **CLOSED**; link here |
| [`stage_open_todos_v1.md`](stage_open_todos_v1.md) | S7B-M* rows **done** |
| [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) | Next: **FIRE7-PLAN-001** implementation waves |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Post-M3 closure rollup |
