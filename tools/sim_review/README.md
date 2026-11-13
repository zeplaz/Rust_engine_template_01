# Sim review

One dump. One matrix. One dashboard.

```powershell
# Debug only. Release / production never writes a dump.
$env:RUST_ENGINE_SIM_DUMP="1"
cargo run -p proc_A_dine01

python tools/sim_review/score_dump.py
# writes debug_runs/sim_review/matrix.json + dashboard.html
```

`sim_run_dump.json` is **counts only** — no `green`. The Python matrix is the verdict.

`*_live.json` writers are **off** (every profile). Do not treat fixture JSON as a run.
