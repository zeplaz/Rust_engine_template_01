# World, terrain & theatre — spec index

**Scope:** **World** lifecycle, **generation**, **terrain/biome/hydrology**, **political** territory, **cities** (procgen + AI growth), and **coupling** to units/logistics (buildings, vehicles, drones, ships, cargo, weapons).  
**Existing topic files** (still canonical for depth): [`../chunks_streaming_v1.md`](../chunks_streaming_v1.md), [`../simulation_lod_v1.md`](../simulation_lod_v1.md), [`../hydrology_v1.md`](../hydrology_v1.md), [`../terrain_tools_brushes_v1.md`](../terrain_tools_brushes_v1.md), [`../tile_sprites_v1.md`](../tile_sprites_v1.md), [`../composite_style_worldgen_v1.md`](../composite_style_worldgen_v1.md) (layered fields + preview), [`../material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md) (registry, tags, rules).  
**Checklist:** [`../implementation_questions_v1.md`](../implementation_questions_v1.md)  
**Matrix:** `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md` · **Preview integration:** `prompts/matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md` · **Material unification:** `prompts/matrix/terrain_biome/material_unification_matrix_v1.md`

| # | File | Contents |
|:---|:---|:---|
| 00 | [`00_world_terrain_political_scope.md`](00_world_terrain_political_scope.md) | Umbrella: world ↔ sim ↔ politics |
| 01 | [`01_world_generation_pipeline.md`](01_world_generation_pipeline.md) | Seeds, params, editors, determinism |
| 02 | [`02_terrain_hydrology_worldgen.md`](02_terrain_hydrology_worldgen.md) | Heightfield, biome, rivers, chunk bind |
| 03 | [`03_political_territory.md`](03_political_territory.md) | Claims, borders, diplomacy on map |
| 04 | [`04_cities_ai_settlements.md`](04_cities_ai_settlements.md) | Procedural cities, AI urban planner |
| 05 | [`05_logistics_settlement_coupling.md`](05_logistics_settlement_coupling.md) | Units, supply, ports, hubs |

**Code:** `src/terrain/`, `src/bevysubengines/world_generator_plugin.rs`, `src/gui/editor/world_gen_ui.rs`, `src/terrain/generation/*`.

**Cross:** `factions/spec/` (political data model), `strategic_platforms/spec/` (platforms), `navigation/spec/`, `production_economy/spec/`.

**Desktop:** Asset editor **World Gen** page and tuning JSON: [`tools_ui/spec/08_world_gen_desktop_tool_v1.md`](../../tools_ui/spec/08_world_gen_desktop_tool_v1.md). **Materials / Tags / Rules:** same spec + `terrain_registry_pages.py`.
