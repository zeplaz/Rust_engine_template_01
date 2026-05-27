# VFX tactical capture status — wave 5 (DESIGN-VFX-CAPTURE-WAVE5-001) `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-26 |
| **Queue** | **DESIGN-VFX-CAPTURE-WAVE5-001** |
| **Variant** | wave 5 capture matrix |
| **Verdict** | **DEFER** (capture matrix is specified; promotion to DONE requires operator/app run captures) |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `/tactical_vfx_witness/all_green` |
| **Do not break** | same witness gate: `/tactical_vfx_witness/all_green == true` |

---
## Capture matrix (what operators should capture in sim)
Capture style: **tactical zoom stills** (no editor replay panel).

| Domain | Expected PNG name pattern | Comparison notes |
|:---|:---|:---|
| Tactical Fire | `fire_tactical_YYYYMMDD.png` | Compare fire sparks vs `elemental_sparks/fire_spark_target_v1.png` |
| Tactical Water (river) | `river_tactical_YYYYMMDD.png` | Check river streaks + motion continuity |
| Tactical Water (lake/ocean) | `water_lake_or_ocean_YYYYMMDD.png` | Verify panel readability + no missing foam |
| Logistics overlay | (if available in sim capture shell) `logistics_tactical_YYYYMMDD.png` | Confirm logistics rows render above terrain |
| Construction MV delta | `construction_mv_tactical_YYYYMMDD.png` | Corridor phase overlays should not hide tactical water/fire |

---
## Acceptance (designer)
1. Captures cover the full tactical set (fire + water + overlay) so coders can compare deltas.
2. Construction MV overlay does not occlude VFX panels.

---
## On-disk interim notes
- As of this spec, do not assume new wave5 PNGs exist yet; once operator runs captures, update the “On disk” section below with final filenames and match notes.

---
## On disk (to fill during operator capture)
| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_YYYYMMDD.png` | TBD | Compare vs target |
| `river_tactical_YYYYMMDD.png` | TBD | Check streaks |
| `water_lake_or_ocean_YYYYMMDD.png` | TBD | Check readability |
| `logistics_tactical_YYYYMMDD.png` | TBD | Overlay legibility |
| `construction_mv_tactical_YYYYMMDD.png` | TBD | Corridor phase + VFX coexistence |

