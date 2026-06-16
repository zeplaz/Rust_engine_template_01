# Fleet snapshot — returns reconcile `v2.1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-28 |
| **Prior** | [`fleet_snapshot_20260528_v1.md`](fleet_snapshot_20260528_v1.md) |
| **Audit** | [`planner_status_audit_v16.md`](planner_status_audit_v16.md) |
| **Phase plan** | [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) — **authoritative execution index** |
| **HANDOFF** | [`tools/orchestrator/queues/HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) |

---

## Executive summary

**@planner, @designer, @coder A, and @coder B have returned.** Product witnesses on disk are **largely green** (WSS rollup, Stage 7 M1–M4 play, construction, Stage 5 readiness). **Queue metadata is partially stale** (planner queue still points audit v14; coder A has no `done_2026_05_28` rows for perf/containment).

**Coder A landed substantial code** (not fully reflected in queue): `TileRasterBudget`, `FireExtractCadence`, perf P1-A, `src/dev/runtime_witness/` slices B–C. **Perf exec DoD and containment Slice D remain open.**

**Only remaining fleet primaries:** **PHASE-NEXT** — OPS-F01 60 s acceptance, PERF-VIS tail, DEV-CONTAIN tail (see [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md)).

---

## Return verdict table

| Role | Returned? | Verdict | Evidence |
|:---|:---:|:---|:---|
| **@planner** | yes | **DRAINED** (horizon exec signed) | `planner_status_audit_v15.md`, exec plans below |
| **@designer** | yes | **DRAINED** (on-call) | `DESIGN-VISUAL-PERF-DEGRADE-001` PASS in queue + registry |
| **@coder A** | yes | **PARTIAL CLOSE** — code ahead of queue | `visual_perf_budget.rs`, `runtime_witness/`, witnesses green |
| **@coder B** | yes | **DRAINED** | `coder_b.active: []`; M3/M4/LOG-E01 in `done_2026_05_27` |

---

## Witness board (disk wins)

| File | Status |
|:---|:---|
| `stage5_full_app_live.json` | `readiness.passes: true`; `log_e01_fullapp_upgrade_001.full_visual_confirm: false` (operator tail) |
| `wss_substrate_live.json` | **green** — `ecs_retire_fixture_green`, `wss_post_spine_001.green`, smoke authority false |
| `stage7_behavioral_live.json` | **green** — M3, steward, **M4 play** (`play_enqueue_wired: true`) |
| `construction_stage_live.json` | `operational_green: true` |
| `minimap_compositor_live.json` | green (prior) |

---

## What each role delivered

### @planner

| ID | Deliverable | Status |
|:---|:---|:---:|
| **PLAN-LEDGER-REFRESH-015** | [`planner_status_audit_v15.md`](planner_status_audit_v15.md) | **SIGNED** |
| **PLAN-VISUAL-PERF-EXEC-001** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) | **SIGNED** |
| **PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001** | [`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md) | **SIGNED** (B–C partial) |
| **PLAN-STAGE7-M4-PLAY-001** | [`plan_stage7_m4_play_001_v1.md`](plan_stage7_m4_play_001_v1.md) | **SIGNED** |

**Planner orders now:** None blocking. Optional **PLAN-LEDGER-REFRESH-016** after perf 60s acceptance.

---

### @designer

| ID | Deliverable | Status |
|:---|:---|:---:|
| **DESIGN-VISUAL-PERF-DEGRADE-001** | [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) | **PASS** |

**Designer orders now:** **On-call hold** only — M4 play read (not needed; disk green), Hanabi prod, R4 product.

---

### @coder A

| ID | Code / witness | Status |
|:---|:---|:---:|
| **P1-A** duplicate CPU minimap skip | `tile_world_fallback.rs` + policy sync | **DONE** |
| **PERF-VIS-002-P2A** `TileRasterBudget` | [`visual_perf_budget.rs`](../render/visual_perf_budget.rs) | **DONE** (release ignores `RASTER_CHUNKS` per tests) |
| **PERF-VIS-002-P2C/D** `FireExtractCadence` | wired in `fire_visual_extract.rs` | **PARTIAL** — verify p95 `view_fire` |
| **WSS witnesses** | `wss_substrate_live.json` rollup | **DONE** on disk |
| **DEV-CONTAIN B–C** | `runtime_witness/{stage6,view_runtime,wave_c,wave_s}.rs` | **DONE** |
| **DEV-CONTAIN Slice 1+** | minimap/construction/… writers | **OPEN** |
| **PERF-VIS-001/003/004** | P1-B/C, viewport, 60s CI | **OPEN** |

**Queue gap:** add `done_2026_05_28` rows for landed slices (hygiene).

---

### @coder B

| ID | Witness | Status |
|:---|:---|:---:|
| **S7B-M3-STEWARD-REMEDY-001** | `s7b_m3_green`, `s7b_steward_green` | **DONE** |
| **S7B-M4-PLAY-REMEDY-001** | `s7b_m4_play_green`, `play_enqueue_wired` | **DONE** |
| **LOG-E01-FULLAPP-UPGRADE-001** | lib witness green; `full_visual_confirm` false | **qualified** |

**Coder B orders now:** **None** (optional operator visual confirm).

---

## Orders going forward

### @planner

| P | Order |
|:---:|:---|
| — | **Stand down** until perf 60s run completes |
| opt | **PLAN-LEDGER-REFRESH-016** → audit v16 |

### @designer

| P | Order |
|:---:|:---|
| — | **Stand down** (on-call only) |

### @coder A

| P | ID | Notes |
|:---:|:---|:---|
| **1** | **PERF-VIS-001-P1BC** | Runbook CI script + GPU minimap default in Simulation |
| **2** | **PERF-VIS-004-P4** | 60s attribution fields in readiness witness |
| **3** | **PERF-VIS-003-P3** | Viewport validity / `RENDER_HOLE_FLIP` tail |
| **4** | **DEV-CONTAIN-SLICE-1** | `runtime_witness/minimap.rs` — first open lane in containment exec |
| **5** | Queue hygiene | `done_2026_05_28` block in `coder_active_queue.json` |

### @coder B

| P | Order |
|:---:|:---|
| — | **Stand down** |

### @operator

| P | Order |
|:---:|:---|
| **1** | `.\tools\orchestrator\scripts\run_visual_test_clean.ps1` + 60s release visual |
| **2** | Set `log_e01_fullapp_upgrade_001.full_visual_confirm: true` if visual passes |
| **3** | Capture p95 + `upd_span` for perf exec baseline table |

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 chunk_grid_tests
.\tools\orchestrator\scripts\check_visual_runbook_no_raster_env.ps1
.\tools\orchestrator\scripts\check_live_proof_containment.ps1
```

---

## Version

| Version | Date | Notes |
|:---|:---|:---|
| v2.1.0 | 2026-05-28 | Planner/designer/coder A/B return reconcile |
