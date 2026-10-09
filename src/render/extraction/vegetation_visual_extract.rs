//! VEG-BURN-EXTRACT-004..005 — read-only vegetation extract frame from sim burn overlay.

use bevy::prelude::*;
use serde_json::json;

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::gui::ViewAuthoritySystemSet;
use crate::render::extraction::FireVisualFrameSet;
use crate::systems::ecology::{
    extract_glyph_for_burn, planning_glyph_for_burn, variant_key_for_burn_row, ActiveBurn,
    LandscapeProgramOnChunk, SuccessionState, SuccessionTopologyStage, VegetationPopulation,
    ACTIVE_BURN_HEAT_EPS,
};
use crate::systems::sim_control::SimStepStamp;
use crate::terrain::generation::Chunk;

pub const LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_extract_live.json";

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

pub fn build_vegetation_extract_frame(
    tick: Res<crate::systems::sim_control::SimTick>,
    sim_time: Res<crate::systems::sim_control::SimTimeMicros>,
    shots: Option<Res<crate::strategic::ShotPulseBook>>,
    mut frame: ResMut<VegetationExtractFrame>,
    q: Query<(
        &Chunk,
        &SuccessionState,
        Option<&ActiveBurn>,
        Option<&VegetationPopulation>,
        Option<&LandscapeProgramOnChunk>,
        Option<&crate::terrain::generation::ChunkCellMatrix>,
    )>,
) {
    // PERF-INSTR-VFX-001: name this per-chunk veg scan inside the Update wall window (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_veg_extract_frame");
    let stamp = SimStepStamp::new(tick.0, sim_time.0);
    if frame.stamp == stamp && !frame.rows.is_empty() {
        return;
    }
    frame.revision = frame.revision.saturating_add(1);
    frame.stamp = stamp;
    frame.rows.clear();
    for (chunk, succ, burn, pop, program, matrix) in &q {
        let burn_active = burn.is_some_and(|b| b.heat > crate::systems::ecology::ACTIVE_BURN_HEAT_EPS);
        let planning_glyph = planning_glyph_for_burn(burn, succ.stage);
        let extract_glyph = extract_glyph_for_burn(burn, succ.stage);
        let variant_key = variant_key_for_extract_row(program, burn, succ.stage);
        let base_density = pop.map(|p| p.mean_density).unwrap_or(0.0);
        let cell_scale = matrix
            .map(center_cell_cover_scale)
            .unwrap_or(1.0);
        let impact = shot_impact_in_chunk(shots.as_deref(), tick.0, chunk, matrix);
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
            frame_index: burn.map(|b| b.frame_index).unwrap_or(0),
        });
    }
}

#[must_use]
pub fn vegetation_extract_witness_green(frame: &VegetationExtractFrame) -> bool {
    frame.rows.iter().any(|r| r.burn_active)
        && frame
            .rows
            .iter()
            .any(|r| r.variant_key.starts_with("veg_burn_") || r.variant_key.starts_with("veg_topo_"))
        && frame.rows.iter().any(|r| r.extract_glyph != '#')
}

#[must_use]
pub fn extract_glyph_deterministic(rows: &[VegExtractRow]) -> bool {
    !rows.is_empty() && rows.iter().all(|r| r.extract_glyph.is_ascii())
}

#[must_use]
pub fn refresh_vegetation_extract_witness(frame: &VegetationExtractFrame) -> bool {
    let green = vegetation_extract_witness_green(frame) && extract_glyph_deterministic(&frame.rows);
    let burn_rows = frame.rows.iter().filter(|r| r.burn_active).count();
    let body = json!({
        "gate": "VEG-BURN-EXTRACT-004",
        "green": green,
        "revision": frame.revision,
        "row_count": frame.rows.len(),
        "burn_active_rows": burn_rows,
        "extract_glyph_deterministic": extract_glyph_deterministic(&frame.rows),
        "sprite_variant_from_program": frame.rows.iter().any(|r| r.variant_key.starts_with("veg_topo_")),
        "sample_variant_keys": frame.rows.iter().take(4).map(|r| r.variant_key.clone()).collect::<Vec<_>>(),
    });
    let wrapped = wrap_debug_run(
        "VEG-BURN-EXTRACT-004",
        "refresh_vegetation_extract_witness",
        LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON, wrapped);
    green
}

/// CDR-A-EXTRACT-SPRITE-001 — program topology variant keys (non-burn rows).
#[must_use]
pub fn harness_topo_extract_witness_green(frame: &VegetationExtractFrame) -> bool {
    frame
        .rows
        .iter()
        .filter(|r| r.variant_key.starts_with("veg_topo_"))
        .count()
        >= 1
        && extract_glyph_deterministic(&frame.rows)
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
        });
    }
    VegetationExtractFrame {
        revision: 1,
        stamp: SimStepStamp::new(0, 0),
        rows,
    }
}

/// Refresh extract witness from sim harness program rows (topology sprite keys).
#[must_use]
pub fn refresh_landscape_extract_sprite_witness(frame: &VegetationExtractFrame) -> bool {
    let green = harness_topo_extract_witness_green(frame);
    let topo_rows = frame
        .rows
        .iter()
        .filter(|r| r.variant_key.starts_with("veg_topo_"))
        .count();
    let body = json!({
        "gate": "CDR-A-EXTRACT-SPRITE-001",
        "gate_id": "VEG-BURN-EXTRACT-004",
        "slice_id": "CDR-A-EXTRACT-SPRITE-001",
        "green": green,
        "revision": frame.revision,
        "row_count": frame.rows.len(),
        "topo_sprite_rows": topo_rows,
        "burn_active_rows": 0,
        "extract_glyph_deterministic": extract_glyph_deterministic(&frame.rows),
        "sprite_variant_from_program": topo_rows >= 1,
        "sample_variant_keys": frame.rows.iter().take(8).map(|r| r.variant_key.clone()).collect::<Vec<_>>(),
    });
    let wrapped = wrap_debug_run(
        "CDR-A-EXTRACT-SPRITE-001",
        "refresh_landscape_extract_sprite_witness",
        LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON, wrapped) && green
}

pub struct VegetationVisualExtractPlugin;

impl Plugin for VegetationVisualExtractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VegetationExtractFrame>()
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
        evaluate_landscape_program, load_landscape_grammar_catalog, ActiveBurn, ChunkEcology,
        LandscapeProgramOnChunk, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::weather::ChunkWeather;

    #[test]
    fn vegetation_extract_builds_burn_rows() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<crate::systems::sim_control::SimTick>()
            .init_resource::<crate::systems::sim_control::SimTimeMicros>()
            .add_plugins(VegetationVisualExtractPlugin);

        app.world_mut().spawn((
            Chunk {
                coord: LG1_PILOT_CHUNK,
            },
            SuccessionState {
                stage: SuccessionTopologyStage::BurnScar,
                ..Default::default()
            },
            ActiveBurn {
                heat: 0.7,
                frame_index: 3,
                started_tick: 0,
                severity: 0.7,
                ..Default::default()
            },
        ));

        app.update();
        let frame = app.world().resource::<VegetationExtractFrame>();
        assert!(vegetation_extract_witness_green(frame));
        assert!(frame.rows.iter().any(|r| r.variant_key.starts_with("veg_burn_")));
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
    fn harness_topo_extract_frame_from_program_rows() {
        use crate::systems::ecology::{
            evaluate_landscape_program, load_landscape_grammar_catalog, ChunkEcology,
            LG1_PILOT_PRESET_ID, VegetationField,
        };
        use crate::systems::weather::ChunkWeather;
        use crate::terrain::generation::Chunk;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let catalog = load_landscape_grammar_catalog();
        let preset = catalog.presets.get(LG1_PILOT_PRESET_ID).expect("pilot");
        for (cx, cy) in [(0, 0), (1, 1), (2, 2)] {
            let eval = evaluate_landscape_program(
                preset,
                IVec2::new(cx, cy),
                &ChunkEcology::default(),
                &VegetationField::default(),
                &ChunkWeather::default(),
            );
            app.world_mut().spawn((
                Chunk {
                    coord: IVec2::new(cx, cy),
                },
                LandscapeProgramOnChunk {
                    preset_id: preset.preset_id.clone(),
                    evaluation: eval,
                },
            ));
        }
        let frame = super::build_harness_topo_extract_frame(app.world_mut());
        assert!(super::harness_topo_extract_witness_green(&frame));
        assert!(frame.rows.iter().any(|r| r.variant_key.starts_with("veg_topo_")));
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
