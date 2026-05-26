# VFX tactical capture status — `VFX-CAPTURE-001`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-25 (updated) |
| **Queue** | **VFX-CAPTURE-001** (operator, optional) |
| **Review** | [`src/dev/vfx_design_review_record_v1.md`](../../../src/dev/vfx_design_review_record_v1.md) |

---

## Capture files

| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_20260524.png` | **INTERIM** | Design-target stand-in (`fire_spark_target_v1.png`) — replace with sim capture |
| `water_river_tactical_20260524.png` | **INTERIM** | Design-target stand-in (`water_surface_target_v1.png`) |
| `water_lake_tactical_20260524.png` | **INTERIM** | Design-target stand-in (`water_surface_target_v1.png`) |

---

## Witness audit (2026-05-25 visual)

From [`debug_runs/stage5_full_app_live.json`](../../../debug_runs/stage5_full_app_live.json) at tactical zoom (`0.85`):

| Domain | Witness | Design checklist |
|:---|:---|:---|
| **Fire** | `fire_spark_rows: 308`, `fire_sparks_above_smoke: true` | D-F09 **PASS** (witness) |
| **Water river** | `water_river_streaks: 27`, `water_particle_rows: 76` | D-W03 **PASS** (witness); PNG interim for visual sign-off |
| **Water lake/ocean** | `water_ocean_tiles: 1715`, `water_particle_coast_foam: 128` | D-W04 **PASS** (witness); PNG interim for lake read |

**Effect:** Water may move **TUNE → PASS (witness + interim PNG)** for operator review; replace PNGs with in-sim stills when convenient.

**Procedure:** `cargo run -p proc_A_dine01 --release` → Simulation → tactical zoom → save three PNGs here → re-run § checklist in review brief.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Interim PNGs committed; ocean witness green |
| v1.0.0 | 2026-05-24 | Witness interim; PNGs pending operator |
