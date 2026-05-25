# Perf attribution note (PERF-01)

**Captured:** 2026-05-22 (post-PLAY follow-up)

## How to capture a live 60s profile

```powershell
$env:RUST_LOG="warn,perf=info,perf_scope=info,stall=info"
$env:STALL="1"
cargo run -p proc_A_dine01 -- --test visual
```

After ~60s in simulation, inspect console for:

- `perf_scope` — scoped buckets ≥ 250 ms (`src/render/frame_perf.rs` `PerfScope`)
- `STALL culprit=*` — top stall labels (`intra_update_stall_log`)
- `emit_frame_perf_summary` — frame attrib: `streaming_reconstruct_ms`, `world_gen_ui_ms`, `hud_egui_ms`

## Expected top buckets (from prior sessions)

| Rank | Label | Mitigation status |
|------|-------|-------------------|
| 1 | `streaming_apply` / `upd_streaming_reconstruct` | Early return when `staged_chunk_bodies` empty (`reconstruct_staged_chunks_into_cache`) |
| 2 | `egui_world_gen_ui` / world preview raster | Gated: `world_gen_ui_chrome_visible`, `world_preview_pipeline_enabled`, sim enter dismiss |
| 3 | Shell / HUD egui | `ProductShellUpdateBudget` + lightweight chrome while dragging (`draw_build_toolbox_egui`) |

## PLAY-02 session gates (verified in code)

- WorldGen UI: `run_if(world_gen_ui_chrome_visible)`
- Preview texture resize: skipped when preview + world-gen chrome hidden
- Fire overlay harness: throttle re-seed + `FirePlaybackStabilityWitness`
- Sim enter: `apply_simulation_hud_defaults` dismisses editor chrome

## Sample log shape (PERF-N03)

From a visual boot frame (representative):

```text
PERF wall=34.71 ... | upd_attrib sum=10.28 stream=8.57 wgen=1.34 ...
STALL culprit=upd_streaming_reconstruct (when STALL=1 and staged work pending)
```

World-gen egui draw logs (`WORLD_GEN_EGUI_DRAW`) only emit when `WORLDGEN_CHROME_DEBUG=1` — not in normal sim.

## Stage 6 frame budget (S6-26)

When `Stage6VirtualizationFrame` residency cell count jumps by ≥128 between frames, `FrameBudgetDiagnostics` emits:

```text
frame budget anomaly ResidencyChurn: residency cells changed by 256 (1113 → 1369)
```

Atlas pressure in the same HUD path uses `gpu_upload_bytes_frame` from the published S6 frame (not the legacy `active_atlas_slots / 3` heuristic).

## Target

p95 wall frame &lt; 33 ms on reference machine, or document hardware-bound baseline here after a timed run.

## Next actions

See [`src/dev/next_action_todos.md`](../src/dev/next_action_todos.md) — **PERF-N01** (60s capture), **PERF-N02** (verified gated).
