# UI-P2B-CODER-B — numbered tasks `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-P2B-CODER-B** |
| **Parent** | **PLAN-UI-SHELL-2B-001** |
| **Gate plan** | [`ui_phase2b_gate_plan_v1.md`](../prompts/guides/ui/ui_phase2b_gate_plan_v1.md) |
| **Owner** | `@coder` B |
| **Status** | **DONE** (tasks 1–6) · optional 7–10 **OPEN** |
| **Witness** | `ui_p2b_coder_b_green` · `phase2b_closed` in [`debug_runs/ui_shell_migration_live.json`](../debug_runs/ui_shell_migration_live.json) |

---

## Scope

**UI-P2B-CODER-B** fixes witness honesty for Phase 2B: **sim-session** egui pass counting and **witness flag sync** so `phase2b_closed` matches PLAY-01 policy. **Does not** replace **UI-P2B-001** (product-shell `run_if` and dock suppression).

**Max files per task:** ≤3 (historical bundle: `shell_diagnostics.rs`, `simulation_session.rs`, `simulation_shell_phase2.rs`).

---

## Tasks 1–6 (CLOSED)

| # | Task | File(s) | Done when | Status |
|:---:|:---|:---|:---|:---:|
| **1** | Add **`egui_pass_count_sim_session`** separate from lifetime `egui_pass_count` | `shell_diagnostics.rs` | Field exists; documented | ☑ |
| **2** | Reset both counters on **`OnEnter(Simulation)`** | `simulation_session.rs` → `apply_simulation_hud_defaults` | `reset_egui_pass_count_for_simulation_session()` called | ☑ |
| **3** | Map JSON **`egui_pass_count_in_sim`** to **sim-session** counter (not lifetime) | `simulation_shell_phase2.rs` → `build_proof_payload` | Witness uses `egui_pass_count_sim_session` | ☑ |
| **4** | Implement **`ui_p2b_coder_b_green`** rollup | `simulation_shell_phase2.rs` | Formula: session count 0 + three witness gates | ☑ |
| **5** | Sync witness gates on sim enter + enforce pass | `simulation_session.rs` | `sync_simulation_egui_shell_gate_witness` in defaults + `enforce_simulation_product_egui_gates` | ☑ |
| **6** | Lib tests for reset, closed rollup, false when passes | `simulation_session.rs`, `simulation_shell_phase2.rs` | `cargo test -p proc_A_dine01 --lib simulation_shell_phase2 ui_p2b` green | ☑ |

### Task detail

#### Task 1 — Sim-session counter resource

```rust
// shell_diagnostics.rs
pub egui_pass_count_sim_session: u64,
```

#### Task 2 — Reset on sim enter

```rust
// apply_simulation_hud_defaults (OnEnter Simulation)
shell_diag.reset_egui_pass_count_for_simulation_session();
```

#### Task 3 — Witness field semantics

```text
egui_pass_count_in_sim  ←  shell_diag.egui_pass_count_sim_session
egui_pass_count_lifetime ← shell_diag.egui_pass_count  (editor inflation visible)
```

#### Task 4 — Green rollup

```rust
pub fn ui_p2b_coder_b_green(witness, shell_diag) -> bool {
    shell_diag.egui_pass_count_sim_session == 0
        && witness.build_toolbox_egui_gated
        && witness.side_status_rail_egui_gated
        && witness.floating_egui_shells_gated
}
```

#### Task 5 — Witness sync

| Flag | Source |
|:---|:---|
| `build_toolbox_egui_gated` | BuildToolbox dock !visible && minimized && !detached |
| `side_status_rail_egui_gated` | `status_side_panel_state == Collapsed` |
| `floating_egui_shells_gated` | `simulation_floating_shells_gated(dock)` |

#### Task 6 — Tests (names)

| Test | Asserts |
|:---|:---|
| `ui_p2b_coder_b_resets_egui_pass_count_on_sim_enter` | Counters 0 after reset |
| `ui_p2b_coder_b_phase2b_closed_when_sim_egui_gates_suppressed` | JSON `phase2b_closed` + `ui_p2b_coder_b_green` true |
| `ui_p2b_coder_b_green_false_when_sim_session_egui_passes` | `record_egui_pass_in_simulation` → green false |

---

## Verify (tasks 1–6)

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2
cargo test -p proc_A_dine01 --lib ui_p2b_coder_b_resets_egui_pass_count_on_sim_enter
cargo test -p proc_A_dine01 --lib stage5
```

**Witness refresh:**

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

Expect: `ui_p2b_coder_b_green: true`, `phase2b_closed: true`, `egui_pass_count_in_sim: 0`.

---

## Tasks 7–10 (OPTIONAL — Phase 2B+)

| # | Task | Owner | Status | Notes |
|:---:|:---|:---|:---:|:---|
| **7** | **UI-P2B-002** — document lifetime vs session in HUD F3 panel | @coder B | ☐ | UX clarity only |
| **8** | **UI-P2B-003** — inner guard on `draw_hud_side_status_panel_egui` | @coder A | ☐ | `side_status_rail_egui_dock_active` |
| **9** | **UI-P2B-004** — PR checklist: no HUD egui via `in_simulation_or_editor` alone | docs | ☐ | `AGENTS.md` one-liner |
| **10** | **UI-P2B-005** — `--test demo` smoke: build rail only, no Construction egui | @coder | ☐ | [`ui_construction_playtest_v1.md`](ui_construction_playtest_v1.md) |

**Do not** block **G-UI-P2B** closure on tasks 7–10.

---

## Copy-paste — @coder B (archive)

```
Lane: UI-P2B-CODER-B
Read: src/dev/ui_p2b_coder_b_numbered_tasks_v1.md
      prompts/guides/ui/ui_phase2b_gate_plan_v1.md
First: confirm tasks 1–6 (sim-session counter + ui_p2b_coder_b_green)
Do NOT: reopen UI-P2B-001 run_if gates without planner
Verify: cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5
Witness: ui_shell_migration_live.json → ui_p2b_coder_b_green, phase2b_closed
```

---

## Steward / operator (not Coder B)

| ID | Owner | Action |
|:---|:---|:---|
| **UI-SHELL-REFRESH-001** | sim-steward | Re-audit stale JSON; **PASS** = no coder blockers |
| **UI-SHELL-REFRESH-001-C** | operator | Re-run sim enter + visual if `witness.*_gated` false at write time |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Numbered tasks 1–6 DONE; 7–10 optional 2B+ |
