# UI Phase 0 panel mocks `v1`

Version: `1.0.2` (2026-05-24)  
**Theme:** [`design_theme.md`](design_theme.md) · **Playbook:** [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md)

Phase 0 defines **layout-only** mocks for the simulation HUD shell. Phase 2A wires read-only consumers; Phase 2B removes legacy F3/egui duplicates.

---

## Panel index

| ID | Panel | Host | Authority |
|:---|:---|:---|:---|
| **P1** | Ops strip — alerts zone | Bevy UI (`OperationsStripRoot`) | `ActiveMissions`, `OperationalTheaterSummary` |
| **P2** | Bottom context tray | Bevy UI (`ContextTrayRoot`) | `ContextTrayState` + `HudInfoLiveData` |
| **P3** | Map frame inset + minimap chrome | Bevy UI (`MapViewportFrameInset`, `MinimapChromeRoot`) | `SimulationMapViewportFill`, `MinimapShellState::last_window_rect` |
| **P4** | Left chrome — dual column | Bevy UI (`LeftContextRail` + `BuildRailRoot`) | `CommandLeftStackState` · `BuildStripState` |

---

## P1 — Ops strip alerts

- **Shape:** 22×22 px square badge (magenta wire) + `ALERTS` label.
- **Number:** mission count (`ActiveMissions::len()`), capped display `99+`.
- **Click:** `ContextTrayState::panel_state = Expanded`, tab `Alerts`.
- **Sources (read-only):** missions, theater threat slots, fracture (via `HudInfoLiveData` in tray body).

## P2 — Context tray

- **Tab bar:** 32px — Alerts | Intel | Logistics | Diag.
- **Body:** 96px when expanded; copy from `HudInfoLiveData`.
- **Chrome:** flat v2 tokens (`hud_chrome::flat_v2_tray_tab` for egui shells; Bevy tabs match palette).
- **Escape:** collapses unpinned tray (`panel_state::hud_panel_escape_collapse_system`).

## P3 — Map / minimap frame

- **Inset:** 4px inner wire on `SimulationMapViewportFill` (`MAP_FRAME_INSET_PX`).
- **Minimap chrome:** `MinimapChromeRoot` tracks egui minimap window rect each frame (logical px / scale).

## P4 — Left rail (Phase 2C **2C-B** — signed 2026-05-24)

**Designer choice:** **2C-B** — canonical **dual column** on absolute overlay; map hole stays full-width (occlusion documented).

### Column layout

| Node | Width | Role |
|:---|:---:|:---|
| `LeftContextRail` | **48px** | HUD glyph column `⏱` `⛭` `◎` `☰`; expand stack on press |
| `BuildRailRoot` | **52px** | Build tools (7 `ToolContext` slots); writes `BuildStripState` |
| `LeftContextStackBody` | **400px** | Narrative / objectives (expanded only) |
| `column_gap` | **6px** | Between visible columns on `CommandLeftStackOverlay` |

### Footprint on map (overlay only — does not inset `SimulationMapViewportFill`)

| `CommandLeftStackState` | Visible columns | **Footprint** |
|:---|:---|:---:|
| **Collapsed** (PLAY-01 default) | context 48 + build 52 | **~106px** |
| **Expanded** | build 52 + stack 400 | **~458px** |

Edge inset: overlay `left: 8px` (`CENTER_ROW_EDGE_PAD_PX`) — not counted in footprint.

### Visibility (`sync_command_left_stack_visibility`)

| State | LeftContextRail | BuildRailRoot | LeftContextStackBody |
|:---|:---|:---|:---|
| Collapsed | Visible | Visible | Hidden |
| Expanded | Hidden | Visible | Visible |

**Authority:** spawn in [`in_game_hud.rs`](../../../src/gui/in_game_hud.rs) · constants in [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs).

**Witness:** `debug_runs/ui_shell_migration_live.json` → `phase2c` block (`layout_option: "2C-B"`, width fields).

---

## Phase 2A closure (coder)

| Item | Status |
|:---|:---|
| Ops strip zones wired | `OpsStripZone` + click routing |
| Alert → tray expanded | `ops_strip_zone_click_system` |
| Intel → `MapCameraDesired` | `OpsStripIntelFocusRequest` |
| Minimap chrome aligned | `sync_minimap_chrome_root_system` |
| Witness JSON | `debug_runs/ui_shell_migration_live.json` |

**Not in 2A:** remove F3 editor egui / `BuildToolbox` (Phase 2B).

---

## Designer sign-off

See [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) · **UI-OH-D2-SIGN-001:** [`ui_oh_d2_signoff_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_oh_d2_signoff_record_v1.md). **Lane index:** [`docs/archive/2026-06-src-dev/plans/ui_overhaul_plan.md`](../../../docs/archive/2026-06-src-dev/plans/ui_overhaul_plan.md).
