# VFX tactical capture status — wave 6 (DESIGN-VFX-CAPTURE-ROUND-003) `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-27 |
| **Queue** | **DESIGN-VFX-CAPTURE-ROUND-003** |
| **Variant** | wave 6 capture matrix (round-003) |
| **Verdict** | **PASS (qualified)** — matrix + witness **PASS**; wave6 dated PNGs optional |
| **Prior round** | [`vfx_capture_status_wave5.md`](vfx_capture_status_wave5.md) · [`vfx_capture_status_wave4.md`](vfx_capture_status_wave4.md) |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `/tactical_vfx_witness/all_green`, `/f2_extract_witness/green` |
| **Do not break** | `/tactical_vfx_witness/all_green == true` |

---

## Capture matrix (wave 6 — operator sim stills)

Capture style: **tactical zoom stills** (WorldMain; no editor replay panel).

| Domain | Expected PNG name pattern | Comparison / witness row |
|:---|:---|:---|
| Tactical Fire (F2 extract) | `fire_tactical_YYYYMMDD.png` | `f2_extract_witness.green`, `fire_instance_buffer_rows > 0` |
| Fire sparks (D-F09) | `fire_sparks_tactical_YYYYMMDD.png` | `fire_spark_011_green`, tactical α ≥ 0.85 |
| Tactical Water (river) | `water_river_tactical_YYYYMMDD.png` | `water_w1_river_read_green` |
| Tactical Water (lake/ocean) | `water_lake_or_ocean_YYYYMMDD.png` | `water_witness_rollup_green`, `water_w2_foam_001_green` |
| Logistics overlay | `logistics_tactical_YYYYMMDD.png` | Rows above terrain; no VFX occlusion |
| Construction MV + parametric | `construction_mv_tactical_YYYYMMDD.png` | `construction_stage_live.json` parametric + R4 MV ghost green |
| Smoke / tactical stack | `smoke_tactical_YYYYMMDD.png` | `fire_sparks_above_smoke: true` when reproducing |
| Hanabi spike (optional) | `hanabi_spike/ember_YYYYMMDD.png` | Post **DESIGN-HANABI-SPIKE-REVIEW-001** — experiment only |

---

## Witness audit (2026-05-27)

| Domain | Witness pointer | Verdict |
|:---|:---|:---|
| **Harness rollup** | `tactical_vfx_witness.all_green: true` | **PASS** |
| **F2 extract** | `f2_extract_witness.green: true`, `fire_instance_buffer_rows: 1` | **PASS** |
| **Fire sparks** | `fire_spark_rows_gt_0`, `fire_spark_011_green`, tactical α 0.85 | **PASS** |
| **Water** | `water_witness_rollup_green`, W1/W2 rows | **PASS** |
| **Strategic cull** | `water_particle_strategic_not_culled` + zero spark at strategic | **PASS** by design |
| **Construction coexistence** | `construction_parametric_placement_001.green`, `construction_r4_mv_ghost_001.green` | **PASS** (construction witness file) |

**Designer rule:** Witness green + signed wave 5/4 interim PNGs = **qualified** PASS; wave6 filenames are **optional refresh** (same policy as round-002 / wave 5).

---

## On disk (interim baseline)

| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_20260524.png` | **ON DISK** (interim) | wave 4/5 baseline |
| `water_river_tactical_20260524.png` | **ON DISK** (interim) | river read green |
| `water_lake_tactical_20260524.png` | **ON DISK** (interim) | lake/ocean green |

**Wave 6 operator optional refresh:**

| Pattern | Status |
|:---|:---|
| `fire_tactical_20260527.png` | TBD — optional |
| `fire_sparks_tactical_20260527.png` | TBD — optional |
| `water_river_tactical_20260527.png` | TBD — optional |
| `water_lake_or_ocean_20260527.png` | TBD — optional |
| `logistics_tactical_20260527.png` | TBD — optional |
| `construction_mv_tactical_20260527.png` | TBD — optional |
| `smoke_tactical_20260527.png` | TBD — optional |
| `hanabi_spike/*_20260527.png` | TBD — optional (experiment lane) |

---

## Acceptance (designer)

1. ☑ Matrix extends wave 5 with F2 extract + construction MV + optional Hanabi spike folder.
2. ☑ Construction/parametric overlays must not occlude tactical fire/water panels.
3. ☑ Interim PNGs acceptable until operator run; does not reopen **DESIGN-VFX-VISUAL-ACCEPT-001**.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-05-27 |

**Unblocks:** operator **VX-P0-04** wave 6 capture round; does not reopen closed VFX acceptance records.
