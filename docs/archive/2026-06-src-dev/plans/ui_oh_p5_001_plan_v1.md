# UI overhaul — Phase 5 closure plan `v1` (UI-OH-P5-001)

| Field | Value |
|:---|:---|
| **Lane ID** | **UI-OH-P5-001** |
| **Planner queue** | **PLAN-UI-P5-PAUSE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **P5-PAUSE-001** **CLOSED** · tails **OPEN** |
| **Pause plan (authoritative)** | [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) |
| **Master lane** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| **Phase 2+3 closure** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) |
| **Live rollup** | [`witness_status_live_v1.md`](witness_status_live_v1.md) |
| **Witness** | `debug_runs/ui_shell_migration_live.json` → `phase5` |

**No Rust in this deliverable.** Maps **PLAN-UI-P5-PAUSE-001** into **UI-OH-P5-001**.

---

## Naming guard

| Term | Meaning |
|:---|:---|
| **UI Phase 5** | Pause / session menu (this plan) |
| **P5_BR** | Phase **4** petroleum icon cell — **not** Phase 5 |
| **Menu pause** | `InGameMenuState::Pause` · `AppState::Paused` |
| **Sim tick pause** | `SimControlState.paused` — ops strip **P** |

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **P5-SCAFFOLD** | **PASS** — Escape toggle + confirm flows |
| **P5-PAUSE-001** | **PASS** — Bevy pause shell (`pause_menu_bevy.rs`); no egui pause in sim |
| **P5-DESIGN-001** | **PASS** — [`ui_p5_design_signoff_v1.md`](ui_p5_design_signoff_v1.md) |
| **P5-SAVE-001** | **DEFERRED** — Save/Load wire to Wave S spine |
| **P5-SETTINGS-001** | **DEFERRED** |
| **UI-OH-P5-001 rollup** | **PASS (qualified)** |

**Qualified:** Run **full** shell witness refresh after P5-only test writes — `ui_p5_pause_001` test can overwrite 2A/2B fields if witness is partial. Use `steward_ui_oh_gate_001_lib_bundle` for combined proof.

**Priority:** **P2** — does **not** block Stage 5 spine, S7B, or UI-OH 2/3 closure.

---

## Gate chain (PLAN-UI-P5-PAUSE-001)

```text
UI-P2B-GATE (egui_pass_count_in_sim: 0)              ☑
PLAY-01 (sim HUD defaults)                            ☑
        │
        ▼
P5-SCAFFOLD — Escape + confirm                        ☑
        │
        ▼
P5-PAUSE-001 — Bevy pause overlay                       ☑
        │
        ├─► P5-DESIGN-001 — vellum mock / tokens          ☑
        ├─► P5-SAVE-001 — save/load wire                  ☐ deferred
        └─► P5-SETTINGS-001                             ☐ deferred
```

---

## PASS gate — P5-SCAFFOLD

| # | Criterion | Evidence |
|:---:|:---|:---|
| S1 | Escape toggles `InGameMenuState::Pause` | `in_game_pause_menu.rs` |
| S2 | UX bridge | `AppState::Paused`, `PauseState` in `ux_orchestration.rs` |
| S3 | Destructive nav confirm | `pause_menu_confirm.rs` |
| S4 | Resume / Quit / Main menu / WorldGen paths | menu + confirm |

---

## PASS gate — P5-PAUSE-001 (Bevy)

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| P5-1 | Bevy path landed | code `pause_menu_bevy.rs` | no egui pause system | ☑ |
| P5-2 | Witness flag | `phase5.pause_menu_bevy` | `true` | ☑ |
| P5-3 | Rollup | `ui_p5_pause_001_green` | `true` | ☑ |
| P5-4 | 2B not regressed | `egui_pass_count_in_sim` | `0` after **full** refresh | ☑ lib / re-run steward bundle |
| P5-5 | Lib test | `ui_p5_pause_001_witness_green_when_bevy_flag_set` | pass | ☑ |

**Lib anchor:**

```powershell
cargo test -p proc_A_dine01 --lib ui_p5_pause
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
```

**Manual (operator):** Simulation → Escape → menu visible → Resume; WorldGen / Main menu confirm still work.

---

## UI-OH-P5-001 rollup (witness block)

| Path | Green when |
|:---|:---|
| `ui_oh_p5_001.gate` | `"UI-OH-P5-001"` |
| `ui_oh_p5_001.green` | `ui_p5_pause_001_green` ∧ scaffold paths OK |
| `ui_oh_p5_001.pause_menu_bevy` | mirrors `phase5.pause_menu_bevy` |

**Formula:**

```text
ui_oh_p5_001.green :=
  phase5.pause_menu_bevy == true
  AND ui_p5_pause_001_green == true
```

Optional writer: extend `build_proof_payload` in `simulation_shell_phase2.rs` (coder, not planner).

Until block exists, use `phase5` + `ui_p5_pause_001_green` as authority.

**Witness choreography:** Run `steward_ui_oh_gate_001_lib_bundle` **after** `ui_p5_pause_001` test if both P5 and 2A/2B must be green in one JSON file (P5-only commit resets other rollup fields).

---

## Witness field table

| Phase | File | Field | Role |
|:---|:---|:---|:---|
| P5 core | `ui_shell_migration_live.json` | `phase5.pause_menu_bevy` | Bevy pause landed |
| P5 rollup | `ui_shell_migration_live.json` | `ui_p5_pause_001_green` | **UI-P5-PAUSE-001** exit |
| OH rollup | `ui_shell_migration_live.json` | `ui_oh_p5_001.green` | **Future** optional |
| Spine | `stage5_full_app_live.json` | `readiness.passes` | unchanged by P5 |

---

## Open tails (not UI-OH-P5-001 blockers)

| ID | Owner | Notes |
|:---|:---|:---|
| **UI-P5-DESIGN-001** | @designer | `assets/ui/simulation/pause_menu_spec_v1.png` |
| **P5-SAVE-001** | @coder | Wave S / save spine |
| **P5-SETTINGS-001** | — | deferred |

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| egui pause window in Simulation after P5-PAUSE-001 | UI-P2B |
| Pause menu mutates gameplay ECS | UX transitions only |
| Confuse **UI-P5** with **P5_BR** | Phase 4 petroleum |
| P5-only witness write without 2A/2B refresh | Partial JSON clobber |

---

## Copy-paste — witness refresh (@coder / operator)

```
Lane: UI-OH-P5-001 — pause witness + shell bundle
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md
      docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md
Verify: cargo test -p proc_A_dine01 --lib ui_p5_pause steward_ui_oh_gate_001_lib_bundle stage5
Witness: phase5.pause_menu_bevy true AND phase2b_closed true (same JSON)
```

---

## Copy-paste — P5-DESIGN-001 (@designer, P2)

See [`ui_phase5_pause_menu_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md) § P5-DESIGN-001 — mock only; does not block **UI-OH-P5-001** qualified PASS.

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — UI-OH-P5-001 / PLAN-UI-P5-PAUSE-001 |
| Coder P5-PAUSE-001 | 2026-05-25 | **CLOSED** — `pause_menu_bevy.rs` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | UI-OH lane mapping for PLAN-UI-P5-PAUSE-001 |
