//! Behaviour: a really generated world carries no vegetation on open water.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::vegetation_suitability::{update_chunk_vegetation_suitability, ChunkVegetationSuitability};
use super::vegetation_field::vegetation_field_tick;
use super::{ChunkEcology, VegetationField};
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::passes::{apply_hydrology_chunk, classify_cells};
use crate::terrain::generation::water_class::tests::generate_hydrology;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{
    hydrate_chunk_matrices_from_dense_terrain, Chunk, ChunkCellMatrix, DenseTerrainHydrationGate,
    WaterClass, WorldGenDenseTerrainCache, WorldWaterGrid,
};
use crate::terrain::material::{TagRegistry, TagSet};
use crate::terrain::world_map_scale::TerrainFieldStorage;
use crate::terrain::default_terrain_families;

const CHUNK: u32 = 32;

/// Real world generation (noise, hydrology, water grid, carved dense cache, hydration into chunk
/// matrices, classification, pass 4) until a seed yields an inland lake.
fn generated_world_chunks() -> (WorldGenParams, Vec<(IVec2, ChunkCellMatrix)>) {
    let mut found = None;
    for seed in 1..80u64 {
        let (p, dem, moist, hydro) = generate_hydrology(seed);
        let grid = WorldWaterGrid::from_hydrology(&hydro, p.width, p.height, seed);
        if grid.count(WaterClass::Lake) > 0 && grid.count(WaterClass::River) > 0 {
            found = Some((p, dem, moist, grid));
            break;
        }
    }
    let (mut params, dem, moist, grid) = found.expect("a seed with a lake and rivers in 80 tries");
    params.field_storage = TerrainFieldStorage::ChunkCellMatrixAuthoritative;
    let n = dem.len();
    let mut cache = WorldGenDenseTerrainCache {
        width: params.width,
        height: params.height,
        elevation: dem,
        moisture: moist,
        temperature: vec![0.3; n],
        family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
        region_index: vec![0; n],
    };
    cache.apply_water_grid(&params, &grid);

    let (cx, cy) = (params.width / CHUNK, params.height / CHUNK);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(params.clone())
        .insert_resource(cache)
        .init_resource::<DenseTerrainHydrationGate>()
        .add_systems(Update, hydrate_chunk_matrices_from_dense_terrain);
    for y in 0..cy as i32 {
        for x in 0..cx as i32 {
            app.world_mut().spawn((
                Chunk { coord: IVec2::new(x, y) },
                ChunkCellMatrix::new(UVec2::splat(CHUNK)),
            ));
        }
    }
    app.update();

    let tag_reg = TagRegistry::load_from_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.ron"),
    )
    .expect("tag registry");
    let mut out = Vec::new();
    let mut q = app.world_mut().query::<(&Chunk, &ChunkCellMatrix)>();
    for (chunk, matrix) in q.iter(app.world()) {
        let mut m = matrix.clone();
        classify_cells(&mut m, &params.biome_tuning, &tag_reg, default_terrain_families());
        apply_hydrology_chunk(
            &mut m,
            chunk.coord,
            &params.biome_tuning,
            &tag_reg,
            &TagSet::ALL,
            Some(&grid),
        );
        out.push((chunk.coord, m));
    }
    (params, out)
}

#[test]
fn generated_world_has_no_canopy_on_lake_river_or_ocean_cells() {
    let (_params, chunks) = generated_world_chunks();
    let open = |m: &ChunkCellMatrix| m.water.iter().filter(|c| c.is_open_water()).count();
    assert!(
        chunks.iter().any(|(_, m)| m.water.iter().any(|c| *c == WaterClass::Lake)),
        "fixture world must contain a lake"
    );
    assert!(chunks.iter().any(|(_, m)| open(m) > 0 && open(m) < m.water.len()));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(100)))
        .insert_resource(SimControlState::default())
        .add_systems(Update, (update_chunk_vegetation_suitability, vegetation_field_tick).chain());
    let mut entities = Vec::new();
    for (coord, matrix) in chunks {
        let e = app
            .world_mut()
            .spawn((
                Chunk { coord },
                matrix,
                ChunkEcology {
                    biomass: 0.8,
                    regrowth_rate: 0.3,
                    shade_factor: 0.6,
                    ..ChunkEcology::default()
                },
                ChunkWeather::default(),
                ChunkSimLod::default(),
                // Forest already standing everywhere, water included.
                VegetationField {
                    canopy_density: 0.9,
                    understory_density: 0.6,
                    ..VegetationField::default()
                },
            ))
            .id();
        entities.push(e);
    }
    for _ in 0..40 {
        app.update();
    }

    let mut land_with_cover = 0usize;
    let mut water_cells = 0usize;
    for e in entities {
        let m = app.world().get::<ChunkCellMatrix>(e).unwrap().clone();
        let veg = *app.world().get::<VegetationField>(e).unwrap();
        let suit = app.world().get::<ChunkVegetationSuitability>(e).unwrap();
        let all_open = m.water.iter().all(|c| c.is_open_water());
        if all_open {
            assert_eq!(veg.canopy_density, 0.0, "all-water chunk keeps canopy");
            assert_eq!(veg.understory_density, 0.0);
            assert_eq!(veg.ground_fuel, 0.0);
            assert_eq!(veg.fuel_load, 0.0);
        }
        for (i, class) in m.water.iter().enumerate() {
            let density = suit.cell_density(veg.canopy_density, i);
            if class.is_open_water() {
                water_cells += 1;
                assert_eq!(density, 0.0, "{class:?} cell {i} carries cover");
                assert_eq!(suit.cell_cap(i), 0.0);
            } else if density > 0.0 {
                land_with_cover += 1;
            }
        }
    }
    assert!(water_cells > 0);
    assert!(land_with_cover > 0, "land must still grow");
}
