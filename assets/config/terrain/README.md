# Terrain config (`assets/config/terrain`)

Designer-edited tables for material / tag / rule unification. Engine load policy: [`material_unification_matrix_v1.md`](../../../prompts/matrix/terrain_biome/material_unification_matrix_v1.md) §2 · Bevy loader extensions: [`bevy_asset_config_migration_matrix_v1.md`](../../../prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md) (Terrain registry section) · Execution: [`bevy_asset_terrain_runbook_v1.md`](../../../prompts/guides/bevy_asset_terrain_runbook_v1.md).

| File | Role |
|:---|:---|
| `material_registry.example.json` | Seed materials; copy to active name when bootstrapping |
| `tag_registry.example.json` | Seed tags |
| `material_rules.example.ron` | Seed rules (RON DSL) |
| `*.json` | Top-level **`schema_version: u32`** required when present |
