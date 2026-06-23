# ENGINE-DEEP-DEBUG-001 — intrusive debug build runbook

| Field | Value |
|:---|:---|
| **Program ID** | ENGINE-DEEP-DEBUG-001 |
| **Goal** | Recover minimap/GPU quality regressions with surgical witnesses |
| **Status** | SHIPPED |

## Build (special profile + feature)

```powershell
tools/orchestrator/scripts/build_deep_debug.ps1
```

Equivalent:

```powershell
cargo build --profile dev-deep-debug --features engine_deep_debug
```

| Flag | Effect |
|:---|:---|
| `--profile dev-deep-debug` | opt-level 0, full debug symbols, overflow checks |
| `--features engine_deep_debug` | compiles deep debug plugin (always-on unless env off) |

Binary: `target/dev-deep-debug/proc_A_dine01.exe`

## Run (env bundle + CLI)

```powershell
tools/orchestrator/scripts/run_deep_debug.ps1 --test visual --stay-open
```

Or manual:

```powershell
$env:RUST_ENGINE_DEEP_DEBUG = "1"
$env:RUST_ENGINE_DEEP_DEBUG_JSONL = "1"
$env:MINIMAP_GPU_DEBUG = "1"
$env:VIEW_RUNTIME_AUDIT = "1"
$env:RUST_LOG = "warn,engine_deep_debug=trace,proc_A_dine01=debug"
target/dev-deep-debug/proc_A_dine01.exe --deep-debug --test visual --stay-open
```

## Runtime-only (no rebuild)

Any dev binary:

```powershell
$env:RUST_ENGINE_DEEP_DEBUG = "1"
cargo run -- --deep-debug
```

Disable: `RUST_ENGINE_DEEP_DEBUG=0`

Minimap-only traces: `RUST_ENGINE_DEEP_DEBUG_MINIMAP_ONLY=1`

## Witness outputs

| Path | Content |
|:---|:---|
| `debug_runs/deep_debug/engine_deep_debug_live.json` | Latest frame sample (minimap shell, RT, compositor, GPU diag, asset counts) |
| `debug_runs/deep_debug/engine_deep_debug_frames.jsonl` | Per-frame history when `RUST_ENGINE_DEEP_DEBUG_JSONL=1` |
| `debug_runs/minimap_compositor_live.json` | Existing compositor proof (still written in visual test) |

## What gets traced

- **Minimap compositor** — skip/dispatch reasons, RT size, terrain handles, overlay rows (ring buffer 256)
- **Minimap egui bind** — CPU vs GPU path decisions
- **View runtime** — violations from `VIEW_RUNTIME_AUDIT=1`
- **Subsystem isolation** — `VisualCadence`, `TileRasterBudget`, `UxFrameSpikeGuard`, overlay mask, shell refresh queue, witness-lane flag
- **Visual memory queues** — per-bucket frame costs (`HudShell`, `MinimapRaster`, `OverlayComposition`, …), GPU upload bytes, minimap RT/heat memory estimates, compositor skip counters
- **Bevy diagnostics** — frame time, entity count, system info (LogDiagnosticsPlugin)
- **Asset inventory** — image/mesh/material counts + sample GPU formats

### Faster flush while reproducing

```powershell
$env:RUST_ENGINE_DEEP_DEBUG_FLUSH_EVERY = "10"   # default 30 frames
```

### Reading `subsystem_isolation` + `visual_memory_queues`

| Signal | Likely meaning |
|:---|:---|
| `spike_guard.spike_active: true` | Main thread over budget — raster chunks capped, preview may defer |
| `tile_raster.effective_chunks_per_frame: 2` | Spike degrade active (normal cap is 4–8) |
| `visual_cadence.minimap_hz: 6` | Simulation play budget (not editor default 10) |
| `witness_lane_active: true` | Test/visual harness — fake ecology/fog data may appear |
| `gpu_compositor_queue.skips_no_change` climbing | Compositor skipping redundant uploads (OK if stamp moves) |
| `bucket_queues` MinimapRaster `avg_ms` high | CPU tile raster still hot — check main map zoom/pan |
| `minimap_heat_est_bytes` | Rough GPU heat-layer footprint at current RT size |

## Minimap recovery checklist

1. Run deep debug build, enter Simulation, reproduce bad minimap
2. Open `engine_deep_debug_live.json` — check:
   - `minimap_registry.committed_size` non-zero
   - `minimap_compositor.stamp` increasing
   - `minimap_gpu_diagnostics.last_skip` not stuck on `NoTerrain` / `NoRenderTarget`
   - `minimap_fallback.main_image` true
3. Compare `minimap_trace.compositor_tail` for repeated `NoChange` vs `RateCapped`
4. Cross-check `debug_runs/minimap_compositor_live.json` composite_ok

## Opt-out

Feature build still respects `RUST_ENGINE_DEEP_DEBUG=0` to run without overhead.
