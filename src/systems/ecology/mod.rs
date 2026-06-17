//! Ecology: macro [`ChunkEcology`](chunk_ecology::ChunkEcology) + meso [`vegetation_field::VegetationField`] (CPU authority).

mod chunk_ecology;
mod landscape_grammar;
mod landscape_grammar_burn;
mod landscape_grammar_lg2;
mod landscape_grammar_map;
mod landscape_atlas_registry;
mod vegetation_field;

pub use chunk_ecology::{chunk_ecology_tick, ChunkEcology};
pub use landscape_grammar::{
    evaluate_landscape_program, evaluate_landscape_program_with_inputs,
    load_landscape_grammar_catalog, load_landscape_preset_from_path, refresh_lg1_witness,
    refresh_composite_eval_witness,
    blend_lambda_with_inputs, effective_topology_graph, macro_topology_subgraph, LandscapeGrammarCatalog, LandscapeProgramEvaluation,
    LandscapeProgramOnChunk, LambdaExternalInputs, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID,
    LANDSCAPE_GRAMMAR_LG1_LIVE_JSON, LANDSCAPE_PRESETS_DIR,
};
pub use landscape_grammar_burn::{
    apply_active_burn_from_surface_fire, advance_regrowth_macro_chain,
    burn_overlay_witness_green, burn_succession_witness_green,
    extract_glyph_for_burn, planning_glyph_for_burn, refresh_burn_overlay_witness,
    remove_mature_active_burn_overlays, variant_key_for_burn_row, veg_burn_frame_index,
    burn_sm_self_check_green, ActiveBurn, LandscapeBurnSet, LandscapeBurnWitness, RegrowthMacroPhase,
    ACTIVE_BURN_HEAT_EPS, ACTIVE_BURN_IGNITE_HEAT, LANDSCAPE_GRAMMAR_BURN_OVERLAY_LIVE_JSON,
    VEG_BURN_FRAME_COUNT, VEG_BURN_FRAME_PERIOD_MS,
};
pub use landscape_grammar_lg2::{
    apply_construction_clear_disturbance, apply_fire_disturbance_on_heat,
    attach_lg2_components_on_pilot, refresh_lg2_witness, refresh_lg4_preview_witness,
    refresh_lg4_preview_witness_with_tint,
    refresh_lg4_preview_witness_with_tint_and_pixel_count,
    fire_corridor_population_fuel_witness_green,
    DisturbanceHistory, DisturbanceKind, lg2_witness_green, lg4_preview_operator_visible,
    lg4_preview_witness_green, LandUseDistrictKind,
    LandUseInfluence, LandscapeDisturbanceQueue, LandscapeGrammarLg2Witness,
    SubcellPopulationGrid, SuccessionState, SuccessionTopologyStage, VegetationPopulation,
    LANDSCAPE_GRAMMAR_LG2_LIVE_JSON, LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON,
};
pub use landscape_grammar::attach_landscape_program_pilot;
pub use landscape_atlas_registry::{
    load_landscape_atlas_registry, topology_kind_to_variant_key, LandscapeAtlasEntry,
    LandscapeAtlasRegistry,
};
pub use landscape_grammar_map::{
    map_rollout_witness_green,     pick_preset_id_for_chunk, refresh_lg3_witness,
    refresh_lg3_witness_from_districts, refresh_lg3_witness_from_districts_with_anchors,
    refresh_lg5_witness, refresh_map_rollout_witness_system,
    lambda_inputs_from_live_fields, pick_preset_id_for_chunk_with_inputs,
    refresh_vegetation_program_close, rollout_landscape_program_on_chunks,
    landscape_lg5_registry_stamped,
    LandscapeMapRolloutWitness, LandscapePresetIndex, VegetationProgramCloseBody,
    LANDSCAPE_GRAMMAR_LG3_LIVE_JSON, LANDSCAPE_GRAMMAR_LG5_LIVE_JSON,
    LANDSCAPE_GRAMMAR_MAP_ROLLOUT_LIVE_JSON, VEGETATION_PROGRAM_CLOSE_LIVE_JSON,
    LG5_ATLAS_ID,
};
pub use vegetation_field::{
    derive_vegetation_structure, integrate_vegetation_field_step, succession_stage_from_vegetation,
    EcologicalSuccessionStage, VegetationField, VegetationStructure,
};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::{
    chunk_fuel_profile_tick, fire_fuel_field_tick, spawn_chunk_fuel_profile_on_new_chunk,
    spawn_fire_fuel_field_on_new_chunk,
};
use crate::terrain::generation::Chunk;
use crate::terrain::material::{invalidate_world, InvalidationReason, WorldPreviewState};
use chunk_ecology::spawn_chunk_ecology_on_new_chunk;
use vegetation_field::{spawn_vegetation_field_on_new_chunk, vegetation_field_tick};

fn ecology_preview_bump_on_vegetation_change(
    preview_state: Option<ResMut<WorldPreviewState>>,
    q: Query<(&Chunk, &VegetationField), Changed<VegetationField>>,
) {
    let Some(mut state) = preview_state else {
        return;
    };
    let coords: Vec<IVec2> = q.iter().map(|(c, _)| c.coord).collect();
    if coords.is_empty() {
        return;
    }
    invalidate_world(
        InvalidationReason::EcologyFields,
        &mut state,
        coords.into_iter(),
    );
}

pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        landscape_grammar::landscape_grammar_plugin(app);
        landscape_grammar_lg2::landscape_grammar_lg2_plugin(app);
        landscape_grammar_burn::landscape_grammar_burn_plugin(app);
        landscape_grammar_map::landscape_grammar_map_plugin(app);
        app.add_systems(
            Update,
            (
                spawn_vegetation_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_ecology_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_fuel_profile_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_fire_fuel_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                chunk_ecology_tick.in_set(ChunkEnvironmentSet::Ecology),
                vegetation_field_tick.in_set(ChunkEnvironmentSet::Ecology),
                fire_fuel_field_tick.in_set(ChunkEnvironmentSet::Ecology),
                chunk_fuel_profile_tick.in_set(ChunkEnvironmentSet::Ecology),
                ecology_preview_bump_on_vegetation_change.in_set(ChunkEnvironmentSet::Ecology),
            )
                .chain(),
        );
    }
}
