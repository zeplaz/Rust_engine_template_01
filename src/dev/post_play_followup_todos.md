# Post-PLAY follow-up — other areas of concern

**Created:** 2026-05-22 · **Closed:** 2026-05-22

**Scope:** Infrastructure, perf, Stage 5 operator exit, construction, representation depth, economy live sim, ops.

**Lane map:** [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) · [`construction_active_progress.md`](construction_active_progress.md)

---

## Status summary

| Track | State | Proof |
|-------|-------|-------|
| INFRA | Done | `infrastructure_view_isolation_live.json` green; VM checklist in `viewport_pipeline.md` runbook |
| PERF | Done | Gates in code + [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) |
| STAGE5-OP | Done | `stage5_full_app_live.json` passes; boards all Done |
| CONSTRUCTION | Done | `construction_stage_live.json` phase2 + operational green |
| REPR | Done (v1) | GPU instanced tile path + per-view fire extract wired; phase GPU tints = future slice |
| BUILD-UX | Done | Mock shapes menu, F7 hint, RON round-trip test |
| ECON-LIVE | Done | Lib + live JSON `open_todos: 0` |
| OPS | Done | 599 lib tests; orchestrate on touch |
| **Deferred** | CONST-DEF-01/02, BUILD-UX-05 | Undo/redo + baked textures — next milestone |

---

## INFRA — View isolation (VM-06…VM-11)

- [x] **INFRA-01** — `infrastructure_view_isolation_live.json` green
- [x] **INFRA-02** — VM-06 minimap lockstep suspect false
- [x] **INFRA-03** — VM-07 preview contracts in stage5 proof
- [x] **INFRA-04** — VM-08 per-view overlay fire flags in isolation JSON
- [x] **INFRA-05** — VM-09 bridge witness + dual_writer false

**Note:** Isolation **witness** green; full VM-06…11 **implementation** → [`stage5_triage_backlog.md`](stage5_triage_backlog.md) / [`stage5_5_open.md`](stage5_5_open.md).
- [x] **INFRA-06** — VM-10 representation per ViewId (spine tests)
- [x] **INFRA-07** — VM-11 runbook checklist added
- [x] **INFRA-08** — VM-B construction uses `ViewProjectionAuthority` + `ConstructionMapProjection`

---

## PERF — Frame budget & attribution

- [x] **PERF-01** — [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md)
- [x] **PERF-02** — `reconstruct_staged_chunks_into_cache` early return when empty
- [x] **PERF-03** — `shell_widget_runs_egui_with_budget` + lightweight drag chrome
- [x] **PERF-04** — Documented WARN bias + `tracing::enabled!` pattern in perf guide
- [x] **PERF-05** — WorldGen/preview gated off in sim

---

## STAGE5 — Operator exit

- [x] **STAGE5-OP-01** — `stage5_full_app_live.json` + visual script
- [x] **STAGE5-OP-02** — `fire_playback` block in proof JSON (`FirePlaybackStabilityWitness`)
- [x] **STAGE5-OP-03** — `live_todo_board.all_done: true`
- [x] **STAGE5-OP-04** — `FINISH-UX-*` Done; map_fit mismatch_frames 0
- [x] **STAGE5-OP-05** — Optional spot-check documented in runbook

---

## CONSTRUCTION — Phase 2 + operational

- [x] **CONST-P2-01…04** — All `PHASE2-BUILD-*` Done in live JSON + witnesses
- [x] **CONST-OP-01** — `operational_green: true`
- [x] **CONST-DEF-01** — Ctrl+Y redo (`replay_road_tiles_for_redo`, 2026-05-22)
- [x] **CONST-DEF-02** — Demolish undo restore (`record_demolish_execution`, 2026-05-22)

---

## REPR — GPU tile & fire depth

- [x] **REPR-01** — `register_tile_debug_instanced_draw` + storage upload path
- [x] **REPR-02** — `tile_flags` FOCUS/FIRE/TERRAIN/FOOTPRINT_* in `gpu_tile_debug.rs`
- [x] **REPR-03** — `sync_visible_fire_chunks_from_views` per ViewManager
- [x] **REPR-04** — `FireChunkLodState` + `build_fire_visual_frames_by_view`
- [x] **REPR-05** — Footprint flags on GPU instances (valid/risky/invalid); phase enum = future

---

## BUILD — Road / UX polish

- [x] **BUILD-UX-01** — `mock_shapes_menu.rs` + Construction toolbox
- [x] **BUILD-UX-02** — `mock_shape_ron_roundtrip_matches_registry_footprint` test
- [x] **BUILD-UX-03** — `IntersectionRegistry` on road commit (`execute_construction_plans_system`)
- [x] **BUILD-UX-04** — F7 tile labels in command shell hint
- [x] **BUILD-UX-05** — Phase/GPU path documented (`view_runtime_architecture_v1.md` §16; CPU `phase_visual` until overlay slice)

---

## ECON-LIVE — Economy live sim validation

- [x] **ECON-01** — `logistics_throughput_live.json` `open_todos: 0`
- [x] **ECON-02** — `industrial_activation_live.json` green
- [x] **ECON-03** — Logistics section in diagnostics UI (F3)
- [x] **ECON-04** — Industrial proof witnesses (I3 grid/thermal in live JSON)

---

## OPS — CI & handoff

- [x] **OPS-01** — `cargo test -p proc_A_dine01 --lib` (599 pass)
- [x] **OPS-02** — `cargo orchestrate` documented in AGENTS.md
- [x] **OPS-03** — Session handoff via this board + `construction_active_progress.md`
- [x] **OPS-04** — 0 test failures on touch

---

## Commands

```powershell
cargo test -p proc_A_dine01 --lib
.\tools\orchestrator\scripts\visual_full_app.ps1
cargo run -p proc_A_dine01 -- --test visual
```
