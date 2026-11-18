//! Ecology: macro [`ChunkEcology`](chunk_ecology::ChunkEcology) + meso [`vegetation_field::VegetationField`] (CPU authority).
//! **DES-06**: stable feedback via [`ecosystem_logistic_step`].

mod cell_stand;
mod stand_flood;
mod fuel_moisture;
mod fuel_break;
mod chunk_ecology;
mod wildlife_flock;
mod landscape_grammar;
mod landscape_grammar_burn;
mod landscape_grammar_lg2;
mod landscape_grammar_map;
mod landscape_atlas_registry;
mod vegetation_field;
mod vegetation_suitability;
mod vegetation_variant_catalog;
#[cfg(test)]
mod water_vegetation_tests;

pub use cell_stand::{
    ChunkCellStand, StandBurnConfig, StandKind, StandPhase, CELL_BURN_HEAT, CELL_BURNING, CELL_BURNT,
    CELL_DEAD, GAME_DAY_SECS, STAND_BURN_RON, ecosystem_logistic_step, load_stand_burn_config,
    parse_stand_burn_ron, CELL_GRASS, CELL_NONE, CELL_OLD, CELL_REGROWTH, CELL_SHRUB, CELL_WATER,
    CELL_YOUNG, SCAR_FRACTION,
};
pub use stand_flood::stand_flood_from_water_system;
pub use fuel_moisture::{
    effective_fuel_moisture01, equilibrium_moisture01, step_timelag01, ChunkCellFuelMoisture,
    TIMELAG_FINE_H, TIMELAG_HUNDRED_H, TIMELAG_TEN_H, TIMELAG_THOUSAND_H,
};
pub use fuel_break::{
    apply_fuel_break_commits, FuelBreakCommitEvent, PendingFuelBreaks,
};
pub use chunk_ecology::{chunk_ecology_tick, ChunkEcology};
pub use landscape_grammar::{
    evaluate_landscape_program, evaluate_landscape_program_with_inputs,
    load_landscape_grammar_catalog, load_landscape_preset_from_path, blend_lambda_with_inputs, effective_topology_graph, macro_topology_subgraph, LandscapeGrammarCatalog, LandscapeProgramEvaluation,
    LandscapeProgramOnChunk, LambdaExternalInputs, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID,
    LANDSCAPE_PRESETS_DIR,
};
pub use landscape_grammar_burn::{
    apply_active_burn_from_surface_fire, advance_regrowth_macro_chain,
    extract_glyph_for_burn, planning_glyph_for_burn, remove_mature_active_burn_overlays, variant_key_for_burn_row, veg_burn_frame_index,
    ActiveBurn, LandscapeBurnSet, LandscapeBurnStats, RegrowthMacroPhase,
    ACTIVE_BURN_HEAT_EPS, ACTIVE_BURN_IGNITE_HEAT, VEG_BURN_FRAME_COUNT, VEG_BURN_FRAME_PERIOD_MS,
};
pub use landscape_grammar_lg2::{
    apply_construction_clear_disturbance, apply_fire_disturbance_on_heat,
    attach_lg2_components_on_pilot, merge_harness_eval_summary,
    drain_landscape_disturbance_queue, push_post_fire_harvest_sim_effect, tick_succession_age_on_ecology,
    DisturbanceHistory, DisturbanceKind, lg4_preview_operator_visible,
    lg4_preview_witness_green, LandUseDistrictKind,
    LandUseInfluence, LandscapeDisturbanceQueue, LandscapeGrammarLg2Stats,
    SubcellPopulationGrid, SuccessionState, SuccessionTopologyStage, VegetationPopulation,
    VegetationStandInstances,
    };
pub use landscape_grammar::attach_landscape_program_pilot;
pub use landscape_atlas_registry::{
    init_landscape_atlas_registry, load_landscape_atlas_registry, topology_kind_to_variant_key,
    LandscapeAtlasEntry, LandscapeAtlasRegistry,
};
pub use landscape_grammar_map::{
    pick_preset_id_for_chunk, hydrology_bias_from_terrain, lambda_inputs_from_live_fields,
    lambda_inputs_from_live_fields_with_terrain, pick_preset_id_for_chunk_with_inputs,
    rollout_landscape_program_on_chunks,
    landscape_lg5_registry_stamped,
    LG5_ATLAS_EXPAND_ROLLUP_JSON,
    LG5_PILOT_ATLAS_ID,
    LandscapeMapRolloutStats, LandscapePresetIndex, VegetationProgramCloseBody,
    LG5_ATLAS_ID,
};
pub use vegetation_field::{
    derive_vegetation_structure, integrate_vegetation_field_step,
    integrate_vegetation_field_step_with_drivers, succession_stage_from_vegetation,
    CanopyDrivers, VegetationCanopySet, CANOPY_CLEARED_DENSITY,
    veg_draw_density, veg_draw_density_after_impact, veg_draw_density_under_snow,
    EcologicalSuccessionStage, VegetationField, VegetationStructure,
};
pub use vegetation_suitability::{
    clamp_stage_to_cap, max_stage_for_canopy_cap, rate_chunk_vegetation_suitability,
    ChunkVegetationSuitability, SuitabilityEntry, SuitabilityParams, VegetationSuitabilityTable,
    VEGETATION_SUITABILITY_RON,
};
pub use vegetation_variant_catalog::{
    catalog_variant_key_set, clamp_vegetation_variant_to_catalog,
    init_vegetation_variant_catalog, load_vegetation_variant_catalog, resolve_vegetation_variant,
    ENGINE_VEG_RESOLVER_KEYS, VegetationVariantCatalog, VegetationVariantCatalogEntry,
    VEGETATION_VARIANT_CATALOG_RON,
};
pub use wildlife_flock::{
    clamp_speed, flock_step, seed_wildlife_herd_from_world_seed, soft_cost_at,
    soft_costs_from_agents, spawn_wildlife_agents, sync_agent_frame_from_wildlife,
    sync_agent_frame_from_wildlife_system, sync_nav_wildlife_soft, sync_nav_wildlife_soft_system,
    tick_wildlife_herd, WildlifeAgent, WildlifeHerdBook, WildlifeSoftCostSample,
    WILDLIFE_DEFAULT_COUNT, WILDLIFE_MAX_SPEED, WILDLIFE_SOFT_COST_MAX,
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
        app.configure_sets(
            Update,
            VegetationCanopySet::Inputs.before(VegetationCanopySet::Apply),
        );
        landscape_grammar::landscape_grammar_plugin(app);
        landscape_grammar_lg2::landscape_grammar_lg2_plugin(app);
        landscape_grammar_burn::landscape_grammar_burn_plugin(app);
        landscape_grammar_map::landscape_grammar_map_plugin(app);
        wildlife_flock::register_wildlife_flock(app);
        app.add_systems(
            Startup,
            (
                init_vegetation_variant_catalog,
                init_landscape_atlas_registry,
                vegetation_suitability::init_vegetation_suitability_table,
            ),
        );
        // Per-cell standing vegetation: fuel capacity, burn-out, regrowth. Its losses reach the canopy only
        // through `CanopyDrivers` (inputs), so `vegetation_field_tick` stays the one canopy writer.
        app.insert_resource(load_stand_burn_config())
            .init_resource::<PendingFuelBreaks>()
            .add_systems(
            Update,
            (
                cell_stand::spawn_chunk_cell_stand,
                fuel_moisture::spawn_chunk_cell_fuel_moisture.after(cell_stand::spawn_chunk_cell_stand),
                fuel_break::apply_fuel_break_commits,
                cell_stand::cell_stand_tick
                    .after(vegetation_suitability::update_chunk_vegetation_suitability)
                    .after(landscape_grammar_lg2::derive_vegetation_population_from_graph)
                    .after(fuel_break::apply_fuel_break_commits),
                stand_flood::stand_flood_from_water_system.after(cell_stand::cell_stand_tick),
                fuel_moisture::cell_fuel_moisture_tick
                    .after(stand_flood::stand_flood_from_water_system),
            )
                .chain()
                .in_set(ChunkEnvironmentSet::Ecology)
                .in_set(VegetationCanopySet::Inputs),
        );
        app.add_systems(
            Update,
            (
                spawn_vegetation_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_ecology_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_fuel_profile_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_fire_fuel_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                chunk_ecology_tick.in_set(ChunkEnvironmentSet::Ecology),
                vegetation_field_tick
                    .in_set(ChunkEnvironmentSet::Ecology)
                    .in_set(VegetationCanopySet::Apply),
                fire_fuel_field_tick.in_set(ChunkEnvironmentSet::Ecology),
                chunk_fuel_profile_tick.in_set(ChunkEnvironmentSet::Ecology),
                ecology_preview_bump_on_vegetation_change.in_set(ChunkEnvironmentSet::Ecology),
            )
                .chain(),
        );
    }
}
