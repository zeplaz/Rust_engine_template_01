//! Weather field systems: init / restore / preset, the step on the sim step clock, and the terrain bake.
//!
//! [`weather_field_step`] is the only system that writes [`WeatherField`]. Save hydrate and the test
//! harness hand it requests ([`PendingWeatherRestore`], [`WeatherPresetRequest`]) instead of writing it.
//! W4: reads [`WeatherSourceTerms`] (built previous frame) inside each solve.

use bevy::prelude::*;

use crate::systems::atmosphere::{AtmosphereDiagnostics, WeatherSourceTerms};
use crate::systems::sim_control::{SimStepClock, SimTimeMicros};
use crate::systems::water::SlowDomainTimeSkip;
use crate::terrain::biome::BiomeBucket;
use crate::terrain::family::default_terrain_families;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WorldWaterGrid};
use crate::terrain::material::MaterializedChunk;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::field::{bake_weather_terrain, TerrainBakeAccum, WeatherField, WeatherFieldSave};
use super::tuning::{WeatherPresetId, WeatherTuning};

/// Scenario / test request to switch the field to a preset (consumed by [`weather_field_step`]).
#[derive(Resource, Debug, Default, Clone)]
pub struct WeatherPresetRequest(pub Option<WeatherPresetId>);

/// Loaded save state waiting for [`weather_field_step`] to adopt it.
#[derive(Resource, Debug, Default, Clone)]
pub struct PendingWeatherRestore(pub Option<WeatherFieldSave>);

/// Weather steps one frame may run when the sim clock catches up (backlog beyond is dropped).
const MAX_WEATHER_STEPS_PER_FRAME: u32 = 4;

fn terrain_key(chunks: usize, water: Option<&WorldWaterGrid>) -> (u64, u64) {
    (chunks as u64, water.map_or(0, |w| w.stamp))
}

/// Accumulate materialised chunk cells into weather cells and bake the factors once.
fn bake_from_chunks(
    field: &WeatherField,
    tuning: &WeatherTuning,
    chunks: &Query<(&Chunk, &ChunkCellMatrix), With<MaterializedChunk>>,
) -> super::field::WeatherTerrain {
    let families = default_terrain_families();
    let rough_family: Vec<bool> = (0..families.families.len())
        .map(|i| {
            let id = crate::terrain::family::TerrainFamilyId(i as u16);
            families.biome_bucket(id) == Some(BiomeBucket::Forest)
                || families.def(id).is_some_and(|d| d.name == "Concrete")
        })
        .collect();
    let mut acc = TerrainBakeAccum::new(field.cell_count());
    for (chunk, m) in chunks.iter() {
        let (sx, sy) = (m.size.x as i32, m.size.y as i32);
        if m.elevation.len() != (sx * sy) as usize {
            continue;
        }
        for y in 0..sy {
            for x in 0..sx {
                let i = (y * sx + x) as usize;
                let tx = chunk.coord.x * sx + x;
                let ty = chunk.coord.y * sy + y;
                if tx < 0 || ty < 0 {
                    continue;
                }
                let pos = Vec2::new(tx as f32 + 0.5, ty as f32 + 0.5) * TILE_METRES_GAMEPLAY;
                if pos.x >= field.extent_m.x || pos.y >= field.extent_m.y {
                    continue;
                }
                let cell = field.cell_index_at(pos);
                let rough = m
                    .family
                    .get(i)
                    .is_some_and(|f| rough_family.get(f.0 as usize).copied().unwrap_or(false));
                let water = m.water.get(i).is_some_and(|w| w.is_open_water());
                acc.add(cell, m.elevation[i], rough, water);
            }
        }
    }
    bake_weather_terrain(field.dims, field.cell_size_m, &acc, tuning)
}

/// Init / restore / preset, then one solve per `weather_step_secs` of sim steps.
#[allow(clippy::too_many_arguments)]
pub fn weather_field_step(
    tuning: Res<WeatherTuning>,
    clock: Option<Res<SimStepClock>>,
    sim_time: Option<Res<SimTimeMicros>>,
    params: Option<Res<WorldGenParams>>,
    water: Option<Res<WorldWaterGrid>>,
    skip: Option<Res<SlowDomainTimeSkip>>,
    chunks: Query<(&Chunk, &ChunkCellMatrix), With<MaterializedChunk>>,
    mut preset_req: ResMut<WeatherPresetRequest>,
    mut restore: ResMut<PendingWeatherRestore>,
    mut field: ResMut<WeatherField>,
    mut terms: Option<ResMut<WeatherSourceTerms>>,
    mut diag: Option<ResMut<AtmosphereDiagnostics>>,
) {
    let Some(params) = params else {
        return;
    };
    let now = sim_time.map_or(0, |t| t.0);
    let chunk_count = chunks.iter().count();
    let key = terrain_key(chunk_count, water.as_deref());
    let extent = Vec2::new(params.width as f32, params.height as f32) * TILE_METRES_GAMEPLAY;

    if let Some(save) = restore.0.take() {
        match WeatherField::from_save(&save, &tuning, now) {
            Ok(mut restored) => {
                let mut terrain = bake_from_chunks(&restored, &tuning, &chunks);
                terrain.key = Some(key);
                restored.set_terrain(terrain);
                restored.revision = field.revision.wrapping_add(1);
                *field = restored;
                return;
            }
            Err(err) => warn!("weather save restore skipped: {err}"),
        }
    }

    let world_changed = !field.is_ready() || field.seed != params.seed || field.extent_m != extent;
    let preset = preset_req.0.take();
    if world_changed || preset.is_some() {
        // Wait for the terrain so the first solve already sees ridges, forests and water.
        if chunk_count == 0 {
            if let Some(p) = preset {
                preset_req.0 = Some(p);
            }
            return;
        }
        let preset = preset.or(if world_changed { tuning.weather_preset } else { field.climate.preset });
        let probe = WeatherField::new(params.seed, extent, &tuning, None, Default::default(), now);
        let mut terrain = bake_from_chunks(&probe, &tuning, &chunks);
        terrain.key = Some(key);
        let revision = field.revision;
        *field = WeatherField::new(params.seed, extent, &tuning, preset, terrain, now);
        field.revision = revision.wrapping_add(1);
        return;
    }

    let Some(clock) = clock else {
        return;
    };
    if clock.due == 0 {
        return;
    }
    let steps_per_solve = ((field.step_secs as f32 / clock.step_secs().max(1e-3)).round() as u32).max(1);
    // DF-1 D4: time-skip advances weather means with coarse dt (same as ecology/level-pool).
    let due_steps = if let Some(skip) = skip.filter(|s| s.enabled) {
        let slow = skip.dt_slow_secs.max(1);
        let step_s = clock.step_secs().max(1e-3);
        ((slow as f32 / step_s).round() as u32)
            .saturating_mul(clock.due)
            .max(clock.due)
    } else {
        clock.due
    };
    field.steps_into_interval = field.steps_into_interval.saturating_add(due_steps);
    let mut solved = 0;
    while field.steps_into_interval >= steps_per_solve {
        field.steps_into_interval -= steps_per_solve;
        if solved >= MAX_WEATHER_STEPS_PER_FRAME {
            field.steps_into_interval %= steps_per_solve;
            break;
        }
        if field.terrain.key != Some(key) {
            let mut terrain = bake_from_chunks(&field, &tuning, &chunks);
            terrain.key = Some(key);
            field.set_terrain(terrain);
        }
        let (moisture, cloud_delta) = match terms.as_ref() {
            Some(t) => field.advance_with_terms(&tuning, now, Some(t.as_ref())),
            None => field.advance_with_terms(&tuning, now, None),
        };
        if let Some(t) = terms.as_mut() {
            t.record_solve_deltas(moisture, cloud_delta);
        }
        // CQ-36: AtmosphereDiagnostics holds solve-applied moisture/cloud only.
        if let Some(d) = diag.as_mut() {
            if moisture > 0.0 || cloud_delta > 0.0 {
                d.pyro_moisture_injected = moisture;
                d.pyro_cloud_delta = cloud_delta;
            }
        }
        solved += 1;
    }
}

/// Load the tuning table once (disk copy over the compiled-in default).
pub fn load_weather_tuning_on_startup(mut commands: Commands) {
    let (tuning, err) = super::tuning::load_weather_tuning();
    if let Some(err) = err {
        warn!("weather tuning: using compiled-in default ({err})");
    }
    commands.insert_resource(tuning);
}
