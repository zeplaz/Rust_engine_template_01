# UI Phase 2B — egui sim-shell retirement gate plan `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@planner` (architecture) · **`UI-P2B-001`** `@coder` (implementation) |
| **Status** | **CLOSED** in repo (2026-05-24) — this doc is the **authoritative gate spec** + **Phase 2B+ hardening** queue |
| **Witness** | [`debug_runs/ui_shell_migration_live.json`](../../../debug_runs/ui_shell_migration_live.json) · profile **`UI_SHELL_MIGRATION_2B`** |
| **Boundary** | [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) |
| **Coder queue (archive)** | [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md) § Sprint 2–3 |
| **Phase 2B+ coders** | [`vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md) *(parallel product lane)* · **this doc § Phase 2B+** |

**Planner checklist:** Embodies **UI-P2B-PLAN** (run_if, dock allowlist, witness metric, files, acceptance, rollback). No Rust in this deliverable.

---

## Summary

**Phase 2B** retires the **egui product shell** in **`BaseState::Simulation`**: gameplay HUD is **Bevy-native** (`simulation_shell_phase2.rs`, `in_game_hud.rs`). Editor keeps **egui** floating docks for Construction, overlays, transmission, and the legacy side status rail. Simulation allows only **F3 diagnostics** and **editor/world-gen tooling** registered outside the product-shell root — not duplicate command/minimap/build-toolbox egui.

Authority: **`product_egui_shell_active`** owns whether **`hud_product_shell_egui_root`** runs; **`simulation_session`** owns dock slot suppression and witness flags on sim enter/enforce.

---

## Current problems (pre-2B — resolved)

| Problem | Resolution |
|:---|:---|
| Duplicate **BuildToolbox** egui + Bevy build rail in sim | `build_toolbox_egui_dock_active` + `SIM_SUPPRESSED_FLOATING_SHELLS` |
| Duplicate **left status rail** (egui vs Bevy context rail) | `side_status_rail_egui_active` + collapsed layout in sim |
| Floating command / overlay / transmission egui over Bevy chrome | `product_egui_shell_active` false in sim; `suppress_simulation_floating_shell_slots` |
| No measurable gate | `egui_pass_count_in_sim` + `phase2b_closed` in live JSON |

**Remaining risks (Phase 2B+ — not blockers for 2B close):**

| Risk | Mitigation slice |
|:---|:---|
| `egui_pass_count` is **lifetime cumulative** — editor passes inflate counter before sim proof | **UI-P2B-002** reset on `OnEnter(Simulation)` |
| `draw_hud_side_status_panel_egui` called unconditionally **inside** gated root (defense OK today; fragile if gate drifts) | **UI-P2B-003** inner `side_status_rail_egui_dock_active` guard |
| Minimap **egui texture** path vs **Bevy GPU** minimap (Phase 3) — sim should not depend on egui minimap | Already gated; document in playtest guide |
| Regression if new egui system uses `in_simulation_or_editor` without 2B review | **UI-P2B-004** lint / checklist in PR template |

---

## Target architecture

```text
BaseState::Simulation
  ├─ Bevy (authoritative player shell)
  │    OperationsStripRoot · ContextTrayRoot · MapViewportFrameInset
  │    MinimapChromeRoot (+ GPU compositor texture when enabled)
  │    LeftContextRail · BuildRailRoot → BuildStripState
  │
  └─ egui (allowed, not product shell)
       DiagnosticsUiPlugin (F3)     run_if: in_simulation_or_editor
       WorldGen / Preview / MapEditor (when AppState permits)
       Agent permissions, production tools (editor tooling table)

BaseState::Editor  (+ AppState != WorldGen)
  └─ egui product shell
       hud_product_shell_egui_root   run_if: product_egui_shell_active
       ├─ side status rail (2B-03)
       ├─ minimap egui texture dock (editor; sim uses Bevy chrome)
       └─ floating shells per dock registry + SIM_SUPPRESSED list (sim: forced closed)
```

**Authority map**

| Concern | Owner | Consumers |
|:---|:---|:---|
| Session profile (sim vs editor) | `State<BaseState>` + `State<AppState>` | `ui_gates.rs`, `simulation_session.rs` |
| Product egui pass on/off | `product_egui_shell_active` | `hud_root_tick.rs` schedule |
| Per-widget dock bodies | `shell_framework::{floating_*, product_shell_widget_egui_dock_active}` | Individual drawers + construction plugin |
| Dock slot visibility in sim | `suppress_simulation_floating_shell_slots` | `HudDockRegistry` |
| Proof / regression | `ProductShellDiagnostics` + `UiShellMigrationWitness` | `build_proof_payload` → `ui_shell_migration_live.json` |

---

## `run_if` strategy

### Layer 1 — Master product-shell gate (schedule)

| System | Schedule | `run_if` | Sim | Editor (non–WorldGen) |
|:---|:---|:---|:---:|:---:|
| `hud_product_shell_egui_root` | `EguiPrimaryContextPass` | `product_egui_shell_active` | **off** | **on** |

**Definition** (`ui_gates.rs`):

```text
product_egui_shell_states_active(base, app) :=
  base == Editor  AND  app != WorldGen
```

**Const helper** for tests / `shell_framework`: `product_egui_shell_base_active(base) := base == Editor`.

**WorldGen exclusivity:** While `AppState::WorldGen`, editor product shell must **not** run (avoids double labels with World Generator + Preview egui).

### Layer 2 — Per-surface gates (inside or beside root)

| Surface | Gate function | Sim | Editor |
|:---|:---|:---:|:---:|
| Left egui status rail | `side_status_rail_egui_active` → aliases `product_egui_shell_active` | off | on |
| BuildToolbox egui window | `build_toolbox_egui_dock_active` | off | on |
| Minimap egui texture dock | `minimap_egui_texture_dock_active` | off | on* |
| Build toolbox drawer | `draw_build_toolbox_egui` | `product_egui_shell_active` | on / off |
| Floating shell body | `floating_product_shell_egui_active(id, base)` | off** | on |

\* Sim minimap presentation uses **Bevy** `MinimapChromeRoot` + optional GPU compositor; not egui texture dock.  
\** Suppressed widgets also fail `sim_suppresses_floating_shell`.

### Layer 3 — Sim-allowed egui (outside product shell)

| Surface | Schedule | `run_if` | Notes |
|:---|:---|:---|:---|
| **Diagnostics F3** | `EguiPrimaryContextPass` | `in_simulation_or_editor` | Listed in `EGUI_SIM_SHELL_WIDGETS` as `Diagnostics_F3` |
| World gen UI | `EguiPrimaryContextPass` | WorldGen plugin rules | `Editor_tools` bucket |
| Map editor chrome | `EguiPrimaryContextPass` | `map_editor_chrome_active` | **off** in sim / InGame / Paused / WorldGen |
| Debug viewport overlay | `EguiPrimaryContextPass` | own plugin | After product shell when both run |

**Rule for new egui:** If it draws **player HUD** or duplicates Bevy shell → must use **`product_egui_shell_active`** or a stricter gate. If it is **dev tooling** → `in_simulation_or_editor` is acceptable with designer note.

### Layer 4 — Enforcement systems (sim only)

| System | Schedule | Role |
|:---|:---|:---|
| `apply_simulation_hud_defaults` | `OnEnter(BaseState::Simulation)` | Collapse layout, suppress dock slots, sync witness |
| `enforce_simulation_product_egui_gates` | `Update` (sim) | Re-close floating shells if something reopens them |
| `enforce_world_gen_chrome_closed_in_simulation` | `Update` | Dismiss WorldGen/Preview windows in play |

---

## Dock allowlist

### A — Floating shells **suppressed** in simulation

**Source:** `shell_framework::SIM_SUPPRESSED_FLOATING_SHELLS`

| `ProductShellWidgetId` | Label | Bevy replacement in sim |
|:---|:---|:---|
| `OverlaysPanel` | Overlay shell | Ops strip + context tray (Bevy) |
| `OverlayTray` | Overlays tray | Same |
| `CommandShell` | Command | Collapsed; Bevy ops/context |
| `BuildToolbox` | Construction | **BuildRailRoot** + `BuildStripState` |
| `IntelTimeline` | Intel | Context tray Intel tab |
| `Explainability` | Explain | Deferred / dev |
| `ConstructionQueue` | Pending builds | Construction flow (non-egui or rail) |
| `Transmission` | Transmission | Bevy / future lane |

**Mechanism:** `suppress_simulation_floating_shell_slots` sets `visible=false`, `minimized=true`, `detached=false` for each.

**Witness:** `witness.floating_egui_shells_gated` ← `simulation_floating_shells_gated(dock)`.

### B — Dock slots **not** in suppression list

| Widget | Sim behavior |
|:---|:---|
| `Minimap` | Slot may stay **visible** for Bevy chrome; **egui body** inactive (`minimap_egui_texture_dock_active` false) |

### C — Witness allowlist string table

**Source:** `shell_framework::EGUI_SIM_SHELL_WIDGETS`

| String ID | Meaning |
|:---|:---|
| `Diagnostics_F3` | `diagnostics_ui_system` (F3 devtools) |
| `Editor_tools` | World-gen, preview, map editor, agent tools — not product shell |

**Not product shell:** Anything under `hud_product_shell_egui_root` when `product_egui_shell_active` is true.

### D — Layout collapse (egui-driven chrome state)

**`collapse_simulation_floating_shell_layout`** sets `HudPanelState::Collapsed` on:

- `layout.overlay_tray_state`, `status_side_panel_state`, `command_tray_state`, `intel_timeline_state`, `command_table_state`
- `tray.tray_panel_state`
- `transmission.panel_state`

**Witness:** `witness.side_status_rail_egui_gated` ← `status_side_panel_state == Collapsed`.

---

## `egui_pass_count_in_sim` — definition

### What it measures

| Field | Type | Source |
|:---|:---|:---|
| `ProductShellDiagnostics.egui_pass_count` | `u64` | Incremented once per **`hud_product_shell_egui_root`** invocation |
| JSON `egui_pass_count_in_sim` | number | Snapshot of **counter at proof write time** |
| JSON `phase2b_closed` | bool | See formula below |

### Increment site

```text
hud_product_shell_egui_root (EguiPrimaryContextPass)
  → panels.shell_diag.record_egui_pass()   // first line of body work
  → only when product_egui_shell_active == true
```

**Therefore:** In a frame where the app is in **Simulation**, `product_egui_shell_active` is false → **root does not run** → counter **does not increase** on that frame.

### `phase2b_closed` formula

From `simulation_shell_phase2::build_proof_payload`:

```text
phase2b_closed :=
  egui_pass_count_in_sim == 0
  AND witness.build_toolbox_egui_gated
  AND witness.side_status_rail_egui_gated
  AND witness.floating_egui_shells_gated
```

### Interpretation caveats (Phase 2B+)

| Caveat | Detail |
|:---|:---|
| **Not per-frame sim counter** | Field name says `_in_sim` but implementation is **global cumulative** since app start |
| **Editor session inflates count** | Entering editor before sim proof can leave `egui_pass_count > 0` while `phase2b_closed` still passes **if** proof taken only after sim with no editor frames — fragile |
| **Recommended hardening** | Reset `egui_pass_count` to `0` in `apply_simulation_hud_defaults` (UI-P2B-002) and/or add `egui_pass_count_sim_session` |

### Operational proof recipe

1. Launch → enter **Simulation** (world-gen complete).
2. Do **not** return to editor product shell before witness flush.
3. Run `cargo run -p proc_A_dine01 --release -- --test visual` or frame harness with witness replay.
4. Expect `egui_pass_count_in_sim: 0`, `phase2b_closed: true`, `egui_sim_shell_widgets: ["Diagnostics_F3","Editor_tools"]`.

Toggle **F3** — allowed; must **not** increment product-shell counter.

---

## Implementation phases

### Phase 2B — **UI-P2B-001** (CLOSED)

| Sprint | IDs | Goal |
|:---|:---|:---|
| **2B-Build** | 2B-01, 2B-04 | Bevy build rail authoritative; hide egui left stack when collapsed |
| **2B-Dedupe** | 2B-02, 2B-03 | Gate BuildToolbox + side status rail; enforce dock suppression |

### Phase 2B+ — follow-up coder slices (OPEN)

| ID | Goal | Owner |
|:---|:---|:---|
| **UI-P2B-002** | Reset `egui_pass_count` on sim enter; optional `egui_pass_count_sim_only` witness field | @coder B |
| **UI-P2B-003** | Guard `draw_hud_side_status_panel_egui` with `side_status_rail_egui_dock_active` | @coder A |
| **UI-P2B-004** | Document + PR checklist: no new `in_simulation_or_editor` HUD egui | @coder / docs |
| **UI-P2B-005** | Smoke: F3 on in sim does not open product shells; `--test demo` build rail only | @coder QA |

---

## UI-P2B-001 — file list + acceptance (@coder)

### Files touched (≤3 per step — historical)

| Step | Files | Change |
|:---:|:---|:---|
| **2B-01** | `src/gui/hud/simulation_shell_phase2.rs` | `build_rail_tool_click_system` → `BuildStripState` |
| **2B-02** | `src/gui/hud/simulation_session.rs`, `src/gui/hud/shell_framework.rs` | `suppress_simulation_floating_shell_slots`, dock helpers |
| **2B-03** | `src/gui/ui_gates.rs`, `src/gui/hud/hud_root_tick.rs` | `product_egui_shell_active` on root; `side_status_rail_egui_active` on panel toggle |
| **2B-04** | `src/gui/in_game_hud.rs` | `sync_command_left_stack_visibility` when tray collapsed |
| **Witness** | `src/gui/hud/simulation_shell_phase2.rs` | `build_proof_payload`, `write_ui_shell_migration_live_proof_system` |
| **Construction** | `src/construction/mod.rs` | `draw_build_toolbox_egui.run_if(product_egui_shell_active)` |
| **Side panel** | `src/gui/hud/hud_side_status_panel.rs`, `src/gui/hud/panel_state.rs` | Rail gated |
| **Diagnostics** | `src/gui/diagnostics_ui.rs` | F3 uses `in_simulation_or_editor` only |

### Read-only authority (do not duplicate)

| File | Role |
|:---|:---|
| `src/gui/hud/hud_root_tick.rs` | Single egui product pass; `record_egui_pass` |
| `src/gui/hud/shell_diagnostics.rs` | Counter resource |
| `src/gui/hud/dock_shell.rs` | Minimized strip, command shell egui |
| `src/construction/build_toolbox.rs` | Editor Construction window body |
| `debug_runs/ui_shell_migration_live.json` | Green witness reference |

### Acceptance criteria — **UI-P2B-001**

| # | Criterion | Verify |
|:---:|:---|:---|
| A1 | `cargo test -p proc_A_dine01 --lib simulation_shell_phase2` green | unit |
| A2 | `cargo test -p proc_A_dine01 --lib stage5` green | regression |
| A3 | `phase2b_closed: true` in `ui_shell_migration_live.json` | witness |
| A4 | `egui_pass_count_in_sim: 0` at proof | witness |
| A5 | `witness.build_toolbox_egui_gated: true` | witness |
| A6 | `witness.side_status_rail_egui_gated: true` | witness |
| A7 | `witness.floating_egui_shells_gated: true` | witness |
| A8 | In sim: **no** Construction egui window; build rail sets tools | `--test demo` manual |
| A9 | In sim: **no** duplicate left egui status rail; Bevy context rail visible | visual |
| A10 | In editor: Construction + product shells still work | editor session |
| A11 | F3 diagnostics still toggles in sim | F3 key |

### Copy-paste — **UI-P2B-001** (archive)

```
Lane: UI-P2B-001 — egui sim-shell retirement (PLAY-01 Phase 2B)
Read: prompts/guides/ui/ui_phase2b_egui_gate_plan_v1.md
      ui_boundary_guide_v1.md
First: confirm product_egui_shell_active on hud_root_tick; suppress SIM_SUPPRESSED_FLOATING_SHELLS in simulation_session
Do NOT: add new egui HUD in sim via in_simulation_or_editor alone
Verify: cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5
        refresh debug_runs/ui_shell_migration_live.json (phase2b_closed)
```

---

## ECS / schedule plan (egui-related)

```text
OnEnter(BaseState::Simulation)
  → apply_simulation_hud_defaults
      → suppress_simulation_floating_shell_slots
      → sync_simulation_egui_shell_gate_witness

Update (Simulation)
  → enforce_simulation_product_egui_gates
  → enforce_world_gen_chrome_closed_in_simulation

EguiPrimaryContextPass
  → hud_product_shell_egui_root          [run_if: product_egui_shell_active]
  → diagnostics_ui_system              [run_if: in_simulation_or_editor]
  → draw_build_toolbox_egui              [run_if: product_egui_shell_active]
  → world_gen / preview / map_editor     [AppState-specific — not product shell]
```

**Bevy shell** (no egui): `SimulationShellPhase2Plugin` systems in `Update` / layout — separate from egui pass.

---

## Diagnostics required

| Artifact | Fields / purpose |
|:---|:---|
| `debug_runs/ui_shell_migration_live.json` | `phase2b_closed`, `egui_pass_count_in_sim`, `egui_sim_shell_widgets`, `witness.*`, `backends.legacy_egui_phase2b` |
| `UiShellMigrationWitness` | Runtime flags synced in `simulation_session` |
| `ProductShellDiagnostics` | `egui_pass_count` for proof |
| Unit tests | `shell_framework::phase2b_product_egui_shell_editor_only`, `simulation_session::simulation_egui_gate_witness_sync` |
| `build_proof_payload` test | `egui_pass_count_in_sim == 0` with default diag |

**Refresh command:**

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Rollback path

### Triggers (when to rollback)

| Trigger | Action |
|:---|:---|
| `phase2b_closed` false after release | Revert UI-P2B-001 commit set; restore prior `hud_root_tick` run_if |
| Build rail stops writing `BuildStripState` | Revert 2B-01 only; keep gates |
| Editor Construction broken | Fix `product_egui_shell_active` WorldGen guard — do not disable entire 2B |

### Rollback steps (safe order)

1. **Feature flag (preferred hotfix):** Add `const BYPASS_SIM_EGUI_GATES: bool` in `ui_gates.rs` — `product_egui_shell_active` returns true for sim when flag set (dev only). *Not in repo today; add only if emergency.*
2. **Git revert** slices in reverse order:
   - Witness-only commits (safe)
   - `enforce_simulation_product_egui_gates` (restores egui shells in sim)
   - `product_egui_shell_active` on `hud_root_tick` (restores full product pass in sim)
3. **Partial rollback — editor-only Construction in sim (violates PLAY-01):** Remove `BuildToolbox` from `SIM_SUPPRESSED_FLOATING_SHELLS` + stop `enforce_simulation_product_egui_gates` for toolbox only — requires designer sign-off ([`ui_construction_playtest_v1.md`](../../../src/dev/ui_construction_playtest_v1.md) § C).

### What rollback must preserve

| Must keep | Reason |
|:---|:---|
| Bevy `BuildRailRoot` | Gameplay construction in sim |
| `BuildStripState` authority | Single tool selection writer |
| Stage 5 spine tests | Unrelated to egui retirement |

### Post-rollback verification

```powershell
cargo test -p proc_A_dine01 --lib stage5 simulation_shell_phase2
```

Expect `phase2b_closed: false` until gates re-applied.

---

## Edge cases

| Case | Expected behavior |
|:---|:---|
| `AppState::WorldGen` + `BaseState::Editor` | Product shell **off**; world-gen egui only |
| `AppState::InGame` / `Paused` | Map editor chrome **off**; sim gates apply when `BaseState::Simulation` |
| Reopen dock via Wave S restore | `enforce_simulation_product_egui_gates` re-closes on next frame |
| Minimap drag in editor | egui minimap window OK; sim uses Bevy chrome rect |
| `MINIMAP_GPU_COMPOSITOR=1` | Minimap texture backend `bevy_ui_gpu`; still no egui product pass in sim |
| Multiview / minimap view | Fire/water particles use view cull — separate from 2B |
| F3 diagnostics open in sim | Allowed; does not increment `egui_pass_count` |

---

## Open questions

| ID | Question | Default until answered |
|:---|:---|:---|
| Q1 | Rename witness field to `product_shell_egui_pass_count`? | Keep name; document cumulative semantics |
| Q2 | Should sim allow **ConstructionQueue** egui ever? | **No** — rail + HUD only |
| Q3 | Phase 3 GPU minimap — remove `Minimap` from dock registry in sim entirely? | Keep slot for chrome; body gated |
| Q4 | Automate `phase2b_closed` in CI from headless harness? | Manual visual + JSON refresh today |

---

## Phase 2B+ — @coder next work (post-close hardening)

Use when **UI-P2B-001** is done but witnesses or UX still fail operator smoke.

| Priority | Slice | Coder | First file |
|:---:|:---|:---|:---|
| P1 | **UI-P2B-002** witness counter reset | B | `shell_diagnostics.rs` + `simulation_session.rs` |
| P2 | **UI-P2B-003** side rail inner guard | A | `hud_root_tick.rs` |
| P2 | **UI-P2B-005** demo/frame smoke script in runbook | either | `ui_construction_playtest_v1.md` |
| P3 | **UI-P2B-004** egui HUD lint note in AGENTS.md | docs | `AGENTS.md` |

**Parallel product (disjoint):** `UI-WP-LAYOUT-001`, `IND-E01`, VFX Phase 2 — see [`coder_execution_plan_v1.md`](../../../src/dev/coder_execution_plan_v1.md).

---

## Document index

| Doc | Role |
|:---|:---|
| [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) | Bevy vs egui decision rule |
| [`ui_overhaul_plan.md`](../../../src/dev/ui_overhaul_plan.md) | Phase 2B CLOSED row |
| [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md) | Sprint 2B task breakdown |
| [`ui_construction_playtest_v1.md`](../../../src/dev/ui_construction_playtest_v1.md) | Where build menus went |
| [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) | P1–P4 SIGNED |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial gate plan — UI-P2B-PLAN checklist; 2B closed + 2B+ queue |
