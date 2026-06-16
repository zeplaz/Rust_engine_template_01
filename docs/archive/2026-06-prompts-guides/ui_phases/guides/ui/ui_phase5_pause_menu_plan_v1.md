# UI Phase 5 — pause menu plan `v1` (PLAN-UI-P5-PAUSE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P5-PAUSE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Priority** | **P2 (lower)** — design/save tails only |
| **Status** | **SIGNED** — P5-PAUSE-001 **CLOSED** · **UI-OH-P5-001** mapped |
| **UI-OH lane** | [`ui_oh_p5_001_plan_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md) |
| **Boundary** | [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) |
| **PLAY-01** | [`simulation_session.rs`](../../../src/gui/hud/simulation_session.rs) |
| **UX orchestration** | [`ux_orchestration.rs`](../../../src/engine/ux_orchestration.rs) · [`ux_states.rs`](../../../src/engine/ux_states.rs) |

**No Rust in this doc.** Planner rollup for **Phase 5** — in-simulation **pause menu** (Escape), distinct from diagnostics **P** tick pause and from **Phase 4** icon cell **P5_BR** (petroleum).

---

## Naming guard

| Term | Meaning |
|:---|:---|
| **UI Phase 5** | This plan — pause / session menu product lane |
| **P5_BR** | Phase **4** icon atlas cell (petroleum tab) — **not** Phase 5 |
| **Menu pause** | `InGameMenuState::Pause` + `AppState::Paused` / `PauseState::Menu` |
| **Sim tick pause** | `SimControlState.paused` — diagnostics / time strip (**P**) |

---

## Executive summary

| Slice | Verdict |
|:---|:---|
| **P5-SCAFFOLD** | **PASS** — Escape + confirm |
| **P5-PAUSE-001** | **CLOSED** — Bevy `pause_menu_bevy.rs` |
| **P5-DESIGN-001** | **CLOSED** — [`ui_p5_design_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_p5_design_signoff_v1.md) |
| **P5-SAVE-001** | **OPEN** (deferred) — wire Save to Wave S / save spine |
| **P5-SETTINGS-001** | **DEFERRED** — `InGameMenuState::Settings` unused |

**Does not block:** Stage 5 FULL_APP, Phase 3 minimap, UI-P2B egui retirement, industrial activation.

---

## Priority & queue position

| Rank | Typical lanes ahead | Policy |
|:---:|:---|:---|
| P0 | BQ-128-APPLY-001, S7B-PLAN-001 | Ship first |
| P1 | OPS-F01, WC-DEPTH-001, UI-P3 optional tails | Infra / polish |
| **P2** | **UI-P5-*** | This plan — schedule when P0/P1 clear |

---

## Master gate chain

```text
UI-P2B-GATE (no egui product shell in sim)           ☑
PLAY-01 (sim HUD defaults on enter)                  ☑
        │
        ▼
P5-SCAFFOLD — Escape egui pause + confirm            ☑ partial
        │
        ▼
P5-DESIGN-001 — pause menu mock / tokens             ☑ CLOSED
        │
        ▼
P5-PAUSE-001 — Bevy pause overlay                      ☑ CLOSED
        │
        ├─► P5-SAVE-001 — save/load wire               ☐ deferred
        └─► P5-SETTINGS-001 — settings surface         ☐ deferred
```

---

## Current implementation (**P5-SCAFFOLD — partial**)

| Component | Path | Status |
|:---|:---|:---|
| Escape toggle | `in_game_pause_menu.rs` `toggle_pause_menu_on_escape` | **DONE** |
| egui window | `pause_menu_egui_system` | **DONE** (transitional) |
| Confirm destructive nav | `pause_menu_confirm.rs` | **DONE** |
| UX mirror | `ux_orchestration.rs` `AppState::Paused`, `PauseState` | **DONE** |
| Resume / Quit / Main menu / WorldGen | menu choices + confirm | **DONE** |
| Save / Load | buttons present | **STUB** (log / flow redirect only) |

**Menu copy today:** *"Simulation paused (menu). Sim tick pause is separate (P)."*

**Choices (egui):** Resume · Save (stub) · Load (stub) · World Generator… · Return to Main Menu · Exit program.

---

## P5-DESIGN-001 — designer deliverable (**CLOSED**)

**Queue ID:** **UI-P5-DESIGN-001** — sign-off [`ui_p5_design_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_p5_design_signoff_v1.md) · implementation [`pause_menu_bevy.rs`](../../../src/gui/pause_menu_bevy.rs).

| # | Deliverable |
|:---:|:---|
| D1 | Centered pause card mock — vellum + wire tokens ([`palette_v2_tokens.md`](palette_v2_tokens.md)) |
| D2 | Button order + labels (match or amend current six actions) |
| D3 | Confirm modal pattern for WorldGen / Main menu (reuse copy or refine) |
| D4 | PLAY-01: pause does **not** expand command tray / WorldGen panels |
| D5 | Optional: link **P** sim pause indicator in ops strip (read-only hint) |

**Output path (proposed):** `assets/ui/simulation/pause_menu_spec_v1.png` + short sign-off row in planner doc appendix.

**Does not block** P5-SCAFFOLD maintenance; **blocks** Bevy migration aesthetic parity.

---

## P5-PAUSE-001 — Bevy pause overlay (**OPEN**)

### Goal

Replace **egui** pause window in **Simulation** with **Bevy UI** (`Node` + observers), keeping behavior parity with scaffold.

### Coder contract

| Rule | Requirement |
|:---|:---|
| Boundary | [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) — player-facing → Bevy |
| Authority | Menu sets `InGameMenuState` / `AppState` / `PauseState` only — no gameplay mutation |
| PLAY-01 | `apply_simulation_hud_defaults` unchanged; pause hides no required sim chrome permanently |
| egui | Remove `pause_menu_egui_system` from sim path; keep confirm logic in `pause_menu_confirm.rs` |
| Max files | **3** per PR — e.g. new `pause_menu_bevy.rs`, `in_game_pause_menu.rs`, `mod.rs` |

### Copy-paste — P5-PAUSE-001

```
Lane: UI-P5-PAUSE-001 — Bevy pause menu (Phase 5)
Read: docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md
      prompts/guides/ui_boundary_guide_v1.md
Prereq: UI-P2B closed (egui_pass_count_in_sim: 0)
First: Bevy-centered pause card; Escape toggle unchanged
Do NOT: new egui sim panels; break ux_orchestration bridge; Stage 5 predicates
Verify: cargo test -p proc_A_dine01 --lib stage5
Manual: Escape → menu → Resume; confirm WorldGen / Main menu still work
Witness: extend ui_shell_migration_live.json → pause_menu_bevy: true (when added)
```

### Acceptance

| # | Criterion |
|:---:|:---|
| P5-1 | Escape opens/closes pause in Simulation |
| P5-2 | No egui pass required for pause UI in sim |
| P5-3 | Destructive exits still use confirm modal |
| P5-4 | `stage5` lib tests green |

---

## P5-SAVE-001 — save / load (**OPEN**, deferred)

| Action | Today | Target |
|:---|:---|:---|
| **Save** | `info!` stub | `WorldSaveSpine` / bundle capture ([`wave_s_open.md`](../../../src/dev/wave_s_open.md)) |
| **Load** | Routes to main-menu load flow | Explicit in-game load picker (product TBD) |

**Coordinate with:** Wave S save bundle · BQ-128 (blueprints) — save menu is **not** blueprint editor.

---

## P5-SETTINGS-001 — settings (**DEFERRED**)

`InGameMenuState::Settings` exists in [`states.rs`](../../../src/engine/states.rs) — no Phase 5 MVP. Fold into Phase 5.5 or settings track when scoped.

---

## Session matrix

| Session | Pause menu |
|:---|:---|
| **Simulation** | **YES** — primary |
| **WorldGen / Editor** | Escape may route to generator/preview chrome — **do not** duplicate full sim menu |
| **Main menu** | N/A — use main menu flows |

---

## Witness (proposed)

| File | Field (future) |
|:---|:---|
| `debug_runs/ui_shell_migration_live.json` | `phase5.pause_menu_bevy: true` |
| `debug_runs/stage5_full_app_live.json` | optional `pause_menu_smoke` |

Until P5-PAUSE-001 lands, **no false green** — scaffold egui is not track exit.

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| egui pause in sim after P5-PAUSE-001 | UI-P2B policy |
| Pause menu writes simulation ECS | UX state transitions only |
| Disable Stage 5 spine for menu work | Lower priority lane |
| Confuse UI-P5 with P5_BR icon | Phase 4 petroleum |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-P5-PAUSE-001 (plan only) |
| Designer P5-DESIGN-001 | 2026-05-25 | **SIGNED — PASS** |
| Coder P5-PAUSE-001 | 2026-05-25 | **CLOSED** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.2 | 2026-05-25 | **P5-DESIGN-001** **CLOSED** — UI-P5-DESIGN-001 sign-off |
| v1.0.1 | 2026-05-25 | **UI-OH-P5-001** — [`ui_oh_p5_001_plan_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md); P5-PAUSE-001 **CLOSED** |
| v1.0.0 | 2026-05-25 | Phase 5 pause menu plan — lower priority P2 |
