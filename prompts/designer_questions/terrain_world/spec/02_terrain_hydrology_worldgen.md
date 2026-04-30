# Terrain, biome & hydrology (worldgen bind) `02`

**Deep dives:** [`../hydrology_v1.md`](../hydrology_v1.md), `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md`, [`../chunks_streaming_v1.md`](../chunks_streaming_v1.md).

## Canonical storage

- Follow matrix: `TerrainClass`, `BiomeWeights`, hydrology fields — ECS vs serializable split per matrix **STATUS**.
- **`TileEnvironmentProfile`** + **`TerrainSurfaceMix`** in `biome.rs` hold per-tile ecology-related **numeric** fields; **`ecology.rs`** adds flora/crop enums + **`EcologicalSuitability`**. Designer + matrix: **merge strategy** (single struct vs layered components) must stay explicit — see `implementation_questions_v1.md` § biome/ecology.

## Runtime vs worldgen

- Which hydro events **re-run** at runtime (dams, breaches) vs baked only at gen — `hydrology_v1.md`.

## Chunk edge continuity

- **Ghost band** width shared with power/production where relevant (`implementation_questions_v1.md` §6).

## Art pipeline

- Terrain **brushes** undo stack: `terrain_tools_brushes_v1.md`.
