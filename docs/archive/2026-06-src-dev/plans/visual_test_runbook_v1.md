# Visual test runbook (`--test visual --stay-open`)

Canonical operator command:

```powershell
cargo run -p proc_A_dine01 -- --test visual --stay-open
```

Release profile (closer to ship perf):

```powershell
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

Helper script (clears noisy env, enables stall probes):

[`tools/orchestrator/scripts/run_visual_test_clean.ps1`](../../tools/orchestrator/scripts/run_visual_test_clean.ps1)

---

## Environment variables — reset for a clean perf run

Unset these in the shell (or use the script). They persist in the terminal session until removed.

| Variable | Effect if left set |
|----------|-------------------|
| `UI_LAYOUT_DEBUG` | Full UI tree dump (~every 60 frames) — very expensive |
| `STAGE5_VERBOSE` | Enables `PERF` verbose path + visual/viewport debug toggles |
| `STAGE5_PER_FRAME_HOOKS` | Per-frame `STAGE5_*` INFO hooks |
| `STAGE5_READINESS_VERBOSE` | Full readiness eval trace every tick |
| `VISUAL_DIAG` | Consolidated visual diagnostics |
| `STREAM_DIAG` | Streaming spine per-frame spam |
| `SIM_VIEW_SYNC_DEBUG` | Map/window hole tracing |
| `VIEWPORT_DEBUG_OVERLAY` | Extra viewport overlay |
| `WORLDGEN_CHROME_DEBUG` | World-gen chrome tracing |
| `STAGE5_FENCE_VERBOSE` | Fence commit verbose |
| `VIEW_RUNTIME_AUDIT` | View runtime audit (use `trace` only when needed) |
| `MINIMAP_GPU_DEBUG` | Minimap compositor debug |
| `TACTICAL_VFX_PROOF` | Locks tactical zoom for VFX proof (not interactive pan/zoom) |
| `PERF_NO_VSYNC` | Disables vsync (changes frame pacing; use only to isolate GPU wait) |

**Debug-only overrides (not ship policy):** see [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md). Do **not** set these for normal visual runs:

| Variable | Debug use only |
|----------|----------------|
| `DEV_RASTER_MINIMAP=0` / `RASTER_MINIMAP=0` | Bisect duplicate CPU minimap pass vs GPU compositor (**debug builds only**) |
| `DEV_RASTER_CHUNKS_PER_FRAME` / `RASTER_CHUNKS_PER_FRAME` | Emergency cap while profiling dirty-chunk storm (**debug builds only**; release uses `TileRasterBudget`) |

**Logging:** narrow `RUST_LOG` — avoid `stage5_live_todos=info`, `ui_layout_tree=info`, or global `trace` unless debugging that subsystem.

**CLI flags (same session):** `--test frame` sets `UI_LAYOUT_DEBUG=1` in `main.rs` — do not combine with visual perf runs.

---

## Environment variables — enable only while debugging stalls

| Variable | Use |
|----------|-----|
| `PERF=1` | Frame perf line + attributed buckets (recommended) |
| `STALL=1` | `STALL {label}: Xms` when segment ≥ 5ms |
| `STALL_SPAN_DEBUG=1` | Extra pre-repr checkpoints + 1ms stall lines |
| `RUST_LOG=warn,stall=info,perf=info` | Stall + perf only |

Example:

```powershell
$env:PERF = "1"
$env:STALL = "1"
$env:STALL_SPAN_DEBUG = "1"
$env:RUST_LOG = "warn,stall=info,perf=info"
cargo run -p proc_A_dine01 -- --test visual --stay-open
```

---

## Reading `upd_span` in perf logs

Requires `STALL_SPAN_DEBUG=1` (extra checkpoints) plus `PERF=1`. Legacy `pre_repr=` alone was **misleading**: it measured PreUpdate→map when no mid-Update checkpoints existed.

| `upd_span` field | Segment |
|------------------|---------|
| `preupd` | First → PreUpdate end |
| `pre_map` | Update start → before map camera input |
| `map_cam` | Map camera input → smooth |
| `to_map` | `pre_map` + `map_cam` (legacy total) |
| `map_view` | Map smooth → view authority sync |
| `view_fire` | View sync → fire build profiles |
| `fire_repr` | Fire build → world repr (LOD registry refresh) |
| `repr` | World repr compute |
| `fire_proj` | Fire ProjectGpu |
| `stream_late` | Late Update streaming spine |

If `to_map` is huge but `map_view` / `view_fire` are tiny, cost is in **Update systems before map camera** (UI layout debug, sim, stage5 hooks). If `fire_repr` is huge, inspect LOD registry refresh before `compute_world_representation_frame`.

---

## Related docs

- [`visual_run_blockers.md`](visual_run_blockers.md)
- [`debug_runs/README.md`](../../debug_runs/README.md)
- Operational perf: [`prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md)
