# World, terrain & theatre — implementation questions `v1`

**Audience:** engineers implementing streaming, LOD, hydrology, tools, **world gen, politics, cities, logistics**.  
**Pair-with:** [`spec/README.md`](spec/README.md), `chunks_streaming_v1.md`, `simulation_lod_v1.md`, `hydrology_v1.md`, `composite_style_worldgen_v1.md`, `material_tag_rule_system_v1.md`, `procedural_world_pipeline_reference_outline_v1.md` (non-authoritative brainstorm), `prompts/matrix/terrain_biome/`.

## Data structures

1. **ChunkId**: `IVec2` global index vs `(sector_id, local_ix, local_iy)`? Wire format vs in-memory?
2. **InterestOrb**: max count per server; merge policy when orbs overlap; priority tie-break.
3. **LOD tier** on chunk: `u8` enum synced with sim tick divisor — where stored (`Resource` vs component on chunk entity)?
4. **Hydrology dirty region**: `Aabb` in chunk space, bitmask, or polygon? Max regions per map.

## Serialization & I/O

5. Serialize queue: **bounded channel** depth; metrics (`drops`, `latency_ms`) for egui panel.
6. **Ghost band** width (cells) for flow/power at chunk edge — same for all subsystems or per-domain?
7. Save file: per-chunk vs bundle — first milestone choice?

## Simulation

8. Server **recomputes LOD** how often — every sim tick vs every N ticks?
9. **Bookmark radius** defaults: config file path (`assets/configs/` RON) vs hardcoded constants first?
10. AI faction **coarse tick**: fixed interval vs event-driven only?

## Tooling / editor

11. Terrain tool **command** trait: `apply(world)`, `undo`, `merge_with_previous` — stable API before UI.
12. Brush stroke **coalescing** window (ms)?

## Political & territory (pair `spec/03_political_territory.md`, `factions/`)

15. **Territory model:** tile owner vs macro polygon vs chunk bitmask — primary representation + wire format?
16. **Border redraw:** event-driven vs periodic; LOD tier that evaluates closures for **navigation**.
17. **Scenario embed:** initial claims in scenario header vs pure post-gen pass?

## Cities & settlements (pair `spec/04_cities_ai_settlements.md`)

18. **City graph:** serialized in save vs regenerated from seed; MP authority for growth commands?
19. **AI planner cadence:** same tick as coarse faction sim vs separate schedule?
20. **Zoning / land use:** data schema for districts; coupling to **production** facility placement?

## Logistics & platforms (pair `spec/05_logistics_settlement_coupling.md`, `strategic_platforms/spec/`)

21. **Port / berth** validity: min depth vs hydrology sampling; failure mode if dam lowers water.
22. **Drone corridor** volume: axis-aligned tube vs polyline + clearance; conflict with buildings.
23. **Promotion bubble** for combat: tie to `InterestOrb` expansion or dedicated `combat_lod` resource 📎?

## Biome, ecology & tile profile (code-synced)

26. **`TileEnvironmentProfile`** (`biome_weights` + `TerrainSurfaceMix`) vs **`EcologicalSuitability`** (`ecology.rs`) — single per-tile record vs derived layers? Save format implications.
27. **`classify_biome()`** outputs normalized weights — do generators other than height/moisture/temp need to call it, or duplicate logic?
28. **`BiomeId::primary()`** tie-break rules when weights equal — document for determinism.

## Parallel world models (technical debt)

29. **`src/terrain/world.rs`** (`GeoRegion`, spatial hash traits) vs **chunk streaming** design in `chunks_streaming_v1.md` — migrate, bridge, or delete path 📎?
30. **`idgen::EntityId`** import style vs `crate::idgen` in terrain — align with rest of crate for saves.

## World gen plugins (code-synced)

31. **`WorldGenToolsPlugin`** wraps `WorldGenerationInGamePlugin` + **`WorldGenerationToolsUiPlugin`** (`world_generation_plugin.rs`); **`WorldGenerationPlugin`** is a separate bundle — which is default for `main` vs `editor` binaries 📎?
32. **F8** toggle wired in `world_gen_key_input` — document alongside other dev hotkeys in `tools_ui/spec/`.

## Testing

33. **Deterministic chunk load order** test: same seed → same `ChunkId` set for fixed orbs?
34. Hydrology **parity** test client/server: which invariants (volume, min/max height)?

## Layered fields & fast preview (pair `composite_style_worldgen_v1.md`, integration matrix)

35. **Chunk vs preview scale:** single authoritative tile size vs separate preview downsample — which is source of truth on wire and in `WorldGenParams.width/height`?
36. **Preview octave budget:** should fast preview use fewer octaves than full `generate_world`, and is that a dedicated `NoiseSamplingTuning` / UI field?
37. **Resource patch overlap:** max concurrent resource masks per tile; tie-break when two densities exceed threshold (order in `ResourceField` list vs priority field)?
38. **Threat / enemy mask:** preview-only red tint vs persisted gameplay layer — authority on server?
39. **Preview LOD pyramid:** how many levels (LOD0–LOD2), max texture dimension per level, and invalidation when only `biome_tuning` changes?
40. **Moisture/temperature seed offsets:** move from hardcoded `seed+1`/`+2` into `NoiseSamplingTuning` (matrix **P1**) — acceptable default migration for saves?
41. **Spawn / starting-area bias:** fixed scenario spawn vs first suitable land tile — how do resource/threat falloff fields anchor?

## Material / tag / rule unification (pair `material_tag_rule_system_v1.md`, `material_unification_matrix_v1.md`; outline brainstorm `procedural_world_pipeline_reference_outline_v1.md`)

42. **Registry size cap:** max `MaterialId` count, max tag count (drives `TagSet` bitset width and reserved id ranges).
43. **`TagSet` representation:** fixed bitset (`u128` / `[u64; N]`) vs `SmallVec<TagId>` — trade-off for memory vs `O(rules × tags)` scan.
44. **Rule memoization:** quantize `BiomeWeights` to 8-bit buckets for `resolve_material` cache — acceptable error vs hit rate.
45. **Save wire format:** `MaterialDef.name` only (recommended) vs name + id pair for fast load — registry version header policy.
46. **`family_filter` on rules:** mandatory vs optional default “match any `TerrainClass`”.
47. **Agent write authority:** which tags LLM/scenario may add or remove at runtime vs read-only outputs of `classify_biome`.
48. **`weight_predicate` in RON rules:** JSON-able subset only vs full expression DSL — schema versioning.

## Hot-reload & invalidation (outline-inspired, non-binding)

49. **`AssetEvent` vs manual reload:** single path for registry/rules/tags — how to avoid double-apply when F8 “reload” and file watcher both fire?
50. **`ChunkDependency` (or equivalent):** which hashes ride on the chunk entity (`ruleset_hash`, `registry_hash`, `tuning_hash`, `tag_registry_hash`) and who computes them?
51. **Partial rebuild rule:** when is it legal to rerun **only** pass 6 (materialize) vs passes 2–6 vs full chunk regen from noise?
52. **Stable runtime `MaterialId` after registry reload:** if file order changes but names unchanged, must ids be preserved? (Outline: stable ids; repo matrix: name-first — **reconcile** 📎.)
53. **Invalidation fan-in:** `MaterialRegistry` change → remap ids → update **texture atlas only** vs full `MaterializedChunk` recompute — exact policy?
54. **`RuleSet` change:** always dirty `MaterializedChunk` only, or also tag passes when rules referenced tag synonyms?

## Tag expansion & spatial passes

55. **Pass 2 “derived tags”** (`wet`+`lowland`→`floodplain`): data-driven combo table (RON/JSON) vs hardcoded small kernel 📎?
56. **Pass 3 neighborhood:** Moore radius 1 vs r=2 for `core_wet_zone`-style tags; ghost-band interaction with [`chunks_streaming_v1.md`](chunks_streaming_v1.md)?
57. **Edge / gradient tags:** finite-difference on elevation/moisture — per-chunk only or cross-chunk consistent (requires neighbor cells)?
58. **Pass ordering Versioning:** adding a new pass bumps `tag_pipeline_version` and invalidates which caches?

## Rule engine complexity

59. **Weighted / scored rules:** optional scoring besides priority (outline “partial matches”) — opt-in per rule or global engine mode?
60. **`weight_curve` string** (`"linear"`, …): interpreter in Rust vs declarative polynomial 📎?
61. **Conflict resolution:** two rules same priority + same tag match — tie-break already `rule_index`; document for LLM editors.
62. **Resource ruleset independence:** separate `resource_rules.ron` + layer vs shared resolver with tag namespace prefixes?

## Multi-layer tilemaps & rendering

63. **Layer stack:** terrain z=0, overlay z=10, resources z=20 — fixed constants or per-world profile?
64. **`bevy_ecs_tilemap` (or equivalent):** one `TilemapId` per layer per chunk vs one tilemap with layered textures?
65. **Shared `TileStorage` grid:** identical `TilemapSize` across layers; validation on chunk bounds 📎?
66. **Overlay data source:** same `ChunkCellMatrix` vs lightweight derived buffer for heatmaps?

## Performance, threading, cache

67. **Bevy 0.18 constraint:** heavy worldgen on `TaskPool` / dedicated thread — where results hand off to main thread `Commands`?
68. **`ChunkCache` resource:** in-memory only vs optional persistence (ties serialization matrix)?
69. **Diff-based tile updates:** API for “set only changed `TileTextureIndex`” vs clear-all — batching policy?
70. **Promotion with LOD:** low-resolution chunk matrix for distant chunks — same tags or fused tags 📎?

## LLM / agent rule editor (policy)

71. **Determinism contract:** agent may change **rules/thresholds files** only; **seed** + **world revision id** unchanged ⇒ same world — enforce how?
72. **Audit log:** append-only JSONL of agent edits — ship with saves or dev-only?
73. **Metric feedback loop** (% traversable, fragmentation, connectivity): which metrics are **real** in-engine vs stub for future?
74. **Safety caps:** max rules injected per session; max priority delta 📎?

## Debug, packs, profiles

75. **Rule trace UX:** per-tile “winning rule id + score” component vs debug overlay only?
76. **World profile:** `assets/config/terrain/profiles/*.ron` selecting rule set + registry + tuning bundle — one `WorldProfileId` in scenario header?
77. **Biome / rules “pack”:** directory convention (`packs/<name>/rules.ron`) vs single merged asset 📎?
78. **Tag visualizer performance:** bitmask GPU blit vs CPU thumb for editor preview?

📎 Unresolved items feed back into designer docs above; do **not** invent numbers without `ASK:`.

### U7 close-out carries (terrain unification, 2026-04)

Promoted from [`u7_steps_v1.md`](../../matrix/terrain_biome/runbook/u7_steps_v1.md) 📎 block — still **`ASK:`** where noted elsewhere: **§62** resource ruleset, **§72** on-disk audit JSONL, cross-process hash (xxh3) **item 50**, **§77** modded priority bands, **§78** tag GPU path, `material_unification_matrix` §7 footnotes.
