# UI overhaul — live witness status `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Cycle** | **PLAN-UI-OH-CLOSURE-004** |
| **Sources** | On-disk JSON (authoritative over stale markdown) |
| **Closure plan** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) · M3 [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) · P4 [`ui_oh_p4_001_plan_v1.md`](ui_oh_p4_001_plan_v1.md) · P5 [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) |

**Rule:** **STALE** = refresh witness, do **not** reopen closed coder lanes without contradicting lib tests.

---

## Shell — `debug_runs/ui_shell_migration_live.json`

| Path | Value (2026-05-25) | Gate |
|:---|:---|:---:|
| `phase2_zones_live` | `true` | **2A** |
| `phase2a_closed` | `true` | **2A** |
| `ui_oh_2a_001.green` | `true` | **2A** |
| `phase2b_closed` | `true` | **2B** |
| `ui_p2b_coder_b_green` | `true` | **2B** |
| `ui_oh_2b_001.green` | `true` | **2B** |
| `egui_pass_count_in_sim` | `0` | **2B** |
| `ui_p2a_coder_b.green` | `true` | 2A tail |
| `ui_p2a_tail.f03_green` | `true` | 2A tail |
| `ui_p2a_tail.p4_auth_green` | `true` | 2A tail |
| `phase2c.phase2c_closed` | `true` | 2C |
| `ui_p3_001.closed` | `false` | **PARTIAL** — compositor authoritative |
| `phase2.minimap_gpu_path` | `false` | **PARTIAL** — timing; see compositor JSON |

---

## Phase 5 — `ui_shell_migration_live.json` → `phase5` (**UI-OH-P5-001**)

| Path | Value (2026-05-25) | Gate |
|:---|:---|:---:|
| `phase5.pause_menu_bevy` | `true` | **P5-PAUSE-001** |
| `ui_p5_pause_001_green` | `true` | **P5-PAUSE-001** |

---

## Phase 4 — `ui_shell_migration_live.json` → `phase4` (**UI-OH-P4-001**)

| Path | Value (2026-05-25) | Gate |
|:---|:---|:---:|
| `phase4.rail_icons` | `["RD","RL","UT","IN","CV"]` | **P4.1** |
| `phase4.p5_br_tab_wired` | `true` | **P4-P5-01** |
| `phase4.atlas_texture` | `textures/ui/icon_atlas_phase4_v1.png` | **P4.1** |
| `phase4.manifest_ron` | `configs/ui/icon_atlas_phase4.icon_atlas.ron` | **P4.1** |
| `phase4.icon_atlas_loaded` | `false` | **STALE** — lib test green |
| `ui_oh_p4_001.green` | *(absent)* | optional writer — use phase4 + lib |

---

## Compositor — `debug_runs/minimap_compositor_live.json`

| Path | Value (2026-05-25) | Gate |
|:---|:---|:---:|
| `composite_ok` | `true` | **M1** |
| `composite_path` | `GpuCompute` | **M1** |
| `dual_minimap_present` | `false` | **M1** |
| `presentation_source` | `SharedRenderTargetImage` | **M1** |
| `ui_p3_001_green` | `true` | **UI-P3-001** |
| `logistics_rows` | `2` | **M2** |
| `ui_oh_m2_001.green` | `true` | **M2** |
| `construction_rows` | `18` | M2 |
| `ui_p3_m2_green` | `true` | M2 |
| `ui_p3_m3_green` | `true` | **UI-P3-M3-001** / **UI-OH-M3-001** (M2 construction + ecology) |
| `ui_p3_m3_units_001_green` | `true` | optional tail |
| `ui_p3_m3_replay_001_green` | `true` | optional tail |
| `ui_oh_m3_001.green` | `true` after refresh | **UI-OH-M3-001** — use `ui_p3_m3_green` if absent |
| `ui_p3_m4_green` | `true` | **UI-P3-M4-001** (design M3 FoW/EW) |

---

## Spine — `debug_runs/stage5_full_app_live.json`

| Path | Value (2026-05-25) | Gate |
|:---|:---|:---:|
| `readiness.passes` | `true` | **UI-OH-GATE-001** col updates |
| `stage5_closure.passes` | `true` | **UI-OH-GATE-001** C |
| `projection_graph.logistics_active_rows` | `0` | **STALE** — see [`log_e01_full_app_witness_spec_v1.md`](log_e01_full_app_witness_spec_v1.md) |

---

## Steward rollup

| Gate ID | Verdict | Record |
|:---|:---:|:---|
| **UI-OH-GATE-001** | **PASS (qualified)** | [`steward_ui_oh_gate_v1.md`](steward_ui_oh_gate_v1.md) |

---

## Refresh commands

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 minimap_compositor stage5
```
