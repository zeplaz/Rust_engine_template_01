# UI overhaul plan — master lane index

| Field | Value |
|:---|:---|
| **Version** | `1.3.0` |
| **Date** | 2026-05-25 |
| **Phase 2+3 closure** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) (**PLAN-UI-OH-CLOSURE-004**) |
| **Live witness rollup** | [`witness_status_live_v1.md`](witness_status_live_v1.md) |
| **Owner** | `@orchestrator` / `ui_layout_agent` |
| **Machine queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Playbook** | [`tools/orchestrator/agents/ui_layout_agent.md`](../../tools/orchestrator/agents/ui_layout_agent.md) |

**Scope:** Simulation HUD shell migration — Bevy panels (P1–P4), PLAY-01 session defaults, egui dedupe, Phase 3 GPU minimap, Phase 4 art.

**Design gates (2026-05-24):** **No blocking gates** for coders. Optional: Phase 4 traced atlas PNG; post-implementation VFX review vs mocks ([`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) § Post-implementation).

**World Preview** (**D-WP**): [`world_preview_d_wp_track_signoff_v1.md`](../prompts/guides/ui/world_preview_d_wp_track_signoff_v1.md) · **DESIGN-D-WP-REVIEW-001 PASS** ([`world_preview_d_wp_review_record_v1.md`](world_preview_d_wp_review_record_v1.md)) · D-01 + D-04 + D-07 **done** · D-02 optional.

**GPU minimap** (**D-MINIMAP-M1/M2** done · **M3** design gate): [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) · [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) · [`minimap_d_m3_signoff_v1.md`](minimap_d_m3_signoff_v1.md) — M3 impl **OPEN**.

---

## Phase status

| Phase | Status | Summary |
|:---|:---|:---|
| **Phase 1** | **CLOSED** | Layout mocks + theme — [`ui_phase0_panel_mocks_v1.md`](../prompts/guides/ui/ui_phase0_panel_mocks_v1.md) |
| **Phase 2 scaffold** | **DONE** | P1–P4 Bevy hosts; PLAY-01; witness writer |
| **Phase 2A** | **CLOSED** | `phase2a_closed` + §1.6 interaction flags green (2026-05-24) |
| **Phase 2B** | **CLOSED** | `phase2b_closed` · `egui_pass_count_in_sim: 0` (2026-05-24) |
| **Phase 2C** | **CLOSED** | **2C-B** dual column — mock § P4 + witness `phase2c` (2026-05-24) |
| **Phase 2 sign-off** | **SIGNED** | [`ui_phase2_designer_signoff_v1.md`](../prompts/guides/ui/ui_phase2_designer_signoff_v1.md) v2.2.0 · **UI-OH-D2-SIGN-001** [`ui_oh_d2_signoff_record_v1.md`](ui_oh_d2_signoff_record_v1.md) |
| **Phase 2+3 closure** | **CLOSED** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) — **PLAN-UI-OH-CLOSURE-004** (2026-05-25) |
| **Sim-steward gate** | **CLOSED** | **UI-P2-GATE** + **UI-OH-GATE-001** PASS (2026-05-25) |
| **Phase 3 M1** | **CLOSED** | `minimap_compositor_live.json` · `composite_ok` · `GpuCompute` path |
| **Phase 3 M1.5** | **CLOSED** | GPU compute compositor; default `MINIMAP_GPU_COMPOSITOR` flip |
| **Phase 3 M2** | **CLOSED** | Logistics heat — `logistics_rows: 2` in `minimap_compositor_live.json` (2026-05-24) |
| **Phase 3 UI-P3-001** | **CLOSED** | GPU minimap sim default · `ui_p3_001_green` witness (2026-05-23) |
| **Phase 4** | **PARTIAL (qualified PASS)** | P4.1 + P5 tab + **UI-OH-P4-ART-001** traced atlas — [`ui_oh_p4_art_signoff_record_v1.md`](ui_oh_p4_art_signoff_record_v1.md) |
| **Phase 5** | **PARTIAL (qualified PASS)** | P5-PAUSE-001 **CLOSED** — [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) · [`ui_phase5_pause_menu_plan_v1.md`](../prompts/guides/ui/ui_phase5_pause_menu_plan_v1.md) |

**Witness profile:** `debug_runs/ui_shell_migration_live.json` → `UI_SHELL_MIGRATION_2B`

**2B gate plan:** [`ui_phase2b_egui_gate_plan_v1.md`](../prompts/guides/ui/ui_phase2b_egui_gate_plan_v1.md) (**PLAN-UI-SHELL-2B-001**)

---

## Handoff table

| Slice ID | Agent | Status | Deliverable |
|:---|:---|:---|:---|
| **UI-P2-GATE** | `@sim-steward` | **done** | CONDITIONAL(shell-only) — spine green |
| **UI-OH-GATE-001** | `@sim-steward` | **done** | 2A/2B + stage5 triage — [`steward_ui_oh_gate_v1.md`](steward_ui_oh_gate_v1.md) **PASS** |
| **UI-P2A-001** | `@coder` | **done** | `phase2a_closed` + §1.6 flags |
| **UI-OH-2A-001** | `@coder` | **done** | alias **UI-P2A-001** — `ui_oh_2a_001.green` + `phase2_zones_live` |
| **UI-P2-DESIGN** | `@designer` | **done** | Sign-off v2.2.0 **SIGNED** |
| **UI-OH-D2-SIGN-001** | `@designer` | **done** | Phase 2 sign-off after **2A** mock parity — [`ui_oh_d2_signoff_record_v1.md`](ui_oh_d2_signoff_record_v1.md) |
| **UI-P2B-001** | `@coder` | **done** | `phase2b_closed` |
| **UI-W3-2B-001** | `@coder B` | **done** | alias 2B — `egui_pass_count_in_sim: 0` + `ui_w3_2b_001.green` |
| **UI-W3-2C-001** | `@coder B` | **done** | **2C-B** mode rail 48px + build rail 52px — `ui_w3_2c_001.green` + `phase2c.phase2c_closed` |
| **@coder A (5)** | `@coder A` | **done** | **2A + M2 + P4 + M3 + theme** — `coder_a_ui_five_lane_001_lib_bundle` |
| **UI-W3-P4-001** | `@coder A` | **done** | Icon atlas + petroleum tab — `ui_w3_p4_001_live_witness_refresh` · `ui_w3_p4_001.green` |
| **UI-W3-M3-001** | `@coder A` | **done** | Stage 7 operational minimap — `ui_w3_m3_001_stage7_operational_witness_refresh` · Track C + `s7b_m3_green` |
| **@coder B (5)** | `@coder B` | **done** | **2B + 2C + P5 + witness + P4** — `coder_b_ui_five_lane_001_lib_bundle` |
| **UI-P3-M1** | `@coder` | **done** | `minimap_compositor_live.json` green |

| **UI-P3-M1.5** | `@coder` | **done** | GpuCompute compositor + default env flip |
| **UI-P2A-F03** | `@coder` | **done** | `ui_p2a_tail.f03_green` + hover replay |
| **UI-P2A-P4-AUTH** | `@coder` | **done** | `ui_p2a_tail.p4_auth_green` + rail authority replay |
| **UI-OH-P5-001** | `@planner` | **done** | Phase 5 qualified PASS — [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) ← **PLAN-UI-P5-PAUSE-001** |
| **UI-W3-P5-001** | `@coder B` | **done** | Wave 3 Bevy pause — `ui_w3_p5_001.green` + `pause_menu_bevy` + `egui_pass_count_in_sim: 0` |
| **UI-W3-WITNESS-001** | `@coder B` | **done** | Lib refresh shell + infra + stage6 + minimap — `coder_b_ui_w3_witness_001_lib_bundle`; operator: `--test visual` |
| **UI-W3-P6-001** | `@coder B` | **done** | Shell perf + multiview — `ui_w3_p6_001.green` · [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) |
| **UI-OH-P4-ART-001** | `@designer` | **done** | Traced atlas — [`ui_oh_p4_art_signoff_record_v1.md`](ui_oh_p4_art_signoff_record_v1.md) |
| **UI-P3-001** | `@coder` | **done** | GPU minimap operationalization — sim default + witness rollup |
| **UI-P3-M3-001** | `@coder` | **done** | **M2** construction + ecology (`ui_p3_m3_green`) — not design M3; see [`ui_phase3_minimap_track_naming_v1.md`](../prompts/guides/ui/ui_phase3_minimap_track_naming_v1.md) |
| **UI-OH-M3-001** | `@planner` | **done** | Phase 3 M2 construction/ecology qualified PASS — [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) ← **PLAN-UI-P3-M3-001** |
| **IND-E01** | `@coder` | **queued** | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) |

**Next agent:** `@coder` **IND-E01** (parallel, disjoint files). Forward: **UI-P3-M4-001** (FoW / multirate polish).

---

## Witness status (2026-05-25 — PLAN-UI-OH-CLOSURE-004)

**Authoritative rollup:** [`witness_status_live_v1.md`](witness_status_live_v1.md) · **Closure gates:** [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md)

**`ui_shell_migration_live.json` refreshed** (2026-05-25): `phase2b_closed: true`, `egui_pass_count_in_sim: 0`, `ui_oh_2a_001/2b_001.green: true` — **UI-OH-GATE-001** **PASS (qualified)**.

**P2A tail (2026-05-25):** `ui_p2a_tail.f03_green` / `p4_auth_green` via replay helpers + `--test visual` harness.

| Field | Value | Fix slice |
|:---|:---|:---|
| `phase2.minimap_gpu_path` | `false` | Shell proof during CPU-path frame; compositor witness has GPU green |

---

## Diagnostics map

Live JSON: `debug_runs/ui_shell_migration_live.json`  
Writer: `write_ui_shell_migration_live_proof_system` · `build_proof_payload`

| JSON path | Meaning | Phase 2 exit |
|:---|:---|:---|
| `phase2a_closed` | zones + tabs + minimap chrome | ✅ |
| `phase2b_closed` | egui gates + pass count 0 | ✅ |
| `phase2_zones_live` | P1 zones live | ✅ |
| `witness.alert_click_expanded_tray` | Alert → tray | ✅ |
| `witness.intel_map_camera_request` | Intel → map | ✅ |
| `witness.escape_collapsed_tray` | Escape collapses tray | ✅ |
| `witness.minimap_chrome_aligned` | P3 ≤2px | ✅ |
| `witness.build_toolbox_egui_gated` | 2B BuildToolbox | ✅ |
| `witness.side_status_rail_egui_gated` | 2B side rail | ✅ |
| `witness.floating_egui_shells_gated` | 2B floating shells | ✅ |
| `egui_pass_count_in_sim` | Must be 0 at 2B exit | ✅ `0` |
| `gpu_minimap_compositor_env` | Env flag at capture | ✅ `true` |

**Phase 3:** `debug_runs/minimap_compositor_live.json`

| Field | Meaning | M1 exit |
|:---|:---|:---|
| `composite_ok` | RT bound + stamp > 0 | ✅ |
| `composite_path` | `GpuCompute` vs `CpuBridge` | ✅ `GpuCompute` |
| `dual_minimap_present` | No egui + Bevy double draw | ✅ `false` |
| `presentation_source` | `SharedRenderTargetImage` | ✅ |
| `logistics_rows` | M2 layer | ✅ `2` |

---

## Phase 3 pointer (active)

| Doc | Role |
|:---|:---|
| [`ui_phase3_minimap_compositor_plan.md`](ui_phase3_minimap_compositor_plan.md) | **M1 authority map + @coder 3.1 file list** |
| [`ux_gpu_minimap_m1_architecture_v1.md`](ux_gpu_minimap_m1_architecture_v1.md) | M1 architecture summary |
| [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) | **Active:** §3.4 M2 logistics |
| [`ux_gpu_minimap_design_v1.md`](ux_gpu_minimap_design_v1.md) | M2/M3 north star |

---

## @coder — UI-P3-M2-001 copy-paste (next primary)

```
Lane: UI Phase 3 — M2 logistics heat (UX-E01)
Read: ui_phase3_coder_queue_v1.md §3.4 + ux_gpu_minimap_design_v1.md §4 M2
Prerequisite: LOG-E01 log_rows≥1 in visual run (code landed)
Do NOT: duplicate logistics extract; read LogisticsVisualSnapshot only
```

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

**Accept:** `minimap_compositor_live.json` → `logistics_rows > 0` when scenario seeded; overlay toggle respected.

---

## @coder — witness tail (optional polish)

```
Lane: UI Phase 2A tail — UI-P2A-F03 + UI-P2A-P4-AUTH
Run sim → hover ops zone → click build rail → refresh ui_shell_migration_live.json
```

---

## Document index

| Doc | Purpose |
|:---|:---|
| [`coder_execution_plan_v1.md`](coder_execution_plan_v1.md) | **@coder master queue** — all active slices |
| [`prompts/guides/ui/README.md`](../prompts/guides/ui/README.md) | UI guides index |
| [`ui_phase2_sprint_queue.md`](ui_phase2_sprint_queue.md) | Phase 2 archive |
| [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) | Phase 3 active queue |
| [`post_stage6_active_todos.md`](post_stage6_active_todos.md) | Product board |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.2.0 | 2026-05-25 | **PLAN-UI-OH-CLOSURE-004** — Phase 2+3 closure plan + witness_status_live_v1 |
| 1.1.1 | 2026-05-24 | Phase **2C** **DEFERRED** — Sprint 2C queue block; designer-first **2C-A/B/C/D** workflow |
| 1.1.0 | 2026-05-24 | Phase 2 **SIGNED**; M1/M1.5 closed; M2 queued; witness tail documented |
| 1.0.1 | 2026-05-24 | UI-P2A-001 + UI-P2-GATE closed |
| 1.0.0 | 2026-05-23 | Initial lane reconcile |
