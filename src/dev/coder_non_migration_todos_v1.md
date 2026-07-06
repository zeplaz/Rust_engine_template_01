# Coder todos — non-migration lane `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-07-03 (active queue refresh) |
| **Bevy** | **0.19** on master — MIG-V1 green |
| **Remaining backlog** | [`coder_todos_v1.md`](coder_todos_v1.md) · [`coder_remaining_post_019_v1.md`](coder_remaining_post_019_v1.md) |
| **Active streams** | **4** — see § Active queue below |
| **Rule** | All **@coder / @coder_a / @coder_b / @coder-mcp** picks **except** closed MIG-V1 mechanical lane |
| **Feature gates** | Keep **`bevy_tilemap_adapter`** and other non-0.19 features **OFF** — see remaining doc § Feature gates |
| **Hub queues** | **[`coder_todos_v1.md`](coder_todos_v1.md)** (authoritative) · [`remaining_lane_todos_v1.md`](remaining_lane_todos_v1.md) · [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) · [`cross_front_pick_queue_v1.md`](cross_front_pick_queue_v1.md) |
| **Ritual** | [`coder_crisis_filter_todos_v1.md`](coder_crisis_filter_todos_v1.md) — witness wins over stale `reopened` |

```powershell
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
python tools/orchestrator/scripts/reconcile_coder_crisis.py
cargo test -p proc_A_dine01 --lib block_recipe building_quality city_g1 stage5 construction
python tools/orchestrator/scripts/scan_queues_hub.py
```

---

## Active queue (4 streams)

**Delegated session board:** [`delegated_lane_todos_v1.md`](delegated_lane_todos_v1.md) — BQ-C4 · APSR-S1/S2 · PERF-INSTR-VFX-002 · BQ-H1/H2 tracked there when handled outside this hub.

Pick **one primary stream per session**; parallel only when file mutex is empty (see [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md)).

### Stream 1 — PERF-INSTR-VFX-002 `@coder_b` (P0)

| ☐ | Step | Exit |
|:---:|:---|:---|
| ☐ | Run acceptance | `cargo run -p proc_A_dine01 --release -- --test vfx` (~60s; **display required**) |
| ☑ | Lib witness | `triage_perf_vfx_002_lib_green` · PerfScope slices ≤5ms |
| ☑ | Witness refresh | `debug_runs/triage_perf_vfx_fix_2026-06-11_live.json` (`display_acceptance_pending: true`) |

**Note:** Phase 2A–2D shipped; full gate needs operator display run for `steady_wall_p50_ms ≤ 33`.

---

### Stream 2 — BQ-C/A + APSR-S `@coder` + `@coder-mcp` (P1) — **primary pick**

**Parallel OK:** coder-mcp on `tools/mcp/` while coder on `src/construction/procedural/`.

| ☐ | ID | Owner | Exit |
|:---:|:---|:---|:---|
| ☑ | **BQ-C1** | coder-mcp | `module_contract_v1` · `pytest -k building_quality_bq_c1` |
| ☑ | **BQ-C2** | coder-mcp | Bounds/pivot validator · `bq_c2_bounds_001_live.json` |
| ☑ | **BQ-C3** | coder-mcp | Seam/pair validator per style pack |
| ☑ | **BQ-C4** | coder | Scale-chain audit · `bq_c4_scale_001_live.json` |
| ☑ | **BQ-A1** | coder | `edge_adjacency.rs` · `bq_a1_adjacency_001_live.json` |
| ☑ | **BQ-A2** | coder | Assembly quality gate · `building_quality_live.json` (`bq_a1_wired`) |
| ☑ | **APSR-S1** | coder-mcp | EventBus + SuiteStateWriter · `apsr_a1_s1_001_live.json` |
| ☑ | **APSR-S2** | coder-mcp | AssemblyService · `apsr_a1_s2_001_live.json` |

**Verify:**

```powershell
cd tools/mcp/python && python -m pytest tests/test_building_quality_bq_c1.py tests/test_aps_mutation_inventory.py tests/test_aps_panel_sync_characterization.py -q
cargo test -p proc_A_dine01 --lib procedural_build_extract assembly_snapshot -q
```

**Closed — do not re-pick:** BQ-F1/F2/F3 · APSR-A0-T1/T2.

Plan: [`plan_building_quality_v1.md`](plan_building_quality_v1.md) · [`plan_aps_refactor_v1.md`](plan_aps_refactor_v1.md)

---

### Stream 3 — CITY post-G3 `@coder` (P2) — **CLOSED 2026-07-03**

G0–G3 + P1/P2 + CITY-DOC-002 done. Only **DR-MIG-TILEMAP** remains (steward, crate not shipped).

| ☑ | **CITY-P1-001** · **CITY-P2-001** · **CITY-DOC-002** | witnesses green |

**Verify:** `cargo test -p proc_A_dine01 --lib city_g0 city_g1 city_g3 city_p1 city_p2 city_c6`

---

### Stream 4 — BQ-H/K/Q + APSR-P/D/Q `@coder` + `@coder-mcp` (P3)

**Gate:** BQ-A2 + APSR-S landed — Stream 4 **unblocked**.

| ☐ | Phase | Owner | Slices |
|:---:|:---|:---|:---|
| ☑ | **H (start)** | coder | BQ-H1 FacadeRule by_massing · `bq_h1_facade_001_live.json` |
| ☑ | **H2** | coder | BQ-H2 street-facing door · `bq_h2_openings_001_live.json` |
| ☑ | **H3** | coder | v0 freeze shim · `bq_h3_v0_retire_001_live.json` |
| ☑ | **K charters** | designer-mcp | BQ-K1/K2/K3 **SIGNED** — @coder-mcp K1 bake + @coder K3 RON merge |
| ☐ | **Q tail** | coder + operator | BQ-Q2/Q3 (Q1 APS strip ☑ · `apsr_a4_q1_001_live.json`) |
| ☐ | **APSR-P/D/Q** | coder-mcp | Panel split · design lint · QC surfaces — **P/D/Q closed** |

---

### Stream 5 — Schedule sync Wave 1 `@coder` (P1)

| ☐ | ID | Exit |
|:---:|:---|:---|
| ☑ | **SCH-W1-E1-001** | Ambiguity warn · `sch_w1_e1_001_live.json` |
| ☑ | **SCH-W1-T1-001** | Pause `dt_scale()` witness · `sch_w1_t1_001_live.json` |
| ☑ | **SCH-W1-E3/E4** | Fire BuildProfiles edge · HybridSim inject edge |

Plan: [`plan_schedule_sync_v1.md`](plan_schedule_sync_v1.md) · picks: [`coder_todos_v1.md`](coder_todos_v1.md)

---

## Deferred / do not pick

| ID | Note |
|:---|:---|
| COD-SIM-HUD-* | **Closed** — `debug_runs/sim_hud_phase2_close_live.json` green; next sim HUD = Bevy migration, not more egui |
| BUILD-READ-REWIRE-004 | Pilot lint transitional |
| PLAN-CLEANUP Phase 2+ | **DR-CLEANUP-P2** — see plan_deferral_registry_v1.md |

---

## Blocked on designer / planner (not on active 4 streams)

| ID | Blocker |
|:---|:---|
| TRIAGE-BUILD-CLICK-PLACE-001 · TRIAGE-CURSOR-UNIFY-001 | Design signed — reconcile queue |
| P0-VFX-ZOOM-LOCK-001 · P0-TERRAIN-BLOB-001 | Planner/product gate |
| APSR-T3 | Designer spec consolidation |

## Downtime lanes

| ID | Owner | Note |
|:---|:---|:---|
| PLAN-CLEANUP Phase 0 | coder_a/b | Hygiene only |
| PERF-GPU-TERRAIN-* | coder | After Stream 1 baseline |
| VEG drain seq 2→82 | coder_a | When primary idle |

---

## Explicitly excluded (migration / mitigation lane)

Do **not** assign these to non-MIG coder sessions:

| Code | Owner | Plan |
|:---|:---|:---|
| MIG-P0-G1-001 | sim-steward | Ecosystem compat matrix (`bevy_egui` gate) |
| MIG-P0-G2-001 | coder | 0.18 baseline witnesses + lockfile snapshot |
| MIG-P1-M1…M9 | — | **SHIPPED** (MIG-V1 green) — do not pick |
| MIG-R1…R6 | — | **SHIPPED** — do not pick |
| plan_schedule_sync Wave 2+ | — | **DR-SCHED-W2** — Wave 1 unblocked |
| plan_cleanup Phase 2+ | — | **DR-CLEANUP-P2** — Phase 0 unblocked |

**Bevy 0.19 migration (2026-07-03):** **PROGRAM CLOSED on master** · `mig_a_program_close.json` · **Do not pick MIG-*** — use [`cross_front_pick_queue_v1.md`](cross_front_pick_queue_v1.md) for product lanes.

---

## Regression bundle (non-MIG session exit)

```powershell
cargo test -p proc_A_dine01 --lib block_recipe city_g1 construction stage5
cd tools/mcp/python && python -m pytest tests/test_building_quality_bq_c1.py tests/test_aps_mutation_inventory.py tests/test_aps_panel_sync_characterization.py -q
cargo orchestrate
```
