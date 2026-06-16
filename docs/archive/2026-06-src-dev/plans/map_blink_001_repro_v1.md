# MAP-BLINK-001 — world map overlay blink repro + fix `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **MAP-BLINK-001** |
| **Blocker** | [`visual_run_blockers.md`](visual_run_blockers.md) **VR-05** · operator [`stauts_26_05.md`](stauts_26_05.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@coder` + debug-intelligence |
| **Status** | **FIX LANDED** (overlay hold + projection lag hold + warmup) |

---

## Symptom

World map **blinks in/out** — fire tint and tactical readability drop for stretches, then return when ecology ignites.

Operator log pattern (frames 23–37 vs 38+):

| Signal | Before ignition | After ignition |
|:---|:---|:---|
| `overlay_rev` | `0` | `1` |
| `overlay_chunk_cells` | `0` | `28` |
| `graph_fire_inst` | `0` | `0` (particles from `chunk_heat` fallback) |
| `fire_particle_rows` | `0` | `336` |

---

## Root causes

| # | Cause | Fix |
|:---:|:---|:---|
| 1 | **PLAY-06c** only held overlay when sim snapshot empty **and** prior overlay non-empty — not when residency filtered sim heat | **PLAY-06d** hold when `sim_has_display_heat` but filtered `next` empty |
| 2 | Cold start: first overlay revision jumps 0→N cells (visible pop-in) | **Warmup blend** 4 frames @ `OVERLAY_WARMUP_BLEND_FRAMES` |
| 3 | `FireProjectionNode` cleared `instance_buffer` on **1-tick** fence lag (`fire.stamp` ahead of `CommittedVisualSnapshotFence`) | **MAP-BLINK-001** retain prior projection when `lag <= 1` |

---

## Code touchpoints

| File | Change |
|:---|:---|
| [`fire_visual_extract.rs`](../render/extraction/fire_visual_extract.rs) | `build_chunk_fire_heat_overlay_map`, PLAY-06d, warmup |
| [`render_projection_graph.rs`](../render/extraction/render_projection_graph.rs) | `fire_projection_stamp_lag` hold |
| [`stage5_closure_witnesses.rs`](../render/stage5_closure_witnesses.rs) | `held_overlay_persist_frames`, `overlay_warmup_frames` |

---

## Repro (operator)

```powershell
$env:RUST_LOG='warn,visual_diag=info,stage5_readiness::live=info'
$env:VISUAL_DIAG='1'
cargo run -p proc_A_dine01 --release
```

Watch for `VISUAL_DIAG render_spine` — `overlay_rev` should not oscillate 0↔N when fire is steady; `held_overlay_persist_frames` increments only on intentional hold.

---

## Witness

`debug_runs/stage5_full_app_live.json` → `fire_playback`:

- `held_empty_snapshot_frames`
- `held_overlay_persist_frames`
- `overlay_warmup_frames`

---

## Verify

```powershell
cargo test -p proc_A_dine01 --lib render::extraction::render_projection_graph
cargo test -p proc_A_dine01 --lib fire_visual_extract
```
