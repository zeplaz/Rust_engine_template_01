# PLAN-PHASE2-EXEC-001 — POST-DRAIN Phase 2 (thin exec) `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-PHASE2-EXEC-001** |
| **Program** | POST-DRAIN-PHASE-2-001 |
| **Dispatch** | $ref:src/dev/planner_dispatch_prompts_20260608_v1.md |
| **Queue** | $ref:tools/orchestrator/queues/post_drain_phase2_queue.json |
| **Planner** | **SIGNED** |
| **Date** | 2026-06-08 |

**Rule:** Witness keys + COMMIT:SPEC only — no grammar / rowhouse / infra E-tail replans.

---

## Cycle 1 slices

### ⟨FIRE-STREAM-CLOSE-001⟩ — @sim-steward

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | Queue close only — code already green |
| **Witness** | $ref:debug_runs/fire_streaming_live.json |
| **Exit keys** | `green: true` · `streaming_wired: true` · `neighbor_wake_observed: true` |
| **COMMIT:OPS** | `continuation_queue.json` SLICE-TRIAGE-FIRE-STREAM → `done` |

---

### ⟨FIRE-FUEL-COUNTERS-001⟩ — @coder · Lane T-FIRE

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | Wire fuel spread counters into `fire_ecology_live_proof` writer |
| **Territory** | $sym:src/systems/fire/ — $ref:src/dev/plan_territory_matrix_002_v1.md |
| **Witness** | $ref:debug_runs/fire_ecology_live.json |

**Target keys** (`fire_f2_fuel_spread_001` block):

| Key | Current | Target |
|:---|:---:|:---:|
| `green` | true | **true** (maintain) |
| `ember_wired` | true | **true** |
| `ember_events_emitted` | ≥1 | **≥1** |
| `fuel_spread_counters_wired` | false | **true** ✅ |
| `fuel_depleted_cells` | 0 | **348** ✅ |
| `neighbor_spread_cells` | 0 | **558** ✅ |

**Close:** SLICE-MD-F2-03 → **done** (2026-06-08). Root cause: harness `elevation=0` below water line → `SurfaceWaterFireGate` blocked spread; fixed elevation 0.5 + schedule finalize after ember apply.

**Regression:** `cargo test -p proc_A_dine01 --lib fire::`

**COMMIT:WIT** `debug_runs/fire_ecology_live.json`

---

### ⟨DESIGN-WX-HUD-IMPL-001⟩ — @designer + @coder C · Lane U-PROD

| Half | Agent | Deliverable |
|:---|:---|:---|
| Spec delta | @designer | `design_weather_player_read_v1.md` **§Implementation** |
| Wire | @coder C | HUD + witness JSON |

**COMMIT:SPEC** (@designer): widget tree, data bindings, witness key table in §Implementation.

**Witness** (new): `debug_runs/weather_hud_player_read_live.json` — full key table in $ref:docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md§Implementation-witness

**P0 rollup:** `ops_wx_wired` · `ops_wx_vis_suffix_wired` · `precip_tactical_band_wired` · `precip_background_band_wired` · `minimap_wx_wash_wired` · `weather_sim_live_maintained` · `acceptance_player_read_at_glance` → **`green: true`**

**Maintain:** $ref:debug_runs/weather_sim_live.json `green: true`

**Regression:** `cargo test -p proc_A_dine01 --lib weather`

**COMMIT:WIT** `debug_runs/weather_hud_player_read_live.json`

---

### ⟨G-PLAY-01⟩ — Operator · Lane V-OPS

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | $ref:src/dev/plan_g_play_close_001_checklist_v1.md |
| **Runbook** | $ref:docs/archive/2026-06-src-dev/plans/play_scenario_acceptance_runbook_v1.md |
| **Exit** | Checklist **EXECUTED** row signed → unblocks PLAN-AUDIT-020 |

**Preconditions:** release build · no `--test visual` · no harness seed env.

---

## Cycle 2 (historical — closed on disk 2026-06-08)

| ⟨ID⟩ | Agent | Note |
|:---|:---|:---|
| TRIAGE-GPU-TILE-001 | @coder | **done** — do not re-assign |
| EGUI-QC-IMPL-001 | — | **cancelled** — lane 4 shipped |
| TRIAGE-REPLAY-001 | @sim-steward | **done** — verify only |
| DSM-SIGNOFF-001 | @orchestrator | **done** |

---

## RT★ / ATL★ (orchestrator — after Cycle 1 sync)

| Criterion | Witness |
|:---|:---|
| RT registry | `rt_registry_001_live.json` green |
| RT brief | `rt_lookup_brief_001_live.json` green |
| RT engine | `procedural_tiles_runtime_live.json` → `rt_eng_001.green` |
| ATL production | `atl_sign_001_live.json` + `aps_atlas_preview_002` |

Tensor: `auth_spine.ATL.phi` → 2 · `auth_spine.RT.phi` → 2

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Cycle 1 witness keys — PLAN-PHASE2-EXEC-001 |
