# Testing

How to run unit tests, scene harnesses, DamFlood playchecks, and the sim-review score path.

**Review model:** the game dumps **counts only** (`sim_run_dump.json`). Verdicts live in the Python matrix (`matrix.json` + `dashboard.html`). No `*_green` predicates in production code.

Package: `proc_A_dine01`. PowerShell examples below; same commands work in bash with env-var syntax adjusted.

---

## Quick map

| Goal | Command |
|:---|:---|
| Lib unit tests | `cargo test -p proc_A_dine01 --lib` |
| DamFlood unit tests | `cargo test -p proc_A_dine01 --lib dam_flood` |
| Interactive DamFlood | `cargo run -p proc_A_dine01 -- --test dam --stay-open` |
| One-shot DamFlood score | `python tools/sim_review/playcheck.py --scene dam` |
| Score an existing dump | `python tools/sim_review/score_dump.py` |
| Compile check (agents) | `python -m rust_engine_mcp.cli validate-report cargo --compress 3` (from `tools/mcp/python`) |

Deeper tooling detail: [`tools/sim_review/README.md`](../tools/sim_review/README.md) · validators: [`tools/validators/README.md`](../tools/validators/README.md).

---

## 1. Unit / lib tests

```powershell
# All lib tests
cargo test -p proc_A_dine01 --lib

# Filter by module / name substring
cargo test -p proc_A_dine01 --lib construction::
cargo test -p proc_A_dine01 --lib dam_flood
cargo test -p proc_A_dine01 --lib playquad
cargo test -p proc_A_dine01 --lib launch_args
```

| Filter | What it covers |
|:---|:---|
| `dam_flood` | Fixture load, valley author helpers, dam placement span |
| `playquad` | Quadrant partition / extent tests |
| `launch_args` | `--test` CLI aliases resolve to the right `TestScene` |
| `construction::` | Construction funnel / guards (example domain filter) |

Ignored tests stay ignored unless you pass `-- --ignored`.

---

## 2. Scene harness (`--test`)

```powershell
cargo run -p proc_A_dine01 -- --test <scene> [--stay-open]
```

`--stay-open` keeps the window after auto-exit scenes (e.g. `visual`). DamFlood / PlayQuad / sandbox already stay open (`DemoOpen`).

### Scenes

| CLI (`--test` / playcheck `--scene`) | Aliases | What it does |
|:---|:---|:---|
| `dam` | `damflood` | **DamFlood benchmark** — authored valley, water ROI, timeline stages S0–S9. World seed pinned to **77**. Scores `dam.*` matrix rows. |
| `playquad` | `quad`, `quadrant` | 512² quadrant world: NW fire · NE build · SW lakes · SE marsh · delayed rain. Scores `playquad.*` rows. |
| `visual` | `capture`, `full` | Full-capture visual smoke (fire/weather/GPU). Auto-exits unless `--stay-open`. Default playcheck scene. |
| `demo` | `open` | Same visual stack, stay-open demo. |
| `sandbox` | `fireweather` | Fire + weather + atmosphere; stay-open. |
| `vfx` | — | Fire / fire-VFX only. |
| `subfx` | — | Weather + atmosphere only. |
| `city` | — | Generated towns; no fire/weather seed. |
| `buildread` | `build_readability`, `readability` | Catalog pilots + massing readability. |
| `weather` / `fire` / `atmosphere` | — | Isolation harnesses for those systems. |
| `frame` | `layout`, `framescreen` | Small-world layout / frame screen test. |
| `unittest` | `unit` | Fixture-driven capture path (pass fixture via CLI when needed). |

Scene enum + aliases: [`src/engine/launch_args.rs`](../src/engine/launch_args.rs).

---

## 3. DamFlood (primary playtest)

Plan: [`src/dev/plan_dam_flood_benchmark_v1.md`](../src/dev/plan_dam_flood_benchmark_v1.md) · operator notes: [`src/dev/playtest_dam_flood_notes_v1.md`](../src/dev/playtest_dam_flood_notes_v1.md) · baseline notes under [`tools/sim_review/expected/`](../tools/sim_review/expected/).

**What it is:** scripted example valley that drives the **general** water / dam / cooling / strike / flood stack. Seed **77** is fixed in the harness (topology from fixture; seed = noise only).

### Preferred: playcheck (build → run → capture → score)

```powershell
# Short interactive / DF smoke (defaults bump capture frame ≥600, view=voxel)
python tools/sim_review/playcheck.py --scene dam

# Longer DF-2-style re-score
python tools/sim_review/playcheck.py --scene dam --view voxel --frame 900 --ticks 1200 --min-secs 90

# Release frame times (binary built with `--features sim_dump`)
python tools/sim_review/playcheck.py --scene dam --release --view voxel --frame 900 --ticks 1200 --min-secs 90

# Skip rebuild if you already built
python tools/sim_review/playcheck.py --scene dam --no-build
```

| Flag | Effect |
|:---|:---|
| `--scene dam` | Engine `--test dam` (`TestScene::DamFlood`) |
| `--view voxel` | Capture PixelWorld3D (auto for dam/playquad if omitted) |
| `--frame N` | GPU still capture frame (`VFX_CAPTURE_FRAME`); dam floor is 600 |
| `--ticks N` | Wait until dump sim tick ≥ N before stop |
| `--min-secs N` | Keep process alive at least N seconds (game loops need wall time) |
| `--secs N` | Hard wall-clock kill limit |
| `--seed` / `--seeds N` | Labels / multi-seed; DamFlood still pins engine seed **77** |
| `--release` | Release build + `sim_dump` feature |
| `--no-build` / `--exe PATH` | Reuse an existing binary |

**Outputs**

| Path | Content |
|:---|:---|
| `debug_runs/sim_run_dump.json` | Counts (`dam_flood.*`, perf, …) — no verdicts |
| `debug_runs/captures/*.png` | Window / RTT stills |
| `debug_runs/sim_review/matrix.json` | Pass/fail rows |
| `debug_runs/sim_review/dashboard.html` | Human-readable matrix |
| `debug_runs/sim_review/playcheck_summary.md` | Panics / perf / score tail |

### Manual DamFlood (no playcheck)

```powershell
$env:RUST_ENGINE_SIM_DUMP = "1"
$env:VFX_CAPTURE = "1"
$env:VFX_CAPTURE_FRAME = "900"
$env:VFX_CAPTURE_VIEW = "voxel"
cargo run -p proc_A_dine01 -- --test dam --stay-open
# play until dump tick is high enough, then:
python tools/sim_review/score_dump.py
```

### Dam matrix rows (what “green” means)

Harness / wiring (expect PASS early):

- `dam.harness.seeded`, `dam.extent`, `dam.stage_s0`
- `dam.water_solver_wired`, `dam.water_gpu_field_wired`

Stage envelopes (many still FAIL until DF-10 baseline lock):

- `dam.s0_gauge_balance`, `dam.s2_downstream_ratio`
- `dam.s3_drowned_cells`, `dam.s4_hydro_mw`, `dam.s5_impact_err_m`
- `dam.s7_wave_arrival_g3`, `dam.s8_inundated_tiles`, `dam.s9_burn_rate_dead_over_live`

Status notes: [`baseline_dam_df2_note.md`](../tools/sim_review/expected/baseline_dam_df2_note.md), [`baseline_dam_df10_note.md`](../tools/sim_review/expected/baseline_dam_df10_note.md). **Do not** treat a short debug playcheck as baseline lock.

### DamFlood unit tests only

```powershell
cargo test -p proc_A_dine01 --lib dam_flood
```

---

## 4. PlayQuad

```powershell
$env:VFX_CAPTURE_VIEW = "voxel"
$env:RUST_ENGINE_STUDIO_MASSING = "fixture"
python tools/sim_review/playcheck.py --scene playquad --frame 2400 --ticks 4000 --min-secs 420
```

Capture frame defaults to ≥2000 (past rain delay ~1800). Details: [`tools/sim_review/README.md`](../tools/sim_review/README.md) · plan: [`src/dev/plan_playtest_quadrant_world_v1.md`](../src/dev/plan_playtest_quadrant_world_v1.md).

---

## 5. Sim review (dump → matrix)

```powershell
# After any dump-writing run
python tools/sim_review/score_dump.py
# → debug_runs/sim_review/matrix.json + dashboard.html

# Score a specific dump
python tools/sim_review/score_dump.py path\to\dump.json

# Multi-dump stats
python tools/sim_review/score_dump.py --multi tools/sim_review/fixtures

# Dry-run on checked-in fixture (no game)
python tools/sim_review/score_dump.py tools/sim_review/fixtures/honest_dump.json

# Balance tune helper (T7)
python tools/sim_review/tune.py --dry-run
```

### Manual dump-only launch (default / visual)

```powershell
$env:RUST_ENGINE_SIM_DUMP = "1"
cargo run -p proc_A_dine01
# or:
cargo run -p proc_A_dine01 -- --test visual --stay-open
python tools/sim_review/score_dump.py
```

Shipped release binaries do **not** write dumps unless built with `--features sim_dump` (playcheck `--release` does that).

---

## 6. playcheck flags (all scenes)

```powershell
python tools/sim_review/playcheck.py --help
```

| Flag | Default | Meaning |
|:---|:---|:---|
| `--scene` | `visual` | Maps to engine `--test` (see §2) |
| `--seed` | `77` | Base seed label; DamFlood engine seed stays 77 |
| `--seeds` | `1` | Run N seeds → `dump_<seed>.json` (prefer 3 for regressions) |
| `--frame` | `1200` | Capture frame (scene floors may raise it) |
| `--ticks` | `3000` | Minimum sim tick before stop |
| `--secs` | `420` | Hard timeout |
| `--min-secs` | `360` | Minimum wall time |
| `--view` | scene-dependent | `voxel` / `tactical` / `topdown` |
| `--release` | off | Release + `sim_dump` |
| `--no-build` | off | Skip `cargo build` |
| `--exe` | — | Run this binary; implies `--no-build` |

Always sets `RUST_ENGINE_SIM_DUMP=1` and `VFX_CAPTURE=1`.

Multi-seed rule: a change is real only if `|mean_after − mean_before| > 2 × max(stderr_before, stderr_after)`.

---

## 7. Useful env vars

| Env | Effect |
|:---|:---|
| `RUST_ENGINE_SIM_DUMP=1` | Write `debug_runs/sim_run_dump.json` (debug / `sim_dump` feature) |
| `RUST_ENGINE_SIM_PAUSE=1` | Start sim paused (stills / Studio) |
| `VFX_CAPTURE=1` | Enable GPU capture path |
| `VFX_CAPTURE_FRAME` | Frame index for stills |
| `VFX_CAPTURE_VIEW` | `voxel` / `tactical` / `topdown` |
| `RUST_ENGINE_STUDIO_MASSING` | `fixture` or `1` — presentation massing override |
| `PERF=1` | Per-frame perf breakdown logs |
| `HITCH_MS` | Override hitch threshold |
| `TERRAIN_GPU_BAKE_SPIKE=0` | Opt out of default GPU terrain bake |

---

## 8. Validate-report (agents / CI-shaped)

Prefer structured JSON over raw cargo walls ([validation-first](../.cursor/skills/validation-first/SKILL.md)):

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.cli validate-report cargo --compress 3
python -m rust_engine_mcp.cli validate-report cargo --cached
python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01
```

Also: [`tools/validators/README.md`](../tools/validators/README.md) (`sole_writer_check.py`, `determinism_lint.py`, asset/MCP validators).

---

## 9. Related docs

| Doc | Role |
|:---|:---|
| [`AGENTS.md`](../AGENTS.md) | Agent rules + short review path |
| [`tools/sim_review/README.md`](../tools/sim_review/README.md) | Dump / matrix / PlayQuad detail |
| [`src/dev/plan_dam_flood_benchmark_v1.md`](../src/dev/plan_dam_flood_benchmark_v1.md) | DamFlood stages, Accept rules |
| [`src/dev/playtest_dam_flood_notes_v1.md`](../src/dev/playtest_dam_flood_notes_v1.md) | Operator eye notes |
| [`debug_runs/README.md`](../debug_runs/README.md) | What lives under `debug_runs/` |
| [`src/dev/coder_todos_v1.md`](../src/dev/coder_todos_v1.md) | Open board (incl. DF-10) |
