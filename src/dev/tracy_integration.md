# Tracy integration (optional — GPU-P3-A)

Tracy is **not** required for ship gates or CI sign-off. Use it when `sim_spectrum` + `render_schedule` witnesses show a regression but you need a CPU/GPU timeline to find the owning system.

**Authoritative perf baseline:** [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md) § Perf truth sign-off (release, **no** env overrides).

---

## Feature gate

Declared in root [`Cargo.toml`](../../Cargo.toml):

```toml
tracy = ["bevy/trace_tracy"]
```

Default builds **do not** link Tracy. Opt in at compile time only.

---

## Build and run

```powershell
# From repo root — release demo matches perf-truth lane
cargo run -p proc_A_dine01 --release --features tracy -- --test demo --stay-open
```

Other harness scenes work the same way (`--test visual`, `--test vfx`, …). Keep **`STALL=1` / `PERF=1` unset** unless you are explicitly bisecting terminal stalls; disk witnesses still populate under `--test`.

Verify the feature is active:

```powershell
cargo test -p proc_A_dine01 --lib gpu_p3a -q
```

Lib builds without `--features tracy` still pass — the witness only checks that docs + Cargo feature wiring exist.

---

## Tracy profiler GUI

1. Install [Tracy profiler](https://github.com/wolfpld/tracy/releases) for your OS (Windows x64 build is typical for this repo).
2. Start **Tracy** before or after launching the game — connect to the running `proc_A_dine01` process when prompted.
3. On Windows, allow local firewall access for the Tracy client/server handshake if connection fails.

Bevy `trace_tracy` publishes `tracing` spans for schedules, systems, and render phases. Look for long blocks in:

- `ExtractSchedule` / `Render` (pairs with `render_schedule.*` in sim-spectrum)
- `Update` systems (pairs with `update_attrib.*` and `perf_scopes` in the witness JSON)

---

## Correlate with disk witnesses

While Tracy captures a timeline, `--test demo` writes structured frames to:

| Artifact | Use with Tracy |
|----------|----------------|
| `debug_runs/sim_spectrum_analytics_live.json` | `last_frame.frame_index`, `render_schedule.render_and_present_ms`, `bottleneck_triage` |
| `debug_runs/perf_frames/frames_*.jsonl` | Per-frame JSONL when `SIM_ANALYTICS_FRAMES=1` (auto on `--test visual` / vfx lanes) |
| `debug_runs/minimap_compositor_live.json` | Minimap GPU path vs CPU fallback |

Workflow:

1. Note the frame index / wall time from a spike in Tracy.
2. Open `sim_spectrum_analytics_live.json` → `last_frame` or matching JSONL row.
3. Trust **`render_thread_draw_and_present`** and **`render_schedule.*`** over raw `STALL substage_*` labels when `bottleneck_triage.attribution_honesty.stall_checkpoint_mismatch` is true.

---

## When to use Tracy vs env probes

| Situation | Tool |
|-----------|------|
| Ship / P0 gate sign-off | Runbook perf truth only — **no** Tracy, **no** `STALL=1` |
| `program_exit_gate.green == false` on demo | Re-read witnesses first; then optional Tracy |
| `render_and_present_ms` p95 high, `tile_raster_ms == 0` | Tracy + `render_schedule` fields |
| Misleading stall substages | Witness triage (`P3-B` contract) — Tracy optional confirmation |

---

## CI / release policy

- CI witnesses (`cargo test --lib`, `stage5`, `gpu_*` lib proofs) remain authoritative.
- Do **not** add `--features tracy` to default CI jobs — compile time and linking cost with no gate value.
- Operator-only deep dives; document findings in `debug_runs/perf_attribution_60s.md` if they change baselines.
