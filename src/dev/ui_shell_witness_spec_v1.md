# UI shell migration witness spec `v1` (PLAN-UI-SHELL-WITNESS-SPEC-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-SHELL-WITNESS-SPEC-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Status** | **SIGNED** |
| **Witness file** | [`debug_runs/ui_shell_migration_live.json`](../../debug_runs/ui_shell_migration_live.json) |
| **Writer** | `simulation_shell_phase2.rs` `write_ui_shell_migration_live_proof` |
| **2B gate** | [`ui_phase2b_gate_plan_v1.md`](../prompts/guides/ui/ui_phase2b_gate_plan_v1.md) |

---

## Profile

| Field | Value |
|:---|:---|
| `profile` | `UI_SHELL_MIGRATION_2B` |

---

## Phase 2B fields (authoritative)

These names are **defined** in code (`ui_p2b_coder_b_phase2b_closed_when_sim_egui_gates_suppressed`).

| Path | Type | Required value (sim) | Meaning |
|:---|:---|:---|:---|
| `phase2b_closed` | bool | `true` | Rollup gate **G-UI-P2B** |
| `ui_p2b_coder_b_green` | bool | `true` | Same predicate as `phase2b_closed` |
| `egui_pass_count_in_sim` | number | `0` | **Sim-session** egui passes only |
| `egui_pass_count_lifetime` | number | any | Lifetime counter (may be > 0 in editor) |

### `ui_p2b_coder_b` object

| Path | Required (sim) |
|:---|:---|
| `green` | `true` |
| `egui_pass_count_in_sim` | `0` |
| `build_toolbox_egui_gated` | `true` |
| `floating_egui_shells_gated` | `true` |
| `side_status_rail_egui_gated` | `true` |

**Formula:**

```text
ui_p2b_coder_b_green :=
  egui_pass_count_sim_session == 0
  AND witness.build_toolbox_egui_gated
  AND witness.side_status_rail_egui_gated
  AND witness.floating_egui_shells_gated
```

---

## Phase 2A tail (optional — not 2B blockers)

| Path | Note |
|:---|:---|
| `ui_p2a_coder_b.green` | Replay helpers — may be `false` until `--test visual` replay |
| `ui_p2a_tail.f03_green` | Build-rail hover border parity |
| `ui_p2a_tail.p4_auth_green` | P4 rail authority replay |
| `ui_p2a_tail.build_rail_authoritative` | Rail sync witness |

**Policy:** **PARTIAL** in ledger — do **not** fail `phase2b_closed` when only `ui_p2a_tail.*` false.

---

## Other rollup fields

| Path | Role |
|:---|:---|
| `phase2a_closed` | Phase 2A exit |
| `phase2c` | Layout **2C-B** widths |
| `phase2c.phase2c_closed` | 2C sign-off |
| `phase5` | Pause menu — [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) |
| `ui_p3_001` | GPU minimap operationalization |
| `witness.*` | Interaction replay flags |
| `backends.legacy_egui_phase2b` | Audit map (editor vs sim) |

---

## STALE JSON handling

If `phase2b_closed: false` but `egui_pass_count_in_sim: 0` and `ui_p2b_coder_b.*_gated: true`:

| Label | Action |
|:---|:---|
| **STALE** | Run `cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_p2b_coder_b_phase2b_closed_when_sim_egui_gates_suppressed` |
| | Or `cargo run -p proc_A_dine01 --release -- --test visual` |

Do **not** reopen **UI-P2B-001** without contradicting proof.

---

| `phase5.pause_menu_bevy` | bool | `true` | **UI-P5-PAUSE-001** |
| `ui_p5_pause_001_green` | bool | `true` | Rollup |
| `ui_w3_p5_001.green` | bool | `true` | **UI-W3-P5-001** (Wave 3 alias) |
| `ui_w3_p5_001.pause_menu_bevy` | bool | `true` | Bevy pause shell |
| `ui_w3_p5_001.egui_pass_count_in_sim` | number | `0` | No egui pause overlay in sim |
| `ui_w3_witness_001.green` | bool | `true` | **UI-W3-WITNESS-001** Wave 3 shell rollup |
| `ui_w3_witness_001.visual_operator` | string | release `--test visual` | Operator timestamp refresh |
| `ui_w3_p6_001.green` | bool | `true` | **UI-W3-P6-001** shell perf slice on shell JSON |
| `ui_w3_p6_001.shell_perf_green` | bool | `true` | P6-1…P6-3 (2B + pause) |

**Cross-file (UI-W3-P6-001 multiview):** `infrastructure_view_isolation_live.json` → `infrastructure_view_isolation_green`; `stage6_virtualization_live.json` → `stage6_virtualization_green`; `minimap_compositor_live.json` → `composite_path: GpuCompute`.

**Lib refresh:**

```powershell
cargo test -p proc_A_dine01 --lib coder_b_ui_w3_witness_001_lib_bundle
cargo test -p proc_A_dine01 --lib coder_b_ui_w3_p6_001_lib_bundle
```

---


```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_p2b_coder_b_phase2b_closed_when_sim_egui_gates_suppressed stage5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-UI-SHELL-WITNESS-SPEC-001 — 2B field catalog |
