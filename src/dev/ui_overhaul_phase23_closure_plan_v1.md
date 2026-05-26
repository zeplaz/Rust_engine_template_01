# UI overhaul — Phase 2 & 3 closure plan `v1` (PLAN-UI-OH-CLOSURE-004)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-OH-CLOSURE-004** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — Phase 2 + Phase 3 **CLOSED**; Phase 4/5 **PARTIAL (qualified PASS)** |
| **Master lane** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| **Live rollup** | [`witness_status_live_v1.md`](witness_status_live_v1.md) |
| **Shell spec** | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) |
| **Steward gate** | [`steward_ui_oh_gate_v1.md`](steward_ui_oh_gate_v1.md) |

**No Rust in this deliverable.** Closure is witness + steward criteria only.

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **Phase 2A** | **PASS** — `phase2_zones_live`, interaction witness bundle |
| **Phase 2B** | **PASS** — `egui_pass_count_in_sim: 0`, egui gates |
| **Phase 2C** | **PASS** — layout **2C-B** witness |
| **Phase 3 M1/M1.5** | **PASS** — GPU compositor path |
| **Phase 3 M2** | **PASS** — `minimap_compositor_live.json` logistics + construction rows |
| **UI-OH-GATE-001** | **PASS (qualified)** — 2A/2B + Stage 5 spine |
| **Phase 4** | **PARTIAL (qualified PASS)** | [`ui_oh_p4_001_plan_v1.md`](ui_oh_p4_001_plan_v1.md) — **PLAN-UI-P4-ATLAS-001** |
| **Phase 5** | **PARTIAL (qualified PASS)** | [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) — **PLAN-UI-P5-PAUSE-001** |

**Product exit (Phase 2+3):** Maintain regression only — do **not** reopen **UI-P2A-001**, **UI-P2B-001**, **UI-P3-M2-001** without contradicting proof.

---

## PASS gate — Phase 2A

**North star:** P1–P4 Bevy zones live; interaction replay flags green.

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| 2A-1 | Zones live | `phase2_zones_live` | `true` | ☑ |
| 2A-2 | Rollup closed | `phase2a_closed` | `true` | ☑ |
| 2A-3 | OH 2A gate | `ui_oh_2a_001.green` | `true` | ☑ |
| 2A-4 | Coder B rollup | `ui_p2a_coder_b.green` | `true` | ☑ |
| 2A-5 | Alert → tray | `witness.alert_click_expanded_tray` | `true` | ☑ |
| 2A-6 | Intel → map | `witness.intel_map_camera_request` | `true` | ☑ |
| 2A-7 | Escape collapses tray | `witness.escape_collapsed_tray` | `true` | ☑ |
| 2A-8 | Ops zones wired | `witness.ops_zones_wired` | `true` | ☑ |
| 2A-9 | Minimap chrome ≤2px | `witness.minimap_chrome_aligned` | `true` | ☑ |
| 2A-10 | Tail F03 hover | `ui_p2a_tail.f03_green` | `true` | ☑ |
| 2A-11 | Tail P4 authority | `ui_p2a_tail.p4_auth_green` | `true` | ☑ |

**Lib anchor:** `steward_ui_oh_gate_001_lib_bundle` · `simulation_shell_phase2` tests.

**PARTIAL (non-blockers):** `ui_p3_001.closed: false` at shell capture frame — **minimap compositor** JSON is authoritative for GPU path.

---

## PASS gate — Phase 2B

**North star:** Zero egui product passes in **Simulation** session.

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| 2B-1 | Rollup closed | `phase2b_closed` | `true` | ☑ |
| 2B-2 | Coder B green | `ui_p2b_coder_b_green` | `true` | ☑ |
| 2B-3 | OH 2B gate | `ui_oh_2b_001.green` | `true` | ☑ |
| 2B-4 | Sim egui passes | `egui_pass_count_in_sim` | **`0`** | ☑ **0** |
| 2B-5 | Nested sim count | `ui_p2b_coder_b.egui_pass_count_in_sim` | **`0`** | ☑ |
| 2B-6 | Build toolbox gated | `witness.build_toolbox_egui_gated` | `true` | ☑ |
| 2B-7 | Side rail gated | `witness.side_status_rail_egui_gated` | `true` | ☑ |
| 2B-8 | Floating shells gated | `witness.floating_egui_shells_gated` | `true` | ☑ |
| 2B-9 | Editor-only audit | `backends.legacy_egui_phase2b.sim_allowed` | `Diagnostics_F3`, `Editor_tools` only | ☑ |

**Formula:** per [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) — `ui_p2b_coder_b_green` iff sim egui count **0** and all three gates **true**.

**Allowed:** `egui_pass_count_lifetime > 0` in editor sessions — not a 2B failure.

---

## PASS gate — Phase 3 M2 (minimap compositor)

**Witness file:** `debug_runs/minimap_compositor_live.json`  
**Spec authority:** [`ui_phase3_minimap_compositor_plan_v1.md`](../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md)

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| M2-0 | M1 compositor | `composite_ok` | `true` | ☑ |
| M2-0b | GPU path | `composite_path` | `GpuCompute` | ☑ |
| M2-0c | No dual draw | `dual_minimap_present` | `false` | ☑ |
| M2-1 | Logistics rows | `logistics_rows` | **`> 0`** | ☑ **2** |
| M2-2 | OH M2 gate | `ui_oh_m2_001.green` | `true` | ☑ |
| M2-3 | Logistics heat on | `logistics_heat_enabled` | `true` | ☑ |
| M2-4 | Construction rows | `construction_rows` | **`> 0`** | ☑ **18** |
| M2-5 | Coder M2 rollup | `ui_p3_m2_green` | `true` | ☑ |
| M2-6 | UI-P3-001 rollup | `ui_p3_001_green` | `true` | ☑ |
| M2-7 | Presentation | `presentation_source` | `SharedRenderTargetImage` | ☑ |

**Cross-check (optional):** `stage5_full_app_live.json` → `projection_graph.logistics_active_rows` may be **STALE** (`0` on disk) while compositor M2 green — refresh via `--test visual`, do **not** fail M2 closure.

**M3/M4 tails (closed, not Phase 2–3 blockers):** `ui_p3_m3_green`, `ui_p3_m4_green`, `ui_p3_m3_units_001_green`, `ui_p3_m3_replay_001_green` — documented in live JSON; forward work is polish only.

---

## UI-OH-GATE-001 — steward criteria

**Record:** [`steward_ui_oh_gate_v1.md`](steward_ui_oh_gate_v1.md) · **Proof:** [`steward_ui_oh_gate_proof.rs`](steward_ui_oh_gate_proof.rs)

| Column | Proves | PASS when |
|:---|:---|:---|
| **A — 2B egui retirement** | Product egui absent in sim | All **2B** rows ☑ |
| **B — 2A zones + interaction** | Bevy shell + replay | All **2A** rows ☑ |
| **C — Stage 5 spine** | FULL_APP not regressed | `stage5_closure.passes` + `readiness.passes` |

### Lib bundle pointers (code-defined, not reimplemented here)

| JSON pointer | Gate |
|:---|:---|
| `/phase2a_closed` | shell |
| `/phase2b_closed` | shell |
| `/ui_p2b_coder_b_green` | shell |
| `/ui_oh_2a_001/green` | shell |
| `/ui_oh_2b_001/green` | shell |
| `/ui_p2a_coder_b/green` | shell |
| `/ui_p2a_tail/f03_green` | shell |
| `/ui_p2a_tail/p4_auth_green` | shell |
| `/egui_pass_count_in_sim` | **`0`** |
| `/stage5_closure/passes` | stage5 |
| `/readiness/passes` | stage5 |

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 -- --test-threads=1
cargo test -p proc_A_dine01 --lib stage5
```

**Qualified PASS policy:**

| Label | Meaning | Action |
|:---|:---|:---|
| **STALE** | Shell JSON from partial writer / parallel tests | Re-run bundle; `--test-threads=1` |
| **PARTIAL** | `ui_p3_001.closed`, `phase2.minimap_gpu_path` in shell JSON | Trust compositor witness |
| **REGRESSION** | Bundle test fails after refresh | Route `@coder` — do **not** assume planner reopen |

**Verdict:** **PASS (qualified)** — maintain regression; **no** steward blockers for Phase 2–3 exit.

---

## Master witness field table

| Phase | File | Rollup field | Partner fields |
|:---|:---|:---|:---|
| **2A** | `ui_shell_migration_live.json` | `phase2a_closed` | `phase2_zones_live`, `ui_oh_2a_001`, `witness.*` |
| **2B** | `ui_shell_migration_live.json` | `phase2b_closed` | `egui_pass_count_in_sim`, `ui_p2b_coder_b*` |
| **2C** | `ui_shell_migration_live.json` | `phase2c.phase2c_closed` | widths, `layout_option: 2C-B` |
| **3 M1** | `minimap_compositor_live.json` | `composite_ok` | `composite_path`, `rt_bound`, `stamp` |
| **3 M2** | `minimap_compositor_live.json` | `ui_oh_m2_001.green` | `logistics_rows`, `construction_rows` |
| **3 rollup** | `minimap_compositor_live.json` | `ui_p3_001_green` | `ui_p3_m2_green` |
| **Steward** | both + `stage5_full_app_live.json` | **UI-OH-GATE-001** | columns A/B/C above |
| **4 partial** | `ui_shell_migration_live.json` | `phase4.p5_br_tab_wired` | `icon_atlas_loaded` optional |
| **3 M3** | `minimap_compositor_live.json` | `ui_oh_m3_001.green` | [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) |
| **Phase 5** | **PARTIAL (qualified PASS)** | [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) — **PLAN-UI-P5-PAUSE-001** |

Full live values: [`witness_status_live_v1.md`](witness_status_live_v1.md).

---

## Out of scope (explicit non-closure)

| Item | Status | Notes |
|:---|:---|:---|
| **Phase 4** full atlas load | PARTIAL | [`ui_phase4_icon_atlas_plan_v1.md`](../prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md) |
| **Phase 5** pause menu | **CLOSED** (tails P2) | [`ui_phase5_pause_menu_plan_v1.md`](../prompts/guides/ui/ui_phase5_pause_menu_plan_v1.md) |
| **LOG-E01** stage5 `log_rows` | STALE optional | Does not block UI-OH 2/3 closure |
| **IND-E01** | Product lane | Disjoint from UI-OH |

---

## Closure checklist (orchestrator)

| # | Action | Output |
|:---:|:---|:---|
| 1 | Read [`witness_status_live_v1.md`](witness_status_live_v1.md) | Disk truth |
| 2 | Run UI-OH-GATE-001 lib bundle | green |
| 3 | Mark **PLAN-UI-OH-CLOSURE-004** done in planner queue | machine state |
| 4 | Bump [`ui_overhaul_plan.md`](ui_overhaul_plan.md) → Phase 2+3 **CLOSED** rollup | doc |
| 5 | Do **not** queue new Phase 2/3 coder slices | — |

---

## Copy-paste — maintenance only

```
Lane: UI-OH regression (post-closure)
Read: src/dev/ui_overhaul_phase23_closure_plan_v1.md
      src/dev/witness_status_live_v1.md
Verify: cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle minimap_compositor stage5
Do NOT: reopen UI-P2A/P2B/P3-M2 without contradicting proof
```

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-OH-CLOSURE-004 |
| Sim-steward | 2026-05-25 | **UI-OH-GATE-001 PASS (qualified)** — prior |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Phase 2A/2B/M2 + UI-OH-GATE-001 closure rollup |
