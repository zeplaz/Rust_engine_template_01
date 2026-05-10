# Terrain config (`assets/config/terrain`)

Designer-edited tables for material / tag / rule unification. Engine load policy: [`material_unification_matrix_v1.md`](../../../prompts/matrix/terrain_biome/material_unification_matrix_v1.md) §2 · Bevy loader extensions: [`bevy_asset_config_migration_matrix_v1.md`](../../../prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md) (Terrain registry section) · Execution: [`bevy_asset_terrain_runbook_v1.md`](../../../prompts/guides/bevy_asset_terrain_runbook_v1.md).

| File | Role |
|:---|:---|
| `material_registry.example.json` | Seed materials (JSON); **fact tags only**; **`schema_version: 2`**; `properties` use **namespaced keys** (`facts.*`, `sim.*`, …) — see `material_tag_rule_system_v1.md` §4.1 |
| `material_registry.example.ron` | **Full mirror** of the JSON example (same `MaterialRegistryFile`); engine and `default.world_profile.ron` prefer `.ron` when present (`MaterialRegistry::load_from_json` / startup). Regenerate via ignored test `emit_material_registry_example_ron_fixture` in `src/terrain/material/registry.rs`. |
| `tag_registry.example.json` | Seed tags (JSON) — physical / ecological facts + pass-2 threshold names + pass-3 biome keys + material tag names |
| `tag_registry.example.ron` | **Full mirror** of the tag JSON example; loaders accept both extensions. |
| `material_rules.example.ron` | Seed rules (RON DSL) |
| `*.json` | Top-level **`schema_version: u32`** required when present |
