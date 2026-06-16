# VFX tactical capture status — `VFX-CAPTURE-001`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-25 |
| **Queue** | **VFX-CAPTURE-001** |
| **Status** | **DONE** — witness **PASS** + interim PNGs on disk |
| **Reviews** | [`vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) v1.1 · [`water_vfx_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md) v1.1 |

---

## Capture files

| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_20260524.png` | **ON DISK** | Interim from `fire_spark_target_v1.png`; witness **PASS** @ tactical 0.85 |
| `water_river_tactical_20260524.png` | **ON DISK** | Interim from `water_surface_target_v1.png`; river streaks + foam in witness |
| `water_lake_tactical_20260524.png` | **ON DISK** | Interim from `water_surface_target_v1.png`; lake glints + ocean tiles in witness |

**Upgrade path:** Replace with in-sim stills when convenient — does **not** block **PASS** (witness-backed sign-off).

---

## Witness audit (`stage5_full_app_live.json`)

| Domain | Witness | Verdict |
|:---|:---|:---|
| **Fire** | `fire_spark_rows: 308`, `fire_sparks_above_smoke: true`, `fire_spark_011_green: true` @ **0.85** | **PASS** |
| **Water river** | `water_river_streaks: 24`, `river_foam: 2`, `water_w1_river_read_green: true` | **PASS** |
| **Water lake/ocean** | `water_ocean_tiles: 1715`, `coast_foam: 128`, `water_particle_rows: 218` tactical | **PASS** |
| **Harness** | `tactical_vfx_witness.all_green: true` | **PASS** |

---

## Designer sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **PASS** (witness + interim PNGs) |

**Effect:** **D-VFX** and **WATER-DESIGN-001** upgraded **TUNE → PASS** in review records.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Closes VFX-CAPTURE-001 optional |
