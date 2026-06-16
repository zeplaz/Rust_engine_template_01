# UI Phase 2B — egui gate plan `v1` (PLAN-UI-SHELL-2B-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-SHELL-2B-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **CLOSED** — gate + **UI-P2B-CODER-B** landed |
| **Coder tasks** | [`ui_p2b_coder_b_numbered_tasks_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_p2b_coder_b_numbered_tasks_v1.md) |
| **Architecture** | [`ui_phase2b_egui_gate_plan_v1.md`](ui_phase2b_egui_gate_plan_v1.md) (full) |
| **Witness** | [`debug_runs/ui_shell_migration_live.json`](../../../debug_runs/ui_shell_migration_live.json) |
| **Ledger** | **G-UI-P2B** — [`stage_tracks_signoff_ledger_v1.md`](../../../docs/archive/2026-06-src-dev/plans/stage_tracks_signoff_ledger_v1.md) |

**Partial deliverable:** gate contract + numbered **UI-P2B-CODER-B** tasks. No Rust in this doc.

---

## Gate verdict (one line)

In **`BaseState::Simulation`**, the **egui product shell does not run**; only **F3 diagnostics** and **editor tooling** buckets may use egui. Proof: **`egui_pass_count_in_sim == 0`** and witness `*_egui_gated` flags.

---

## Master gate

| Gate | `run_if` / mechanism | Sim | Editor |
|:---|:---|:---:|:---:|
| **G-2B-01** Product shell root | `product_egui_shell_active` on `hud_product_shell_egui_root` | **off** | on |
| **G-2B-02** Floating shells | `SIM_SUPPRESSED_FLOATING_SHELLS` + `enforce_simulation_product_egui_gates` | suppressed | on |
| **G-2B-03** BuildToolbox egui | `build_toolbox_egui_dock_active` | **off** | on |
| **G-2B-04** Side status rail egui | `side_status_rail_egui_active` + collapsed layout | **off** | on |
| **G-2B-05** Sim-allowed egui | `EGUI_SIM_SHELL_WIDGETS` only | F3 + editor tools | — |

**Authority:** `src/gui/ui_gates.rs` · `src/gui/hud/simulation_session.rs` · `src/gui/hud/shell_framework.rs`

---

## Proof formula (UI-P2B-CODER-B)

```text
ui_p2b_coder_b_green :=
  egui_pass_count_sim_session == 0
  AND witness.build_toolbox_egui_gated
  AND witness.side_status_rail_egui_gated
  AND witness.floating_egui_shells_gated

phase2b_closed := ui_p2b_coder_b_green   // JSON rollup
egui_pass_count_in_sim := egui_pass_count_sim_session   // witness field name (sim-session scoped)
```

**Do not reopen UI-P2B-001** if only stale JSON flags disagree — run **UI-SHELL-REFRESH-001** (witness refresh) per ledger policy.

---

## Witness bundle

| Field | Required | Fleet (2026-05-25) |
|:---|:---:|:---|
| `egui_pass_count_in_sim` | `0` | `0` |
| `egui_pass_count_lifetime` | informational | `0` |
| `phase2b_closed` | `true` | `true` |
| `ui_p2b_coder_b_green` | `true` | `true` |
| `witness.build_toolbox_egui_gated` | `true` | `true` |
| `witness.side_status_rail_egui_gated` | `true` | `true` |
| `witness.floating_egui_shells_gated` | `true` | `true` |
| `egui_sim_shell_widgets` | `["Diagnostics_F3","Editor_tools"]` | ☑ |

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 ui_p2b stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Lane map (PLAN-UI-SHELL-2B-001)

| Lane | ID | Status |
|:---|:---|:---:|
| Core retirement | **UI-P2B-001** | **DONE** |
| Witness + sim-session counter | **UI-P2B-CODER-B** | **DONE** — tasks 1–6 |
| Steward re-audit | **UI-SHELL-REFRESH-001** | **PASS** |
| Phase 2A tail | **UI-P2A-CODER-B**, F03, P4-AUTH | **DONE** |
| Optional hardening | **UI-P2B-002…005** | OPEN — see numbered tasks §7+ |

---

## Do not re-queue

| ID | Policy |
|:---|:---|
| **UI-P2B-001** | Gates landed — maintenance only |
| **UI-P2B-CODER-B** | Tasks 1–6 **DONE** — no counter/formula rework without regression proof |
| **UI-SHELL-REFRESH-001** | Proof-only PASS — refresh JSON if drift, not feature slice |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — 2B gate plan |
| Coder B | 2026-05-23–25 | **UI-P2B-CODER-B** tasks 1–6 **DONE** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-UI-SHELL-2B-001 partial — gate + CODER-B task index |
