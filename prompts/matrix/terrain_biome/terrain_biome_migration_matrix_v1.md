# Terrain/Biome Consolidation Migration Matrix v1

> **STATUS:** ✅ **Core type consolidation applied** in `src/terrain/biome.rs`, `WorldData` maps, legacy aliases · ⏳ **Parameter merge** (river fields, `RegionMethod`) still **Partial** — see tables below. **Paired designer:** `prompts/designer_questions/terrain_world/` · **Structured spec:** [`terrain_world/spec/README.md`](../../designer_questions/terrain_world/spec/README.md) (world gen, politics, cities, logistics). **Layered gen + preview addendum:** [`composite_style_preview_integration_matrix_v1.md`](composite_style_preview_integration_matrix_v1.md). **Material / tag / rule unification:** [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) (pipeline §§1–12; **hot-reload / partial rebuild / multi-layer tilemaps / packs:** §§13–18 + optional **U7**). **Outline brainstorm (non-authoritative):** [`procedural_world_pipeline_reference_outline_v1.md`](../../designer_questions/terrain_world/procedural_world_pipeline_reference_outline_v1.md).

Version: `v1.3.1`
Scope: Canonical biome + terrain storage model with ECS/runtime separation and UI plugin touchpoints.

> **Prompt use:** Pair `designer_questions/terrain_world/` · verify table rows vs `src/terrain/` · cite `path` + symbol only · `ASK:` for unset tunables.

## Canonical Data Boundary

- Serializable world data uses:
  - `TerrainClass` (dominant tile terrain label)
  - `BiomeWeights` (continuous ecological blend per tile)
  - `TileEnvironmentProfile` (**implemented** — bundles `BiomeWeights` + `TerrainSurfaceMix` soil fractions in `src/terrain/biome.rs`)
  - `classify_biome(height, moisture, temperature)` → `BiomeClassification` (weights + `TerrainClass`) in same file
- Parallel **ecology** tunables (flora/crop/suitability) in `src/terrain/ecology.rs` — not the same as `BiomeWeights`; **design how they merge** for tiles/saves (chunk docs + matrix must stay explicit).
- **Legacy / duplicate terrain storage:** `src/terrain/world.rs` (`GeoRegion`, spatial hash traits) coexists with chunk/streaming design in designer docs — **reconcile or quarantine** in implementation questions; do not assume one world model.
- ECS-only data uses components that reference canonical serializable fields.
- Render/gameplay bucket (`BiomeBucket`) is derived from canonical data and should not become source-of-truth storage.

## Cross-matrix: procedural pipeline downstream

`TerrainClass` / `BiomeWeights` remain the **canonical biome boundary** here; material variants, tag passes, rule resolution, chunk-scoped invalidation, and multi-layer presentation are owned by the unification matrix.

| Topic | Doc | Anchor |
|:---|:---|:---|
| Invalidation graph, partial chunk rebuild, tag-pass depth, multi-layer stack, perf/cache, LLM-edit policy, world profiles / packs | [`material_unification_matrix_v1.md`](material_unification_matrix_v1.md) | §§**13–18**, phase **U7** |
| Outline-derived questions (checklist) | [`../../designer_questions/terrain_world/implementation_questions_v1.md`](../../designer_questions/terrain_world/implementation_questions_v1.md) | **49–78** |
| Non-authoritative outline → naming alignment | [`../../designer_questions/terrain_world/procedural_world_pipeline_reference_outline_v1.md`](../../designer_questions/terrain_world/procedural_world_pipeline_reference_outline_v1.md) | full doc |
| Layered fields + preview mental model | [`../../designer_questions/terrain_world/composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md) | — |
| **Rust execution orchestrator (U3–U7)** | [`../../guides/terrain_unification_runbook_v1.md`](../../guides/terrain_unification_runbook_v1.md) | invariants §1, loop §5 |
| **Per-phase atomic step packs** | [`runbook/README.md`](runbook/README.md) | `u3_steps_v1.md` … `u7_steps_v1.md` |
| **Paired Bevy terrain assets (A1–A3)** | [`../../guides/bevy_asset_terrain_runbook_v1.md`](../../guides/bevy_asset_terrain_runbook_v1.md) + [`../assets/runbook/README.md`](../assets/runbook/README.md) | extensions + example audit + post-U3 gates |
| **Paired downstream queue (S, P, C)** | [`../../guides/terrain_paired_runbooks_queue_v1.md`](../../guides/terrain_paired_runbooks_queue_v1.md) | Q0–Q6 creation + sync gates |

## Migration Matrix

| Old Symbol | File | New Symbol | Action | Status |
|---|---|---|---|---|
| `BiomeType` (local enum) | `src/terrain/generation/world_generator_enhanced.rs` | `TerrainClass` | Replace enum with deprecated alias to canonical type | Applied |
| `BiomeType` (local enum) | `src/bevysubengines/world_generator_plugin.rs` | `TerrainClass` | Replace enum with deprecated alias to canonical type | Applied |
| `GameBiomeType` (storage-like usage) | `src/bevysubengines/world_generator_plugin.rs` | `BiomeBucket` | Keep as derived output only | Applied |
| `TerrainType` (legacy terrain labels) | `src/terrain/bevy_terrain.rs` | `LegacyTerrainType` + deprecated alias | Preserve old parser path while migrating runtime/storage | Applied |
| `WorldData.biome_map: Vec<BiomeType>` | `src/bevysubengines/world_generator_plugin.rs` | `Vec<TerrainClass>` | Canonical dominant terrain map | Applied |
| N/A | `src/bevysubengines/world_generator_plugin.rs` | `biome_weights_map: Vec<BiomeWeights>` | Add continuous ecological storage | Applied |
| Legacy flora/crop/flower tags in malformed enums | `src/terrain/bevy_terrain.rs` | `Legacy*` enums + canonical ecology types in `src/terrain/ecology.rs` | Account all legacy variants with migration mapping | Applied |
| Invalid prototype tile ecology structs | `src/terrain/generation/bevy_terrain_gen.rs` | `TerrainPoint`, `VegetationCandidates` | Replace invalid definitions with serializable generation models | Applied |

## Parameter Conflict Matrix (next-stage)

| Conflict | File A | File B | Canonical Direction | Status |
|---|---|---|---|---|
| `river_count/lake_count` vs `river_threshold/lake_threshold` | `world_generator_enhanced.rs` | `world_generator_plugin.rs` | Keep count now, retain thresholds as legacy serialized fields until unified generator config lands | Partial |
| `RegionMethod` vs `RegionMethodType` | `world_generator_enhanced.rs` | `world_generator_plugin.rs` | Merge into single shared enum module | Pending |

## UI Plugin Integration Points

1. `src/terrain/generation/world_generation_plugin.rs`
   - Aggregates `WorldGeneratorPlugin`, `WorldGenUiPlugin`, `WorldPreviewPlugin`.
   - Acts as primary place to attach new generator-tooling UI resources.
2. `src/gui/editor/world_gen_ui.rs`
   - Existing editor-side control surface for generation params.
   - Needs follow-up to expose multi-noise layer selection and generative system profile selectors.
3. `src/bevysubengines/world_generator_plugin.rs`
   - Separate world generator UI flow already present.
   - Should eventually consume same shared canonical param resources to avoid divergence.
4. `src/terrain/generation/world_generation_plugin.rs`
   - `WorldGenerationInGamePlugin`: runtime generator only.
   - `WorldGenerationToolsUiPlugin`: editor/testing UI + preview.
   - `WorldGenToolsPlugin` composes both and keeps F8 tooling toggle.

## Vegetation/Crop/Flower Accounting

Legacy tags now explicitly tracked:
- Flora: moss, vines, shrubs, broad-leafed trees, coniferous trees, grass
- Crop subtags: cereal, legumes
- Flower subtags: dafidals, forgetmenoghts, bella, dandlions, blue iris, a

Canonical ecology layer:
- `FloraType`, `CropType`, `FlowerType` in `src/terrain/ecology.rs`
- Legacy string mapper: `legacy_flower_name_to_type`
- Simulation hooks in serialized world data:
  - `flora_density_map`
  - `crop_yield_map`
  - `flower_density_map`

## Noise/Generative Context Plan (prompt-ready)

Define a shared serializable config:

```rust
enum NoiseKind { Fbm, Perlin, Simplex }

struct NoiseLayer {
    kind: NoiseKind,
    frequency: f32,
    amplitude: f32,
    octaves: u8,
    seed_offset: u64,
}

struct GeneratorContext {
    terrain_layers: Vec<NoiseLayer>,
    moisture_layers: Vec<NoiseLayer>,
    temperature_layers: Vec<NoiseLayer>,
    feature_profile_id: String,
}
```

## Deprecation Trace Rules

- Keep deprecated type aliases in-place for one migration cycle.
- Every deprecated alias must name its canonical replacement in the `note`.
- Remove alias only after all references are moved and save/load compatibility adapters are in place.

## Prompt Fragment for Subsequent Agent Pass

Use this exactly to continue migration in a later review:

1. Read `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`.
2. Enforce `TerrainClass + BiomeWeights` as canonical serializable terrain/biome storage.
3. Keep ECS mutations on main thread only; generation remains data-first.
4. Ensure UI plugins consume shared generation context resources.
5. Resolve outstanding conflicts:
   - `RegionMethod`/`RegionMethodType`
   - `river_count/lake_count` vs thresholds
   - duplicate world generation entrypoints.

## Production System Split (Concrete + Electrical)

Applied structural split for expansion-ready modular manufacturing:

| Concern | Active Location | Boundary |
|---|---|---|
| Concrete serializable policy/config | `src/entities/production/concrete/components.rs` (`ConcreteProductionConfig`) | Serializable |
| Concrete runtime mutable state/systems | `src/entities/production/concrete/components.rs` + `src/entities/production/concrete/systems.rs` | ECS runtime |
| Power runtime systems | `src/entities/production/power/systems.rs` | ECS runtime |
| Production runtime plugin aggregation | `src/systems/production/runtime.rs` | Runtime orchestration |
| Production tooling UI state | `src/systems/production/tools_ui.rs` | Tools/editor boundary |
| Legacy reference files | `src/entities/production/concrete/sys.rs`, `src/systems/production/power_systems.rs`, `src/systems/production/production_consumption.rs` | Migration-only |

Applied same split pattern for aluminum and generic manufacturing scaffolding:

| Concern | Active Location | Boundary |
|---|---|---|
| Aluminum serializable policy/config | `src/entities/production/aluminum/components.rs` (`AluminumProductionConfig`) | Serializable |
| Aluminum runtime mutable state/systems | `src/entities/production/aluminum/components.rs` + `src/entities/production/aluminum/systems.rs` | ECS runtime |
| Generic manufacturing blueprint super-structure | `src/entities/production/core/manufacturing.rs` | Serializable + ECS bridge |
| Legacy aluminum plugin file | `src/entities/production/aluminum/production_sys.rs` | Migration-only |

