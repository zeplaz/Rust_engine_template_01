# PLAN-VISUAL-PERF-PRODUCTION-EXEC-001 — Production visual perf budgets `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-VISUAL-PERF-EXEC-001** (alias **PLAN-VISUAL-PERF-PRODUCTION-EXEC-001**) |
| **Artifact** | `plan_visual_perf_production_exec_001_v1.md` |
| **Parent draft** | [`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md) |
| **Runbook** | [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@planner` → **`@coder`** (render/gui lane) |
| **Status** | **SIGNED** — P1-A **DONE**; P1-C **PARTIAL**; execute P1-B then Phase 2 |

**Board rows:** `PERF-VIS-001` (Phase 1) · `PERF-VIS-002` (Phase 2) · `PERF-VIS-003` (Phase 3)

---

## 1. Scope + non-goals

### Scope

- Replace emergency operator env throttles (`RASTER_MINIMAP`, `RASTER_CHUNKS_PER_FRAME`) with **production budget resources** derived from `VisualBudgetSettings`, `VisualCadence`, `FrameBudgetDiagnostics`, and `UxFrameSpikeGuard`.
- Hold **60 s** `cargo run -p proc_A_dine01 --release -- --test visual --stay-open` under measurable p95 targets **without** `RASTER_*` env.
- Preserve Stage 5 spine contracts: `RepresentationResult`, render projection graph, single minimap presentation authority (`resolve_minimap_texture_source`).
- Degrade optional lanes under spike via existing `UxFrameSpikeGuard` — never drop authoritative contracts silently.

### Non-goals

- Rewriting tilemap adapter / full GPU terrain (Phase 5 track only).
- Changing witness JSON schemas beyond optional `perf.p95_*` telemetry fields (Phase 4).
- Making gameplay correctness depend on perf budgets or witness I/O.
- Using `STALL_SPAN_DEBUG=1` as the default operator recipe (profiling-only).

---

## 2. Current-state attribution table

| Stall / PERF bucket | Typical symptom (inv 260+) | Root cause | Owner file(s) |
|:---|:---|:---|:---|
| `upd_streaming_reconstruct` / `stream_late` | 300–900 ms bursts after enter-sim | Streaming reconstruct + tile apply chain; interest churn | `src/io/streaming/mod.rs`, `src/engine/test_harness.rs` |
| `raster_b` / `MinimapRaster` | ~200 ms/frame, `tile_raster_ran=true` steady | CPU fallback dirty chunks + **duplicate minimap sub-pass** (P1-A fixes duplicate) | `src/render/tile_world_fallback.rs` |
| `view_fire` / fire extract | 50–200 ms | Full-world ECS fire scan every frame; spike-only skip band-aid | `src/render/extraction/fire_visual_extract.rs` |
| `to_map` / `pre_map` (legacy) | Misleading “map camera” blame | Update work **before** map camera (UI layout debug, sim, stage5 hooks) | `src/gui/hud/*`, `src/dev/stage5_live_todos.rs` |
| `RENDER_HOLE_FLIP` / viewport | Scaling blink, ortho 17×9 | Sim viewport invalid pre-layout; scissor vs ortho `view_px` mismatch | `src/gui/authoritative_viewport.rs`, `src/gui/map_camera.rs` |
| Env `RASTER_CHUNKS_PER_FRAME` | Masks dirty-chunk storms | No `TileRasterBudget` resource; hardcoded env read | `tile_world_fallback.rs` `raster_chunks_per_frame_budget()` |
| Env `RASTER_MINIMAP=0` | Hides duplicate CPU minimap | Policy in env instead of `resolve_minimap_texture_source` | Same + `src/gui/map_view/backend/mod.rs` |

### Already landed (mark DONE)

| ID | What | Evidence |
|:---|:---|:---|
| **P1-A** | Skip CPU minimap raster when GPU RT committed | `tile_fallback_cpu_minimap_raster_needed()` mirrors `resolve_minimap_texture_source`; `sync_tile_fallback_raster_policy` + `TileFallbackRasterPolicy`; unit tests in `tile_world_fallback.rs` (`chunk_grid_tests`) |
| Schedule | Fire build after map smooth | `FireVisualFrameSet::BuildProfiles` after `MapCameraSystemSet::Smooth` |
| Attribution | `upd_span` split in PERF line | `src/render/frame_perf.rs`, `src/render/stall_watch.rs` (`STALL_SPAN_DEBUG`) |
| Fire band-aid | Skip extract on spike | `extract_fire_simulation_snapshot` + `UxFrameSpikeGuard` (replace in P2-C) |

---

## 3. Target architecture

### Resources (ship path)

| Resource | Source | Consumers |
|:---|:---|:---|
| `VisualBudgetSettings` | Plugin init / profile (FULL_APP vs editor) | Authoring Hz policy |
| `VisualCadence` | `VisualCadence::from(&budgets)` each frame | Minimap GPU, preview, overlay `run_if` |
| **`TileRasterBudget`** (new) | `VisualBudgetSettings` + world size + `FrameBudgetDiagnostics` EMA | `tile_world_fallback_rasterize`, dirty marking |
| **`FireExtractCadence`** (new) | `overlay_hz`, sim tick, residency | `extract_fire_simulation_snapshot`, overlay sync |
| `UxFrameSpikeGuard` | Frame wall time vs 33 ms | Optional lane suppress (preview, diagnostics, raster cap) |
| `TileFallbackRasterPolicy` (exists) | `sync_tile_fallback_raster_policy` | `cpu_minimap_pass` gate in rasterize |

Proposed definitions (new file: `src/gui/visual_perf_budget.rs` or `src/render/visual_perf_budget.rs`):

```rust
// TileRasterBudget — ship policy
pub struct TileRasterBudget {
    pub chunks_per_frame: usize,
    pub minimap_cpu_allowed: bool,
    pub fire_overlay_mark_interval_frames: u32,
    pub zoom_band_quantum: f32,
}

// FireExtractCadence — ship policy
pub struct FireExtractCadence {
    pub min_interval_secs: f32,
    pub full_scan_on_sim_tick: bool,
    pub residency_scoped: bool,
}
```

### Schedule order (authoritative)

```text
PreUpdate → …
Update:
  MapCameraSystemSet::ApplyInput
  → DeriveDesired
  → Smooth
  → ViewAuthoritySystemSet::SyncViewManager
  → FireVisualFrameSet::BuildProfiles … ProjectGpu
  → WorldRepresentationSystemSet::ComputeFrame
  → Streaming spine (late)
  → TileWorldFallbackAfterFireExtract (raster)
PostUpdate → readiness → egui
```

Enforce via `configure_sets` in `engine_with_worldgen.rs` / lane plugins — do not rely on stall labels alone.

### Authority rules

| Rule | Enforcement |
|:---|:---|
| **One minimap texture source** | `resolve_minimap_texture_source` is sole authority; `TileRasterBudget.minimap_cpu_allowed` must match |
| **No env in release** | `RASTER_*` reads only under `cfg(debug_assertions)` behind `DEV_RASTER_*` |
| **Spike degrade** | `UxFrameSpikeGuard` may lower `chunks_per_frame` (e.g. min 2) and skip preview — not projection graph merge · **Designer:** [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) (**DESIGN-VISUAL-PERF-DEGRADE-001** PASS) |
| **Viewport single commit** | `SimulationMapViewport` from `measure_sim_map_fill_viewport` only; camera scissor uses same `view_px` |

---

## 4. Phased migration (P1–P4)

### Phase 1 — Authority & duplicate work (P0)

| Task | Status | Exit criteria |
|:---|:---:|:---|
| **P1-A** Skip duplicate CPU minimap pass | **DONE** | GPU compositor on → no `minimap_image` sub-pass in rasterize |
| **P1-B** Default minimap presentation = GPU in Simulation when compositor green | OPEN | `MinimapShellState.presentation_source` + witness `presentation_source` = `SharedRenderTargetImage` |
| **P1-C** Runbook + script: no `RASTER_*` in clean recipe; CI warning | **PARTIAL** | Runbook debug-only table + `run_visual_test_clean.ps1` clears `RASTER_*`; `check_visual_runbook_no_raster_env.ps1` passes — wire into CI (P4-C) |

### Phase 2 — Budget resources (P0)

| Task | Exit criteria |
|:---|:---|
| **P2-A** `TileRasterBudget` resource; remove release `std::env::var("RASTER_*")` | Release visual test with zero `RASTER_*`; chunks/frame from budget |
| **P2-B** Zoom band dirty policy + spike feedback from `FrameBudgetDiagnostics` | No `mark_all_dirty` on zoom-only during spike; p95 raster ≤ 12 ms (GPU minimap on) |
| **P2-C** `FireExtractCadence`; extract on sim tick / overlay revision | p95 `view_fire` &lt; 8 ms steady (**OPS-F01 / live perf attribution** — not lib-only) |
| **P2-D** Residency-scoped extract (`ActiveFireChunkSet` + `ChunkResidencyTable`) | Extract cost ∝ resident chunks; **p95 acceptance = operator 60s run** |

### Phase 3 — Viewport stability (P1)

| Task | Exit criteria |
|:---|:---|
| **P3-A** Single `SimulationMapViewport` commit after `PendingHudLayoutCommit` | No `SIM_MAP_VIEWPORT_VALIDITY_CHANGED` flap in steady sim (60 s) |
| **P3-B** Ortho `view_px` aligned with scissor (`MainWorldCameraOrthoTrace`) | No full-window ortho + hole scissor mismatch |
| **P3-C** MAP-BLINK-001 closure row | [`map_blink_001_repro_v1.md`](map_blink_001_repro_v1.md) signed + witness |

### Phase 4 — Observability (P1)

| Task | Exit criteria |
|:---|:---|
| **P4-A** `debug_runs/perf_attribution_60s.md` protocol | One script/doc: 60 s run → p95 summary |
| **P4-B** Optional `perf.p95_upd_span` in `stage5_full_app_live.json` when `PERF_CAPTURE=1` | Witness keys documented; default off |
| **P4-C** CI: runbook must not require `RASTER_*` | Grep/lint fails on regression |

---

## 5. PR-sized execution slices

Parallelizable after **Slice 2** (P2-A/B can split by file).

| Slice | Phase | Files touched | Tests | Expected witness / perf |
|:---:|:---|:---|:---|:---|
| **1** | P1-B | `src/gui/minimap_shell.rs`, `src/render/minimap_compositor/mod.rs`, `src/engine/engine_with_worldgen.rs` | `cargo test -p proc_A_dine01 --lib minimap_compositor` | `minimap_compositor_live.json`: `composite_ok`, `presentation_source`, `gpu_budget.justified` |
| **2** | P1-C | `visual_test_runbook_v1.md`, `run_visual_test_clean.ps1`, `tools/orchestrator/scripts/check_visual_runbook_no_raster_env.ps1` (new) | script self-test | Clean recipe has no required `RASTER_*` |
| **3** | P2-A | **New** `src/render/tile_raster_budget.rs` or `src/gui/visual_perf_budget.rs`, `tile_world_fallback.rs`, `src/gui/mod.rs`, `src/render/mod.rs`, `engine_with_worldgen.rs` | `tile_world_fallback` unit tests | PERF: `raster_b` bounded; no env |
| **4** | P2-B | `tile_world_fallback.rs`, `frame_budget_diagnostics.rs` (read p95 EMA), `ux_states.rs` | lib tests + 60 s manual | Zoom scroll no 100+ ms raster spikes |
| **5** | P2-C/D | `fire_visual_extract.rs`, `fire_chunk_runtime.rs`, `world_representation.rs` | `cargo test -p proc_A_dine01 --lib fire` | `fire_ecology_live.json` stable; `view_fire` p95 ↓ |
| **6** | P3 | `authoritative_viewport.rs`, `map_camera.rs`, `simulation_shell_phase2.rs` | visual + `stage5` tests | `viewport_drift.json` quiet in steady sim |
| **7** | P4 | `stage5_full_app_harness.rs`, `debug_run_envelope.rs`, `perf_attribution_60s.md`, CI script | `stage5` lib test for optional perf block | `stage5_full_app_live.json` optional `perf.p95_*` |

**Deprecated after Slice 3:** direct `std::env::var("RASTER_CHUNKS_PER_FRAME")` in release builds (debug-only `DEV_RASTER_CHUNKS_PER_FRAME` optional).

---

## 6. Verification + witness matrix

### Reference hardware baseline (document before signoff)

Record in this doc §Baseline when measured:

| Field | Value (fill at signoff) |
|:---|:---|
| Machine | e.g. Win10, 8c/16t, RTX * |
| Profile | `--release` |
| World | Default visual test harness (320×320 or current `WorldGenParams`) |
| Command | `.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release` |
| Duration | 60 s after enter Simulation (exclude worldgen window) |

### Commands

```powershell
# Clean ship-like run (no RASTER_*, no STALL_SPAN_DEBUG)
.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release

# Profiling session (not acceptance)
.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release -StallDebug

# Lib regression
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib minimap_compositor
cargo test -p proc_A_dine01 --lib tile_world_fallback
cargo test -p proc_A_dine01 --lib fire
```

### Measurable acceptance targets

| Metric | Target | How to read |
|:---|:---|:---|
| Frame p95 | **< 33 ms** | PERF line `wall=` or `FrameBudgetDiagnostics` history |
| Frame p99 | **< 50 ms** | Same, 60 s window |
| `raster_b` p95 (GPU minimap on) | **< 12 ms** | PERF `raster_b=` when `tile_raster_ran` |
| `view_fire` p95 | **< 8 ms** | PERF `upd_span view_fire=` (requires `-StallDebug` for span) |
| Env | **No `RASTER_*` set** | `Get-ChildItem Env:RASTER_*` empty before run |
| Witness green | unchanged contracts | See matrix below |

### Witness JSON keys (must stay green)

| File | Keys to check |
|:---|:---|
| `debug_runs/stage5_full_app_live.json` | `readiness.passes`, `representation_valid`, `fire_playback.stable`, `f2_extract_witness.green` |
| `debug_runs/minimap_compositor_live.json` | `composite_ok`, `presentation_source`, `dual_minimap_present: false`, `extent` stable |
| `debug_runs/stage6_virtualization_live.json` | virtualization / residency fields per existing spec |
| `debug_runs/infrastructure_view_isolation_live.json` | view isolation — no regression from perf changes |

### Release-mode / no-env validation

```powershell
Remove-Item Env:RASTER_* -ErrorAction SilentlyContinue
cargo run -p proc_A_dine01 --release -- --test visual --stay-open
# Assert: app runs 60s; no requirement to set RASTER_*; p95 within targets
```

---

## 7. Observability (60 s visual run protocol)

**Daily operator path** — not `STALL_SPAN_DEBUG`:

1. `run_visual_test_clean.ps1 -Release` (clears debug env; sets minimal `PERF=1` + `RUST_LOG=warn,error` only).
2. Enter Simulation; wait 10 s settle.
3. Record 60 s of PERF lines (or enable `PERF_CAPTURE=1` once Slice 7 lands).
4. Summarize p95 for: `wall`, `raster_b`, `view_fire` (if `-StallDebug` used for span), `stream_late`.

**Profiling path** (bisect only): `PERF=1`, `STALL=1`, optional `STALL_SPAN_DEBUG=1` per [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md).

Deliverable: [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) (create in Slice 7) with worked example.

---

## 8. Risks and mitigations

| Risk | Phase | Mitigation | Rollback |
|:---|:---|:---|:---|
| GPU minimap off in editor/build | P1-B | Keep CPU fallback when `registry.committed_image` default | Feature flag `MINIMAP_GPU_COMPOSITOR` |
| Under-raster stale minimap | P2-A | Dirty revision + cadence still drive main map; compositor refresh Hz | Raise `chunks_per_frame` in budget profile |
| Fire extract stale tactical view | P2-C | Full scan on sim tick + overlay revision bump | Re-enable per-frame extract behind `DEV_FIRE_EVERY_FRAME` (debug) |
| Stage 5 contract regression | All | `cargo test -p proc_A_dine01 --lib stage5` each slice | Revert slice; keep P1-A |
| CI false positive on runbook | P4-C | Allow explicit "debug-only" table rows; fail only "clean run" section | Narrow grep pattern |
| Coder B queue starvation | Planning | Dedicated **PERF-VIS** lane parallel to S7B after steward | Fleet dispatch row |

---

## 9. Definition of done

- [ ] **P1-A** DONE (duplicate CPU minimap skip).
- [ ] **P1-B/C** DONE — runbook + script + compositor default aligned.
- [ ] **`TileRasterBudget`** + **`FireExtractCadence`** initialized from `VisualBudgetSettings`; no release `RASTER_*` reads.
- [ ] 60 s `--test visual --stay-open` **release**, no `RASTER_*`, p95 frame **< 33 ms** on documented baseline.
- [ ] p95 `raster_b` **< 12 ms** with GPU minimap on; p95 `view_fire` **< 8 ms**.
- [ ] Stage 5 / minimap witnesses green (matrix §6).
- [ ] CI fails if clean runbook requires `RASTER_*`.
- [ ] `plan_visual_perf_production_v1.md` marked superseded by this exec doc for tracking.

---

## 10. Immediate next 48-hour action list

| Priority | Owner | Action |
|:---:|:---|:---|
| 1 | Coder | **Slice 2** (P1-C): extend `run_visual_test_clean.ps1` to `Remove-Item Env:RASTER_*`; add CI grep script |
| 2 | Coder | **Slice 1** (P1-B): wire Simulation default to GPU minimap when compositor witness green |
| 3 | Coder | **Slice 3** (P2-A): introduce `TileRasterBudget`; replace `raster_chunks_per_frame_budget()` env in release |
| 4 | Coder | **Slice 5** (P2-C/D): `FireExtractCadence` + remove spike-only extract as sole policy |
| 5 | Operator | Run 60 s release clean script; fill §Baseline table in this doc |
| 6 | Planner | Add `PERF-VIS-*` rows to `coder_active_queue.json` / fleet dispatch after S7B steward |

---

## Start Here (@coder — next 48 h)

1. Confirm **P1-A** on your branch: grep `tile_fallback_cpu_minimap_raster_needed` and run `cargo test -p proc_A_dine01 --lib tile_world_fallback`.
2. Implement **Slice 2** (P1-C) — lowest risk, unblocks CI guardrail.
3. Implement **Slice 3** (P2-A) — `TileRasterBudget` resource + init in `engine_with_worldgen.rs` from `VisualBudgetSettings` + world size (chunks/frame: release **4**, debug **8** default).
4. Wire rasterize to read `Res<TileRasterBudget>` and `Res<TileFallbackRasterPolicy>`; keep `DEV_RASTER_*` override only in `#[cfg(debug_assertions)]`.
5. Run `.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release` for 60 s; capture PERF p95 before/after Slice 3.
6. Only then start **Slice 5** (fire cadence) — do not block P1 on fire work.

### Prioritized queue (coder B after S7B, or split perf lane)

| Order | ID | Blocker |
|:---:|:---|:---|
| 1 | `PERF-VIS-001-P1BC` | None — after S7B disk witness or parallel |
| 2 | `PERF-VIS-002-P2A` | P1-C merged |
| 3 | `PERF-VIS-002-P2CD` | P2-A merged |
| 4 | `PERF-VIS-003-P3` | P2 steady p95 green |
| 5 | `PERF-VIS-004-P4` | Optional telemetry |

**Do not** set `RASTER_*` on the acceptance run. Use budgets + `UxFrameSpikeGuard` only.

---

## Ownership map

| Role | Responsibility |
|:---|:---|
| **Planner** | This exec plan, acceptance targets, fleet row routing |
| **Coder (render)** | `tile_world_fallback.rs`, `TileRasterBudget`, fire extract, `frame_perf` |
| **Coder (gui)** | `VisualCadence`, minimap shell, viewport (`authoritative_viewport`, `map_camera`) |
| **Operator** | 60 s baseline run, witness refresh, signoff §Baseline |
| **Designer** | — (only if HUD layout drives viewport churn) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | PLAN-VISUAL-PERF-PRODUCTION-EXEC-001 — P1-A DONE; slices P1-B → P4 |
