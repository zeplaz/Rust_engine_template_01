# Production jank sweep `v1` (2026-06-02)

| Field | Value |
|:---|:---|
| Scope | Runtime/build-path code under `src/` (not doc-only drift) |
| Goal | Identify non-production patterns still compiled or executed in real app paths |
| Method | `rg` sweep for harness hooks, witness shortcuts, env toggles, scaffolds, `allow(dead_code)` |
| Status | **PHASE-STABLE P1 ACTIVE** — P0 #1 done; open = DEHACK-ENG/RENDER/LOG (exec-002) |
| Exec | [`plan_fleet_stability_integrity_exec_002_v1.md`](plan_fleet_stability_integrity_exec_002_v1.md) |

---

## Critical findings (P0)

### 1) Test harness plugin on production app path

- File: `src/engine/engine_with_worldgen.rs`
- Finding: `TestHarnessPlugin` was added unconditionally in `EnginePlugin`.
- Risk: non-test runs carried test-harness systems in Update schedule.
- Fix landed: add plugin only when `EngineLaunchArgs.test_mode()` is true.
- **DEHACK-ENG-001 landed:** split `TestHarnessStatePlugin` + `TestHarnessMenuPlugin` (always) vs `TestHarnessPlugin` (CLI `--test` only); removed harness types from `engine::*` root re-exports; `main` inserts active harness only when `test_mode()`.
- Verification: `cargo check -p proc_A_dine01` passes.

### 2) LOG-E01 fixture vs visual closure confusion

- Files: `src/render/stage5_full_app_harness.rs`, `docs/archive/2026-06-src-dev/plans/log_e01_visual_acceptance_v1.md`, `docs/archive/2026-06-src-dev/plans/log_e01_full_app_witness_spec_v1.md`
- Finding: historical contract allowed fixture narratives to look like visual closure.
- Risk: witness theater in ship decisions.
- Status: PLAY-TRUTH-003 landed:
  - `full_visual_confirm` is visual-run-only (`capture_lane: visual_run`)
  - fixture lane uses `log_e01_fixture_green`
  - `qualified_close` no longer substitutes green in non-fixture grades

---

## High-risk production-path jank (P1)

### A) Test/bootstrap logic still deeply coupled to runtime binary

- Files:
  - `src/engine/test_harness.rs`
  - `src/engine/mod.rs` (`pub mod test_harness`, exports)
  - `src/engine/ux_orchestration.rs` (harness interplay)
  - `src/main.rs` (always inserts `TestWorldHarness` resource)
- Why risky:
  - test-only seed flows (logistics, industrial, minimap, fire) are compiled into ship binary paths.
  - plugin is now gated, but harness resources/exports remain broad.
- Recommendation:
  - Move harness behind a crate feature (e.g. `dev_harness`) or stronger module boundary.
  - Keep CLI `--test` support via explicit opt-in build/run mode.

### B) Witness refresh functions exported from production render API

- Files:
  - `src/render/mod.rs` re-exports many `refresh_*_live_witness` functions
  - `src/dev/*_bundle_proof.rs` call chains invoke refreshes as pseudo-acceptance.
- Why risky:
  - Dev witness mutation helpers are easy to call from non-test code by accident.
  - Blurs runtime authority vs proof-authoring tooling.
- Recommendation:
  - Gate re-exports with `#[cfg(test)]` or `#[cfg(feature = "dev_witness_tools")]` where possible.
  - Keep runtime writer systems in `runtime_witness` authoritative path; keep refresh helpers dev-only.

### C) Remaining logistics shortcut code (fixture-only by policy, still present)

- Files:
  - `src/economy/logistics/witness.rs`
  - `src/economy/logistics/witness_collectors.rs`
  - `src/economy/logistics/mod.rs` exports shortcut helpers
- Why risky:
  - Known hack entrypoints (`apply_s7p_*`, `patch_s7p_*`) still exist and are callable.
- Current guard:
  - `ProofGrade::VisualCapture` blocks shortcut effects + grep gate test.
- Recommendation:
  - Continue DEHACK-LOG-001: remove shortcut calls from non-fixture flows, reduce public export surface.

---

## Medium-risk jank (P2)

### D) Env toggles that can alter runtime behavior

- Files (runtime-impacting):
  - `src/engine/engine_with_worldgen.rs` (`PERF_NO_VSYNC`)
  - `src/render/minimap_compositor/pass.rs` (`MINIMAP_GPU_COMPOSITOR`)
  - `src/render/visual_diagnostics.rs` (`VISUAL_DIAG`, `STAGE5_VERBOSE`)
  - `src/substrate/mod.rs`, `src/substrate/shim.rs` (`RUST_ENGINE_SUBSTRATE*`)
  - `src/render/stage5_full_app_harness.rs` (`RUST_ENGINE_FIRE_DEGRADED_OVERLAY`, proof flags)
  - `src/io/streaming/budget.rs` (streaming budget env knobs)
- Why risky:
  - Environment dependence can diverge behavior between operator/dev/prod paths.
- Recommendation:
  - Keep env controls debug/ops scoped; define release profile defaults in resources/config.
  - For each env, classify as: `debug-only`, `ops-runbook`, `deprecated`.

### E) Transitional scaffolds and dead-code allowances in runtime trees

- Files:
  - `src/construction/witness_collectors.rs` (`#[allow(dead_code)]`)
  - `src/economy/activation/witness_collectors.rs` (`#[allow(dead_code)]`)
  - `src/economy/logistics/witness_collectors.rs` (`#[allow(dead_code)]`)
  - `src/gui/map_camera.rs` (`#[allow(dead_code)]`)
  - `src/render/gpu_surface_teardown.rs` (`#[allow(dead_code)]`)
  - plus multiple scaffold-marked modules (`strategic_icon_instances`, `fire_chunk_runtime`, etc.)
- Why risky:
  - Unfinished scaffolds drift into long-term runtime dependencies.
- Recommendation:
  - Add/refresh `ScaffoldContract` expiry tags per module and enforce via CI grep/lint check.

---

## Open removal candidates (from containment manifest)

Current `allowed_shim_paths` still includes 5 paths in `src/dev/runtime_witness/exceptions_manifest.json`:

- `src/io/streaming/wave_c_live_proof.rs`
- `src/io/save/wave_s_live_proof.rs`
- `src/render/stage6_live_proof.rs`
- `src/render/minimap_compositor/live_proof.rs`
- `src/render/view_runtime/live_proof.rs`

Plan alignment:
- `CONTAIN-D-001` currently targets 4 immediate retirements (all except minimap shim).
- Keep minimap shim until explicit handoff row says direct runtime_witness import is safe.

---

## Recommended next sweep actions

1. **P1-A:** ~~Move `test_harness` module + exports behind dev feature boundary.~~ **DONE** — DEHACK-ENG-001 (state/menu vs CLI plugin split; narrow `engine` re-exports).
2. **P1-B:** Reduce `render/mod.rs` dev witness re-exports from production API.
3. **P1-C:** Execute `DEHACK-LOG-001` and shrink public shortcut symbols.
4. **P2-A:** Create env-toggle registry file with owner + allowed mode + sunset date.
5. **P2-B:** Add CI check for `#[allow(dead_code)]` in non-whitelisted runtime files.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | First full runtime jank sweep; includes landed TestHarnessPlugin gate |
| v1.1.0 | 2026-06-02 | Planner sign-off: exec-002 three PRs; env registry DEHACK-ENV-001 |

