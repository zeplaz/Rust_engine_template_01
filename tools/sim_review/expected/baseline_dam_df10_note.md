# DamFlood DF-10 — baseline lock **BLOCKED**

**Scene:** `DamFlood` (`playcheck.py --scene dam --seeds 1`, seed 77)  
**Date:** 2026-11-16

## Honest verdict

Do **not** commit `baseline_dam_v1.json` yet.

This wave owns DF-4 / DF-9 counters + dump/matrix wiring. Playcheck (debug, ~60s, frame 600) showed:

| Row | Result | Notes |
|-----|--------|--------|
| `dam.harness.seeded` | PASS | DF-1 |
| `dam.extent` | PASS | DF-1 |
| `dam.stage_s0` | PASS | DF-1/2 |
| `dam.water_solver_wired` | PASS | DF-2 |
| `dam.s0_gauge_balance` | FAIL | 0.88 ≫ 0.20 interim — short run / latch gap |
| `dam.s2_downstream_ratio` | PASS | DF-2 |
| `dam.s8_inundated_tiles` | PASS | DF-9 live depth count (8840) |
| `dam.water_gpu_field_wired` | PASS | DF-3/10 |
| `dam.water_upload_ms` | path fixed → `dam_flood.water_upload_ms` | ~0.5 ms on prior dump |
| S3/S4/S5/S7/S9 | FAIL | unowned / honest stubs |

## Next for BASELINE v1

Longer playcheck / timeline latch for S0 balance; land remaining DF slices before claiming green lock.
