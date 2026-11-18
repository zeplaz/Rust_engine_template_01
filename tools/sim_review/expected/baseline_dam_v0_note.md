# DamFlood baseline v0 (intentional fail)

**Plan:** PLAN-DAM-FLOOD-BENCHMARK-v1 DF-1 Accept  
**Date:** 2026-11-16  
**Scene:** `DamFlood` (`--test dam` / `playcheck.py --scene dam`)

## Intent

Baseline **v0** is not a green lock. It records that the review path sees the dump block and that every **DF-dependent** `dam.*` matrix row **fails** until the owning slice lands (DF-2…9). That proves the benchmark detects missing systems.

## Expected FAIL rows (DF-dependent)

| Matrix id | Owning slice |
|:---|:---|
| `dam.water_solver_wired` | DF-2 |
| `dam.s0_gauge_balance` | DF-2 |
| `dam.s2_downstream_ratio` | DF-2 |
| `dam.s3_drowned_cells` | DF-5 |
| `dam.s4_hydro_mw` | DF-6 |
| `dam.s5_impact_err_m` | DF-7 |
| `dam.s7_wave_arrival_g3` | DF-8 |
| `dam.s8_inundated_tiles` | DF-9 |
| `dam.s9_burn_rate_dead_over_live` | DF-5 |

## Expected PASS rows (DF-1 harness)

| Matrix id | Notes |
|:---|:---|
| `dam.harness.seeded` | Fixture + GaugeBook seeder |
| `dam.extent` | MediumSmall 320² |
| `dam.stage_s0` | Stage book at S0River |

## Scored artifact (2026-11-16)

```text
python tools/sim_review/score_dump.py debug_runs/sim_review/dam_flood_baseline_v0_dump.json --out debug_runs/sim_review/dam_v0
```

DamFlood-scoped result: harness rows **pass** (`dam.harness.seeded`, `dam.extent`, `dam.stage_s0`); all nine DF-dependent `dam.*` rows **fail** as required for baseline v0. Dashboard: `debug_runs/sim_review/dam_v0/dashboard.html`.

## Next

DF-2 CPU level-pool + virtual-pipes ROI (`SurfaceWaterField` sole writer). Do not mark DF-2 ADOPTED without a production `file:line` caller.
