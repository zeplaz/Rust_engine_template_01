# Stage 7 behavioral live witness spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **S7B-PLAN-001** (witness contract) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Witness file** | `debug_runs/stage7_behavioral_live.json` |
| **Writer (target)** | `src/dev/stage7_behavioral_live_proof.rs` |
| **Envelope** | `debug_run_envelope_v1` — [`debug_run_envelope.rs`](debug_run_envelope.rs) |
| **Impl plan** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) |

---

## Profile

| Field | Type | Value |
|:---|:---|:---|
| `profile` | string | `STAGE7_BEHAVIORAL` |
| `source_system` | string | `stage7_behavioral_live_proof` |

---

## Rollup gates

| Path | Type | Green when |
|:---|:---|:---:|
| `s7b_steward_green` | bool | `s7b_m3_green` (final); partial: `s7b_m1_green` during M1-only |
| `s7b_m1_green` | bool | `behavioral_contract_ok` && play witness still green |
| `s7b_m2_green` | bool | `dispatch_delay_ticks` > 0 && delay test passes |
| `s7b_m3_green` | bool | `recon_overlay_enabled` && `logistics_stress_overlay_enabled` |

**Cross-check (maintain, do not merge files):**

| File | Field |
|:---|:---|
| `stage7_play_live.json` | `s7p_steward_green: true` |

---

## M1 — contract fields

| Path | Type | Required (M1) | Meaning |
|:---|:---|:---:|:---|
| `behavioral_contract_ok` | bool | `true` | DTOs + queue resource exist; lib tests pass |
| `communication_plane_v1` | string | `StrategicCommand` | **D-S7-01 A** |
| `mission_kinds_supported` | array | `["MoveCorridor","SecureCorridor"]` | **D-S7-03 A** |
| `overlay_channels_v1` | array | `["Recon","LogisticsStress"]` | **D-S7-02 A** (M3 enables publish) |
| `dispatch_delay_model` | string | `fixed_ticks` | **D-S7-04 A** |
| `intel_stale_surface` | string | `tray_and_map_tint` | **D-S7-05 A** |
| `explainability_surface` | string | `f3_and_context_tray` | **D-S7-06 C** |
| `s7p_play_witness_ok` | bool | `true` | Copied check from play JSON |

---

## M2 — dispatch delay fields

| Path | Type | Green when |
|:---|:---|:---:|
| `dispatch_delay_ticks` | number | `> 0` (default **8** in impl plan) |
| `pending_dispatch_count` | number | any (telemetry) |
| `stale_intel_surface` | bool | `true` when stale policy active |
| `orders_pending_ui_hook` | bool | `true` when HUD DTO wired |

---

## M3 — overlay fields

| Path | Type | Green when |
|:---|:---|:---:|
| `recon_overlay_enabled` | bool | `true` |
| `logistics_stress_overlay_enabled` | bool | `true` |
| `recon_overlay_sample_count` | number | `>= 1` in sim proof |
| `logistics_stress_sample_count` | number | `>= 1` in sim proof |
| `minimap_compositor_crosscheck` | bool | optional: read `minimap_compositor_live.json` rows |

---

## `decisions` block (audit trail)

Mirror worksheet picks for agents:

```json
"decisions": {
  "d_s7_01": "StrategicCommand_only",
  "d_s7_02": "Recon_logistics_stress",
  "d_s7_03": "Move_secure_corridor",
  "d_s7_04": "fixed_ticks",
  "d_s7_05": "tray_and_map_tint",
  "d_s7_06": "f3_and_context_tray"
}
```

---

## Example stub (pre-M1 — schema only)

See [`debug_runs/stage7_behavioral_live.json`](../debug_runs/stage7_behavioral_live.json).

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play comms_contract
```

**Pass (M1):** test writes JSON with `behavioral_contract_ok: true`, `s7b_m1_green: true`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | S7B-PLAN-001 witness schema |
