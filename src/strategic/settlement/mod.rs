//! Settlement hierarchy + organic growth (SET-P5 / ECON-OG / PROC-OG).

mod actors;
mod assign;
mod block;
mod block_archetype;
mod block_frame;
mod block_lod_impostor;
mod block_perf;
mod block_recipe;
mod block_rollout;
mod block_street_visual;
mod district;
mod execute;
mod growth;
mod ids;
mod market;
mod policy;
mod pressure;
mod seed_chain;
mod town;
mod zoning;

pub use actors::{
    growth_actor_may_enqueue, BuildingUsage, GrowthActorLayer, GrowthReasonCode,
};
pub use assign::{
    assign_block_for_tile, block_cluster_key, district_for_tile, register_site_on_commit,
    BLOCK_GRID_SIZE,
};
pub use block::{BlockBook, BlockRecord};
pub use block_frame::{
    block_bounds_from_tiles, block_frame_debug_overlay_wired_witness_green,
    build_block_frame, build_city_g1_c2_001_witness_body, city_g1_c2_001_block_frame_witness_green,
    fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
    orientation_from_street_side, rebuild_block_frames, refresh_city_g1_c2_001_block_frame_witness,
    scatter_interior_tiles, street_edge_tiles, street_side_from_junction, sync_block_frames_system,
    nearest_junction_tile, BlockFrame, BlockFrameBook, StreetSide, CITY_G1_C2_LIVE_JSON,
};
pub use block_archetype::{
    build_city_g1_c1_001_witness_body, city_g1_c1_001_block_archetype_witness_green,
    city_g1_c1_001_per_band_tests_green, fixture_score_for_archetype,
    load_block_archetype_registry, load_block_archetype_threshold_table,
    load_block_archetype_threshold_table_from_path, noise_jitter_from_block_seed,
    refresh_city_g1_c1_001_block_archetype_witness, resolve_block_archetype, BlockArchetype,
    BlockArchetypeBand, BlockArchetypeRegistry, BlockArchetypeThresholdTable, BlockScore,
    BLOCK_ARCHETYPE_THRESHOLDS_RON, CITY_G1_C1_LIVE_JSON,
};
pub use block_recipe::{
    block_recipe_lot_list_stable_hash, build_city_g1_c3_001_witness_body,
    city_g1_c3_001_block_recipe_witness_green, evaluate_block_recipe,
    evaluate_block_recipe_for_archetype, load_block_recipe_from_path,
    load_block_recipe_registry, refresh_city_g1_c3_001_block_recipe_witness,
    BlockEdgePlacement, BlockLotPlacement, BlockParkFillPlacement, BlockPlazaPlacement,
    BlockRecipe,
    BlockRecipeEvaluation, BlockRecipeRegistry, BlockRecipeStep, BlockScatterPlacement,
    BLOCK_RECIPES_DIR, CITY_G1_C3_LIVE_JSON,
};
pub use block_rollout::{
    advance_block_rollout_staging_system, begin_block_rollout_staging_system,
    block_rollout_fixture_witness_green, build_city_g3_rollout_witness_body,
    city_g3_rollout_witness_green,     evaluate_block_recipes_system, execute_staged_block_growth_system,
    push_block_recipe_visual_requests_system, refresh_city_g3_rollout_witness,
    release_staged_block_lot_proposals_system, resolve_block_archetypes_system, BlockAssembled,
    BlockCommitted, BlockPlanned, BlockRecipeEvaluationBook, BlockRolloutGrowthBook,
    BlockRolloutStagingBook, CITY_G3_ROLLOUT_LIVE_JSON,
};
pub use block_street_visual::{
    block_street_visual_fixture_witness_green, build_city_c6_bsn_witness_body,
    city_c6_bsn_witness_green, load_block_street_furniture_scenes,
    refresh_city_c6_bsn_witness, spawn_block_street_furniture_system,
    BlockStreetFurnitureCatalog, BlockStreetFurniturePiece, BlockStreetFurnitureRoot,
    BlockStreetVisualBook,     PresentationOnlyStreetVisual, CITY_C6_BSN_LIVE_JSON,
};
pub use block_lod_impostor::{
    block_lod_impostor_fixture_witness_green, build_city_p2_witness_body, city_p2_witness_green,
    load_block_lod_impostor_scene, refresh_city_p2_witness, spawn_block_lod_impostors_system,
    sync_block_lod_impostor_visibility_system, BlockLodImpostor, BlockLodImpostorBook,
    BlockLodImpostorCatalog, CITY_P2_LIVE_JSON,
};
pub use block_perf::{
    build_city_p1_witness_body, city_p1_witness_green, refresh_city_p1_witness,
    CITY_P1_LIVE_JSON,
};
pub use district::{
    DevelopmentPressure, DevelopmentPressureBook, DistrictBook, DistrictMetrics,
    DistrictMetricsBook, DistrictRecord, DistrictStyleRules,
};
pub use execute::{
    approve_all_growth_proposals_into_pending, approve_growth_proposal_into_pending,
    enqueue_approved_growth_proposal, growth_approve_execute_pipeline_witness_green,
    pending_blueprint_from_growth_proposal, site_archetype_for_growth_proposal,
};
pub use growth::{growth_proposal_tick_system, score_proposal, GrowthProposal, GrowthProposalQueue};
pub use ids::{ArchetypeId, BlockId, DistrictId, RegionId, TownId};
pub use market::{
    compute_market_saturation_for_district, compute_market_saturation_system,
    niche_factor, proposal_rejected_by_saturation, MarketSaturation, MarketSaturationBook,
    SaturationCell,
};
pub use policy::{
    push_proposal_ghosts_to_visual_requests, sync_growth_proposal_ghosts_system, AutoBuildPolicy,
    AutoBuildPolicyBook, GrowthProposalGhostState,
};
pub use pressure::{compute_district_pressure_system, rollup_district_metrics_system};
pub use seed_chain::{
    block_id_for_site, block_seed, building_grammar_seed, building_grammar_seed_chain,
    building_grammar_seed_for_site, city_g1_c4_001_seed_chain_witness_green,
    build_city_g1_c4_001_witness_body, refresh_city_g1_c4_001_seed_chain_witness,
    lot_idx_from_site_id, lot_seed, mix_u64, town_seed, CitySeedContext, DEFAULT_TOWN_ID,
    DEFAULT_WORLD_SEED, CITY_G1_C4_LIVE_JSON,
};
pub use town::{portland_fixture_town, TownBook, TownRecord};
pub use zoning::ZoningClass;

#[cfg(test)]
pub use district::portland_fixture_district;

use bevy::prelude::*;

use crate::strategic::InfrastructureSiteSet;

/// **SET-P5-002** witness — block assignment wired on commit.
#[must_use]
pub fn set_p5_002_block_assignment_witness_green() -> bool {
    assign::three_sites_same_block_witness_green()
}

/// **ECON-OG-1-C** witness rollup.
#[must_use]
pub fn construction_organic_growth_001_witness_green() -> bool {
    actors::building_usage_serde_witness_green()
        && actors::growth_actor_layer_serde_witness_green()
        && market::market_saturation_witness_green()
        && pressure::district_pressure_witness_green()
        && growth::growth_proposal_witness_green()
        && policy::proposal_ghost_witness_green()
}

pub struct SettlementPlugin;

fn settlement_registry_startup(mut commands: Commands) {
    commands.insert_resource(load_block_recipe_registry());
    commands.insert_resource(load_block_archetype_registry());
}

impl Plugin for SettlementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, settlement_registry_startup)
            .add_systems(
                Startup,
                (
                    load_block_street_furniture_scenes,
                    load_block_lod_impostor_scene,
                ),
            )
            .init_resource::<TownBook>()
            .init_resource::<DistrictBook>()
            .init_resource::<BlockBook>()
            .init_resource::<DistrictMetricsBook>()
            .init_resource::<DevelopmentPressureBook>()
            .init_resource::<MarketSaturationBook>()
            .init_resource::<GrowthProposalQueue>()
            .init_resource::<AutoBuildPolicyBook>()
            .init_resource::<BlockFrameBook>()
            .init_resource::<GrowthProposalGhostState>()
            .init_resource::<BlockRecipeEvaluationBook>()
            .init_resource::<BlockRolloutGrowthBook>()
            .init_resource::<BlockRolloutStagingBook>()
            .init_resource::<BlockStreetFurnitureCatalog>()
            .init_resource::<BlockStreetVisualBook>()
            .init_resource::<BlockLodImpostorCatalog>()
            .init_resource::<BlockLodImpostorBook>()
            .add_message::<BlockPlanned>()
            .add_message::<BlockAssembled>()
            .add_message::<BlockCommitted>()
            .add_systems(
                Update,
                (
                    rollup_district_metrics_system,
                    compute_district_pressure_system,
                    compute_market_saturation_system,
                    growth_proposal_tick_system,
                    sync_growth_proposal_ghosts_system,
                    push_proposal_ghosts_to_visual_requests,
                    sync_block_frames_system,
                    resolve_block_archetypes_system,
                    evaluate_block_recipes_system,
                    advance_block_rollout_staging_system,
                    begin_block_rollout_staging_system,
                    release_staged_block_lot_proposals_system,
                    execute_staged_block_growth_system,
                    spawn_block_street_furniture_system,
                    spawn_block_lod_impostors_system,
                    sync_block_lod_impostor_visibility_system,
                    push_block_recipe_visual_requests_system,
                )
                    .chain()
                    .after(InfrastructureSiteSet::Planning),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_p5_002_witness_green() {
        assert!(set_p5_002_block_assignment_witness_green());
    }

    #[test]
    fn construction_organic_growth_001_witness() {
        assert!(construction_organic_growth_001_witness_green());
    }
}
