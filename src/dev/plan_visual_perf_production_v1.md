# Visual / simulation perf — production plan (not env hacks)

**Status:** Superseded for execution by [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) · **Owner:** render + gui + sim  
**Context:** `--test visual --stay-open` and `BaseState::Simulation` must hold **≤33 ms** steady-state on a reference machine without operator env tuning.

**Problem statement:** Recent stall logs showed multi-hundred-ms frames driven by (1) mis-ordered Update work counted as “map camera”, (2) full-world fire ECS extract every frame, (3) CPU tile fallback raster (main + duplicate minimap pass), and (4) viewport scissor mode flips. Temporary env vars (`RASTER_MINIMAP`, `RASTER_CHUNKS_PER_FRAME`) are **debug overrides only** — they are not ship policy.

---

## Principles

| Principle | Meaning |
|-----------|---------|
| **Budget resources, not env vars** | Ship path reads `VisualBudgetSettings` → `VisualCadence`, `FrameBudgetDiagnostics`, and dedicated `TileRasterBudget` / fire cadence resources. Env overrides exist only under `cfg(debug_assertions)` or explicit `DEV_*` flags documented in runbook. |
| **One minimap authority** | When GPU minimap compositor is active and presentation binds `GpuRenderTarget`, **do not** CPU-repaint `TileWorldFallbackState::minimap_image`. |
| **Incremental sim extract** | Fire snapshot extract is **residency- and cadence-scoped**, not O(all chunks) every frame unless sim tick advanced. |
| **Schedule truth** | Map input → smooth → view sync → fire build → world repr → streaming — enforced in `configure_sets`, not inferred from stall labels. |
| **Degrade under budget** | `UxFrameSpikeGuard` may skip *optional* lanes (preview, diagnostics, second-pass raster), never silently drop authoritative spine contracts. |

---

## What already landed (2026-05-28)

- `FireVisualFrameSet::BuildProfiles` **after** `MapCameraSystemSet::Smooth` (pan/zoom no longer blocked by fire extract).
- `extract_fire_simulation_snapshot` skips full scan while `UxFrameSpikeGuard::spike_active` (recovery band-aid — replace with cadence policy in Phase 2).
- Map-hole scissor **symmetric hysteresis** (`CAM_HOLE_INVALID_STREAK`) to reduce `RENDER_HOLE_FLIP` / scaling blink.
- Stall `upd_span` split (`pre_map`, `map_cam`, `view_fire`, …) — use for attribution, not env throttles.

---

## Phase 1 — Authority & duplicate work removal (P0, ~2–3 days)

**Goal:** Remove redundant CPU minimap repaint and wire presentation source in code.

| Task | Detail | Exit |
|------|--------|------|
| **P1-A** | In `tile_world_fallback_rasterize`, skip minimap sub-pass when `MinimapPresentationSource` + `MinimapRenderTargetRegistry` resolve to `MapTextureSource::GpuRenderTarget` (same logic as `resolve_minimap_texture_source`). | **DONE 2026-05-28** — `TileFallbackRasterPolicy` + `sync_tile_fallback_raster_policy` in `tile_world_fallback.rs`. |
| **P1-B** | Default `MinimapShellState.presentation_source` to GPU path in Simulation when compositor proof green (already default env: `MINIMAP_GPU_COMPOSITOR` true). Document fallback to CPU raster only when compositor disabled or RT not committed. | `debug_runs/minimap_compositor_live.json` green + visual test shows minimap without `minimap_image` dirty work. |
| **P1-C** | Reclassify `RASTER_MINIMAP` / `RASTER_CHUNKS_PER_FRAME` in runbook as **debug-only**; no mention in “clean run” recipe. | Runbook updated. |

**Not in scope:** Deleting `minimap_image` asset yet — keep CPU fallback for editor / compositor-off builds.

---

## Phase 2 — Budget resources (P0, ~3–5 days)

**Goal:** Production frame pacing without per-operator env.

| Resource | Source | Consumers |
|----------|--------|-----------|
| `VisualCadence` (exists) | `VisualBudgetSettings` | Minimap GPU pass, preview, overlays |
| **`TileRasterBudget`** (new) | Derived from cadence + world size + `FrameBudgetDiagnostics` p95 | `tile_world_fallback_rasterize` chunk cap, spike clamp |
| **`FireExtractCadence`** (new) | `overlay_hz` / sim tick | `extract_fire_simulation_snapshot`, overlay sync |

| Task | Detail | Exit |
|------|--------|------|
| **P2-A** | Add `TileRasterBudget { chunks_per_frame, minimap_cpu_allowed }` initialized from world dimensions and profile (FULL_APP vs editor). Replace `std::env::var("RASTER_*")` with `Option<Res<TileRasterBudget>>`; env override only in debug builds. | Release visual test: no env set; steady raster ≤ budget line in `PERF`. |
| **P2-B** | Tie main-map raster cadence to zoom band + dirty revision (already partial); **never** `mark_all_dirty` on zoom band change during spike (done). | Zoom cycle does not cause 100+ ms raster spikes. |
| **P2-C** | Fire extract: run full ECS pass on sim tick advance OR overlay dirty revision; otherwise reuse snapshot (remove spike-only skip as sole policy). | `view_fire` stall ≤ 5 ms typical; spikes only on world load / teleport. |
| **P2-D** | Scope extract query to `ChunkResidencyTable` + active fire chunks (existing `ActiveFireChunkSet` path) — no full-world iterator when residency known. | Extract cost ∝ visible/resident chunks, not map size. |

**Acceptance (Phase 2):** 60 s `--test visual --stay-open` on reference HW: p95 frame &lt; 33 ms, p99 &lt; 50 ms, zero env overrides; `upd_span view_fire` &lt; 8 ms p95; `raster_b` &lt; 12 ms p95 when GPU minimap on.

---

## Phase 3 — Viewport stability (P1, ~2 days)

**Goal:** Fix “screen all weird” / scaling without hiding layout bugs.

| Task | Detail | Exit |
|------|--------|------|
| **P3-A** | Audit `SimulationMapViewport` writers — single commit per frame after HUD layout settle (`PendingHudLayoutCommit`). | No `SIM_MAP_VIEWPORT_VALIDITY_CHANGED` flapping during steady sim. |
| **P3-B** | Ortho fit uses same `view_px` as scissor decision (`sync_main_world_camera_viewport_and_projection`) — witness in `MainWorldCameraOrthoTrace` when `VISUAL_DIAG=1` only. | No full-window ortho + hole scissor mismatch. |
| **P3-C** | Map-blink repro doc: [`map_blink_001_repro_v1.md`](map_blink_001_repro_v1.md) closed with witness JSON. | Signoff row in planner matrix. |

---

## Phase 4 — Observability (P1, ~1 day)

**Goal:** Operators diagnose without `STALL_SPAN_DEBUG` every run.

| Deliverable | Detail |
|-------------|--------|
| `FrameBudgetDiagnostics` anomaly | Already logs `FrameSpike` — link to dominant bucket (fire / raster / egui). |
| `debug_runs/perf_attribution_60s.md` playbook | One command: run visual 60s + summarize p95 per `upd_span` field from perf log. |
| Witness | Extend `stage5_full_app_live.json` with `perf.p95_upd_span` when `PERF=1` capture mode set (optional CLI flag, not default). |

---

## Phase 5 — Longer-term (P2, tracked separately)

| Item | Notes |
|------|--------|
| **Tilemap adapter / GPU terrain** | **→ ACTIVE:** [`plan_gpu_terrain_production_exec_001_v1.md`](plan_gpu_terrain_production_exec_001_v1.md) (PERF-GPU-TERRAIN-001..004). `bevy_tilemap_adapter` + retire CPU fallback as Simulation default. |
| **Hanabi / GPU particles** | Keep extract on projection graph; don’t duplicate CPU overlay rebuild. |
| **WSS substrate fire** | Slab path already partially wired; ensure extract does not dual-scan ECS + slab. |

---

## What operators should run today (no hacks)

```powershell
# Clean shell — unset debug noise (see visual_test_runbook_v1.md)
.\tools\orchestrator\scripts\run_visual_test_clean.ps1

# Ship-like perf (release strongly recommended for FPS truth)
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
```

Use `PERF=1` + `STALL=1` only while actively profiling. Do **not** set `RASTER_*` unless bisecting tile raster in a debug session.

---

## Tracking

| Phase | Board row | Proof |
|-------|-----------|-------|
| 1 | `PERF-VIS-001` | minimap compositor live + raster budget witness |
| 2 | `PERF-VIS-002` | 60s perf attribution + p95 gates |
| 3 | `PERF-VIS-003` | viewport validity witness, no RENDER_HOLE_FLIP in steady sim |

Link: [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md) · [`visual_run_blockers.md`](visual_run_blockers.md) · [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md)
