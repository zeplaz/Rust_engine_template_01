# PLAN-ELEMENTAL-WAVE2-INDEX-001 — Elemental + WSS navigation index `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-ELEMENTAL-WAVE2-INDEX-001** |
| **Version** | `1.2.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** (navigation only — no Rust) |
| **Audit** | [`planner_status_audit_v16.md`](planner_status_audit_v16.md) |
| **Phase index** | [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) |

**Planner sign-off:** PASS (2026-05-27). Single entry point for operators and coders after wave 3 closure.

---

## Purpose

One rollup doc so teams do not hunt across WSS, fire F2, weather v2, smoke, hydrology, and Hanabi charters. **Witness JSON wins** for done/not-done.

---

## North star

[`wssr_index_v1.md`](wssr_index_v1.md) — World Simulation Spine (L1 substrate → L2 extract → L3 GPU).

---

## Authority stack (read order)

| # | Doc | Role |
|:---:|:---|:---|
| 1 | [`wssr_index_v1.md`](wssr_index_v1.md) | WSS parent index |
| 2 | [`planner_elemental_vfx_domain_charter_v1.md`](planner_elemental_vfx_domain_charter_v1.md) | Fire/weather/water VFX routing |
| 3 | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) | L0–L3 clipmap + zoom bands |
| 4 | [`planner_status_audit_v16.md`](planner_status_audit_v16.md) | Fleet CLOSED vs OPEN |

---

## WSS substrate (L1)

| Lane | Plan | Witness | Status (v16) |
|:---|:---|:---|:---:|
| Chunk slab | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) | `wss_substrate_live.json` | **CLOSED** |
| Dual-write PR-2 | [`plan_wss_slab_pr2_dual_write_v1.md`](plan_wss_slab_pr2_dual_write_v1.md) | `dual_write_shim_enabled` | **CLOSED** |
| Active runtime PR-3 | [`plan_wss_slab_pr3_exec_001_v1.md`](plan_wss_slab_pr3_exec_001_v1.md) | `active_runtime_*` | **CLOSED** |
| Active-chunk policy | [`plan_wss_active_chunk_001_v1.md`](plan_wss_active_chunk_001_v1.md) | `active_runtime_policy_wired` | **CLOSED** |
| PR-4 retire | [`plan_wss_pr4_exec_001_v1.md`](plan_wss_pr4_exec_001_v1.md) | persist + overlay | **CLOSED** |
| PR-5 fixture + smoke prod | hybrid retire + smoke prod plans | `ecs_retire_fixture_green`, `hybrid_ecs_smoke_authoritative: false` | **CLOSED** |
| Atmos clipmap | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) | `wss_atmos_clipmap_001` | **CLOSED** |
| Hydro runtime | [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) | `wss_hydro_runtime_001` | **CLOSED** |
| Post-spine | fleet snapshot / substrate writer | `wss_post_spine_001.green` | **CLOSED** |
| Hydro ↔ construction | [`plan_construction_hydro_coupling_001_v1.md`](plan_construction_hydro_coupling_001_v1.md) | `construction_hydro_coupling_wired` | **CLOSED** |
| Deformation slab | PHASE-NEXT P3 optional | — | **OPEN** (depth) |

---

## Fire + streaming (extract / F7)

| Lane | Plan | Witness | Status |
|:---|:---|:---|:---:|
| F2 extract | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) | `f2_extract_witness.green` | **OPEN** (PHASE-NEXT P3) |
| F7 streaming | [`plan_f7_stream_exec_001_v1.md`](plan_f7_stream_exec_001_v1.md) | `fire_streaming_live.json` | **CLOSED** |
| Visual perf | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) | `perf_attribution_60s.md`, stage5 | **PARTIAL** (PHASE-NEXT P0–P1) |
| Witness containment | [`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md) | containment script | **PARTIAL** (PHASE-NEXT P2) |

---

## Construction + Wave S

| Lane | Plan | Witness | Status |
|:---|:---|:---|:---:|
| Parametric | archived exec — **do not reopen** | `construction_parametric_placement_001` | **CLOSED** |
| R4 corridor/MV | archived exec — **do not reopen** | `construction_r4_*` | **CLOSED** |
| BQ-128 apply-001 | [`plan_bq128_apply_exec_001_v1.md`](plan_bq128_apply_exec_001_v1.md) | `construction_bq128_apply_ghost_001` | **CLOSED** |
| BQ-128 apply-002 | same + wave S | `construction_bq128_apply_merge_replace_002` | **CLOSED** |
| R4 product board | [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) | `product_board_open` | **OPEN** (policy SIGNED) |

---

## Stage 7 behavioral

| Lane | Plan | Witness | Status |
|:---|:---|:---|:---:|
| M3 + steward + M4 play | [`plan_stage7_m4_play_001_v1.md`](plan_stage7_m4_play_001_v1.md) | `stage7_behavioral_live.json` | **CLOSED** |

---

## Hanabi (L3 only)

| Lane | Plan | Witness | Status |
|:---|:---|:---|:---:|
| Designer bounds | [`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md) | design PASS | **SIGNED** |
| Adoption charter | [`plan_hanabi_adoption_v1.md`](plan_hanabi_adoption_v1.md) | `experiments/.../report_v1.md` | **QUALIFIED CLOSED** |
| H-A2 exec | [`plan_hanabi_h_a2_exec_001_v1.md`](plan_hanabi_h_a2_exec_001_v1.md) | `hanabi_l3_plugin_wired: false` | **POLICY CLOSED** (feature-only) |

---

## Operator

| Doc | Use |
|:---|:---|
| [`plan_ops_witness_cadence_001_v1.md`](plan_ops_witness_cadence_001_v1.md) | When to refresh proofs |
| [`operator_visual_signoff_bundle_plan_v1.md`](operator_visual_signoff_bundle_plan_v1.md) | Optional `--test visual` |

---

## Do not re-plan (hard)

Archived exec in `planner_active_queue.json` `done`: parametric, R4 impl, M3, replay, hydro coupling, PR-3/PR-4 exec — see audit v16. **Do not reopen.**

Active execution: [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) · coder detail [`fleet_wave7_coder_dispatch_v1.md`](fleet_wave7_coder_dispatch_v1.md).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Initial navigation index |
| v1.2.0 | 2026-05-28 | PHASE-NEXT routing; v16 audit; WSS/S7 CLOSED on disk |
| v1.1.0 | 2026-05-27 | PR-5 fixture + BQ-128 **CLOSED**; smoke prod + H-A2 exec rows; audit v14 |

---

## Verification (navigation sanity)

```powershell
python tools/orchestrator/scripts/refresh_008_sync.py
```
