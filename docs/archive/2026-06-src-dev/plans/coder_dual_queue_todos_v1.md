# Coder dual queue — open todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-26 |
| **Authority** | [`coder_dual_queue_v2.md`](coder_dual_queue_v2.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) |
| **Lib bundle** | `cargo test -p proc_A_dine01 --lib coder_a_dual_queue_14_closure_bundle` |

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue_14_closure_bundle
```

---

## @coder A — P1 (pick one primary)

| ☑ | ID | Task | Plan / entry | Exit witness |
|:---:|:---|:---|:---|:---|
| ☑ | **FIRE7-F7-A-001** | Per-view fire extract hardening (after FIRE7-PLAN-001) | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) | `fire7_f7_a_001` in infrastructure JSON |
| ☑ | **P2-FIRE-SPARK-010** | Sparks above smoke / weather field draw order | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) | `fire_sparks_above_smoke: true` |
| ☑ | **P2-FIRE-SPARK-011** | Spark compute tune @ tactical zoom | same · `fire_spark_compute.wgsl` | `fire_spark_011_green` @ tactical zoom |
| ☑ | **P2-WATER-POLISH-001** | River ribbon + ocean tile player read | same · D-W01/D-W04 | `water_w1_river_read_green` |
| ☑ | **UX-E03-CODER-A** | Transmission shell wiring (no order enqueue from UI) | [`ux_e03_transmission_shell_note_v1.md`](ux_e03_transmission_shell_note_v1.md) | `ux_e03_coder_a.green` |
| ☑ | **S7B-M4-PLAY-001** | Mission enqueue / playtest hooks (Move/Secure corridor) | [`s7b_closure_plan_v1.md`](s7b_closure_plan_v1.md) | `s7b_m4_play_001.green` |
| ☑ | **INFRA-GPU-TILE-001** | Instanced tile authoritative; drop CPU gizmo fallback | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) | `readiness/instanced_dispatch_ok` |

---

## @coder A — P2 (optional polish)

| ☑ | ID | Task | Notes |
|:---:|:---|:---|:---|
| ☑ | **UI-WP-PIPELINE** | Preview raster/GPU/viewport bugs (not layout) | `ui_wp_pipeline_green` in `wave_p_live.json` |
| ☑ | **UI-WP-L4-001** | Raster look from signed refs | `ui_wp_l4_001_green` |
| ☑ | **UI-WP-MOTION-001** | World preview motion table | `ui_wp_motion_001_green` |
| ☑ | **UI-WP-LAYOUT-003** | Paper frames + D-09 offsets | `ui_wp_layout_003_green` |
| ☑ | **P4-VEH-01** | Vehicle icon row consumers | `p4_veh_01.green` in shell JSON |
| ☑ | **INFRA-PERF-001** | Frame budget fixes from OPS-F01 | qualified via `wc_d04.green` (stage6) |
| ☑ | **S7B-TUNE-DELAY-001** | Tune `dispatch_delay_ticks` (default 8) | `s7b_tune_delay_001_green` |

---

## @coder B — P1 + optional (lib bundle)

| ☑ | ID | Task | Bundle |
|:---:|:---|:---|:---|
| ☑ | **LOG-E01-WITNESS** | FULL_APP logistics rollup | `refresh_log_e01_and_tactical_vfx_stage5_live_witness` |
| ☑ | **IND-E02-DEFAULT** | `ind_e02_green` default path | `refresh_ind_e02_default_live_witness` |
| ☑ | **P2-VFX-WITNESS-001** | Tactical fire+water gates | same refresh |
| ☑ | **P2-WATER-WITNESS-002** | Water particle JSON fields | same refresh |
| ☑ | **INFRA-VM10-001** | Minimap lockstep | `refresh_infrastructure_view_isolation_live_witness` |
| ☑ | **INFRA-VM11-001** | Preview vs main audit | same |
| ☑ | **INFRA-VM09-STRAY-001** | Stray `MapCameraDesired` audit | `infra_vm09_stray_map_camera_writer_audit_green` |
| ☑ | **WITNESS-SHELL-P4** | `icon_atlas_loaded` + P2A tail | `refresh_ui_p2a_001_live_witness` |
| ☑ | **UI-P2A-WITNESS-TAIL** | ops hover + build rail | same |
| ☑ | **WAVE-P-WITNESS** | `wave_p_live.json` | `refresh_coder_a_ui_wp_wave_p_witness` |
| ☑ | **WAVE-C-WITNESS** | `wave_c_live.json` | `commit_wave_c_live_proof` |
| ☑ | **UI-WP-LAYOUT-D02-OPT** | map dominance witness | `ui_wp_layout_d02_opt_green` in wave_p |
| ☑ | **CONSTRUCTION-MV-001** | construction gate | bundle qualified — full MV when sim runs |
| ☑ | **S7P-GRID-UX-001** | grid overload UX | witness fields only — **no toast UI** |

**Verify:**

```powershell
cargo test -p proc_A_dine01 --lib coder_b_queue_bundle
```

---

## Mark done

- **Coder A (14):** [`coder_a_dual_queue_closure_v1.rs`](coder_a_dual_queue_closure_v1.rs) — `coder_a_dual_queue_14_closure_bundle`
- **Coder B (14):** [`coder_b_queue_bundle_proof.rs`](coder_b_queue_bundle_proof.rs) — `coder_b_queue_bundle_001_lib_refresh`

**FIRE7-F7-A-001** here = **witness bundle only**. Product gate = **FIRE7-F7-A-EXIT-001** in [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md).

Do not re-queue unless regression fails. **Next wave:** [`coder_dual_queue_v3.md`](coder_dual_queue_v3.md).
