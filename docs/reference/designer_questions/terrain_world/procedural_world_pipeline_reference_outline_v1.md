# Procedural world pipeline — reference outline only `v1`

**Status:** **Non-authoritative.** Brainstorm / external prompt fragment. The **binding** specs are:

- [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md)
- [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md)
- [`implementation_questions_v1.md`](implementation_questions_v1.md) (items **42+**, including extensions from this outline)

Use this file to spawn **new matrix rows** and **new implementation questions**, not as runtime truth.

---

## Naming alignment (repo vs outline)

| Outline phrase | Canonical in this repo | Notes |
|:---|:---|:---|
| “Migration Matrix” | **`ChunkCellMatrix`** | Avoid clashing with `prompts/matrix/` migration docs |
| Material registry in RON only | **JSON today** (`material_registry.json`); RON migration **ASK** | See matrix §2 “format evolution” |
| Stable numeric `MaterialId` in rules (`result: 10`) | **Preferred: `result: "loam_wet"`** resolving to runtime id at load | Saves stay **name-based**; stable runtime ids are an **engine** concern |
| “NO terrain logic hardcoded” | **`TerrainClass` enum remains** as `MaterialFamily` | Gameplay still needs exhaustive handles; *variant* materials and visuals stay data-driven |

---

## Outline summary (compressed)

- **Pipeline:** noise fields → semantic cell grid → tag expansion passes → rule resolution → material assignment → tilemap layers → multi-layer render.
- **Hot-reload:** asset change → invalidate dependent ECS → rerun **minimal** pipeline stages → prefer **chunk-level** partial rebuild.
- **Tag passes:** raw thresholds → derived tag combos → spatial (clusters, edges, gradients) → agent/LLM overlay (audited, seed-stable config only).
- **Rules:** priority, deterministic tie-break, optional weighted / partial matching (design 📎).
- **Rendering:** terrain / overlay / resource layers share tile space, independent visibility, z-order convention 📎.
- **Performance:** dirty chunks, caches keyed by hashes, batched ECS updates, diff-friendly tilemap writes.
- **Debug:** tag viz, rule trace, material resolution explain, heatmaps.
- **Extensibility:** “biome packs”, modded rule sets, world profiles.

Each bullet is expanded into **checklist items** in `implementation_questions_v1.md` and **tables** in `material_unification_matrix_v1.md`.

---

## Cross-links

- Layered fields + preview: [`composite_style_worldgen_v1.md`](composite_style_worldgen_v1.md)
- Chunk streaming / dirty regions: [`chunks_streaming_v1.md`](chunks_streaming_v1.md)
- Tools UI / asset editor: [`../tools_ui/spec/08_world_gen_desktop_tool_v1.md`](../tools_ui/spec/08_world_gen_desktop_tool_v1.md)
