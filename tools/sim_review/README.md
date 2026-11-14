# Sim review

One dump. One matrix. One dashboard.

```powershell
# Debug (or release built with the non-default `sim_dump` feature). Shipped release never writes a dump.
$env:RUST_ENGINE_SIM_DUMP="1"
cargo run -p proc_A_dine01

python tools/sim_review/score_dump.py
# writes debug_runs/sim_review/matrix.json + dashboard.html

python tools/sim_review/playcheck.py            # debug: build + run + capture + score
python tools/sim_review/playcheck.py --release  # release built with `--features sim_dump`
python tools/sim_review/playcheck.py --seeds 3  # T6: build once, N seeds → dump_<seed>.json (prefer N=3)
python tools/sim_review/score_dump.py --multi tools/sim_review/fixtures
# → matrix_multi.json (mean/stdev/min/max/n/stderr) + dashboard_multi.html
python tools/sim_review/tune.py --dry-run       # T7: FrontBalance whitelist + planned perturbations
```

Multi-seed verdict: a change is real only if `|mean_after − mean_before| > 2 × max(stderr_before, stderr_after)`.

Frame time is always tracked. `perf` in the dump: avg / p50 / p95 / p99 / max (after warm-up),
`hitches` (frames over 50 ms release / 500 ms debug, `HITCH_MS` overrides), `stalls` (> 1 s),
the worst hitch frames with main-schedule vs outside-main time and their top `PerfScope`s, and a
downsampled series. Matrix perf rows use `when_build` (debug vs release thresholds). The dashboard
draws the frame-time sparkline and the hitch table.

Hitches also ship in release logs: one rate-limited `WARN hitch: HITCH frame=… dt=… tick=… top=[…]`
line (stalls always log). Grep `hitch` in any run log. `PERF=1` adds the per-frame breakdown.

`sim_run_dump.json` is **counts only** — no `green`. The Python matrix is the verdict.

`*_live.json` writers are **off** (every profile). Do not treat fixture JSON as a run.
