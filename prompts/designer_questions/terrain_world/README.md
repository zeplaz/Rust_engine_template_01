# Designer Q — world, terrain & theatre

**Paired matrix:** `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md` · **Layered gen + preview:** [`composite_style_worldgen_v1.md`](composite_style_worldgen_v1.md) + [`prompts/matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md`](../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md) · **Materials / tags / rules:** [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md) + [`prompts/matrix/terrain_biome/material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) · **Paired runbook queue:** [`../../guides/terrain_paired_runbooks_queue_v1.md`](../../guides/terrain_paired_runbooks_queue_v1.md)  
**Related:** `prompts/matrix/serialization/` (chunk saves), `prompts/matrix/assets/` (textures, RON), **`prompts/designer_questions/strategic_platforms/`** (theatre / fires / EW need terrain + LOD), **`factions/faction_editor/`** (political stance + territory when wired).

> **Prompt use:** Ground answers in `src/terrain/`, `src/gui/editor/` symbols · cite briefly · paired matrix is boundary truth · `ASK:` for sim LOD numbers.

## Structured spec (world, gen, terrain, politics, cities, units)

| File / folder | Role |
|:---|:---|
| **`spec/README.md`** | **Start here** — umbrella: world lifecycle, gen, terrain/hydro, **political territory**, **cities & AI planners**, **logistics/units** (buildings, vehicles, drones, ships, cargo, weapons) |
| `spec/00_world_terrain_political_scope.md` … `spec/05_logistics_settlement_coupling.md` | `00`–`05` stubs + cross-links |

## Topic files (depth)

| File | Role |
|:---|:---|
| `chunks_streaming_v1.md` | Chunk grid, interest orbs, server vs client, bookmarks |
| `simulation_lod_v1.md` | 3 LOD layers, pockets, consistency vs X4 |
| `hydrology_v1.md` | Worldgen detail vs runtime triggers, MP parity |
| `terrain_tools_brushes_v1.md` | Terrain tools, undo/command stack |
| `tile_sprites_v1.md` | Pixel vs logic decouple, PNG → future formats |
| `composite_style_worldgen_v1.md` | Layered scalar fields, deterministic chunks, fast pixel preview (mental model + code map) |
| `material_tag_rule_system_v1.md` | Material registry, interned tags, RON rules, `ChunkCellMatrix` — unified with `TerrainClass` |
| `procedural_world_pipeline_reference_outline_v1.md` | Non-authoritative outline (hot-reload graph, multi-pass tags, layers) → spawns matrix §§13–18 + checklist **49–78** |
| `llm_world_evolution_reference_outline_v1.md` | Non-authoritative outline (LLM rule-edit loop, memory tiers, metric system) → feeds future Qs §§79–83 |
| **`implementation_questions_v1.md`** | **Engineering checklist** — types, queues, IPC |
