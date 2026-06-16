# Runtime environment policy registry `v1` (DEHACK-ENV-001)

| Field | Value |
|:---|:---|
| **ID** | **DEHACK-ENV-001** |
| **Owner** | `@planner` (registry) · `@coder A` (review / sunset PRs) |
| **Status** | **ACTIVE (P2)** |
| **Source** | [`production_jank_sweep_20260602_v1.md`](production_jank_sweep_20260602_v1.md) §D |
| **Parent plan** | [`plan_fleet_stability_integrity_exec_002_v1.md`](plan_fleet_stability_integrity_exec_002_v1.md) |

**Policy classes**

| Class | Meaning |
|:---|:---|
| **debug-only** | Dev/CI only; must not change default ship behavior when unset |
| **ops-runbook** | Operator/visual acceptance runs; documented in `debug_runs/README.md` or runbooks |
| **ship-config** | Intentional product tuning via env (prefer migration to RON/resources) |
| **deprecated** | Remove on next touch; do not add new callers |

---

## Registry (sweep §D + engine paths)

| Env var | Class | Primary file(s) | Behavior when set | Sunset / notes |
|:---|:---|:---|:---|:---|
| `PERF_NO_VSYNC` | **ops-runbook** | `src/engine/engine_with_worldgen.rs` | Disables vsync present mode | Document in perf runbook; not default ship |
| `MINIMAP_GPU_COMPOSITOR` | **debug-only** | `src/render/minimap_compositor/pass.rs`, `mod.rs` | Forces GPU compositor path vs default | GPU default is ship; env = override for A/B |
| `MINIMAP_GPU_DEBUG` | **debug-only** | `src/render/minimap_compositor/diagnostics.rs` | Extra compositor logging | — |
| `VISUAL_DIAG` | **debug-only** | `src/render/visual_diagnostics.rs` | Enables visual diag plugin trace | Prefer `--debug-visual-diag` long-term |
| `STAGE5_VERBOSE` | **debug-only** | `src/render/visual_diagnostics.rs`, `frame_perf.rs` | Verbose stage5 / visual diag | Pair with `RUST_LOG=visual_diag=...` |
| `STAGE5_READINESS_VERBOSE` | **debug-only** | `src/render/frame_perf.rs` | Readiness logging | — |
| `STAGE5_FENCE_VERBOSE` | **debug-only** | `src/render/visual_snapshot_commit.rs` | Fence commit logging | — |
| `RUST_ENGINE_SUBSTRATE` | **ops-runbook** | `src/substrate/mod.rs` | Substrate mode selection | DEHACK-WSS done — compare-only default |
| `RUST_ENGINE_SUBSTRATE_DUAL_WRITE` | **debug-only** | `src/substrate/shim.rs` | Enables dual-write mirror | **deprecated** for ship; opt-in debug only |
| `RUST_ENGINE_FIRE_DEGRADED_OVERLAY` | **debug-only** | `src/render/stage5_full_app_harness.rs` | Overlay bootstrap for proofs | DEHACK-FIRE-001: default off |
| `RUST_ENGINE_STAGE7_PLAY_SEED` | **debug-only** | `src/economy/activation/concrete_chain_e2e.rs` | One-shot Portland chain seed | Prefer `PlayScenarioPlugin` (PLAY-TRUTH-001) |
| `RUST_ENGINE_CONSTRUCTION_INSTANT` | **debug-only** | `src/economy/activation/concrete_chain_e2e.rs` | Skips staged tick; fast-forwards Portland to Operational | CON-P2-001 default off; witness/legacy proofs only |
| `RUST_ENGINE_S7P_STEWARD` | **deprecated** | — | *(sunset DEHACK-ENV-002)* | Removed 2026-05-28; use `PlayScenarioPlugin` |
| `RUST_ENGINE_IND_E03_SEED` | **debug-only** | `src/economy/activation/concrete_chain_e2e.rs` | IND-E03 seed | Scenario-driven activation target |
| `TACTICAL_VFX_PROOF` | **ops-runbook** | `src/render/stage5_full_app_harness.rs` | Tactical VFX proof lane | Visual acceptance only |
| `STREAMING_HYDRATE_BUDGET` | **ship-config** | `src/io/streaming/budget.rs` | Hydrate budget override | Prefer RON tuning file |
| `STREAMING_RECONSTRUCT_BUDGET` | **ship-config** | `src/io/streaming/budget.rs` | Reconstruct budget | Same |
| `MAX_STREAMING_PENDING_CHUNKS` | **ship-config** | `src/io/streaming/budget.rs` | Pending chunk cap | Same |
| `STREAMING_SYNC_HYDRATE` | **ops-runbook** | `src/io/streaming/budget.rs` | Sync hydrate mode | CI stress only |
| `STREAM_DIAG` | **debug-only** | `src/io/streaming/diagnostics.rs` | Streaming diagnostics | — |
| `PERF` | **debug-only** | `src/render/frame_perf.rs` | Perf logging | — |
| `STALL` | **debug-only** | `src/render/frame_perf.rs`, `stall_watch.rs` | Stall detection | — |
| `STALL_SPAN_DEBUG` | **debug-only** | `src/render/stall_watch.rs` | Span-level stall debug | — |
| `FIRE_SPARK_COMPUTE` | **debug-only** | `src/render/gpu_particles.rs` | Spark compute path toggle | — |
| `VIEW_RUNTIME_AUDIT` | **debug-only** | `src/render/view_runtime/plugin.rs` | View runtime audit | — |
| `RASTER_MINIMAP` | **deprecated** | tests / `minimap_compositor/mod.rs` | Legacy raster path | PERF-VIS-P1B GPU default — do not use in ship |
| `RASTER_CHUNKS_PER_FRAME` | **deprecated** | tests / compositor | Legacy raster throttle | Same |
| `MINIMAP_STRESS_CHROME` | **debug-only** | `src/gui/hud/ui_stress_state.rs` | UI stress chrome | — |
| `WAVE_S_AUTOLOAD_SHELL` | **ops-runbook** | `src/io/save/wave_s_artifacts.rs` | Autoload shell snapshot | Dev convenience |
| `STAGE5_VERBOSE` (witness envelope) | **debug-only** | `src/dev/debug_run_envelope.rs` | Recorded in `_agent_meta` | Proof JSON only |

**CLI / launch (not env, documented for policy):**

| Surface | Class | Notes |
|:---|:---|:---|
| `EngineLaunchArgs::test_mode()` / `--test` | **ops-runbook** | Enables harness plugin (P0 #1 gate) |
| `--debug-visual-diag` | **debug-only** | Equivalent intent to `VISUAL_DIAG` |

---

## Maintenance rules

1. New `std::env::var` in `src/` (outside tests) **must** add a row here in the same PR or be rejected at review.
2. **ship-config** envs need a tracked issue to move to RON (`registry_serde_path` policy).
3. **deprecated** envs: grep gate in CI (future STAB-CI slice) — zero new references.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Initial registry from production jank sweep §D (DEHACK-ENV-001) |
