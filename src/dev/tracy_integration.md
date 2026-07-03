# Tracy integration (optional — PLAN-GPU-TERRAIN-P3)

Tracy is **not** required for ship gates. Use it for deep GPU/CPU timeline dives when `RenderScheduleWitness` is insufficient.

## Build

```powershell
cargo run -p proc_A_dine01 --release --features tracy -- --test demo --stay-open
```

Enables Bevy `trace_tracy` via the `tracy` crate feature.

## Capture

1. Run the Tracy profiler GUI on the same machine.
2. Connect to the running `proc_A_dine01` process.
3. Correlate spans with `debug_runs/sim_spectrum_analytics_live.json` frame index.

## When to use

- `render_schedule.render_and_present_ms` p95 regression with `spine.tile_raster_ms == 0`
- Suspected GPU bubble with `attribution_honesty.stall_checkpoint_mismatch == true`

CI witnesses remain authoritative for sign-off; Tracy is operator triage only.
