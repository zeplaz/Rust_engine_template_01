# DamFlood DF-2 matrix lock (sim core)

**Plan:** PLAN-DAM-FLOOD-BENCHMARK-v1 DF-2  
**Date:** 2026-11-16  
**Scene:** `DamFlood` (`playcheck.py --scene dam`)

## DF-2 Accept rows (PASS)

| Matrix id | Notes |
|:---|:---|
| `dam.harness.seeded` | Fixture + GaugeBook |
| `dam.extent` | 320² |
| `dam.stage_s0` | `s0_entered` |
| `dam.water_solver_wired` | `SurfaceWaterField.wired` |
| `dam.s0_gauge_balance` | ≤0.20 interim (latched) |
| `dam.s2_downstream_ratio` | ≤0.45 interim (latched) |

## Still FAIL until later slices

| Matrix id | Owning slice |
|:---|:---|
| `dam.s3_drowned_cells` | DF-5 |
| `dam.s4_hydro_mw` | DF-6 finish (peak latch + crest) |
| `dam.s5_impact_err_m` | DF-7 |
| `dam.s7_wave_arrival_g3` | DF-8 |
| `dam.s8_inundated_tiles` | often PASS early (inundation live) |
| `dam.s9_burn_rate_dead_over_live` | DF-5 |

## Re-score

```powershell
python tools/sim_review/playcheck.py --scene dam --view voxel --frame 900 --ticks 1200 --min-secs 90
python tools/sim_review/score_dump.py
```
