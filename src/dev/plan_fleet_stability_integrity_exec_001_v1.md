# PLAN-FLEET-STABILITY-INTEGRITY-EXEC-001 — Coder execution slices `v1`

| Field | Value |
|:---|:---|
| **Parent** | [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) |
| **Queue ID** | **PLAN-FLEET-STABILITY-INTEGRITY-EXEC-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@coder A` + `@coder B` |
| **Status** | **SUPERSEDED (open work → exec-002)** |
| **Horizon** | 2 weeks (Horizon A + start B) |

PR-sized slices only — one workstream row per PR unless noted.

---

## Sprint 1 (week 1) — proof honesty + play scenario seed

| Slice | ID | Owner | Files | Tests | Exit |
|:---:|:---|:---|:---|:---|:---|
| 0 | **PLAY-TRUTH-002** | A | `src/dev/proof_grade.rs` (new), `src/dev/runtime_witness/mod.rs`, `src/render/stage5_full_app_harness.rs`, `src/economy/logistics/witness.rs`, `src/economy/logistics/witness_collectors.rs`, `src/engine/test_harness.rs` | `cargo test -p proc_A_dine01 --lib stage5 logistics`; grep gate test | No `patch_*_witness` / `apply_*_shortcut` on `ProofGrade::VisualCapture` path; G-PROOF-01 |
| 1 | **PLAY-TRUTH-003** | A | `src/render/stage5_full_app_harness.rs`, `src/dev/log_e01_visual_acceptance_v1.md` | `log_e01_visual_confirm_001_*` lib tests | `log_e01_fixture_green` (lib) vs `full_visual_confirm` (visual only) in JSON |
| 2 | **PLAY-TRUTH-001** | B | `src/engine/play_scenario.rs` (new), `src/engine/engine_with_worldgen.rs`, `src/gui/hud/simulation_session.rs`, `src/construction/`, `src/economy/activation/` | `cargo test -p proc_A_dine01 --lib construction stage7`; manual G-PLAY-01 script | `PlayScenarioId::DefaultIndustrial` — build → activate → logistics without `test_harness` bootstrap |
| 3 | **DEHACK-LOG-001** | B | `src/economy/logistics/witness.rs`, `witness_collectors.rs`, `mod.rs`, `src/engine/test_harness.rs` | `cargo test -p proc_A_dine01 --lib logistics` (48/48) | `routes_open > 0` from sim graph in `VisualCapture`; shortcuts lib-only |

**Start today:** A → **PLAY-TRUTH-002** · B → **PLAY-TRUTH-001** (parallel, disjoint files).

---

## Sprint 2 (week 2) — de-hack + containment tail

| Slice | ID | Owner | Files | Tests | Exit |
|:---:|:---|:---|:---|:---|:---|
| 4 | **DEHACK-VIEW-001** | A | `src/gui/map_camera.rs`, `src/gui/authoritative_viewport.rs`, `src/render/visual_diagnostics.rs`, `src/gui/map_view/projection/mod.rs` | `stage5`, viewport lib tests | Single commit path; no stray `MapCameraDesired` writes in sim default |
| 5 | **DEHACK-FIRE-001** | A | `src/render/extraction/fire_visual_extract.rs`, `src/render/fire_view_extract.rs`, `src/render/stage5_full_app_harness.rs` | `fire`, `stage5` | `fire_degraded_overlay_bootstrap: false` in default scenario; overlay = explicit `DegradedMode` only |
| 6 | **CONTAIN-D-001** | B | Delete shims: `src/io/streaming/wave_c_live_proof.rs`, `src/io/save/wave_s_live_proof.rs`, `src/render/stage6_live_proof.rs`, `src/render/view_runtime/live_proof.rs`; update plugin imports → `runtime_witness::*`; `exceptions_manifest.json` | `check_live_proof_containment.ps1 -HardFail` | 4 shim paths removed; plugins import writers directly |
| 7 | **DEHACK-WSS-001** | B | `src/substrate/shim.rs`, `src/substrate/ecs_retire.rs`, `src/dev/runtime_witness/wss_substrate.rs` | `wss_substrate` lib | Slab authoritative; shim compare-only; `dual_write_drift_max` witness key |

---

## Sprint 3+ (weeks 3–5, parallel tail)

| Slice | ID | Owner | Files | Exit |
|:---:|:---|:---|:---|:---|
| 8 | **STAB-PERF-001** | A | `src/render/visual_perf_budget.rs`, `debug_runs/perf_attribution_60s.md` | OPS-F01 p95 table filled |
| 9 | **STAB-VT-001** | A + operator | `src/dev/visual_run_blockers.md`, VT harness | VR-04 closed or won't-fix |
| 10 | **STAB-CI-001** | A | `.github/workflows/ci.yml`, `tools/orchestrator/ci/run.ps1` | `-D warnings` scoped green |
| 11 | **OPS-PLAY-001** | operator | `debug_runs/perf_attribution_60s.md`, runbook | 60s release + play checklist |

---

## Operator (parallel)

| ID | Task | Exit |
|:---|:---|:---|
| **OPS-PLAY-001** | `run_visual_test_clean.ps1 -Release` 60s + G-PLAY-01 manual script | Measured p95 in `perf_attribution_60s.md` |
| **OPS-VT5-001** | Capture VR-04 log during visual run | Feeds STAB-VT-001 |

---

## Designer (on-call)

| ID | Task |
|:---|:---|
| **PLAY-TRUTH-001 UX** | Default scenario readability — build/activate/logistics HUD copy |
| **FEAT-WSS-002 lang** | Hydrology player-read visual language (after DEHACK-WSS-001 plan) |

---

## Regression (every slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 logistics
.\tools\orchestrator\scripts\check_live_proof_containment.ps1
```

After CONTAIN-D-001:

```powershell
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail
```

---

## Grep gate (PLAY-TRUTH-002 acceptance)

Visual capture path must not call:

- `patch_s7p_logistics_throughput_witness_for_play_proof`
- `apply_s7p_logistics_throughput_witness_shortcut`
- `qualified_close` as green substitute when `ProofGrade::VisualCapture`

Suggested lib test: `proof_grade_visual_capture_has_no_witness_shortcuts` in `src/dev/proof_grade.rs`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | Sprint slices for PHASE-STABLE-2026-06 |
