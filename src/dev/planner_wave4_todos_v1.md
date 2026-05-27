# Planner wave 4 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` — **12/12 CLOSED** |
| **Date** | 2026-05-26 |
| **Trigger** | Coder dual-queue closure — [`coder_fleet_return_recap_wave3_v1.md`](coder_fleet_return_recap_wave3_v1.md) |
| **Coder backlog** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) |
| **Workboard** | [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/planner_active_queue.json`](../tools/orchestrator/queues/planner_active_queue.json) |

**Rule:** Planner delivers **docs only** (plans, specs, acceptance matrices). No Rust.

**Regression after ledger refresh:** `cargo orchestrate --plan-slice --skip-cargo` optional.

---

## P1 — pick one (unblocks coders)

| ☐ | # | Queue ID | Deliverable | Unblocks coder |
|:---:|:---:|:---|:---|:---|
| ☑ | 1 | **PLAN-F7-A-EXIT-001** | [`fire7_f7_a_exit_acceptance_v1.md`](fire7_f7_a_exit_acceptance_v1.md) — A1–A5 checklist + witness field names + test list | **FIRE7-F7-A-EXIT-001** (A #1) |
| ☑ | 2 | **PLAN-F7-B-STREAM-001** | [`fire7_f7_b_streaming_impl_plan_v1.md`](fire7_f7_b_streaming_impl_plan_v1.md) — B1–B4 signoff | **FIRE7-F7-B-001** ☑ coded |
| ☑ | 3 | **PLAN-F7-C-LOD-001** | [`fire7_f7_c_lod_impl_plan_v1.md`](fire7_f7_c_lod_impl_plan_v1.md) — C1–C3 signoff | **FIRE7-F7-C-001** ☑ coded |
| ☑ | 4 | **PLAN-CONSTRUCTION-MV-001** | [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) — MV ghost fields + sim writer contract | **CONSTRUCTION-MV-SIM-001** **CLOSED** |
| ☑ | 5 | **PLAN-IND-E02-PLAY-001** | [`ind_e02_default_play_spec_v1.md`](ind_e02_default_play_spec_v1.md) — default writer vs seed; exit JSON | **IND-E02-DEFAULT-PLAY-001** **CLOSED** |

---

## P2 — parallel / follow-on

| ☐ | # | Queue ID | Deliverable | Unblocks coder |
|:---:|:---:|:---|:---|:---|
| ☑ | 6 | **PLAN-LOG-E01-VISUAL-001** | [`log_e01_visual_acceptance_v1.md`](log_e01_visual_acceptance_v1.md) — visual vs lib fixture distinction | **LOG-E01-VISUAL-CONFIRM-001** (B #4) |
| ☑ | 7 | **PLAN-VISUAL-RUN-GATE-001** | [`visual_run_acceptance_matrix_v1.md`](visual_run_acceptance_matrix_v1.md) — VR-01…10 + lib vs visual ownership | **VFX-VISUAL-SIGNOFF-001**, **UI-WP-VISUAL-001** |
| ☑ | 8 | **PLAN-M3-MINMAP-001** | [`minimap_m3_units_replay_impl_plan_v1.md`](minimap_m3_units_replay_impl_plan_v1.md) — units + replay; witness CLOSED / product PARTIAL | **UI-P3-M3-UNITS-001**, **UI-P3-M3-REPLAY-001** |
| ☑ | 9 | **PLAN-REPLAY-PARITY-001** | [`replay_editor_parity_impl_plan_v1.md`](replay_editor_parity_impl_plan_v1.md) — lib CLOSED / live ring PARTIAL | **REPLAY-PARITY-001** (B #7) |
| ☑ | 10 | **PLAN-S7B-M4-SIM-001** | [`s7b_m4_sim_playtest_spec_v1.md`](s7b_m4_sim_playtest_spec_v1.md) — M4 play witness CLOSED | **S7B-M4-SIM-001** (A #8) |
| ☑ | 11 | **PLAN-LEDGER-REFRESH-005** | [`planner_status_audit_v7.md`](planner_status_audit_v7.md) · [`plan_ledger_refresh_005_checklist_v1.md`](plan_ledger_refresh_005_checklist_v1.md) | orchestrator / fleet truth |
| ☑ | 12 | **PLAN-PHASE-D-PARITY-001** | [`overlay_parity_stress_plan_v1.md`](overlay_parity_stress_plan_v1.md) — VM-08 baseline CLOSED; stress P2 | **TRIAGE-PHASE-D-PARITY-001** (B #8) |

---

## Blocked — start P2 instead

| Blocked ID | Waits on | Planner start instead |
|:---|:---|:---|
| ~~**PLAN-F7-B/C impl block**~~ | Coder landed wave 3 | **#4** construction MV or **#11** ledger refresh |
| **PLAN-CONSTRUCTION-R4-001** | Product Round 4 board | **#4 PLAN-CONSTRUCTION-MV-001** |

---

## Suggested session order

1. **#1** F7-A exit acceptance (critical path)  
2. **#4** + **#5** construction MV + IND-E02 play (parallel)  
3. **#2** + **#3** F7-B/C impl plans (draft; mark blocked until A exit)  
4. **#7** visual run matrix (unblocks visual sign-off coders)  
5. **#8** + **#9** minimap M3 + replay  
6. **#11** ledger refresh  

---

## Copy-paste — @planner primary

```
@planner — PLAN-F7-A-EXIT-001
Read: fire_sim_phase7_architecture_v1.md § F7-A exit · coder_fleet_return_recap_wave3_v1.md
Deliver: src/dev/fire7_f7_a_exit_acceptance_v1.md
Include: A1–A5 table, exact JSON paths, cargo test list, anti-patterns (no F7-B stub JSON)
Handoff: update tools/orchestrator/queues/coder_active_queue.json notes for FIRE7-F7-A-EXIT-001
Do NOT: Rust
```

---

## Already done (do not re-plan)

FIRE7-PLAN-001 · S7B closure · LOG-E01 impl plan · IND board reconcile · UI shell/P4/P5/M3 compositor · WC-D04 · VM-09 v2 plan · fire_spark/water closure plans · BQ-128 · Phase 6 multiview **plan** (coder needs sim spec #4)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-26 | **12/12 CLOSED** — PLAN-LEDGER-REFRESH-005 |
| v1.0.0 | 2026-05-26 | Post dual-queue; 12 planner rows for wave 4 |
