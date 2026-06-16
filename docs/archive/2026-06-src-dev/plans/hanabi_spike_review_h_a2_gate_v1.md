# DESIGN-HANABI-SPIKE-REVIEW-001 — Hanabi spike formal review (H-A2 gate) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-HANABI-SPIKE-REVIEW-001** |
| **Spike report** | [`experiments/hanabi_validation/report_v1.md`](../../experiments/hanabi_validation/report_v1.md) |
| **Bounds contract** | [`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md) (**DESIGN-HANABI-BOUNDS-001**) |
| **Adoption plan** | [`plan_hanabi_adoption_v1.md`](plan_hanabi_adoption_v1.md) — phases H-A0 → H-A2 |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Spike verdict (coder)** | **PASS (qualified)** per report_v1 |
| **Designer review verdict** | **PASS (qualified)** |
| **H-A2 gate** | **CLEARED (qualified)** — feature-flag plugin path only |
| **No Rust** | Review record only |

---

## Executive summary

Designer re-review of **H-A-SPIKE-001** (`report_v1.md`) against signed bounds confirms all three **ACCEPT** L3 presets are within numeric limits and rubric intent. The documented **REJECT** anti-pattern (`reject_arcade_muzzle_stack`) correctly exceeds bounds and must never ship. Main `EnginePlugin` remains unlinked — charter intact.

**H-A2** may proceed for `@coder` behind `hanabi_l3` feature flag per adoption plan. **Default engine binary** must not add `HanabiPlugin` until witness row `hanabi_spike_report_present` is wired and tactical VFX rollup stays green.

---

## Bounds crosswalk

| Bounds token | Limit | Spike aggregate / worst ACCEPT preset | Designer |
|:---|:---|:---|:---|
| Max instances / event | ≤ 32 | Peak 24 (`fire_ember_burst`) | **PASS** (upper PASS band — prefer headroom in production) |
| Peak α | ≤ 0.45 | Worst 0.35 | **PASS** |
| Lifetime window | 0.2–1.2 s | All ACCEPT presets in window | **PASS** |
| Material kick-up (fire/water) | ACCEPT | `fire_ember_burst`, `water_splash_mist` | **PASS** |
| Construction micro-spark | ACCEPT | `construction_micro_spark` (8 inst, α 0.22) | **PASS** |
| Arcade muzzle / neon stack | REJECT | `reject_arcade_muzzle_stack` classified REJECT | **PASS** (reference only) |
| L1 / weather write | REJECT | Not attempted in spike | **PASS** (charter) |
| Minimap / strategic | REJECT | Not attempted | **PASS** (charter) |

---

## Per-preset designer disposition

| Preset | Report verdict | Designer disposition | Production wiring |
|:---|:---|:---|:---|
| `fire_ember_burst` | PASS | **ACCEPT** | Allowed L3 fire-edge embellishment candidate |
| `water_splash_mist` | PASS | **ACCEPT** | Allowed L3 water-surface candidate |
| `construction_micro_spark` | PASS | **ACCEPT** | Allowed one-shot commit spark |
| `reject_arcade_muzzle_stack` | REJECT | **REJECT** | Anti-pattern reference — do not merge |

**TUNE note:** `fire_ember_burst` at 24 instances is within bounds but near the experiment **TUNE** band (>24 in `bounds.rs`). Production presets should target ≤ 20 instances unless witness proves fill-rate headroom.

---

## H-A2 gate checklist

| Criterion | Status | Evidence |
|:---|:---|:---|
| Spike report on disk with counts / α / histogram | **PASS** | `report_v1.md` |
| All ACCEPT presets within designer numeric bounds | **PASS** | Per-preset table in report |
| REJECT anti-pattern documented | **PASS** | `reject_arcade_muzzle_stack` |
| Bevy 0.18 experiment compiles | **PASS** | `cargo check -p hanabi_validation` |
| Main app does not link `bevy_hanabi` | **PASS** | Report § Regression |
| `stage5_full_app_live.json` tactical VFX green | **PASS** | `tactical_vfx_witness.all_green: true` |
| `hanabi_spike_report_present` witness wired | **PENDING** | @coder — not a designer blocker |
| Operator PNGs under `hanabi_spike/` | **OPTIONAL** | Not required for H-A2 designer clearance |

**H-A2 gate outcome:** **CLEARED (qualified)** for coder implementation of optional `HanabiEmbellishmentPlugin` + `hanabi_l3` feature flag. **Not cleared** for default-binary plugin registration.

---

## Forbidden (unchanged)

- `app.add_plugins(HanabiPlugin)` on default engine binary until H-A2 coder gates complete.
- Weather / smoke authority via Hanabi.
- Minimap or strategic-zoom particle draws.
- Any preset derived from `reject_arcade_muzzle_stack`.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** — H-A2 **CLEARED (qualified)** | 2026-05-27 |

**Unblocks:** `H-A2` / `hanabi_l3` feature-flag work · `PLAN-HANABI-ADOPTION-001` exec · optional `W4-C` event embellishment planning. **Does not** authorize main-plugin merge without coder witness row.
