# Faction editor — spec index

**Parent:** [`../faction_editor_tooling_matrix_v1.md`](../faction_editor_tooling_matrix_v1.md) (FE-01…FE-06)  
**Checklist:** [`../implementation_questions_v1.md`](../implementation_questions_v1.md)

Read in order:

| # | File | Contents |
|:---|:---|:---|
| 00 | [`00_scope.md`](00_scope.md) | Goals, out-of-scope, MP note |
| 01 | [`01_data_model.md`](01_data_model.md) | `FactionBlueprint`, tags, diplomacy DTOs, IDs |
| 02 | [`02_ui_egui_panels.md`](02_ui_egui_panels.md) | Tools UI: panels, plugin pattern, hotkeys |
| 03 | [`03_persistence.md`](03_persistence.md) | Scenario vs save vs live overlay |
| 04 | [`04_validation.md`](04_validation.md) | Naming, colors, scale without hard caps |
| 05 | [`05_integration_tests.md`](05_integration_tests.md) | Headless round-trip, regression hooks |

**Repo patterns to mirror:** `src/systems/production/tools_ui.rs` (`ProductionToolsUiPlugin`), `src/gui/agent_permissions_ui.rs` (egui window + resources), `src/gui/ui_windows.rs` (window wiring if present).
