# Stage tracks — execution index `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.6.0` |
| **Date** | 2026-05-26 (orchestrator sync) |
| **Snapshot** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) |
| **Audit** | [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) |
| **Owner** | `@orchestrator` / `@planner` |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) ← **truth table** |
| **Designer board** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Coder board** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) |
| **Planner board** | [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Designer queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../../tools/orchestrator/queues/designer_active_queue.json) |
| **Coder hub** | [`coder_execution_plan_v1.md`](coder_execution_plan_v1.md) |

**Rule:** One **primary track** per cycle. One **secondary** infra or witness row allowed. Witness JSON wins over markdown checkboxes.

---

## Closed gates (do not reopen for feature work)

| Gate | Doc |
|:---|:---|
| Stage 5 FULL_APP | [`stage5_operational_signoff.md`](stage5_operational_signoff.md) |
| Stage 6 virtualization | [`stage6_operational_signoff.md`](stage6_operational_signoff.md) |
| UI Phase 2 + Phase 3 M1–M2 | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| Construction operational | [`construction_invariants.md`](construction_invariants.md) |

---

## Active tracks (plans + agents)

| Track | Plan | Primary agent | Designer? | First slice |
|:---|:---|:---|:---:|:---|
| **VFX Phase 2 closure** (fire + shared proof) | [`stages/vfx_phase2_closure_plan_v1.md`](stages/vfx_phase2_closure_plan_v1.md) · triage [`vfx_triage_v1.md`](vfx_triage_v1.md) | — | **CLOSED** | maintain regression only |
| **Water VFX closure** | [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) · [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) | — | **CLOSED** | maintain regression only |
| **UI Phase 4** | [`stages/ui_phase4_execution_plan_v1.md`](stages/ui_phase4_execution_plan_v1.md) · handoff [`ui_phase4_handoff_plan_v1.md`](../prompts/guides/ui/ui_phase4_handoff_plan_v1.md) | `@coder` | **D-WP** + [`world_preview_product_full_plan_v1.md`](../prompts/guides/ui/world_preview_product_full_plan_v1.md) | optional D-02 · WP-L3/L4 |
| **UI Phase 3 minimap** | compositor + M3/M4 plans | — | **CLOSED** | optional units/replay |
| **Infra 5.5+** | [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) · [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) | operator | VM-09 + PROJ2 + WC-D04 **CLOSED** | **OPS-F01** · TRIAGE-VM-09-v2 deferred |
| **Wave C depth** | [`stages/wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md) · [`post_stage6_infra_wave_c_plan_v1.md`](post_stage6_infra_wave_c_plan_v1.md) | operator | — | **WC-DEPTH-001** + **WC-D04** **done** |
| **Fire sim Phase 7** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) | — | **FIRE7-DESIGN-001 SIGNED** | **CLOSED** — F7-A/B/C witness green |
| **Stage 7 Play** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) | — | **CLOSED** | maintain witnesses |
| **Stage 7 Behavioral** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) | — | **S7B-DESIGN SIGNED** | **M1–M3 CLOSED** · M4 play tail optional |

---

## Recommended 6-cycle rhythm

| Cycle | Primary track | Secondary | Milestone |
|:---:|:---|:---|:---|
| — | **UI shell 2B** | [`ui_phase2b_gate_plan_v1.md`](../prompts/guides/ui/ui_phase2b_gate_plan_v1.md) · [`ui_p2b_coder_b_numbered_tasks_v1.md`](ui_p2b_coder_b_numbered_tasks_v1.md) | — | **CLOSED** | **UI-P2B-CODER-B** tasks 1–6 done |
| 2 | **Fire VFX** tune | Operator PNGs | F-T01…T03 |
| 3 | **UI Phase 4** | — | D-04 **DONE**; next D-07 / optional D-02 |
| 4 | **S7P-DESIGN-001** | — | Stage 7 Play designer SIGNED |
| 5 | **Infra 5.5+** | PERF 60s | VM-09 (S-VM-09 code done; witness refresh) |
| 6 | **Wave C** | — | depth + churn |
| 7+ | **Behavioral / Fire P7** | — | gated |

**Done (audit 2026-05-25):** S7-PLAY CLOSED · VFX-P2 + FX-WATER tactical witness · UI-P2 shell · UI-P4 LAYOUT-002 · UI-P3 M1–M4 · BQ-128-APPLY · WC-DEPTH · WC-D04 · INFRA-PROJ2 · planner batch 12/12

**Active (one primary per cycle):** **LOG-E01-VISUAL-CONFIRM** · **INFRA-VM-DEEP** · **OPS-F01/F03** (operator) — see [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) § Open

---

## Global commands (every coder slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo orchestrate --skip-cargo
```

Product / render witness refresh:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Agent routing

| Agent | Read first |
|:---|:---|
| **@coder** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) → track plan |
| **@designer** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **@orchestrator** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) · [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) |
| **@planner** | [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) |
| **@sim-steward** | Infra + Fire7 preflight sections |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.5.0 | 2026-05-25 | Orchestrator sync — active = S7B-PLAN-001; closed BQ-128/WC/PROJ2/VFX tracks |
| v1.4.0 | 2026-05-25 | Ledger refresh audit — tactical VFX/water/S7/UI-P4 signed |
| v1.3.0 | 2026-05-24 | Six PLAN deliverables + planner workboard + infra/wave execution plan |
| v1.2.0 | 2026-05-24 | Sign-off ledger + designer/coder workboards; audit 2026-05-24 |
| v1.1.0 | 2026-05-24 | Added **FX-WATER** dedicated closure track (not done) |
| v1.0.0 | 2026-05-24 | Initial seven-track execution index |
