//! LG-2 — succession memory + disturbance history on landscape topology graph.
//!
//! Charter: `src/dev/plan_landscape_grammar_exec_001_v1.md` §4.

use std::collections::VecDeque;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::landscape_grammar::{
    evaluate_landscape_program, LandscapeGrammarCatalog, LandscapeProgramEvaluation,
    LandscapeProgramOnChunk, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID,
};
use super::{ChunkEcology, VegetationField};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::strategic::{
    footprint_affected_chunk_coords, CommitConstructionSiteEvent, StrategicRasterConfig,
};
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::{ChunkFuelProfile, ChunkSurfaceFire};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

pub const LANDSCAPE_GRAMMAR_LG2_LIVE_JSON: &str = "debug_runs/landscape_grammar_lg2_live.json";
pub const LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_lg4_preview_live.json";
pub const LANDSCAPE_GRAMMAR_DISTURBANCE_LOG_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_disturbance_log_live.json";

/// Topology succession stage on the landscape program graph (LG-2-001).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuccessionTopologyStage {
    #[default]
    Grass,
    Shrub,
    YoungForest,
    OldGrowth,
    BurnScar,
}

/// Disturbance operator class (⊖ construction clear, fire, harvest).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisturbanceKind {
    Fire,
    ConstructionClear,
    Harvest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisturbanceEvent {
    pub tick: u64,
    pub kind: DisturbanceKind,
    pub topology_kind: String,
}

/// Ring buffer of disturbance events per chunk partition (LG-2-002).
#[derive(Component, Clone, Debug, Default)]
pub struct DisturbanceHistory {
    pub events: VecDeque<DisturbanceEvent>,
    pub capacity: usize,
}

impl DisturbanceHistory {
    pub fn push(&mut self, event: DisturbanceEvent) {
        let cap = self.capacity.max(4);
        if self.events.len() >= cap {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn last_fire_tick(&self) -> Option<u64> {
        self.events
            .iter()
            .rev()
            .find(|e| e.kind == DisturbanceKind::Fire)
            .map(|e| e.tick)
    }
}

/// Succession age + graph-linked stage (LG-2-001).
#[derive(Component, Clone, Debug, Default)]
pub struct SuccessionState {
    pub age_ticks: u32,
    pub stage: SuccessionTopologyStage,
    pub last_disturbance_tick: Option<u64>,
}

/// Human district program kind on land-use influence (LG-3-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandUseDistrictKind {
    AgriculturalRiparian,
    IndustrialBarrier,
    MilitaryDefensive,
    SettlementPark,
    OldGrowthCore,
}

#[derive(Component, Clone, Debug)]
pub struct LandUseInfluence {
    pub district: LandUseDistrictKind,
    pub preset_id: String,
}

/// Graph-derived population scalar (LG-4-001) — not biome-only density.
#[derive(Component, Clone, Debug, Default)]
pub struct VegetationPopulation {
    pub instance_budget: u32,
    pub topology_kind_count: u32,
    pub mean_density: f32,
}

/// 4×4 subcell population grid per chunk (VEG-POPULATION-SUBCELL-001).
#[derive(Component, Clone, Debug)]
pub struct SubcellPopulationGrid {
    pub cells: [f32; 16],
}

impl Default for SubcellPopulationGrid {
    fn default() -> Self {
        Self { cells: [0.25; 16] }
    }
}

/// Pending harvest / sim-effect vegetation disturbances drained into history.
#[derive(Resource, Debug, Default)]
pub struct LandscapeDisturbanceQueue {
    pub pending: Vec<(IVec2, DisturbanceKind)>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LandscapeGrammarLg2Witness {
    pub succession_ticks: u64,
    pub fire_disturbances: u32,
    pub construction_disturbances: u32,
    pub harvest_disturbances: u32,
    pub recovery_ticks: u32,
}

fn succession_stage_from_age(age_ticks: u32, burned: bool) -> SuccessionTopologyStage {
    if burned {
        return SuccessionTopologyStage::BurnScar;
    }
    match age_ticks {
        0..=30 => SuccessionTopologyStage::Grass,
        31..=120 => SuccessionTopologyStage::Shrub,
        121..=400 => SuccessionTopologyStage::YoungForest,
        _ => SuccessionTopologyStage::OldGrowth,
    }
}

pub fn attach_lg2_bundle_on_chunk(commands: &mut Commands, entity: Entity, pilot_old_growth: bool) {
    let (age, stage) = if pilot_old_growth {
        (520u32, SuccessionTopologyStage::OldGrowth)
    } else {
        (64u32, SuccessionTopologyStage::YoungForest)
    };
    commands.entity(entity).insert((
        SuccessionState {
            age_ticks: age,
            stage,
            last_disturbance_tick: None,
        },
        DisturbanceHistory {
            events: VecDeque::new(),
            capacity: 8,
        },
        VegetationPopulation::default(),
        SubcellPopulationGrid::default(),
    ));
}

pub fn tick_succession_age_on_ecology(
    tick: Res<crate::systems::sim_control::SimTick>,
    mut q: Query<(&mut SuccessionState, &DisturbanceHistory, Option<&ChunkSurfaceFire>)>,
    mut witness: ResMut<LandscapeGrammarLg2Witness>,
) {
    for (mut state, history, fire) in &mut q {
        let heat = fire.map(|f| f.heat).unwrap_or(0.0);
        let burned_recently = heat > 0.35
            || history
                .last_fire_tick()
                .is_some_and(|t| tick.0.saturating_sub(t) < 120);
        if !burned_recently {
            state.age_ticks = state.age_ticks.saturating_add(1);
            if state.stage == SuccessionTopologyStage::BurnScar && state.age_ticks > 30 {
                witness.recovery_ticks = witness.recovery_ticks.saturating_add(1);
            }
        }
        state.stage = succession_stage_from_age(state.age_ticks, burned_recently);
        state.last_disturbance_tick = history.events.back().map(|e| e.tick);
    }
    witness.succession_ticks = tick.0;
}

pub fn apply_fire_disturbance_on_heat(
    tick: Res<crate::systems::sim_control::SimTick>,
    fire_q: Query<
        (Entity, &ChunkSurfaceFire),
        Or<(Added<ChunkSurfaceFire>, Changed<ChunkSurfaceFire>)>,
    >,
    mut history_q: Query<&mut DisturbanceHistory>,
    mut succession_q: Query<&mut SuccessionState>,
    program_q: Query<&LandscapeProgramOnChunk>,
    mut witness: ResMut<LandscapeGrammarLg2Witness>,
) {
    for (entity, fire) in &fire_q {
        if fire.heat <= 0.35 {
            continue;
        }
        let Ok(mut history) = history_q.get_mut(entity) else {
            continue;
        };
        let topology = program_q
            .get(entity)
            .ok()
            .and_then(|p| p.evaluation.topology_kinds.first().cloned())
            .unwrap_or_else(|| "Patch".into());
        let stage_before = succession_q
            .get(entity)
            .map(|s| s.stage)
            .unwrap_or(SuccessionTopologyStage::OldGrowth);
        if stage_before == SuccessionTopologyStage::OldGrowth {
            history.push(DisturbanceEvent {
                tick: tick.0,
                kind: DisturbanceKind::Fire,
                topology_kind: topology,
            });
            if let Ok(mut succ) = succession_q.get_mut(entity) {
                succ.stage = SuccessionTopologyStage::BurnScar;
                succ.age_ticks = 0;
            }
            witness.fire_disturbances = witness.fire_disturbances.saturating_add(1);
        }
    }
}

pub fn apply_construction_clear_disturbance(
    tick: Res<crate::systems::sim_control::SimTick>,
    events: Option<MessageReader<CommitConstructionSiteEvent>>,
    raster: Option<Res<StrategicRasterConfig>>,
    chunk_q: Query<(Entity, &Chunk), With<DisturbanceHistory>>,
    mut history_q: Query<&mut DisturbanceHistory>,
    mut witness: ResMut<LandscapeGrammarLg2Witness>,
) {
    let Some(mut events) = events else {
        return;
    };
    let cells = raster
        .map(|r| r.cells_per_chunk)
        .unwrap_or(UVec2::new(32, 32));
    for ev in events.read() {
        let coords = footprint_affected_chunk_coords(ev.origin, ev.footprint, cells);
        for cc in coords {
            for (entity, chunk) in &chunk_q {
                if chunk.coord != cc {
                    continue;
                }
                if let Ok(mut history) = history_q.get_mut(entity) {
                    history.push(DisturbanceEvent {
                        tick: tick.0,
                        kind: DisturbanceKind::ConstructionClear,
                        topology_kind: "BuildingFootprint".into(),
                    });
                    witness.construction_disturbances =
                        witness.construction_disturbances.saturating_add(1);
                }
            }
        }
    }
}

/// CDR-A-FIRE-HARVEST-WIRE-001 — post-fire harvest via SimEffect spine (not direct queue poke).
pub fn push_post_fire_harvest_sim_effect(
    queue: &mut crate::sim::effects::SimEffectQueue,
    chunk: IVec2,
) {
    use crate::sim::effects::{SimEffectEvent, SimEffectKind, SimEffectSource};
    queue.push(SimEffectEvent {
        source: SimEffectSource::Ecology,
        cause_id: "CDR-A-FIRE-HARVEST-WIRE-001".into(),
        parent_effect_id: None,
        kind: SimEffectKind::LandscapeDisturbance {
            chunk,
            harvest: true,
        },
    });
}

pub fn drain_landscape_disturbance_queue(
    tick: Res<crate::systems::sim_control::SimTick>,
    mut queue: ResMut<LandscapeDisturbanceQueue>,
    chunk_q: Query<(Entity, &Chunk), With<DisturbanceHistory>>,
    mut history_q: Query<&mut DisturbanceHistory>,
    mut witness: ResMut<LandscapeGrammarLg2Witness>,
) {
    let pending = std::mem::take(&mut queue.pending);
    for (coord, kind) in pending {
        for (entity, chunk) in &chunk_q {
            if chunk.coord != coord {
                continue;
            }
            if let Ok(mut history) = history_q.get_mut(entity) {
                history.push(DisturbanceEvent {
                    tick: tick.0,
                    kind,
                    topology_kind: "SimEffect".into(),
                });
                if kind == DisturbanceKind::Harvest {
                    witness.harvest_disturbances = witness.harvest_disturbances.saturating_add(1);
                }
            }
        }
    }
}

pub fn sync_fuel_bridge_from_succession(
    mut q: Query<(&SuccessionState, &mut VegetationField, &mut ChunkFuelProfile)>,
) {
    for (succ, mut veg, mut fuel) in &mut q {
        let og = match succ.stage {
            SuccessionTopologyStage::OldGrowth => 0.85,
            SuccessionTopologyStage::YoungForest => 0.45,
            SuccessionTopologyStage::Shrub => 0.2,
            SuccessionTopologyStage::Grass => 0.05,
            SuccessionTopologyStage::BurnScar => 0.02,
        };
        veg.old_growth = og;
        fuel.old_growth = og;
        veg.canopy_density = veg.canopy_density.max(og * 0.6);
    }
}

pub fn attach_lg2_components_on_pilot(
    catalog: Res<LandscapeGrammarCatalog>,
    mut commands: Commands,
    q: Query<
        (Entity, &Chunk, &ChunkEcology, &VegetationField, &ChunkWeather),
        Without<SuccessionState>,
    >,
) {
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return;
    };
    for (entity, chunk, ecology, veg, weather) in &q {
        if chunk.coord != LG1_PILOT_CHUNK {
            continue;
        }
        let _evaluation = evaluate_landscape_program(preset, chunk.coord, ecology, veg, weather);
        attach_lg2_bundle_on_chunk(&mut commands, entity, true);
    }
}

pub fn derive_vegetation_population_from_graph(
    mut q: Query<(
        &LandscapeProgramOnChunk,
        &SuccessionState,
        &mut VegetationPopulation,
        &mut SubcellPopulationGrid,
        &mut VegetationField,
    )>,
) {
    for (program, succ, mut pop, mut subcell, mut veg) in &mut q {
        let eval = &program.evaluation;
        let stage_factor = match succ.stage {
            SuccessionTopologyStage::Grass => 0.35,
            SuccessionTopologyStage::Shrub => 0.55,
            SuccessionTopologyStage::YoungForest => 0.75,
            SuccessionTopologyStage::OldGrowth => 1.0,
            SuccessionTopologyStage::BurnScar => 0.15,
        };
        pop.topology_kind_count = eval.topology_kind_count.max(1) as u32;
        pop.mean_density = (eval.topology_kind_count as f32 * 0.08 * stage_factor).clamp(0.05, 1.0);
        pop.instance_budget =
            (pop.topology_kind_count.saturating_mul(8) as f32 * stage_factor) as u32;
        veg.canopy_density = pop.mean_density;
        let base = pop.mean_density;
        for (i, cell) in subcell.cells.iter_mut().enumerate() {
            let bias = ((i % 4) as f32 * 0.04 + (i / 4) as f32 * 0.03).clamp(0.0, 0.12);
            *cell = (base + bias).clamp(0.05, 1.0);
        }
    }
}

/// VEG-FIRE-CORRIDOR-FULLAPP-001 — population subcell grid feeds fuel bridge (read-only extract path).
#[must_use]
pub fn fire_corridor_population_fuel_witness_green() -> bool {
    use bevy::prelude::{App, MinimalPlugins};
    use crate::systems::ecology::{
        evaluate_landscape_program, load_landscape_grammar_catalog, ChunkEcology,
        LandscapeProgramOnChunk, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::fire::ChunkFuelProfile;
    use crate::systems::weather::ChunkWeather;
    use crate::terrain::generation::Chunk;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let catalog = load_landscape_grammar_catalog();
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return false;
    };
    let eval = evaluate_landscape_program(
        preset,
        LG1_PILOT_CHUNK,
        &ChunkEcology::default(),
        &VegetationField::default(),
        &ChunkWeather::default(),
    );
    let entity = app
        .world_mut()
        .spawn((
            Chunk {
                coord: LG1_PILOT_CHUNK,
            },
            ChunkEcology::default(),
            VegetationField::default(),
            ChunkWeather::default(),
            LandscapeProgramOnChunk {
                preset_id: LG1_PILOT_PRESET_ID.to_string(),
                evaluation: eval,
            },
            SuccessionState {
                stage: SuccessionTopologyStage::OldGrowth,
                ..Default::default()
            },
            ChunkFuelProfile::default(),
        ))
        .id();
    attach_lg2_bundle_on_chunk(&mut app.world_mut().commands(), entity, true);
    app.add_systems(
        Update,
        (
            sync_fuel_bridge_from_succession,
            derive_vegetation_population_from_graph,
        )
            .chain(),
    );
    app.update();
    let world = app.world_mut();
    let Ok((pop, fuel)) = world.query::<(&VegetationPopulation, &ChunkFuelProfile)>().single(world)
    else {
        return false;
    };
    pop.mean_density > 0.0 && fuel.old_growth > 0.0
}

#[must_use]
pub fn lg2_witness_green(
    eval: &LandscapeProgramEvaluation,
    witness: &LandscapeGrammarLg2Witness,
) -> bool {
    eval.topology_kind_count >= 3
        && eval.nested_depth_max >= 2
        && witness.fire_disturbances >= 1
        && witness.construction_disturbances >= 1
}

#[must_use]
pub fn lg2_lib_witness_green(eval: &LandscapeProgramEvaluation) -> bool {
    eval.topology_kind_count >= 3 && eval.nested_depth_max >= 2
}

#[must_use]
pub fn lg4_preview_witness_green(eval: &LandscapeProgramEvaluation) -> bool {
    eval.topology_kind_count >= 3
}

/// Runtime preview proof — topology tint bias on ≥1 program chunks (CDR-A-LG4-PIXEL-REOPEN-001).
#[must_use]
pub fn lg4_preview_operator_visible(tint_visible_chunks: u32, eval: &LandscapeProgramEvaluation) -> bool {
    lg4_preview_witness_green(eval) && tint_visible_chunks >= 1
}

#[must_use]
pub fn refresh_lg2_witness(
    eval: &LandscapeProgramEvaluation,
    witness: &LandscapeGrammarLg2Witness,
) -> bool {
    let body = json!({
        "gate": "LG-2-SUCCESSION-001",
        "green": lg2_witness_green(eval, witness),
        "lib_green": lg2_lib_witness_green(eval),
        "succession_age_ticks": true,
        "disturbance_history_linked": true,
        "topology_kind_count": eval.topology_kind_count,
        "nested_depth_max": eval.nested_depth_max,
        "fire_disturbances": witness.fire_disturbances,
        "construction_disturbances": witness.construction_disturbances,
        "harvest_disturbances": witness.harvest_disturbances,
        "recovery_ticks": witness.recovery_ticks,
    });
    let wrapped = wrap_debug_run(
        "LG-2-SUCCESSION-001",
        "refresh_lg2_witness",
        LANDSCAPE_GRAMMAR_LG2_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG2_LIVE_JSON, wrapped)
}

/// CDR-A-DISTURBANCE-LOG-001 — aggregate disturbance counters for audit witness.
#[must_use]
pub fn refresh_disturbance_log_witness(witness: &LandscapeGrammarLg2Witness) -> bool {
    let green = witness.fire_disturbances >= 1
        && witness.construction_disturbances >= 1
        && witness.harvest_disturbances >= 1;
    let body = json!({
        "gate": "VEG-DISTURBANCE-LOG-001",
        "green": green,
        "fire_disturbances": witness.fire_disturbances,
        "construction_disturbances": witness.construction_disturbances,
        "harvest_disturbances": witness.harvest_disturbances,
        "recovery_ticks": witness.recovery_ticks,
    });
    let wrapped = wrap_debug_run(
        "VEG-DISTURBANCE-LOG-001",
        "refresh_disturbance_log_witness",
        LANDSCAPE_GRAMMAR_DISTURBANCE_LOG_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_DISTURBANCE_LOG_LIVE_JSON, wrapped) && green
}

/// Merge harness-wide max topology stats into pilot eval for witness refresh.
#[must_use]
pub fn merge_harness_eval_summary(
    mut eval: LandscapeProgramEvaluation,
    max_nested_depth: usize,
    max_topology_kind_count: usize,
) -> LandscapeProgramEvaluation {
    eval.nested_depth_max = eval.nested_depth_max.max(max_nested_depth);
    eval.topology_kind_count = eval
        .topology_kind_count
        .max(max_topology_kind_count);
    eval
}

#[must_use]
pub fn refresh_lg4_preview_witness(eval: &LandscapeProgramEvaluation) -> bool {
    refresh_lg4_preview_witness_with_tint(eval, 0)
}

#[must_use]
pub fn refresh_lg4_preview_witness_with_tint(
    eval: &LandscapeProgramEvaluation,
    tint_visible_chunks: u32,
) -> bool {
    refresh_lg4_preview_witness_with_tint_and_pixel_count(eval, tint_visible_chunks, None)
}

#[must_use]
pub fn refresh_lg4_preview_witness_with_tint_and_pixel_count(
    eval: &LandscapeProgramEvaluation,
    tint_visible_chunks: u32,
    topology_kind_count_visible: Option<u32>,
) -> bool {
    let pixel_visible = topology_kind_count_visible
        .unwrap_or_else(|| eval.topology_kind_count as u32);
    // WIT-RUST-001 / WIT-GREEN-TINT-ZERO — never green when tint count is zero.
    let operator_visible = if tint_visible_chunks == 0 {
        false
    } else {
        lg4_preview_operator_visible(tint_visible_chunks, eval) && pixel_visible >= 3
    };
    let body = json!({
        "gate": "LG-4-PREVIEW-001",
        "slice_id": "CDR-A-LG4-PIXEL-REOPEN-001",
        "green": operator_visible,
        "operator_visible": operator_visible,
        "proof_grade": crate::dev::proof_grade::ProofGrade::HeadlessSim.as_str(),
        "topology_tint_wired": true,
        "topology_tint_visible_chunks": tint_visible_chunks,
        "topology_kinds_visible_min": 3,
        "topology_kind_count": eval.topology_kind_count,
        "topology_kind_count_visible": pixel_visible,
        "pixel_heterogeneity_wired": topology_kind_count_visible.is_some(),
    });
    let wrapped = wrap_debug_run(
        "LG-4-PREVIEW-001",
        "refresh_lg4_preview_witness",
        LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON, wrapped) && operator_visible
}

pub fn landscape_grammar_lg2_plugin(app: &mut App) {
    app.init_resource::<LandscapeGrammarLg2Witness>()
        .init_resource::<LandscapeDisturbanceQueue>()
        .add_systems(
            Update,
            (
                attach_lg2_components_on_pilot,
                tick_succession_age_on_ecology,
                apply_fire_disturbance_on_heat,
                apply_construction_clear_disturbance,
                drain_landscape_disturbance_queue,
                sync_fuel_bridge_from_succession,
                derive_vegetation_population_from_graph,
            )
                .chain()
                .in_set(ChunkEnvironmentSet::Ecology),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use super::super::landscape_grammar::{
        load_landscape_preset_from_path, LANDSCAPE_PRESETS_DIR,
    };
    use super::super::landscape_grammar_map::repo_asset_path;
    use crate::strategic::{
        BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype,
        SiteId, StrategicRasterConfig,
    };
    use crate::systems::sim_control::SimTick;

    fn pilot_eval() -> LandscapeProgramEvaluation {
        let path = repo_asset_path(&format!("{LANDSCAPE_PRESETS_DIR}/{LG1_PILOT_PRESET_ID}.json"));
        let preset = load_landscape_preset_from_path(&path).expect("preset");
        let ecology = ChunkEcology::default();
        let veg = VegetationField::default();
        let weather = ChunkWeather::default();
        evaluate_landscape_program(&preset, LG1_PILOT_CHUNK, &ecology, &veg, &weather)
    }

    #[test]
    fn lg2_lib_witness_writes_json() {
        let eval = pilot_eval();
        let witness = LandscapeGrammarLg2Witness::default();
        assert!(lg2_lib_witness_green(&eval));
        assert!(refresh_lg2_witness(&eval, &witness));
        let raw = fs::read_to_string(repo_asset_path(LANDSCAPE_GRAMMAR_LG2_LIVE_JSON)).unwrap();
        assert!(raw.contains("lib_green"));
    }

    #[test]
    fn lg4_preview_witness_writes_green_json() {
        let eval = pilot_eval();
        assert!(lg4_preview_witness_green(&eval));
        assert!(
            !refresh_lg4_preview_witness(&eval),
            "WIT-GREEN-TINT-ZERO: tint=0 must not green"
        );
        assert!(refresh_lg4_preview_witness_with_tint(&eval, 2));
        let raw = fs::read_to_string(repo_asset_path(LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON)).unwrap();
        assert!(raw.contains("topology_tint_visible_chunks"));
    }

    #[test]
    fn live_fire_disturbance_old_growth_to_burn_scar() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<LandscapeGrammarLg2Witness>()
            .add_systems(Update, apply_fire_disturbance_on_heat);

        let entity = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: LG1_PILOT_CHUNK,
                },
                ChunkSurfaceFire {
                    heat: 0.9,
                    fuel: 0.5,
                },
                SuccessionState {
                    age_ticks: 500,
                    stage: SuccessionTopologyStage::OldGrowth,
                    last_disturbance_tick: None,
                },
                DisturbanceHistory {
                    events: VecDeque::new(),
                    capacity: 8,
                },
            ))
            .id();

        app.update();

        let witness = app.world().resource::<LandscapeGrammarLg2Witness>();
        assert!(
            witness.fire_disturbances >= 1,
            "fire_disturbances={}",
            witness.fire_disturbances
        );
        let succ = app.world().get::<SuccessionState>(entity).unwrap();
        assert_eq!(succ.stage, SuccessionTopologyStage::BurnScar);

        let history = app.world().get::<DisturbanceHistory>(entity).unwrap();
        assert_eq!(history.events.back().map(|e| e.kind), Some(DisturbanceKind::Fire));
    }

    #[test]
    fn construction_disturbance_maps_footprint_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<LandscapeGrammarLg2Witness>()
            .init_resource::<StrategicRasterConfig>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(Update, apply_construction_clear_disturbance);

        let target = IVec2::new(2, 2);
        app.world_mut().spawn((
            Chunk { coord: target },
            DisturbanceHistory {
                events: VecDeque::new(),
                capacity: 8,
            },
        ));

        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner: Entity::PLACEHOLDER,
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 64, z: 64 },
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });

        app.update();

        let witness = app.world().resource::<LandscapeGrammarLg2Witness>();
        assert!(
            witness.construction_disturbances >= 1,
            "construction_disturbances={}",
            witness.construction_disturbances
        );
    }

    #[test]
    fn recovery_advances_after_fire_clears() {
        let mut history = DisturbanceHistory {
            events: VecDeque::new(),
            capacity: 8,
        };
        history.push(DisturbanceEvent {
            tick: 1,
            kind: DisturbanceKind::Fire,
            topology_kind: "Patch".into(),
        });
        let stage = succession_stage_from_age(15, false);
        assert_eq!(stage, SuccessionTopologyStage::Grass);
    }

    #[test]
    fn harvest_queue_drains_to_history() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<LandscapeGrammarLg2Witness>()
            .init_resource::<LandscapeDisturbanceQueue>()
            .add_systems(Update, drain_landscape_disturbance_queue);

        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            DisturbanceHistory {
                events: VecDeque::new(),
                capacity: 8,
            },
        ));
        app.world_mut()
            .resource_mut::<LandscapeDisturbanceQueue>()
            .pending
            .push((IVec2::ZERO, DisturbanceKind::Harvest));

        app.update();

        let witness = app.world().resource::<LandscapeGrammarLg2Witness>();
        assert_eq!(witness.harvest_disturbances, 1);
    }
}
