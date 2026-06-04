# PLAN-FLEET-PHASE-NEXT-EXEC-001 — Coder execution slices `v1`

| Field | Value |
|:---|:---|
| **Parent** | [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) |
| **Queue ID** | **PLAN-FLEET-PHASE-NEXT-EXEC-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@coder A` (primary) · `@operator` (P0-1) |
| **Status** | **READY** |

PR-sized slices only — one lane per PR unless noted.

---

## P0 — Ship acceptance

| Slice | ID | Files | Tests | Witness keys | Exit |
|:---:|:---|:---|:---|:---|:---|
| 0 | **OPS-F01** | — (operator) | `run_visual_test_clean.ps1 -Release` | `perf_attribution_60s.md` §2026-05-28 | p95 table + HW baseline |
| 1 | **P0-2** | `src/render/visual_readiness_witness.rs`, `src/render/perf_attribution_witness.rs`, `src/render/stage5_full_app_harness.rs`, `src/render/mod.rs` | `cargo test -p proc_A_dine01 --lib stage5` | `stage5_full_app_live.json` → `visual_witness`, `perf_attribution_60s` | lib refresh writes perf block |
| 2 | **P0-3** | CI scripts (verify only) | `check_visual_runbook_no_raster_env.ps1`, `check_live_proof_containment.ps1 -HardFail` | — | both exit 0 |

---

## P1 — PERF-VIS tail

| Slice | ID | Files | Tests | Witness | Exit |
|:---:|:---|:---|:---|:---|:---|
| 3 | **P1-1** P1-B | `src/gui/hud/simulation_session.rs`, `src/render/minimap_compositor/pass.rs`, `src/engine/engine_with_worldgen.rs` | `minimap_compositor`, `stage5` | `presentation_source`, `composite_ok` | GPU default when RT committed (no env) |
| 4 | **P1-2** P2-B | `src/render/visual_perf_budget.rs`, `src/gui/hud/frame_budget_diagnostics.rs`, `src/render/tile_world_fallback.rs` | `chunk_grid_tests`, lib tile fallback | PERF `raster_b` | EMA tightens `chunks_per_frame` on spike |
| 5 | **P1-3** P2-D | `src/render/extraction/fire_visual_extract.rs`, `src/render/fire_chunk_runtime.rs` | `fire`, `stage5` (residency wiring) | **Runtime** p95 via OPS-F01 | `residency_scoped: true`; bounded query — p95 not lib-gated |
| 6 | **P1-4** P3 | `src/gui/map_camera.rs`, `src/gui/authoritative_viewport.rs`, `src/render/visual_readiness_witness.rs` | visual + `stage5` | `render_hole_steady_flip_count: 0` | 60 s sim no RENDER_HOLE flap |
| 7 | **P1-5** P4 | `src/render/perf_attribution_witness.rs`, `src/dev/plan_visual_perf_production_exec_001_v1.md` §Baseline | operator re-run + lib | perf block green vs targets | DoD §9 perf exec |

---

## P2 — DEV-CONTAIN tail

| Slice | ID | Files | Tests | Witness |
|:---:|:---|:---|:---|:---|
| 8 | **P2-1** | `src/dev/runtime_witness/construction.rs`, `src/construction/live_proof.rs` (shim) | `construction` | `construction_stage_live.json` |
| 9 | **P2-2** | `runtime_witness/industrial.rs`, `runtime_witness/logistics.rs` | industrial, logistics lib | `industrial_activation_live.json`, `logistics_throughput_live.json` |
| 10 | **P2-3** | `runtime_witness/fire.rs`, `runtime_witness/wave_p.rs` | fire, wave_p | `fire_ecology_live.json`, `wave_p_live.json` |
| 11 | **P2-4** | `runtime_witness/stage7_behavioral.rs`, `runtime_witness/stage7_play.rs` | `stage7_behavioral`, `stage7_play` | behavioral + play JSON |
| 12 | **P2-5** | `runtime_witness/wss_substrate.rs` | `wss_substrate` | `wss_substrate_live.json` |
| 13 | **P2-6** | `exceptions_manifest.json`, remove shims, `tools/orchestrator/ci/run.ps1` | `-HardFail` | parity diff all lanes |

---

## P3 — Optional depth (pick ≤2 per cycle)

| Slice | ID | Files | Tests | Witness |
|:---:|:---|:---|:---|:---|
| 14 | **P3-1** F2 | `src/render/fire_view_extract.rs`, projection graph, harness | `stage5` | `f2_extract_witness.green: true` |
| 15 | **VFX-VECTOR-SHAPES** | `src/render/tactical_vector_overlay.rs`, `Cargo.toml` | unit + stage5 harness | `tactical_vector_overlay.drawn_shapes > 0` |
| 16 | **STAGE5-VT-FLICKER** | `vt_spatial_invariants.rs`, harness seeds | `vt_ci_matrix` lib + **`--test visual`** | VR-04 confirmed live; lib collapse test ≠ visual-only |

---

## Regression (every slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7
cargo test -p proc_A_dine01 --lib chunk_grid_tests
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | Coder slices for PHASE-NEXT-2026-05-28 |
