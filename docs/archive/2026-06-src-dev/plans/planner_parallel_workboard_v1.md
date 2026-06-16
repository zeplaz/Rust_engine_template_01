# Planner parallel workboard `v1` (while coders execute)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-PARALLEL-WAVE-001** |
| **Version** | `1.5.0` |
| **Date** | 2026-05-27 |
| **Coder matrix** | [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) |
| **Rule** | **Docs only** — do not block active coder PRs; prep **next** exec plans |

**Parallel with coders:** Planners expand specs **ahead** of witness checkpoints so coders never wait on empty stubs.

> **Clarity note (2026-05-27):** References to `B-C*` cycles are historical sequencing context.  
> Parametric lane is closed; live planner assignments are in `planner_active_queue.json` `active` + `next_phase`.

---

## P0 — unblocks coders in cycles 1–4 (do first)

| ☐ | ID | Deliverable | Unblocks coder | Coder cycle |
|:---:|:---|:---|:---|:---:|
| ☑ | **PLAN-FIRE-F2-EXEC-001** | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) | **A-V2** FIRE-F2-EXTRACT | 3–4 |
| ☑ | **PLAN-WSS-SMOKE-BRIDGE-001** | [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) | **A-V3** / A-W4 | 5–6 |
| ☑ | **PLAN-CONSTRUCTION-PARAM-P3P4-001** | [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md) § P3/P4 expanded | **B-C4**, **B-C5** | 5–7 |

**PLAN-FIRE-F2-EXEC-001 must include:** authority map, VX-P2-01 acceptance, ≤3 files/PR, witness schema, hybrid overlay_bootstrap retirement plan.

---

## P1 — WSS spine follow-ons (prep while A-W1/W2 run)

| ☐ | ID | Deliverable | Trigger / parallel |
|:---:|:---|:---|:---|
| ☑ | **PLAN-WSS-SLAB-PR-2-EXEC-001** | [`plan_wss_slab_pr2_dual_write_v1.md`](plan_wss_slab_pr2_dual_write_v1.md) **SIGNED v1.0.0** | checkpoint met (`wss_chunk_slab_001.green`) |
| ☑ | **PLAN-WSS-SLAB-PR-3-EXEC-001** | [`plan_wss_slab_pr3_exec_001_v1.md`](plan_wss_slab_pr3_exec_001_v1.md) READY | **WSS-SLAB-PR-3** |
| ☑ | **PLAN-WSS-ACTIVE-CHUNK-001** | [`plan_wss_active_chunk_001_v1.md`](plan_wss_active_chunk_001_v1.md) READY | **WSS-SLAB-PR-3** policy |
| ☑ | **WEATHER-SIM-PLAN-001** | [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md) **SIGNED v1.0.0** | checkpoint met (`wss_atmos_clipmap_001.green`) |

---

## P1 — construction + product (prep while B-C1..C3 run)

| ☐ | ID | Deliverable | Unblocks |
|:---:|:---|:---|:---|
| ☑ | **PLAN-CONSTRUCTION-R4-EXEC-001** | [`plan_construction_r4_exec_001_v1.md`](plan_construction_r4_exec_001_v1.md) READY | **R4-CORRIDOR-001** |
| ☑ | **PLAN-CONSTRUCTION-R4-MV-EXEC-001** | [`plan_construction_r4_mv_exec_001_v1.md`](plan_construction_r4_mv_exec_001_v1.md) READY | **R4-MV-GHOST-001** |
| ☑ | **PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001** | [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) | policy **SIGNED**; board open on disk |
| ☑ | **PLAN-REPLAY-RING-EXEC-001** | [`plan_replay_ring_exec_001_v1.md`](plan_replay_ring_exec_001_v1.md) finalized | **B-P2** fallback |
| ☑ | **PLAN-M3-DEPTH-EXEC-001** | [`plan_m3_depth_exec_001_v1.md`](plan_m3_depth_exec_001_v1.md) finalized | **B-P1** next phase |
| ☑ | **PLAN-CONSTRUCTION-HYDRO-COUPLING-001** | [`plan_construction_hydro_coupling_001_v1.md`](plan_construction_hydro_coupling_001_v1.md) READY | **WSS-HYDRO-BOUNDARY-001** (B-H2) |

---

## P2 — infrastructure & adoption (non-blocking)

| ☐ | ID | Deliverable | Notes |
|:---:|:---|:---|:---|
| ☑ | **PLAN-HANABI-ADOPTION-001** | [`plan_hanabi_adoption_v1.md`](plan_hanabi_adoption_v1.md) READY | **H-A-SPIKE-001** |
| ☑ | **PLAN-ELEMENTAL-WAVE2-INDEX-001** | [`plan_elemental_wave2_index_001_v1.md`](plan_elemental_wave2_index_001_v1.md) | Nav index |
| ☑ | **PLAN-LEDGER-REFRESH-007** | [`planner_status_audit_v9.md`](planner_status_audit_v9.md) | audit v9 **SIGNED** |
| ☑ | **PLAN-OPS-WITNESS-CADENCE-001** | [`plan_ops_witness_cadence_001_v1.md`](plan_ops_witness_cadence_001_v1.md) READY | operator cadence |
| ☑ | **PLAN-LEDGER-REFRESH-008** | [`planner_status_audit_v10.md`](planner_status_audit_v10.md) | audit v10 **SIGNED** |
| ☑ | **PLAN-WSS-HYBRID-RETIRE-PR4-001** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) READY | PR-4/PR-5 |
| ☑ | **PLAN-BQ128-APPLY-EXEC-001** | [`plan_bq128_apply_exec_001_v1.md`](plan_bq128_apply_exec_001_v1.md) READY | BQ-128-APPLY-001 |
| ☑ | **PLAN-STAGE7-M3-STEWARD-001** | [`plan_stage7_m3_steward_001_v1.md`](plan_stage7_m3_steward_001_v1.md) READY | **S7B-M3-STEWARD-REMEDY-001** (@coder B) |

**Archived (2026-05-27):** PLAN-CONSTRUCTION-R4-EXEC-001 · PLAN-CONSTRUCTION-R4-MV-EXEC-001 · PLAN-M3-DEPTH-EXEC-001 · PLAN-REPLAY-RING-EXEC-001

---

## Do not re-plan (signed — reference only)

WSS-PLAN-001..004 · PLAN-CONSTRUCTION-PARAM-001 · FIRE7-PLAN-001 · elemental charter · plan_wss_chunk_slab_exec (READY)

---

## Suggested planner session order (next phase)

1. **PLAN-WSS-POST-SPINE-001** (deferred)  
2. Coder: [`plan_stage7_m3_steward_001_v1.md`](plan_stage7_m3_steward_001_v1.md) → **S7B-M3-STEWARD-REMEDY-001** (@coder B)  

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Parallel planner wave while multistage coders run |
| v1.1.0 | 2026-05-27 | Signed PR-2 + weather plans; finalized M3 depth/replay exec docs |
| v1.2.0 | 2026-05-27 | Drained active queue: R4 exec + M3 depth + replay ring archived |
| v1.3.0 | 2026-05-27 | P1 prep: hydro coupling + PR-3 exec + ledger audit v9 |
| v1.4.0 | 2026-05-27 | P2 prep: active-chunk criteria + Hanabi charter + ops cadence |
| v1.5.0 | 2026-05-27 | Ledger-008 audit v10 + elemental index + PR-4 retire + BQ128 exec |
