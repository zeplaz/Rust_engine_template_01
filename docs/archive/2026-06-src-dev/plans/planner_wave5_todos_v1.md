# Planner wave 5 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` — **8/8 CLOSED** |
| **Date** | 2026-05-26 |
| **Trigger** | Wave 4 **12/12 CLOSED** — [`planner_delivery_signoff_matrix_v1.md`](planner_delivery_signoff_matrix_v1.md) |
| **Prior wave** | [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/planner_active_queue.json`](../tools/orchestrator/queues/planner_active_queue.json) |

**Rule:** Docs only. Wave 5 = **operator tails** + **product depth** + **Round 4 prep** — not re-author wave 4.

---

## P1 — operator / qualified close

| ☐ | # | Queue ID | Deliverable | Unblocks |
|:---:|:---:|:---|:---|:---|
| ☑ | 1 | **PLAN-OPERATOR-VISUAL-BUNDLE-001** | [`operator_visual_signoff_bundle_plan_v1.md`](operator_visual_signoff_bundle_plan_v1.md) | **LOG-E01-VISUAL-CONFIRM**, **VFX-VISUAL**, **UI-WP-VISUAL** |
| ☑ | 2 | **PLAN-S7B-M4-LIVE-001** | [`s7b_m4_live_sim_playtest_plan_v1.md`](s7b_m4_live_sim_playtest_plan_v1.md) | Live sim `play_enqueue_wired` (not lib seed only) |

---

## P2 — product depth (optional)

| ☐ | # | Queue ID | Deliverable | Unblocks |
|:---:|:---:|:---|:---|:---|
| ☑ | 3 | **PLAN-M3-PRODUCT-DEPTH-001** | [`m3_minimap_product_depth_plan_v1.md`](m3_minimap_product_depth_plan_v1.md) | Real unit reader + live replay ring (P2) |
| ☑ | 4 | **PLAN-F7-STREAM-DEEP-001** | [`fire7_streaming_depth_plan_v1.md`](fire7_streaming_depth_plan_v1.md) | Neighbor-wake fixtures + residency tie-in (P2) |
| ☑ | 5 | **PLAN-REPLAY-LIVE-RING-001** | [`replay_live_ring_impl_plan_v1.md`](replay_live_ring_impl_plan_v1.md) | Simulation `CommittedSimReplayRing` commits (P2) |

---

## P3 — product gate prep

| ☐ | # | Queue ID | Deliverable | Unblocks |
|:---:|:---:|:---|:---|:---|
| ☑ | 6 | **PLAN-CONSTRUCTION-R4-001** | [`construction_round4_product_gate_plan_v1.md`](construction_round4_product_gate_plan_v1.md) | **CONSTRUCTION-R4-PREP-001** prep ☑ |
| ☑ | 6b | **R4-PLAN-001** | [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) | **R4-CORRIDOR-001** when board opens |
| ☑ | 6c | **R4-PLAN-002** | [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) | **R4-MV-GHOST-001** when board opens |
| ☑ | 7 | **PLAN-OPS-F01-F03-001** | [`operator_ops_witness_refresh_plan_v1.md`](operator_ops_witness_refresh_plan_v1.md) | OPS-F01 perf · OPS-F03 stage6 sim |
| ☑ | 8 | **PLAN-LEDGER-REFRESH-006** | [`planner_status_audit_v8.md`](planner_status_audit_v8.md) | Fleet closed / tails only |

---

## Do not re-plan (wave 4 closed)

All **PLAN-F7-*** / **PLAN-CONSTRUCTION-MV** / **PLAN-M3-MINMAP** / **PLAN-PHASE-D** wave 4 specs — maintain regression only.

---

## Suggested session order

1. **#1** operator visual bundle (single `--test visual` session)  
2. **#2** S7B M4 live sim playtest spec  
3. **#6** construction R4 when product board opens  
4. **#8** ledger refresh  

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-26 | **8/8 CLOSED** — PLAN-LEDGER-REFRESH-006 |
| v1.0.0 | 2026-05-26 | Wave 5 opened after wave 4 sign-off |
