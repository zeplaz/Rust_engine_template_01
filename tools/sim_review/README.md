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

### PlayQuad + voxel view + weather + massing → score (V2 D3)

One loop that proves **quadrant harness weather**, **PixelWorld3D (voxel view)**, **Studio massing
override**, GPU capture, and the **counts-only** dump → matrix. Does not replace D0 matrix rows;
`score_dump.py` reads whatever `presentation.*`, `weather.*`, and `atmosphere.*` keys the dump
already carries.

**Harness:** `--test playquad` → `TestScene::PlayQuad` (512² Standard, fixed seed
[`PLAYQUAD_WORLD_SEED`](../../src/engine/launch_args.rs), NW fire / NE build pilots / SE
StormFront after ~frame 1800). Module map: [`src/engine/test_harness.rs`](../../src/engine/test_harness.rs).

**Preferred (playcheck wraps build, env, capture, score):**

```powershell
$env:VFX_CAPTURE_VIEW = "voxel"              # PixelWorld3D for capture + presentation dump
$env:RUST_ENGINE_STUDIO_MASSING = "fixture"  # one-shot pilot JSON → massing_columns (presentation only)
python tools/sim_review/playcheck.py --scene playquad --frame 2400 --ticks 4000 --min-secs 420
```

playcheck always sets `RUST_ENGINE_SIM_DUMP=1`, `VFX_CAPTURE=1`, `VFX_CAPTURE_FRAME` (see
`--frame`), and `WORLD_GEN_SEED` (PlayQuad still pins world seed 8 in the harness). Outputs:

- `debug_runs/sim_run_dump.json` — inspect `launch.test_scene` (`PlayQuad`), `presentation.*`
  (`massing_columns`, `voxel_pixel_world_active`, …), `weather.*`, `atmosphere.*` (if wired).
- `debug_runs/captures/window_live.png` + `tactical_rtt_live.png` — voxel-view stills.
- `debug_runs/sim_review/matrix.json` + `dashboard.html` — verdict (no `*_green` in the binary).
- `debug_runs/sim_review/playcheck_summary.md` — panics, perf, score tail.

**Manual (same env, no playcheck):**

```powershell
$env:RUST_ENGINE_SIM_DUMP = "1"
$env:VFX_CAPTURE = "1"
$env:VFX_CAPTURE_FRAME = "2400"
$env:VFX_CAPTURE_VIEW = "voxel"
$env:RUST_ENGINE_STUDIO_MASSING = "fixture"
cargo run -p proc_A_dine01 -- --test playquad --stay-open
# stop after dump tick rises; then:
python tools/sim_review/score_dump.py
```

**Dry-run (no game — documents scoring only):**

```powershell
python tools/sim_review/playcheck.py --help
python tools/sim_review/score_dump.py tools/sim_review/fixtures/honest_dump.json
```

Multi-seed regression (T6) on PlayQuad: repeat with `--seeds 3` once a baseline exists; same
`|mean_after − mean_before| > 2 × max(stderr_before, stderr_after)` rule as the default visual scene.

Ref-books (dual cite): **DBG-04** (GPU capture + `VFX_CAPTURE`) · **DBG-02** (dump discipline) ·
**LGT-04** (voxel weather presentation) · LEGACY `debug_tools` / `geometry_visual_effects` when
orienting capture regressions.

See also [`src/dev/plan_playtest_quadrant_world_v1.md`](../../src/dev/plan_playtest_quadrant_world_v1.md)
(P5 voxel + precip) and [`src/dev/plan_voxel_massing_presentation_v1.md`](../../src/dev/plan_voxel_massing_presentation_v1.md).

### Studio massing (presentation demo)

Ghost silhouette override is **not** a matrix row — counts live in construction visual requests.
One-shot env (no per-frame poll):

```powershell
$env:RUST_ENGINE_STUDIO_MASSING="fixture"   # checked-in pilot JSON
# or "1" after APS export to debug_runs/studio_dash/massing_preview.json
cargo run -p proc_A_dine01
```

See [`tools/studio_dash/demo_massing_override.md`](../studio_dash/demo_massing_override.md).
