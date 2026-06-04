# Planner delivery sign-off matrix `v1` (wave 4 + fleet)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-DELIVERY-SIGNOFF-001** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Wave 4 board** | [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) — **12/12** |
| **Audit** | [`planner_status_audit_v10.md`](planner_status_audit_v10.md) |
| **Orchestrator** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) |
| **Next plans** | [`planner_wave5_todos_v1.md`](planner_wave5_todos_v1.md) |

**Authority:** Witness JSON + lib bundles **beat** markdown checkboxes. **STALE** = refresh writer, not reopen planner specs.

---

## Sign-off legend

| Symbol | Meaning |
|:---:|:---|
| **P☑** | Planner spec on disk + **SIGNED** |
| **C☑** | Coder lane closed (lib bundle or witness green) |
| **D☑** | Designer ACCEPT (qualified or full) |
| **O◐** | Operator / live sim tail optional |
| **STALE** | Disk JSON old vs last lib refresh — re-run writer |

---

## Wave 4 — planner deliverables

| # | Planner ID | plan_doc | P | C | D | O | Notes |
|:---:|:---|:---|:---:|:---:|:---:|:---:|:---|
| 1 | **PLAN-F7-A-EXIT-001** | [`fire7_f7_a_exit_acceptance_v1.md`](fire7_f7_a_exit_acceptance_v1.md) | ☑ | ☑ | — | — | `fire7_f7_a_exit_001.green` |
| 2 | **PLAN-F7-B-STREAM-001** | [`fire7_f7_b_streaming_impl_plan_v1.md`](fire7_f7_b_streaming_impl_plan_v1.md) | ☑ | ☑ | — | ◐ | P2 neighbor-wake depth optional |
| 3 | **PLAN-F7-C-LOD-001** | [`fire7_f7_c_lod_impl_plan_v1.md`](fire7_f7_c_lod_impl_plan_v1.md) | ☑ | ☑ | ☑ | — | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) |
| 4 | **PLAN-CONSTRUCTION-MV-001** | [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) | ☑ | ☑ | — | — | `construction_mv_001.green` |
| 5 | **PLAN-IND-E02-PLAY-001** | [`ind_e02_default_play_spec_v1.md`](ind_e02_default_play_spec_v1.md) | ☑ | ☑ | — | — | `concrete_chain_e2e.ind_e02_green` |
| 6 | **PLAN-LOG-E01-VISUAL-001** | [`log_e01_visual_acceptance_v1.md`](log_e01_visual_acceptance_v1.md) | ☑ | ◐ | — | ◐ | `logistics_active_rows: 1` on disk; visual re-run optional |
| 7 | **PLAN-VISUAL-RUN-GATE-001** | [`visual_run_acceptance_matrix_v1.md`](visual_run_acceptance_matrix_v1.md) | ☑ | ◐ | ☑ | ◐ | [`vfx_visual_acceptance_record_v1.md`](vfx_visual_acceptance_record_v1.md) PASS qualified |
| 8 | **PLAN-M3-MINMAP-001** | [`minimap_m3_units_replay_impl_plan_v1.md`](minimap_m3_units_replay_impl_plan_v1.md) | ☑ | ☑ | ☑ | ◐ | Witness green; product readers P2 |
| 9 | **PLAN-REPLAY-PARITY-001** | [`replay_editor_parity_impl_plan_v1.md`](replay_editor_parity_impl_plan_v1.md) | ☑ | ☑ | — | ◐ | Lib `parity_green`; live sim ring P2 |
| 10 | **PLAN-S7B-M4-SIM-001** | [`s7b_m4_sim_playtest_spec_v1.md`](s7b_m4_sim_playtest_spec_v1.md) | ☑ | ☑ | — | ◐ | Lib refresh greens disk; live enqueue P2 |
| 11 | **PLAN-LEDGER-REFRESH-005** | [`planner_status_audit_v7.md`](planner_status_audit_v7.md) | ☑ | — | — | — | `refresh_005_sync.py` OK |
| 12 | **PLAN-PHASE-D-PARITY-001** | [`overlay_parity_stress_plan_v1.md`](overlay_parity_stress_plan_v1.md) | ☑ | ☑ | — | — | S1–S3 in `phase_d_parity_stress.rs` |

---

## Designer ACCEPT (blocks qualified coder close)

| ID | Record | Coder lane | Verdict |
|:---|:---|:---|:---:|
| **DESIGN-VFX-VISUAL-ACCEPT-001** | [`vfx_visual_acceptance_record_v1.md`](vfx_visual_acceptance_record_v1.md) | **VFX-VISUAL-SIGNOFF-001** | **PASS (qualified)** |
| **DESIGN-WP-VISUAL-ACCEPT-001** | [`world_preview_visual_acceptance_record_v1.md`](world_preview_visual_acceptance_record_v1.md) | **UI-WP-VISUAL-001** | **PASS (qualified)** |
| **DESIGN-M3-UNITS-001** | [`minimap_unit_marker_visual_spec_v1.md`](minimap_unit_marker_visual_spec_v1.md) | **UI-P3-M3-UNITS-001** | **SIGNED** |
| **DESIGN-M3-REPLAY-001** | [`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) | **UI-P3-M3-REPLAY-001** | **SIGNED** |

---

## Coder wave 3 — closure bundles

```powershell
cargo test -p proc_A_dine01 --lib coder_a_wave3_closure coder_b_wave3_bundle_001
```

| Bundle | Lanes |
|:---|:---|
| [`coder_a_wave3_closure_v1.rs`](coder_a_wave3_closure_v1.rs) | F7-A/B/C, VFX qualified, S7B M4 lib refresh, infra |
| [`coder_b_wave3_bundle_proof.rs`](coder_b_wave3_bundle_proof.rs) | IND-E02, CONSTRUCTION-MV, REPLAY, PHASE-D, LOG-E01 lib |

**Round 4 + parametric + M3/replay:** **CLOSED** per audit v9 — do not re-plan archived exec docs.

---

## Wave 6/7 — planner P1 prep (2026-05-27)

| ID | plan_doc | P | C | Notes |
|:---|:---|:---:|:---:|:---|
| **PLAN-CONSTRUCTION-HYDRO-COUPLING-001** | [`plan_construction_hydro_coupling_001_v1.md`](plan_construction_hydro_coupling_001_v1.md) | ☑ | ◐ | B-H2 ready; witness field pending impl |
| **PLAN-WSS-SLAB-PR-3-EXEC-001** | [`plan_wss_slab_pr3_exec_001_v1.md`](plan_wss_slab_pr3_exec_001_v1.md) | ☑ | ◐ | After PR-2 signed |
| **PLAN-LEDGER-REFRESH-007** | [`planner_status_audit_v9.md`](planner_status_audit_v9.md) | ☑ | — | Fleet reconcile |

---

## STALE vs CURRENT (2026-05-27)

| Witness | Field | Verdict | Action |
|:---|:---|:---:|:---|
| `stage5_full_app_live.json` | `vfx_visual_signoff_001.visual_run_pending` | **Optional** | `--test visual` upgrades from qualified |
| `stage5_full_app_live.json` | `logistics_active_rows` | **CURRENT** if > 0 | Re-visual if 0 |
| `stage7_behavioral_live.json` | `s7b_m4_play_001.green` | **STALE** after non-M4 writer | `cargo test … coder_a_wave3_closure` |
| `ui_shell_migration_live.json` | `phase4.icon_atlas_loaded` | **STALE** possible | `steward_ui_oh_gate_001_lib_bundle` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Post wave 4 delivery audit + wave 5 routing |
| v1.1.0 | 2026-05-27 | Audit v9; P1 hydro + PR-3; R4/M3 closed |
