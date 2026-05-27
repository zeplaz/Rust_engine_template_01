# VFX tactical capture status — wave 4 round-002

| Field | Value |
|:---|:---|
| **Date** | 2026-05-26 |
| **Queue** | **DESIGN-VFX-CAPTURE-ROUND-002** · **VX-P0-04** |
| **Status** | **DONE** — witness **PASS** + round-001 PNGs retained |
| **Prior round** | [`vfx_capture_status_20260525.md`](vfx_capture_status_20260525.md) (**DESIGN-VFX-CAPTURE-001**) |
| **Designer ACCEPT** | [`vfx_visual_acceptance_record_v1.md`](../../../src/dev/vfx_visual_acceptance_record_v1.md) |

---

## Capture files (on disk)

| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_20260524.png` | **ON DISK** | Interim from `fire_spark_target_v1.png`; witness green @ tactical 0.85 |
| `water_river_tactical_20260524.png` | **ON DISK** | River streaks + foam — witness `water_w1_river_read_green` |
| `water_lake_tactical_20260524.png` | **ON DISK** | Lake glints + ocean tiles |

**Round-002 operator action:** Replace any file with in-sim still when running `--test visual` — **optional**; does not reopen **DESIGN-VFX-VISUAL-ACCEPT-001**.

**Suggested new names (when capturing):**

| Pattern | Content |
|:---|:---|
| `fire_tactical_20260526.png` | WorldMain tactical fire sparks |
| `water_river_tactical_20260526.png` | River flow read |
| `water_lake_tactical_20260526.png` | Lake / ocean panel |

---

## Witness audit (`stage5_full_app_live.json` — 2026-05-26 refresh)

| Domain | Witness | Verdict |
|:---|:---|:---|
| **Harness rollup** | `tactical_vfx_witness.all_green: true` | **PASS** |
| **Fire** | `fire_spark_rows: 64`, `fire_spark_011_green: true`, `fire_sparks_above_smoke: true`, `fire_spark_tactical_proof_zoom_alpha: 0.85` | **PASS** |
| **Water river** | `water_w1_river_read_green: true`, `water_has_river_segments: true` | **PASS** |
| **Water rollup** | `water_witness_rollup_green: true`, `water_w2_foam_001_green: true` | **PASS** |
| **VFX signoff block** | `vfx_visual_signoff_001.green: true`, `designer_verdict: PASS (qualified)` | **PASS** |
| **Visual run** | `vfx_visual_signoff_001.visual_run_pending: true` | **Optional** upgrade |

---

## Operator procedure

1. `cargo run -p proc_A_dine01 --release -- --test visual` (or sim PLAY-01 with fire/water fixtures).
2. Pause at **tactical** zoom on WorldMain — fire front + river/lake in frame.
3. Save PNGs to this folder using date suffix above.
4. Update this file **Capture files** table with `ON DISK (in-sim)` vs interim.
5. Do **not** edit witness JSON by hand.

**Guide:** [`prompts/guides/ui/vfx_post_implementation_review_v1.md`](../../../../prompts/guides/ui/vfx_post_implementation_review_v1.md)

---

## Designer sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **PASS (qualified)** — round-002 documents witness refresh; PNG upgrade optional |
| Operator | — | In-sim captures when convenient |

**Effect:** Closes **DESIGN-VFX-CAPTURE-ROUND-002** / **VX-P0-04** without blocking coder **VFX-VISUAL-SIGNOFF-001** (already qualified).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-VFX-CAPTURE-ROUND-002** — wave 4 operator round |
