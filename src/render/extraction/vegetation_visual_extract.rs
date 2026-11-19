//! VEG-BURN-EXTRACT-004..005 — read-only vegetation extract frame from sim burn overlay.

use std::sync::Arc;

use bevy::prelude::*;


use crate::gui::ViewAuthoritySystemSet;
use crate::render::extraction::FireVisualFrameSet;
use crate::render::vegetation_species::VegSpeciesTable;
use crate::systems::ecology::{
    extract_glyph_for_burn, planning_glyph_for_burn, variant_key_for_burn_row, ActiveBurn,
    ChunkCellStand, ChunkVegetationSuitability, LandscapeProgramOnChunk, SuccessionState, SuccessionTopologyStage, VegetationPopulation,
    ACTIVE_BURN_HEAT_EPS, CELL_BURNING, VEG_BURN_FRAME_COUNT, VEG_BURN_FRAME_PERIOD_MS,
};
use crate::systems::sim_control::SimStepStamp;
use crate::terrain::generation::Chunk;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VegExtractModifiers {
    pub burn_active: bool,
    pub mean_density: f32,
}

#[derive(Clone, Debug)]
pub struct VegExtractRow {
    pub coord: IVec2,
    pub planning_glyph: char,
    pub extract_glyph: char,
    pub modifiers: VegExtractModifiers,
    pub variant_key: String,
    pub succession_stage: SuccessionTopologyStage,
    pub burn_active: bool,
    pub frame_index: u8,
    /// Per-cell stand state straight from the sim ([`ChunkCellStand::render`]): live stage, burning, burnt
    /// snags, regrowth, water. `None` for a chunk without per-cell stands.
    pub cells: Option<Arc<[u8]>>,
    /// Changes whenever `cells` does.
    pub cells_rev: u32,
    /// Species per cell ([`VegSpeciesTable::species_cell_codes`]), row-major like `cells`. A pure function
    /// of the chunk's cell matrix and the species table, cached until either changes. `None` without a
    /// matrix: consumers then use [`crate::render::vegetation_species::SPECIES_DEFAULT`].
    pub species: Option<Arc<[u8]>>,
    /// Changes whenever `species` is recomputed.
    pub species_rev: u32,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct VegetationExtractFrame {
    pub revision: u64,
    pub stamp: SimStepStamp,
    pub rows: Vec<VegExtractRow>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VegetationExtractFrameSet {
    BuildProfiles,
}

#[must_use]
fn variant_key_for_extract_row(
    program: Option<&LandscapeProgramOnChunk>,
    burn: Option<&ActiveBurn>,
    succ: SuccessionTopologyStage,
) -> String {
    if burn.is_some_and(|b| b.heat > ACTIVE_BURN_HEAT_EPS) {
        return variant_key_for_burn_row(burn, succ);
    }
    let Some(program) = program else {
        return "veg_topo_patch".to_owned();
    };
    let kind = program
        .evaluation
        .topology_kinds
        .first()
        .map(|s| s.as_str())
        .unwrap_or("Patch");
    format!(
        "veg_topo_{}",
        kind.chars()
            .flat_map(|c| {
                if c.is_ascii_alphanumeric() {
                    vec![c.to_ascii_lowercase()]
                } else {
                    vec![]
                }
            })
            .collect::<String>()
    )
}

fn center_cell_cover_scale(matrix: &crate::terrain::generation::ChunkCellMatrix) -> f32 {
    let scales = crate::terrain::generation::cell_canopy_scales(matrix);
    if scales.is_empty() {
        return 1.0;
    }
    let cx = matrix.size.x / 2;
    let cy = matrix.size.y / 2;
    scales
        .get(matrix.idx(cx, cy))
        .copied()
        .unwrap_or(1.0)
}

/// Biome gate: the row samples the centre cell, so water cells draw nothing and land cells get the
/// chunk cover re-based onto the land fraction. No suitability data leaves the density unchanged.
#[must_use]
fn vegetation_density_on_centre_cell(
    chunk_density: f32,
    matrix: Option<&crate::terrain::generation::ChunkCellMatrix>,
    suitability: Option<&ChunkVegetationSuitability>,
) -> f32 {
    match (matrix, suitability) {
        (Some(m), Some(suit)) => suit.cell_density(chunk_density, m.idx(m.size.x / 2, m.size.y / 2)),
        _ => chunk_density,
    }
}

fn shot_impact_in_chunk(
    book: Option<&crate::strategic::ShotPulseBook>,
    tick: u64,
    chunk: &Chunk,
    matrix: Option<&crate::terrain::generation::ChunkCellMatrix>,
) -> f32 {
    let (Some(book), Some(matrix)) = (book, matrix) else {
        return 0.0;
    };
    let origin_x = chunk.coord.x.saturating_mul(matrix.size.x as i32);
    let origin_y = chunk.coord.y.saturating_mul(matrix.size.y as i32);
    let w = matrix.size.x as i32;
    let h = matrix.size.y as i32;
    let mut best = 0.0f32;
    for shot in &book.recent {
        let tile = shot.target;
        if tile.x < origin_x || tile.y < origin_y || tile.x >= origin_x + w || tile.y >= origin_y + h
        {
            continue;
        }
        best = best.max(crate::strategic::shot_impact_at(shot, tick, tile));
    }
    best
}

/// Burning-tree flicker frame from the shared fire clock (`Time::elapsed_secs`, the clock the flame
/// and smoke particles animate on), so trees, flames and embers never run on different timers and
/// the flicker keeps moving while the sim is paused.
#[must_use]
pub fn veg_burn_frame_from_clock(time_s: f32) -> u8 {
    let ms = (time_s.max(0.0) * 1000.0) as u64;
    let frame = (ms / u64::from(VEG_BURN_FRAME_PERIOD_MS.max(1))) % u64::from(VEG_BURN_FRAME_COUNT.max(1));
    u8::try_from(frame).unwrap_or(0)
}

/// Whether a chunk row draws burning. Per-cell stand codes are the one truth (the same
/// `CELL_BURNING` the particles are gated on); the chunk's `ActiveBurn` heat is only the fallback
/// for a chunk without stand codes, so it can no longer keep a whole chunk burning after its cells burnt out.
#[must_use]
pub fn row_burn_active(cells: Option<&[u8]>, burn: Option<&ActiveBurn>) -> bool {
    match cells {
        Some(codes) => codes.contains(&CELL_BURNING),
        None => burn.is_some_and(|b| b.heat > ACTIVE_BURN_HEAT_EPS),
    }
}

pub fn build_vegetation_extract_frame(
    time: Res<Time>,
    tick: Res<crate::systems::sim_control::SimTick>,
    sim_time: Res<crate::systems::sim_control::SimTimeMicros>,
    shots: Option<Res<crate::strategic::ShotPulseBook>>,
    mut frame: ResMut<VegetationExtractFrame>,
    q: Query<(
        Entity,
        &Chunk,
        &SuccessionState,
        Option<&ActiveBurn>,
        Option<&VegetationPopulation>,
        Option<&LandscapeProgramOnChunk>,
        Option<Ref<crate::terrain::generation::ChunkCellMatrix>>,
        Option<&ChunkVegetationSuitability>,
        Option<&ChunkCellStand>,
    )>,
    mut centre_scale_cache: Local<
        bevy::platform::collections::HashMap<Entity, (bevy::ecs::change_detection::Tick, f32)>,
    >,
    species_table: Res<VegSpeciesTable>,
    mut species_cache: Local<
        bevy::platform::collections::HashMap<Entity, (bevy::ecs::change_detection::Tick, u32, Arc<[u8]>)>,
    >,
    mut species_rev_counter: Local<u32>,
) {
    // PERF-INSTR-VFX-001: name this per-chunk veg scan inside the Update wall window (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_veg_extract_frame");
    let stamp = SimStepStamp::new(tick.0, sim_time.0);
    let burn_frame = veg_burn_frame_from_clock(time.elapsed_secs());
    if frame.stamp == stamp && !frame.rows.is_empty() {
        // No new sim step (paused / between ticks): burning rows still follow the shared clock.
        if frame.rows.iter().any(|r| r.burn_active && r.frame_index != burn_frame) {
            for r in frame.rows.iter_mut().filter(|r| r.burn_active) {
                r.frame_index = burn_frame;
            }
            frame.revision = frame.revision.saturating_add(1);
        }
        return;
    }
    frame.revision = frame.revision.saturating_add(1);
    frame.stamp = stamp;
    frame.rows.clear();
    if species_table.is_changed() {
        species_cache.clear();
    }
    for (entity, chunk, succ, burn, pop, program, matrix, suitability, stand) in &q {
        let matrix_ref = matrix.as_ref();
        let matrix = matrix_ref.map(|m| &**m);
        let burn_active = row_burn_active(stand.map(|s| &s.render[..]), burn);
        let planning_glyph = planning_glyph_for_burn(burn, succ.stage);
        let extract_glyph = extract_glyph_for_burn(burn, succ.stage);
        let variant_key = variant_key_for_extract_row(program, burn, succ.stage);
        let base_density = pop.map(|p| p.mean_density).unwrap_or(0.0);
        let base_density = vegetation_density_on_centre_cell(base_density, matrix, suitability);
        // The centre-cell cover scale needs full-chunk slope + curvature passes; the matrix only
        // changes on (re)materialize, so cache it per chunk keyed on the matrix change tick.
        let cell_scale = matrix_ref.map_or(1.0, |m| {
            let changed_at = m.last_changed();
            match centre_scale_cache.get(&entity) {
                Some((tick, scale)) if *tick == changed_at => *scale,
                _ => {
                    let scale = center_cell_cover_scale(m);
                    centre_scale_cache.insert(entity, (changed_at, scale));
                    scale
                }
            }
        });
        let impact = shot_impact_in_chunk(shots.as_deref(), tick.0, chunk, matrix);
        // Species depends only on the matrix (static between materializations) and the table.
        let (species, species_rev) = match (matrix, matrix_ref) {
            (Some(m), Some(r)) => {
                let changed_at = r.last_changed();
                match species_cache.get(&entity) {
                    Some((t, rev, codes)) if *t == changed_at => (Some(codes.clone()), *rev),
                    _ => {
                        let codes = species_table.species_cell_codes(m, chunk.coord);
                        *species_rev_counter = species_rev_counter.wrapping_add(1);
                        species_cache.insert(entity, (changed_at, *species_rev_counter, codes.clone()));
                        (Some(codes), *species_rev_counter)
                    }
                }
            }
            _ => (None, 0),
        };
        frame.rows.push(VegExtractRow {
            coord: chunk.coord,
            planning_glyph,
            extract_glyph,
            modifiers: VegExtractModifiers {
                burn_active,
                mean_density: crate::systems::ecology::veg_draw_density_after_impact(
                    base_density,
                    cell_scale,
                    impact,
                ),
            },
            variant_key,
            succession_stage: succ.stage,
            burn_active,
            frame_index: if burn_active { burn_frame } else { 0 },
            cells: stand.map(|s| s.render.clone()),
            cells_rev: stand.map_or(0, |s| s.render_rev),
            species,
            species_rev,
        });
    }
    if centre_scale_cache.len() > frame.rows.len() {
        centre_scale_cache.retain(|e, _| q.contains(*e));
    }
    if species_cache.len() > frame.rows.len() {
        species_cache.retain(|e, _| q.contains(*e));
    }
}

#[must_use]
pub fn extract_glyph_deterministic(rows: &[VegExtractRow]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.extract_glyph.is_ascii())
}



/// Build extract rows from live program chunks without active burn (sim harness path).
#[cfg(test)]
pub fn harness_row_density(
    mean: f32,
    matrix: Option<&crate::terrain::generation::ChunkCellMatrix>,
) -> f32 {
    let scale = matrix.map(center_cell_cover_scale).unwrap_or(1.0);
    crate::systems::ecology::veg_draw_density(mean, scale)
}

/// Drawn density after snow. Depth 0 matches [`harness_row_density`].
#[must_use]
pub fn harness_row_density_under_snow(
    mean: f32,
    matrix: Option<&crate::terrain::generation::ChunkCellMatrix>,
    snow_depth: f32,
) -> f32 {
    let scale = matrix.map(center_cell_cover_scale).unwrap_or(1.0);
    crate::systems::ecology::veg_draw_density_under_snow(mean, scale, snow_depth)
}

#[must_use]
pub fn build_harness_topo_extract_frame(world: &mut World) -> VegetationExtractFrame {
    let mut rows = Vec::new();
    let mut q = world.query::<(
        &Chunk,
        &LandscapeProgramOnChunk,
        Option<&SuccessionState>,
        Option<&ActiveBurn>,
        Option<&VegetationPopulation>,
        Option<&crate::terrain::generation::ChunkCellMatrix>,
        Option<&crate::systems::weather::ChunkWeather>,
    )>();
    for (chunk, program, succ, burn, pop, matrix, weather) in q.iter(world) {
        if burn.is_some_and(|b| b.heat > ACTIVE_BURN_HEAT_EPS) {
            continue;
        }
        let stage = succ
            .map(|s| s.stage)
            .unwrap_or(SuccessionTopologyStage::YoungForest);
        let variant_key = variant_key_for_extract_row(Some(program), burn, stage);
        if !variant_key.starts_with("veg_topo_") {
            continue;
        }
        rows.push(VegExtractRow {
            coord: chunk.coord,
            planning_glyph: planning_glyph_for_burn(burn, stage),
            extract_glyph: extract_glyph_for_burn(burn, stage),
            modifiers: VegExtractModifiers {
                burn_active: false,
                mean_density: harness_row_density_under_snow(
                    pop.map(|p| p.mean_density).unwrap_or(0.0),
                    matrix,
                    weather.map(|wx| wx.snow_depth).unwrap_or(0.0),
                ),
            },
            variant_key,
            succession_stage: stage,
            burn_active: false,
            frame_index: 0,
            cells: None,
            cells_rev: 0,
            species: None,
            species_rev: 0,
        });
    }
    VegetationExtractFrame {
        revision: 1,
        stamp: SimStepStamp::new(0, 0),
        rows,
    }
}



pub struct VegetationVisualExtractPlugin;

impl Plugin for VegetationVisualExtractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VegetationExtractFrame>()
            .insert_resource(VegSpeciesTable::load())
            .configure_sets(
                Update,
                VegetationExtractFrameSet::BuildProfiles
                    .after(FireVisualFrameSet::BuildProfiles)
                    .after(ViewAuthoritySystemSet::SyncViewManager),
            )
            .add_systems(
                Update,
                build_vegetation_extract_frame.in_set(VegetationExtractFrameSet::BuildProfiles),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::{
        evaluate_landscape_program, load_landscape_grammar_catalog, ChunkEcology,
        LandscapeProgramOnChunk, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::weather::ChunkWeather;

    #[test]
    fn row_burns_exactly_when_a_cell_code_is_burning() {
        use crate::systems::ecology::{CELL_BURNT, CELL_GRASS};
        let hot = ActiveBurn { heat: 0.9, ..Default::default() };
        assert!(row_burn_active(Some(&[CELL_GRASS, CELL_BURNING]), None));
        assert!(
            !row_burn_active(Some(&[CELL_GRASS, CELL_BURNT]), Some(&hot)),
            "chunk ActiveBurn heat must not keep burnt-out cells drawn burning"
        );
        assert!(row_burn_active(None, Some(&hot)), "no stand codes: chunk burn is the fallback");
        assert!(!row_burn_active(None, None));
    }

    #[test]
    fn burn_frame_follows_wall_clock_and_wraps() {
        assert_eq!(veg_burn_frame_from_clock(0.0), 0);
        let period_s = VEG_BURN_FRAME_PERIOD_MS as f32 / 1000.0;
        assert_eq!(veg_burn_frame_from_clock(period_s * 1.5), 1);
        assert_eq!(veg_burn_frame_from_clock(period_s * f32::from(VEG_BURN_FRAME_COUNT) * 3.0 + 0.01), 0);
        assert_eq!(veg_burn_frame_from_clock(-5.0), 0);
    }

    #[test]
    fn water_centre_cell_draws_no_vegetation_and_land_is_rebased() {
        use crate::systems::ecology::{rate_chunk_vegetation_suitability, VegetationSuitabilityTable};
        use crate::terrain::generation::{ChunkCellMatrix, WaterClass};
        use crate::terrain::{default_terrain_families, TerrainFamilyId};

        let families = default_terrain_families();
        let id = |n: &str| -> TerrainFamilyId { families.id(n).unwrap() };
        let mut m = ChunkCellMatrix::new(UVec2::new(4, 4));
        for i in 0..16 {
            m.elevation[i] = 0.6;
            m.family[i] = id("DenseForest");
        }
        // Left half is ocean; the centre cell (2,2) is land.
        for y in 0..4 {
            for x in 0..2 {
                let i = m.idx(x, y);
                m.water[i] = WaterClass::Ocean;
            }
        }
        let suit = rate_chunk_vegetation_suitability(&m, families, &VegetationSuitabilityTable::default());
        let land = vegetation_density_on_centre_cell(0.4, Some(&m), Some(&suit));
        assert!((land - 0.8).abs() < 1e-6, "{land}");
        // Same chunk with the centre flipped to water draws nothing.
        let c = m.idx(2, 2);
        m.water[c] = WaterClass::Lake;
        let suit = rate_chunk_vegetation_suitability(&m, families, &VegetationSuitabilityTable::default());
        assert_eq!(vegetation_density_on_centre_cell(0.4, Some(&m), Some(&suit)), 0.0);
        assert_eq!(vegetation_density_on_centre_cell(0.4, Some(&m), None), 0.4);
    }

    #[test]
    fn vegetation_extract_uses_topo_variant_when_program_not_burning() {
        let catalog = load_landscape_grammar_catalog();
        let preset = catalog.presets.get(LG1_PILOT_PRESET_ID).expect("pilot");
        let eval = evaluate_landscape_program(
            preset,
            LG1_PILOT_CHUNK,
            &ChunkEcology::default(),
            &VegetationField::default(),
            &ChunkWeather::default(),
        );
        let program = LandscapeProgramOnChunk {
            preset_id: preset.preset_id.clone(),
            evaluation: eval,
        };
        let key = variant_key_for_extract_row(
            Some(&program),
            None,
            SuccessionTopologyStage::YoungForest,
        );
        assert!(
            key.starts_with("veg_topo_"),
            "expected veg_topo_* got {key}"
        );
        assert_ne!(key, "veg_burn_02");
    }

    #[test]
    fn harness_density_follows_center_curvature() {
        use crate::terrain::generation::ChunkCellMatrix;
        let size = bevy::math::UVec2::new(7, 7);
        let mut hollow = ChunkCellMatrix::new(size);
        let mut ridge = ChunkCellMatrix::new(size);
        for v in hollow.elevation.iter_mut().chain(ridge.elevation.iter_mut()) {
            *v = 0.4;
        }
        let center = hollow.idx(3, 3);
        hollow.elevation[center] = 0.05;
        ridge.elevation[center] = 0.95;
        let a = super::harness_row_density(0.6, Some(&hollow));
        let b = super::harness_row_density(0.6, Some(&ridge));
        assert!(a > b, "hollow {a} ridge {b}");
    }

    #[test]
    fn fold_harness_zero_mean_draws_nothing() {
        assert_eq!(super::harness_row_density(0.0, None), 0.0);
        assert_eq!(super::harness_row_density_under_snow(0.0, None, 1.0), 0.0);
        assert_eq!(super::harness_row_density(-1.0, None), 0.0);
    }

    #[test]
    fn fold_harness_overfull_mean_clamps_at_one() {
        let drawn = super::harness_row_density(4.0, None);
        let again = super::harness_row_density(4.0, None);
        let snow = super::harness_row_density_under_snow(4.0, None, 0.0);
        assert!((drawn - 1.0).abs() < 1e-5);
        assert_eq!(drawn.to_bits(), again.to_bits());
        assert!((snow - drawn).abs() < 1e-5);
    }
}
