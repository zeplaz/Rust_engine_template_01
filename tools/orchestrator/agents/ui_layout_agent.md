# ui_layout_agent

**Lane:** HUD shell, map_view spine, egui pass ordering, world preview UI.

## Read first

- `tools/orchestrator/runbooks/ui_pipeline.md`
- `knowledge/map_view_spine.json`

## Rules

- Respect `MapViewPlugin` system ordering (ResolveViewport → PostUpdate consumers → EguiPrimaryContextPass).
- Map view presentation is **STABLE** (`map_view/mod.rs`) — coordinate with `viewport_cleanup_agent` on viewport authority only; do not re-open presentation spine without planner scope.

## STAGE5

- FINISH-UX rows in `stage5_finish_todos.rs`.

## F-10 — simulation HUD focus order (Phase 2A)

Tab / keyboard focus order for the **Bevy simulation shell** (top → bottom, outer → inner). Presentation-only; does not change `BuildStripState` authority.

| Order | Region | Root / component | Notes |
|:---:|:---|:---|:---|
| 1 | Operations strip | `OperationsStripRoot` | TIME → ALERTS → INTEL → WX → PWR → ▼ TRAY (left → right) |
| 2 | Context tray tabs | `ContextTrayRoot` | Alerts · Intel · Logistics · Diagnostics |
| 3 | Context tray body | `ContextTrayBodyRoot` | Scroll when Expanded/Pinned |
| 4 | Build tool rail | `BuildRailRoot` | Rd/Rl/Ut/… top → bottom; toggles `BuildStripState` |
| 5 | Left context rail | `LeftContextRail` | Icon grid (editor-adjacent; collapsed in PLAY-01) |
| 6 | Map viewport hole | `SimulationMapViewportFill` | World picks pass through when focused |
| 7 | Minimap chrome | `MinimapChromeRoot` | Inset ≤2px vs texture (`F-09`) |

**Escape:** collapses context tray when not pinned (`collapse_context_tray_on_escape`).

**Capture witness (1.6):** `--test capture` replays ALERTS + INTEL + Escape once at sim frame ≥30 (`replay_ui_shell_witness_interactions_system`) so `ui_shell_migration_live.json` interaction flags commit without manual clicks.

**Verify:** `cargo run -p proc_A_dine01 -- --test frame` — Tab through strip zones; Escape from expanded tray.
