# Production & economy — spec index

**Deep dive:** [`../power_damage_ui_persistence_v1.md`](../power_damage_ui_persistence_v1.md)  
**Checklist:** [`../implementation_questions_v1.md`](../implementation_questions_v1.md)  
**Matrix:** `prompts/matrix/production/production_migration_matrix_v1.md`, `prompts/matrix/serialization/`, `prompts/matrix/assets/`

| # | File | Contents |
|:---|:---|:---|
| 00 | [`00_scope.md`](00_scope.md) | Domains: concrete, power, aluminum, manufacturing, damage |
| 01 | [`01_data_model_manifest.md`](01_data_model_manifest.md) | DTOs, `ProductionManifest`, serializers |
| 02 | [`02_tools_ui_production.md`](02_tools_ui_production.md) | `ProductionToolsUiPlugin`, inspectors |
| 03 | [`03_persistence_snapshots.md`](03_persistence_snapshots.md) | Binary chunks, repair jobs, versioning |
| 04 | [`04_power_damage_repair.md`](04_power_damage_repair.md) | Grid, phases, damage/repair queues |
| 05 | [`05_integration_tests.md`](05_integration_tests.md) | Round-trip production state |
| 06 | [`06_power_plants_data_scripting_v1.md`](06_power_plants_data_scripting_v1.md) | Plant archetypes, research, JSON vs Lua vs expressions, tool-editable params |

**Code:** `src/entities/production/*`, `src/systems/production/*`, `src/entities/damages.rs`, `src/gui/*` (HUD vs tools split per power_damage doc).
