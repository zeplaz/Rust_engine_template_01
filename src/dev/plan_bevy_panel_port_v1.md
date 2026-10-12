# PLAN-BEVY-PANEL-PORT-v1 — Bevy panel port (post egui drop)

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-BEVY-PANEL-PORT-v1** |
| **Date** | 2026-10-10 |
| **Owner** | `@coder` (impl) · decision packet (session scope) |
| **Status** | **CLOSED** (Session 3 done · residual **P9 terrain_conform** / PHASE2-BUILD-20 Open) |
| **Witness** | `debug_runs/bevy_panel_port_live.json` |
| **Parent** | [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) (Cargo drop CLOSED) |
| **Boundary** | Bevy `Node` trees only — do **not** re-add `bevy_egui` |

## Goal

Replace retired empty `draw_*_egui` / panel stubs with honest Bevy UI roots for the session ship set, and stop witness fake-green from `include_str!(…).contains("draw_*_egui")`.

## Session 1 ship (done)

| ID | Slice | Done when |
|:---|:---|:---|
| PORT-0-WITNESS-HONESTY | Zero `include_str!(…).contains("draw_*_egui")` in `src/` | Honest markers (Bevy root / mesh sync / `PanelPresence`); fake-green → honest false OK for DEFER |
| PORT-F3-DIAGNOSTICS | `DiagnosticsBevyRoot` ← `DiagnosticsUiState`; F3 spawn/despawn | Sections collapse in Simulation (`sections_default_open`) |
| PORT-F1-KEYBINDINGS | Bevy list from `InputBindings` / `KeybindingsUiState` | Toggle scheduled; pointer gate blocks map pick while open |
| PORT-LOGISTICS-ORDERS-PANEL | Orders + targets Bevy panels ← `LogisticsOrdersSnapshotRes` | Edits via `draft_commands` only; change-detected rebuild |
| PORT-LOGISTICS-OVERLAY | `sync_world_map_orders_meshes` in `world_map_network_mesh` | Consumes `build_orders_overlay()` |

## Session 2 ship (done)

| ID | Slice | Done when |
|:---|:---|:---|
| PORT-EDITOR-DOCK | `EditorDockBevyRoot` ← `HudDockRegistry` / CommandShell + OverlayTray + minimized strip + side status rail | `product_shell_active` (Editor, ¬WorldGen); `PanelPresence.editor_dock_root` |
| PORT-CONSTRUCTION-TOOLBOX | `BuildToolboxBevyRoot` ← `ActiveBuildTool` + `BuildingDefinitionRegistry` | Tool-select intents only; `ensure_build_toolbox_shell_visible` dock slot; `toolbox_panel` true when root spawned |
| PORT-MAP-EDITOR-CHROME | `MapEditorBevyShell` palette \| minimap (`MapEditorMapTexture` → `ImageNode` / Text fallback) \| scenario (F10) | `map_editor_chrome_active`; mesh/visual_authority world paint unchanged |

## Session 3 ship (done)

| ID | Slice | Done when |
|:---|:---|:---|
| PORT-ROAD-POPUP-BEVY | `RoadToolPopupBevyRoot` under `product_shell_active` | Build/Cancel/snap/upgrade via `commit_road_path_to_queue` / `enqueue_road_upgrade`; `road_popup` true when root spawned |
| PORT-STAGED-MULTILINE | Sim tray Build tab + `staged_placements_tray_lines` | Parametric + staged multi-line readout; no execute outside funnel |
| PORT-DEV-CHROME | Faction / AI explain / Pressure composer / Assembly QC Bevy roots | `PanelPresence` + `apply_dev_bevy_panel_pointer_blocks_system` |
| PORT-MASS-STUB-PRUNE | Empty `draw_*_egui` removal | Compile green; no fake-green `include_str` predicates |
| PORT-PRODUCT-SHELL-ALIAS-DROP | Drop `product_egui_shell_*` | All sites → `product_shell_*` |
| PORT-P9-TERRAIN-CONFORM-DOC | Document-only | `terrain_conform` stays honest false; PHASE2-BUILD-20 Open |

Gate: `product_shell_active` only (alias removed).

## Residual (not program-blocking)

- **P9 / PHASE2-BUILD-20** — `terrain_conform` honest false until live `height_grid` (not height_norm_stub)

## Authority

- Pointer: `SimulationMapPointerGate` (+ floating panel / editor Bevy roots → `chrome_blocks`)
- Presence: `PanelPresence` resource (root entity flags)
- Orders writes: `LogisticsOrderRequest` only after `draft_commands`
- Construction toolbox: tool-select on `ActiveBuildTool` only — no execute outside construction funnel
- Road popup: commit only via construction funnel
- World paint: `Mesh2d` ribbons / visual_authority — never egui strokes

## Patterns

- `diagnostics_ui.rs` (`DiagnosticsBevyRoot` + `FocusPolicy::Block`)
- `build_toolbox.rs` (`BuildToolboxBevyRoot`)
- `sim_road_power_sheets_bevy.rs` (sim road sheet actions)
- `simulation_shell_phase2.rs` Node trees
