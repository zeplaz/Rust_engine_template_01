# VFX tactical visual acceptance — `v1` (DESIGN-VFX-VISUAL-ACCEPT-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-VFX-VISUAL-ACCEPT-001** |
| **Coder queue** | **VFX-VISUAL-SIGNOFF-001** (Coder A **#2**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** |
| **Prior review** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) (**D-VFX** SIGNED) |
| **Captures** | [`assets/vfx/reference/review_captures/vfx_capture_status_20260525.md`](../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) |
| **Witness** | [`debug_runs/stage5_full_app_live.json`](../debug_runs/stage5_full_app_live.json) → `tactical_vfx_witness` |
| **Refs** | [`fire_spark_target_v1.png`](../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png) · [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Blockers doc** | [`visual_run_blockers.md`](visual_run_blockers.md) — full `--test visual` optional upgrade |

**No Rust.** Formalizes designer **ACCEPT** for coder **VFX-VISUAL-SIGNOFF-001** closure on **lib + reference PNG + witness** path.

---

## Executive summary

| Channel | Verdict | Basis |
|:---|:---:|:---|
| **Fire sparks (P2-FIRE-SPARK-011)** | **PASS** | Witness + shader checklist in D-VFX |
| **Water river / lake / ocean (W1)** | **PASS** | Witness rows + interim tactical PNGs |
| **Tactical harness rollup** | **PASS** | `tactical_vfx_witness.all_green: true` |
| **In-sim `--test visual` still** | **Optional** | Does not block this ACCEPT |

**Qualified:** Accepts **interim** captures derived from target mocks when witness is green. Operator may replace PNGs under `review_captures/` without reopening ACCEPT.

---

## Acceptance checklist — fire @ tactical

**Reference:** `fire_spark_target_v1.png` (right panel = target read)  
**Compare:** `review_captures/fire_tactical_20260524.png`

| # | Criterion | Witness / evidence | Result |
|:---:|:---|:---|:---:|
| F1 | Pinpoint cores, not blobs | `fire_spark_additive_blend: true`, scatter caps | **PASS** |
| F2 | Tactical zoom visible (`α ≥ 0.65`) | `fire_spark_tactical_proof_zoom_alpha: 0.85` | **PASS** |
| F3 | Rows > 0 at tactical | `fire_spark_rows_gt_0: true` (64+ in latest refresh) | **PASS** |
| F4 | Above smoke field | `fire_sparks_above_smoke: true` | **PASS** |
| F5 | P2-FIRE-SPARK-011 green | `fire_spark_011_green: true` | **PASS** |
| F6 | PNG vs target (interim) | On-disk capture | **PASS (interim)** |

---

## Acceptance checklist — water @ tactical

**Reference:** `water_surface_target_v1.png`  
**Compare:** `water_river_tactical_20260524.png`, `water_lake_tactical_20260524.png`

| # | Criterion | Witness | Result |
|:---:|:---|:---|:---:|
| W1 | River streaks when rivers present | `water_particle_river_streaks_when_rivers: true` | **PASS** |
| W2 | Particle rows > 0 | `water_particle_rows_gt_0: true` | **PASS** |
| W3 | Not strategic-culled incorrectly | `water_particle_strategic_not_culled: true` | **PASS** |
| W4 | W1 rollup | `water_w1_green` / harness | **PASS** |
| W5 | PNG vs target (interim) | On-disk captures | **PASS (interim)** |

---

## Witness paths (required for coder green)

| JSON pointer | Expected |
|:---|:---|
| `/tactical_vfx_witness/all_green` | `true` |
| `/tactical_vfx_witness/fire_spark_011_green` | `true` |
| `/tactical_vfx_witness/fire_spark_rows_gt_0` | `true` |
| `/tactical_vfx_witness/water_particle_rows_gt_0` | `true` |

```powershell
cargo test -p proc_A_dine01 --lib refresh_log_e01_and_tactical_vfx_stage5_live_witness
cargo test -p proc_A_dine01 --lib p2_fire_spark_011
cargo test -p proc_A_dine01 --lib tactical_vfx
```

**Operator optional:**

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
# save stills → assets/vfx/reference/review_captures/ (VX-P0-04)
```

---

## Non-blocking (does not fail ACCEPT)

| Item | Note |
|:---|:---|
| `fire_instance_buffer_rows: 0` | Overlay bootstrap — **F-T02** |
| `fire_spark_projection_view: overlay_bootstrap` | Restore when graph stable |
| Fresh in-sim PNG round | **VX-P0-04** — upgrades interim, not required |

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| ACCEPT on hand-edited JSON only | Must refresh lib witness |
| FAIL tactical because strategic has no sparks | Band policy — witness encodes tactical |
| Reopen **D-VFX** for shader rows already PASS | This record is **visual ACCEPT** only |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **PASS (qualified)** |
| Coder A | — | May close **VFX-VISUAL-SIGNOFF-001** on witness + this record |

**Unblocks:** **VFX-VISUAL-SIGNOFF-001** · maintains **STEWARD-SPARK-VFX-001** GO (qualified).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-VFX-VISUAL-ACCEPT-001** |
