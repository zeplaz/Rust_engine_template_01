# UI overhaul plan — master lane index

| Field | Value |
|:---|:---|
| **Version** | `1.1.1` |
| **Date** | 2026-05-24 |
| **Owner** | `@orchestrator` / `ui_layout_agent` |
| **Machine queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Playbook** | [`tools/orchestrator/agents/ui_layout_agent.md`](../../tools/orchestrator/agents/ui_layout_agent.md) |

**Scope:** Simulation HUD shell migration — Bevy panels (P1–P4), PLAY-01 session defaults, egui dedupe, Phase 3 GPU minimap, Phase 4 art.

**Design gates (2026-05-24):** **No blocking gates** for coders. Optional: Phase 4 traced atlas PNG; post-implementation VFX review vs mocks ([`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) § Post-implementation).

**World Preview** (**D-WP**): [`world_preview_d_wp_track_signoff_v1.md`](../prompts/guides/ui/world_preview_d_wp_track_signoff_v1.md) · D-01 **done** ([`world_preview_d01_shell_signoff_v1.md`](../prompts/guides/ui/world_preview_d01_shell_signoff_v1.md)) · D-02 optional · **UI-WP-LAYOUT-002** queued.

**GPU minimap** (**D-MINIMAP-M1/M2**): [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) · [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) — **done** 2026-05-24.

---

## Phase status

| Phase | Status | Summary |
|:---|:---|:---|
| **Phase 1** | **CLOSED** | Layout mocks + theme — [`ui_phase0_panel_mocks_v1.md`](../prompts/guides/ui/ui_phase0_panel_mocks_v1.md) |
| **Phase 2 scaffold** | **DONE** | P1–P4 Bevy hosts; PLAY-01; witness writer |
| **Phase 2A** | **CLOSED** | `phase2a_closed` + §1.6 interaction flags green (2026-05-24) |
| **Phase 2B** | **CLOSED** | `phase2b_closed` · `egui_pass_count_in_sim: 0` (2026-05-24) |
| **Phase 2C** | **CLOSED** | **2C-B** dual column — mock § P4 + witness `phase2c` (2026-05-24) |
| **Phase 2 sign-off** | **SIGNED** | [`ui_phase2_designer_signoff_v1.md`](../prompts/guides/ui/ui_phase2_designer_signoff_v1.md) v2.2.0 — P4 **PASS** |
| **Sim-steward gate** | **CLOSED** | **UI-P2-GATE** CONDITIONAL(shell-only) — spine green (2026-05-24) |
| **Phase 3 M1** | **CLOSED** | `minimap_compositor_live.json` · `composite_ok` · `GpuCompute` path |
| **Phase 3 M1.5** | **CLOSED** | GPU compute compositor; default `MINIMAP_GPU_COMPOSITOR` flip |
| **Phase 3 M2** | **CLOSED** | Logistics heat — `logistics_rows: 2` in `minimap_compositor_live.json` (2026-05-24) |
| **Phase 3 UI-P3-001** | **CLOSED** | GPU minimap sim default · `ui_p3_001_green` witness (2026-05-23) |
| **Phase 4** | **PARTIAL** | Atlas code done · **optional** traced PNG · P5/vehicles open — [`ui_phase4_icon_atlas_brief_v1.md`](../prompts/guides/ui/ui_phase4_icon_atlas_brief_v1.md) |

**Witness profile:** `debug_runs/ui_shell_migration_live.json` → `UI_SHELL_MIGRATION_2B`

---

## Handoff table

| Slice ID | Agent | Status | Deliverable |
|:---|:---|:---|:---|
| **UI-P2-GATE** | `@sim-steward` | **done** | CONDITIONAL(shell-only) — spine green |
| **UI-P2A-001** | `@coder` | **done** | `phase2a_closed` + §1.6 flags |
| **UI-P2-DESIGN** | `@designer` | **done** | Sign-off v2.1.1 **SIGNED** |
| **UI-P2B-001** | `@coder` | **done** | `phase2b_closed` |
| **UI-P3-M1** | `@coder` | **done** | `minimap_compositor_live.json` green |

| **UI-P3-M1.5** | `@coder` | **done** | GpuCompute compositor + default env flip |
| **UI-P2A-F03** | `@coder` | **open** | `witness.ops_zone_hover_token: true` (interaction replay) |
| **UI-P2A-P4-AUTH** | `@coder` | **open** | `witness.build_rail_authoritative: true` (rail click replay) |
| **UI-P3-M2-001** | `@coder` | **done** | M2 logistics heat — `logistics_rows > 0` witness green |
| **UI-P3-001** | `@coder` | **done** | GPU minimap operationalization — sim default + witness rollup |
| **UI-P3-M3-001** | `@coder` | **done** | M3 construction + ecology heat on GPU compositor; witness `ui_p3_m3_green` |
| **IND-E01** | `@coder` | **queued** | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) |

**Next agent:** `@coder` **IND-E01** (parallel, disjoint files). Optional polish: **UI-P2A-F03** / **UI-P2A-P4-AUTH**. Forward: **UI-P3-M4-001** (FoW / multirate polish).

---

## Witness status (2026-05-24 audit)

**`ui_shell_migration_live.json` may be STALE** (`phase2b_closed: false` while `egui_pass_count_in_sim: 0`). Treat as **proof refresh** needed — see **UI-SHELL-REFRESH-001** in [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md). Do not assume Phase 2B code regressed without replaying sim interactions.

Historical tail gaps (when witness refreshed):

| Field | Value | Fix slice |
|:---|:---|:---|
| `witness.ops_zone_hover_token` | `false` | **UI-P2A-F03** — hover ops zone in sim |
| `witness.build_rail_authoritative` | `false` | **UI-P2A-P4-AUTH** — click build rail tool |
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
| 1.1.1 | 2026-05-24 | Phase **2C** **DEFERRED** — Sprint 2C queue block; designer-first **2C-A/B/C/D** workflow |
| 1.1.0 | 2026-05-24 | Phase 2 **SIGNED**; M1/M1.5 closed; M2 queued; witness tail documented |
| 1.0.1 | 2026-05-24 | UI-P2A-001 + UI-P2-GATE closed |
| 1.0.0 | 2026-05-23 | Initial lane reconcile |
