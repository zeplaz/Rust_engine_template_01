# World map scale v1 (symbolic metres)

**Status:** SHIPPED (scale contract + derived WG params). **ECS chunk storage:** SHIPPED for `--test` harness; editor still `PerTileEntities` by default.

## Contract

| Layer | Rule |
|-------|------|
| **Sim grid** | 1 tile = 1 logical world unit on XZ (unchanged). |
| **Symbolic scale** | `WorldMapScale.meters_per_tile` — default **100 m**. Lore, UI, feature derivation. |
| **Buildings** | May stay oversized in tiles (game scale). |
| **Land features** | Rivers, ridges, macro regions derived from **km targets** → tile counts. |

Code: [`src/terrain/world_map_scale.rs`](../terrain/world_map_scale.rs) · wired in [`WorldGenParams`](../terrain/generation/world_generator_enhanced.rs).

## Map presets (@ 100 m/tile)

| Preset | Tiles | Symbolic extent | Role |
|--------|-------|-----------------|------|
| TacticalSmall | 192² | ~19 km | Frame / maneuver tests |
| **MediumSmall** | **320²** | **~32 km** | Visual harness, medium-small play |
| Standard | 512² | ~51 km | Editor default |
| LargeStrategic | 1024² | ~102 km | Needs chunk-authoritative storage |

End-game target: **battlefields over hundreds of km²** → tile count × m/tile, not bigger ECS entity spam.

## Derived land features

`LandFeatureRhythm` (defaults):

- Macro patch **8 km** → `num_regions`
- Relief wavelength **4 km** → `noise_scale` ≈ `1 / tiles_for_km(4)`
- River spacing **6 km** → `river_count` ≈ `√area_km² / 6`
- Lakes **0.3 per 100 km²** → `lake_count`

`WorldGenParams::recompute_symbolic_land_features()` runs on default, presets, and at each gen job when `auto_symbolic_land_features` is true.

## ECS tile storage (broken at scale)

**Current (legacy):** full world gen spawns **one entity per tile** (`TileMarker`) when `field_storage = PerTileEntities` (editor default).

**Shipped (harness):** `TerrainFieldStorage::ChunkCellMatrixAuthoritative` — dense [`WorldGenDenseTerrainCache`](../terrain/generation/world_gen_dense_cache.rs), **0 tile entities**, hydrate into ~100 [`ChunkCellMatrix`] slabs. Raster index reads chunks, not 102k entities.

**Migration lanes (not interchangeable):**

1. **WG full pass** — stop spawning all `TileMarker` entities; write chunk matrices directly (chunk_worldgen_scheduler path).
2. **Editor sync** — `editor_chunk_tile_sync` today copies tiles → chunks; invert to chunk-first.
3. **Strategic / construction** — already chunk/slab oriented; keep tile coords as `(i,j)` with scale resource for display.

Do not increase `MAX_WORLD_GEN_TILES` without completing (1).

## Witness

World-gen debug JSON includes `meters_per_tile`, `extent_km_*`, `area_km2` (`world_gen_diagnostics.rs`).
