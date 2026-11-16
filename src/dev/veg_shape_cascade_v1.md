# Vegetation shape cascade `v1` (VEG-SHAPE-001)

One parametric path from cell data to plants in **both** presentation modes (TacticalFlat2D symbols, PixelWorld3D voxel trees). Species and shape are RON rows, not code. Flora never enters `VoxelMassing` / `massing_columns`; no dig-voxels.

## Authority map

| Surface | Single writer | Readers |
|:---|:---|:---|
| `ChunkCellMatrix` (biome weights, moisture, temperature, water) | terrain generation | extract (species) |
| `ChunkCellStand::render` (live stage, fire, snags, regrowth, water) | `cell_stand_tick` | extract |
| `VegSpeciesTable` (`assets/configs/ecology/veg_species_v1.ron`) | `VegetationVisualExtractPlugin` (insert at build, `load()` = disk if valid else embedded) | extract, both views |
| `VegExtractRow.species` / `species_rev` | `build_vegetation_extract_frame` (cached per chunk on matrix change tick) | both views |
| `VegInstance` list | none stored: `sample_row_instances` is a pure function of a row | both views |
| `TreeShape` | none stored: `VegSpeciesTable::resolve_shape` is pure, `Copy`, no alloc | both views |

Species is derived, not simulated: no new sim state, no second writer. Sim -> View (extract) -> Render; consumers never write back.

## Cascade

```text
veg_species_v1.ron  (species rows: biome/moisture/temp affinity + height, crown width, trunk, taper, tiers, palette)
  │
  ├─ species_for_cell(cell, BiomeWeights, moisture, temp)         affinity x low-freq patch noise, argmax
  │     └─ species_cell_codes(matrix, chunk)  ─▶ VegExtractRow.species   (per cell u8, Arc, cached)
  │
VegExtractRow { cells, species, cells_rev, species_rev }
  └─ sample_row_instances(row, chunk, step) ─▶ [VegInstance { pos, cover, species, rand, riparian, burn_phase }]
        │   (Halton jitter + topology layout weight + per-cell code; THE one site sampler)
        └─ resolve_shape(species, Cover::maturity(), rand) ─▶ TreeShape { profile, height, crown_width,
              │                                                trunk_frac, trunk_width, tiers[..5], crown, tip }
              ├─ Tactical  (`world_map_vegetation_mesh::push_symbol`): polygon by CrownProfile
              │        Round=hex  Cone=triangle  Column=diamond  Spread=octagon; trunk rect at Fine
              └─ PixelWorld3D (`pixel_world::layers::push_tree`): trunk box + one box per tier
                       (tier width falls with `taper`; coarse bands drop the trunk)
```

Shrubs, young and old stands share a species and scale through `maturity` (`shrub/young/old` width and height). Fire states (`Burning`, `Burnt`) take the species' mature envelope and keep their own fire/charcoal colours. `Grass` and `Regrowth` stay species-independent.

## Authoring a species

Append a row to `species` in `veg_species_v1.ron`. Ids are the table index: **append only**, reordering reshuffles every forest. `VegSpeciesTable::from_ron` validates ranges, tiers (1..=5), unique ids and table size (<= 64); a bad disk table logs a warning and falls back to the embedded one. Tests: `cargo test -p proc_A_dine01 --lib -- vegetation_species world_map_vegetation_mesh pixel_world::layers`.

| Field | Meaning |
|:---|:---|
| `biome` / `moisture` / `temperature` / `breadth` | where it grows; `breadth` is the tolerated stray from the preferred moisture+temperature |
| `profile` | `Round`, `Cone`, `Column`, `Spread` silhouette family |
| `height`, `crown_width` | mature range in tiles; the per-site rand picks inside it |
| `trunk_frac`, `trunk_width`, `taper`, `tiers` | voxel build: bare trunk share, trunk size, tier narrowing (0..1), tier count |
| `crown_old`, `crown_young`, `crown_tip`, `crown_shrub` | palette (tier colour lerps crown -> tip) |

Adding a **new silhouette** needs one enum arm in `CrownProfile` plus its Tactical polygon in `crown_symbol`; the voxel path is already generic over tiers.

## Determinism

Species: pure function of `(world cell, matrix cell data, table.seed, table)`. Shape: pure function of `(species, maturity, site rand)`. Site rand and positions: existing `site_rand` / Halton helpers keyed by chunk coord and site index. No unseeded randomness, no time, no frame counters except the existing burn flicker phase.

## Instancing status

`VegInstance` and `TreeShape` are fixed-size POD, so a chunk's instance list is already upload-shaped. Today each view still stamps them CPU-side into one merged mesh per chunk (the existing contract: Tactical one mesh for visible chunks, PixelWorld3D one mesh per chunk, nearest-first rebuild budget). That is **template stamping, not GPU instancing**.

## Residual debt (not taken)

| Id | Item |
|:---|:---|
| VEG-SHAPE-D1 | GPU instancing: per-(species, maturity) shared meshes + an instance buffer in PixelWorld3D (same debt class as RND-09 for massing columns). Needs a custom instanced material or batched `Mesh3d` entities; defer until the tree count per frame justifies it. |
| VEG-SHAPE-D2 | `Grass` has no voxel form in PixelWorld3D (skipped as before); flowers / vines / crops (`FlowerType`, `CropType`, `FloraType::Vines`) have no species rows. |
| VEG-SHAPE-D3 | Species is per cell from biome + patch noise. No ecological succession by species, no species-specific fuel (`StandKind` is still Grass/Brush/Timber), no seeding or dispersal. If species must affect gameplay it belongs in `systems/ecology`, with this table as its authoring source. |
| VEG-SHAPE-D4 | Species noise salts with `table.seed` and world cell only, not the world-gen seed; two worlds with the same biome map get the same stands. |
| VEG-SHAPE-D5 | No hot reload: `VegSpeciesTable` is read at startup. A change would invalidate caches (both consumers already react to `is_changed()`), but nothing watches the file. |
| VEG-SHAPE-D6 | Wash band (Tactical) is still cover-class colour blocks; species only shows from the Blob band up. PixelWorld3D uses an 8-tile site step at Wash. |
| VEG-SHAPE-D7 | LG-5 atlas stamps (`queue_pixel_world_landscape_atlas_stamps`) and the minimap compositor read `VegetationExtractFrame` cover only; they do not read `species`. Art stays `pilot` tier (VEG-F01). |
| VEG-SHAPE-D8 | Voxel shapes are box stacks; no per-tier rotation, no asymmetric crowns, no age-driven trunk lean. Visual sign-off needs a windowed capture (VEG-C13). |
