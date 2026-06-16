# Stage tracks — sign-off ledger `v1`



| Field | Value |

|:---|:---|

| **Version** | `1.2.6` |

| **Date** | 2026-05-26 (**ORCH-SIGNOFF-20260526**) |

| **Snapshot** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) |

| **Authority** | Witness JSON wins over markdown checkboxes |

| **Hub** | [`stage_tracks_execution_index_v1.md`](stage_tracks_execution_index_v1.md) |

| **Planner board** | [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) |

| **Planner batch** | [`planner_queue_todos_v1.md`](planner_queue_todos_v1.md) **CLOSED** |
| **Last audit** | [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) v5 (**PLAN-LEDGER-REFRESH-003**) |

| **Refresh** | [`stage_tracks_ledger_refresh_runbook_v1.md`](stage_tracks_ledger_refresh_runbook_v1.md) (**PLAN-LEDGER-REFRESH**) |



**Legend:** **CLOSED** = exit criteria met · **SIGNED** = designer review recorded · **DONE** = coder slice landed · **OPEN** = active work · **STALE** = proof JSON out of date vs code · **PARTIAL** = slice landed but witness gap remains



---



## Executive summary (2026-05-26)

**Orchestrator snapshot:** [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md)

| Area | Verdict | Next |
|:---|:---|:---|
| **Stage 5 / 6 gates** | **CLOSED** | `cargo test -p proc_A_dine01 --lib stage5` |
| **Steward preflights** | **ALL CLOSED** | Regression only — see snapshot § Steward |
| **Dual-queue + wave 3** | **CLOSED** | `coder_a_wave3` · `coder_b_wave3_bundle_001` |
| **Fire P7 (F7-A/B/C)** | **CODER CLOSED** | `fire7_f7_a_exit_001` + `fire_streaming_live.json` green |
| **Stage 7 Behavioral** | **M1–M3 CLOSED** | Qualified tail: `s7b_m4_play_green` optional sim |
| **VFX / Water tactical** | **WITNESS CLOSED** | Operator PNG / `--test visual` optional |
| **Industrial / S7 Play** | **CLOSED** | maintain witnesses |
| **UI shell 2B / P3 minimap** | **CLOSED** | `phase2b_closed` · compositor M1–M4 |
| **UI Phase 4 / 5** | **CODER CLOSED** (P5 save deferred) | WP-L3/L4 optional polish |
| **Infra / Wave C** | **WC-D04 CLOSED** | **OPS-F01** / **INFRA-VM-DEEP** P2 tails |



---



## Closed gates (do not reopen)



| ID | Milestone | Evidence | Status |

|:---|:---|:---|:---:|

| **G-S5** | Stage 5 FULL_APP operational | `stage5_full_app_live.json` → `readiness.passes: true` | **CLOSED** |

| **G-S6** | Stage 6 virtualization | `stage6_operational_signoff.md` | **CLOSED** |

| **G-CON-OP** | Construction operational | `construction_stage_live.json` → `operational_green: true` | **CLOSED** |

| **G-WAVE-S** | Wave S save spine code | `wave_s_open.md` | **CLOSED** (code) |

| **G-S7P** | Stage 7 Play product slice | `stage7_play_live.json` → `production_green: true` | **CLOSED** (2026-05-25) |

| **G-UI-P2B** | Simulation shell Phase 2B | `egui_pass_count_in_sim: 0` + `phase2b_closed: true` + witness gates | **CLOSED** — **UI-P2B-CODER-B** |



---



## Track sign-off matrix



### Stage 7 Play (`S7-PLAY`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **S7P-IND-001** | coder | **DONE** | `stage7_play_live.json` → `activation_green: true` |

| **S7P-DESIGN-001** | designer | **SIGNED** | [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) |

| **S7P-STEWARD-001** | sim-steward | **DONE** | `stage7_play_live.json` writer + seed env |

| **CON P9** | coder | **DONE** | `construction_stage_live.json` → `con_e01_p9_green: true` |

| **Track exit** | — | **CLOSED** | Scenario signed + play witness green |



### VFX Phase 2 — fire (`VFX-P2`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **FX-FIRE-SPARK-001…006** | coder | **DONE** | queue + code landed |

| **P2-VFX-VISUAL-001** | coder | **DONE** | `fire_spark_rows: 308`, tactical `all_green: true` |

| **P2-VFX-WITNESS-001** | coder | **DONE** | lib tests + harness `all_green` |

| **P2-FIRE-SPARK-010** | coder | **DONE** | `fire_sparks_above_smoke: true` |

| **P2-FIRE-SPARK-011** | coder | **DONE** | tune @ `zoom_alpha: 0.85` |

| **D-VFX** (`VFX-POST-REVIEW`) | designer | **SIGNED — PASS** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) v1.1 |

| **VFX-CAPTURE-001** | designer | **DONE** | [`vfx_capture_status_20260525.md`](../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) |

| **Track exit** | — | **CLOSED** | witness + designer PASS |
| **PLAN-FIRE-VFX-CLOSURE-001** | planner | **DONE** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) — do not re-queue FX-FIRE-SPARK / P2-FIRE |



### Water VFX (`FX-WATER`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **FX-WATER-SHADER-001/002** | coder | **DONE** | `water_w1_green: true` |

| **FX-WATER-PARTICLE-001/002** | coder | **DONE** | tactical `water_particle_rows: 218` |

| **WATER-W1-OCEAN-001** | coder | **DONE** | `water_ocean_tiles: 1715` |

| **WATER-W1-RIVER-001** | coder | **DONE** | `water_w1_river_read_green: true` |

| **WATER-W2-FOAM-001** | coder | **DONE** | tactical `coast_foam: 128`, `river_foam: 2` |

| **WATER-STRATEGIC-001** | coder | **DONE** | `water_strategic_001_green: true` (strategic cull expected) |

| **WATER-DESIGN-001** | designer | **SIGNED — PASS** | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) v1.1 |

| **WATER-DESIGN-002** | designer | **DONE** | [`water_ocean_fixture_request_v1.md`](water_ocean_fixture_request_v1.md) |

| **STEWARD-WATER-WITNESS-001** | sim-steward | **PASS** | [`steward_water_witness_gate_v1.md`](steward_water_witness_gate_v1.md) |

| **PLAN-WATER-TRACK-001** | planner | **DONE** | [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) v2 — **closure sign-off only**; do not re-queue foam |

| **Track exit** | — | **CLOSED** | W2 + strategic + steward PASS + designer PASS |



### UI Phase 2 shell (`UI-P2`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **UI-P2B-001 / 2A / 2C** | coder | **DONE** | historical proofs + lib tests |

| **UI-P2-GATE** | sim-steward | **DONE** | CONDITIONAL historical |

| **ui_shell_migration_live.json** | operator | **CURRENT** | `phase2b_closed: true`, `phase2_zones_live: true` |

| **UI-SHELL-REFRESH-001** | sim-steward | **DONE** | PASS proof-only (2026-05-24) |

| **UI-P2A-CODER-B** | coder | **DONE** | `ui_p2a_coder_b_green` in shell witness |



### UI Phase 3 minimap (`UI-P3`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **UI-P3-M1/M2/001** | coder | **DONE** | `composite_ok`, `logistics_rows: 2`, `construction_rows: 18` |

| **Plan v1** (M1 spine) | planner | **APPROVED** v2 | [`ui_phase3_minimap_compositor_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_compositor_plan_v1.md) |
| **PLAN-UI-P3-COMPOSITOR-001** | planner | **DONE** | [`ui_phase3_minimap_compositor_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_compositor_full_plan_v1.md) — M1+M2+M3 rollup |

| **UI-P3-M2-PLAN** | planner | **DONE** | overlay plan + D-MINIMAP-M2 |
| **PLAN-UI-P3-M2-IMPL-001** | planner | **DONE** | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_m2_impl_full_plan_v1.md) — unblocks **UI-P3-M2-CODER-A** |

| **UI-P3-M2-TRAY-OPT** | coder | **DONE** | `dock_shell` tray → `MinimapOverlayMask` (witness refresh optional) |

| **UI-P3-M2-CODER-A** / **UI-P3-M3-001** (code name) | coder | **DONE** | `seed_minimap_m2_overlay_witness` + witness `ui_p3_m2_green` / `ui_p3_m3_green` |

| **D-MINIMAP-M3** | design **SIGNED** / FoW+EW **DONE** | [`minimap_d_m3_signoff_v1.md`](minimap_d_m3_signoff_v1.md) — **UI-P3-M4-001** FoW/EW; units/replay open |



### UI Phase 4 (`UI-P4`)



| Slice / gate | Agent | Status | Evidence |

|:---|:---|:---|:---|

| **PLAN-WP-DECISION-001** | planner | **DONE** | [`world_preview_product_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_product_full_plan_v1.md) — unblocks **DESIGN-D-WP-REVIEW** |

| **UI-WP-DESIGN** | designer | **SIGNED** | layout + D-04 spec |

| **UI-P4-PLAN** | planner | **DONE** | handoff plan |
| **PLAN-UI-P4-ATLAS-001** | planner | **DONE** | [`ui_phase4_icon_atlas_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md) — icons + petroleum tab |
| **P4-P5-01** | coder | **DONE** | `phase4.p5_br_tab_wired` · `IconId::P5Br` on petroleum tab |
| **P4-ART-01** / **P4-VEH-01** | design / coder | **ART DONE** · vehicles **OPEN** | **UI-OH-P4-ART-001** · vehicle row consumers |

| **UI-WP-LAYOUT-001** | coder | **DONE** | unified workspace tests |

| **UI4-DESIGN-001** | designer | **SIGNED** | [`world_preview_d04_slide_sheet_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_d04_slide_sheet_spec_v1.md) |

| **UI-WP-LAYOUT-002** | coder | **DONE** | D-04 dim + sheet (`window.rs`); lib `wave_p_live_proof` |

| **UI-WP-LAYOUT-D07** | coder | **DONE** | `ui_wp_layout_d07_green: true` in `wave_p_live.json` |

| **Track exit (product)** | — | **CODER CLOSED** | WP-L3/L4 optional |



### UI Phase 5 — pause menu (`UI-P5`)



| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **PLAN-UI-P5-PAUSE-001** | planner | **DONE** | [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) |
| **P5-SCAFFOLD** | coder | **PARTIAL** | egui pause + confirm — `in_game_pause_menu.rs` |
| **UI-P5-DESIGN-001** | designer | **DONE** | [`ui_p5_design_signoff_v1.md`](ui_p5_design_signoff_v1.md) |
| **UI-P5-PAUSE-001** | coder | **DONE** | [`pause_menu_bevy.rs`](../gui/pause_menu_bevy.rs) · `ui_p5_pause_001_green` |
| **S7B-M3-001** | coder | **DONE** | overlay readers → `s7b_m3_green` · `s7b_steward_green` |
| **UI-P5-SAVE-001** | coder | **DEFERRED** | Wire Save/Load to save spine |



### Infra 5.5+ (`INFRA-55`) · Wave C · Fire P7 · Behavioral



| Slice / track | Agent | Status | Note |

|:---|:---|:---|:---|

| **PLAN-INFRA-C-WC** | planner | **DONE** | execution plan v1 |

| **INFRA-55** | coder | **slice 2 CLOSED** | CODER-B + PROJ2 green — [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) |

| **STEWARD-VM-09-001** | sim-steward | **CLOSED** | [`steward_vm09_gate_v1.md`](steward_vm09_gate_v1.md) |

| **WAVE-C** | coder + operator | **PARTIAL CLOSED** | **WC-D04-CODER-B DONE** · **OPS-F01** open |

| **FIRE-P7** | coder | **CLOSED** (2026-05-26) | F7-A-EXIT + F7-B + F7-C — [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) |

| **PLAN-STAGE7-BEHAVIORAL** | planner | **DONE** | handoff v1 |

| **S7-BEHAV** | coder | **M1–M3 CLOSED** | **S7B-PREFLIGHT GO** · qualified `s7b_m4_play_green` tail |



---



## Designer sign-offs (orchestrator registry)

**Registry:** [`designer_signoff_registry.json`](../../tools/orchestrator/queues/designer_signoff_registry.json) · **Todos:** [`stage_designer_todos_v1.md`](stage_designer_todos_v1.md) · **Queue:** [`designer_active_queue.json`](../../tools/orchestrator/queues/designer_active_queue.json)

| @designer ID | Sign-off | Status |
|:---|:---|:---:|
| **UI4-DESIGN-001** | **SIGNED** 2026-05-24 | **DONE** |
| **S7P-DESIGN-001** | **SIGNED** 2026-05-24 | **DONE** |
| **DESIGN-UI-P2-SIGNOFF-001** | **SIGNED** 2026-05-24 | **DONE** |
| **UI-OH-P4-ART-001** | **SIGNED** 2026-05-25 **PASS** | **DONE** — traced atlas |
| **DESIGN-VFX-CAPTURE-001** | **SIGNED** 2026-05-25 **PASS** | **DONE** |
| **WATER-DESIGN-002** | **SIGNED** 2026-05-24 | **DONE** |
| **DESIGN-MINIMAP-M2-001** | **SIGNED** 2026-05-24 **M2 COMPLETE** | **DONE** |
| **DESIGN-D-WP-REVIEW-001** | **SIGNED** 2026-05-25 **PASS** | **DONE** |
| **DESIGN-D-VFX-POST-001** | **SIGNED** 2026-05-25 **PASS** | **DONE** |
| **UX-E02-BQ128-001** | **SIGNED** 2026-05-25 | **DONE** — [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) |
| **PLAN-UX-BQ128-001** | planner | **DONE** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) — unblocks **BQ-128-APPLY-001** |
| **MINIMAP-DESIGN-M3-001** | **SIGNED** 2026-05-25 | **DONE** |
| **S7B-DESIGN-001** | **SIGNED** 2026-05-25 | **DONE** — [`stage7_behavioral_d_signoff_v1.md`](stage7_behavioral_d_signoff_v1.md) |

| Priority | Queue ID | Status |
|:---:|:---|:---:|
| **4** | **UI-P3-M2-TRAY-OPT** | **DONE** |
| **DONE** | **S7B-DESIGN-001** | worksheet **SIGNED** 2026-05-25 — [`stage7_behavioral_d_signoff_v1.md`](stage7_behavioral_d_signoff_v1.md) |
| **DONE** | **S7B-PLAN-001** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) + [`stage7_behavioral_live_witness_spec_v1.md`](stage7_behavioral_live_witness_spec_v1.md) |
| **CLOSED** | **S7B-PREFLIGHT-001** | [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) **GO** |
| **CLOSED** | **S7B-M1/M2/M3** | `stage7_behavioral_live.json` greens |
| **QUALIFIED** | **S7B-M4-PLAY** | `s7b_m4_play_green: false` — optional sim |
| **CLOSED** | **FIRE7-PREFLIGHT-001** | [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) **GO** |
| **CLOSED** | **FIRE7-F7-A-EXIT / B / C** | infra + `fire_streaming_live.json` |



---



## Coder workboard (active only)



| Priority | ID | Track | First action | Status |

|:---:|:---|:---|:---|:---:|

| 1 | **UI-P3-M4-001** | UI-P3 | Design M3 fog/EW — **not** UI-P3-M3-001 | **DONE** (FoW+EW) |

| 2 | **BQ-128-APPLY-001** | WAVE-S | Preset apply-to-ghost | **OPEN** |
| 3 | **WC-DEPTH-001** | WAVE-C | **BQ-101** closed — `wc_depth_001_green` | **DONE** |

| — | **S7P-LOG-001** | S7-PLAY | `logistics_throughput_live.json` green | **DONE** |

| — | **TRIAGE-VM-09-CODER-B** / **STEWARD-VM-09-001** | INFRA-55 | slice 2 | **DONE** |

| — | **UI-P3-M2-CODER-A** / **UI-P3-M3-001** | UI-P3 | M2 construction + ecology witness | **DONE** |
| — | **INFRA-PROJ2-001** / **INFRA-PROJ2-CODER-B** | INFRA-55 | [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) — hit-test + ViewManager sole writer | **DONE** |

| — | **UI-WP-LAYOUT-002** | UI-P4 | — | **DONE** |

| — | **WATER-*** / **P2-FIRE-*** | VFX | — | **DONE** |

| — | **UI-SHELL-REFRESH-001** | UI-P2 | — | **DONE** |



---



## Planning todos (orchestrator queue)



| Queue ID | Deliverable | Status |

|:---|:---|:---|

| **PLAN-WP-DECISION-001** | [`world_preview_product_full_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/world_preview_product_full_plan_v1.md) | **DONE** |

| **UI-P3-M2-PLAN** | `ui_phase3_m2_minimap_overlay_plan_v1.md` | **DONE** |

| **PLAN-UI-P3-M2-IMPL-001** | `ui_phase3_minimap_m2_impl_full_plan_v1.md` | **DONE** |

| **PLAN-UI-P3-COMPOSITOR-001** | `ui_phase3_minimap_compositor_full_plan_v1.md` | **DONE** |

| **PLAN-UI-P4-ATLAS-001** | `ui_phase4_icon_atlas_plan_v1.md` | **DONE** |

| **PLAN-IND-E03-001** | `industrial_grid_overload_impl_plan_v1.md` | **DONE** |

| **PLAN-INFRA-PROJ2-001** | `infra_proj2_sole_writer_plan_v1.md` | **DONE** |

| **UI-P4-PLAN** | `ui_phase4_handoff_plan_v1.md` | **DONE** |

| **PLAN-INFRA-C-WC** | `post_stage6_infra_wave_c_plan_v1.md` | **DONE** |

| **PLAN-STAGE7-BEHAVIORAL** | `stage7_behavioral_planner_handoff_v1.md` | **DONE** |

| **PLAN-STAGE7-BEHAVIORAL-001** | [`stage7_behavioral_full_plan_v1.md`](stage7_behavioral_full_plan_v1.md) | **DONE** |

| **PLAN-UI-SHELL-2B-001** | [`ui_phase2b_gate_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase2b_gate_plan_v1.md) · [`ui_p2b_coder_b_numbered_tasks_v1.md`](ui_p2b_coder_b_numbered_tasks_v1.md) | **DONE** |

| **PLAN-LEDGER-REFRESH** | runbook + [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) | **DONE** (2026-05-25 cycle) |

| **PLAN-WATER-TRACK-001** | [`water_vfx_track_closure_plan_v1.md`](water_vfx_track_closure_plan_v1.md) | **DONE** |

| **PLAN-FIRE-VFX-CLOSURE-001** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) | **DONE** |

| **PLAN-UX-BQ128-001** | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) | **DONE** |

| **PLAN-UI-P5-PAUSE-001** | [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) | **DONE** |

| **PLAN-LEDGER-REFRESH-003** | [`stage_tracks_ledger_refresh_003_plan_v1.md`](stage_tracks_ledger_refresh_003_plan_v1.md) | **DONE** |

| **PLAN-WAVE-P-WITNESS-SPEC-001** | [`wave_p_witness_spec_v1.md`](wave_p_witness_spec_v1.md) | **DONE** |

| **PLAN-UI-SHELL-WITNESS-SPEC-001** | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) | **DONE** |

| **PLAN-LOGISTICS-PROJECTION-001** | [`logistics_projection_impl_plan_v1.md`](logistics_projection_impl_plan_v1.md) | **DONE** |

| **PLAN-IND-BOARD-RECONCILE-001** | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) | **DONE** |

| **PLAN-INFRA-SLICE2-001** | Hub [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) v2 | **SPLIT** |



---



## Witness refresh commands



```powershell

# Spine + VFX + industrial

cargo test -p proc_A_dine01 --lib stage5

cargo run -p proc_A_dine01 --release -- --test visual



# Industrial board (refresh stale standalone JSON)

cargo run -p proc_A_dine01 --release

# optional: $env:RUST_ENGINE_STAGE7_PLAY_SEED=1



# UI shell (interaction replay if extending witness)

cargo run -p proc_A_dine01 --release

```



---



## Changelog



| Version | Date | Notes |

|:---|:---|:---|

| v1.0.0 | 2026-05-24 | Initial reconciliation vs debug_runs + review records |

| v1.1.0 | 2026-05-24 | Six PLAN todos delivered; world preview product SIGNED |

| v1.2.5 | 2026-05-25 | **PLAN-LEDGER-REFRESH-003** — planner batch 12; witness specs; IND-E01/E02/E03 reconcile |
| v1.2.4 | 2026-05-25 | **PLAN-LEDGER-REFRESH-002** — witness↔done; stage6/wc_d04 CURRENT; queue hygiene |
| v1.2.3 | 2026-05-25 | **PLAN-LEDGER-REFRESH-001** urgent — queue restored; 28× stage5; shell/stage6 STALE policy |
| v1.2.2 | 2026-05-25 | Fleet truth: D-07 green; logistics/minimap current |
| v1.2.1 | 2026-05-25 | Designer batch **DONE**; VFX **PASS**; designer registry expanded |
| v1.2.0 | 2026-05-25 | First audit cycle |


