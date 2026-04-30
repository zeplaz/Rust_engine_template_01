# Tools UI — plugins & schedule `01`

## Patterns

- One **plugin per domain** (`ProductionToolsUiPlugin`, `WorldGenerationToolsUiPlugin`, future `FactionToolsUiPlugin` per `factions/faction_editor/02_ui_egui_panels.md`).
- **Init resource** for UI state + systems in `build()`; avoid coupling to unrelated `App` types.

## Ordering

- Document **relative order** to `EguiPrimaryContextPass` in central engine setup (`implementation_questions_v1.md` §3).
- **Keyboard:** egui `wants_keyboard` vs game bindings (`implementation_questions_v1.md` §4).

## Naming

- `*ToolsUiPlugin` for egui operator tools; `*InGamePlugin` for player-facing Bevy UI when split.
