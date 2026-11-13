//! Settlement hierarchy + organic growth (SET-P5 / ECON-OG / PROC-OG).

mod admin_ink;
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
mod faction_step;
mod growth;
mod hierarchy_gap;
mod ids;
mod market;
mod need_utility;
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
pub use faction_step::{FactionPairStance, FactionStanceBook};
pub use block_frame::{
    block_bounds_from_tiles, build_block_frame, fixture_block_record_with_tiles, fixture_transport_graph_for_block_frame,
    orientation_from_street_side, rebuild_block_frames, scatter_interior_tiles, street_edge_tiles, street_side_from_junction, sync_block_frames_system,
    nearest_junction_tile, BlockFrame, BlockFrameBook, StreetSide, };
pub use block_archetype::{
    fixture_score_for_archetype,
    load_block_archetype_registry, load_block_archetype_threshold_table,
    load_block_archetype_threshold_table_from_path, noise_jitter_from_block_seed,
    resolve_block_archetype, BlockArchetype,
    BlockArchetypeBand, BlockArchetypeRegistry, BlockArchetypeThresholdTable, BlockScore,
    BLOCK_ARCHETYPE_THRESHOLDS_RON, };
pub use block_recipe::{
    block_recipe_lot_list_stable_hash, evaluate_block_recipe,
    evaluate_block_recipe_for_archetype, load_block_recipe_from_path,
    load_block_recipe_registry, BlockEdgePlacement, BlockLotPlacement, BlockParkFillPlacement, BlockPlazaPlacement,
    BlockRecipe,
    BlockRecipeEvaluation, BlockRecipeRegistry, BlockRecipeStep, BlockScatterPlacement,
    BLOCK_RECIPES_DIR, };
pub use block_rollout::{
    advance_block_rollout_staging_system, begin_block_rollout_staging_system,
    evaluate_block_recipes_system, execute_staged_block_growth_system,
    push_block_recipe_visual_requests_system, release_staged_block_lot_proposals_system, resolve_block_archetypes_system, BlockAssembled,
    BlockCommitted, BlockPlanned, BlockRecipeEvaluationBook, BlockRolloutGrowthBook,
    BlockRolloutStagingBook, };
pub use block_street_visual::{
    load_block_street_furniture_scenes,
    spawn_block_street_furniture_system,
    BlockStreetFurnitureCatalog, BlockStreetFurniturePiece, BlockStreetFurnitureRoot,
    BlockStreetVisualBook,     PresentationOnlyStreetVisual, };
pub use block_lod_impostor::{
    load_block_lod_impostor_scene, spawn_block_lod_impostors_system,
    sync_block_lod_impostor_visibility_system, BlockLodImpostor, BlockLodImpostorBook,
    BlockLodImpostorCatalog, };

pub use admin_ink::{
    admin_ring_should_close, admin_zone_corner_tile, admin_zone_drag_rect, admin_zone_drag_stamps,
    admin_zone_tile_from_world, append_admin_ring_vertex, arm_admin_zone_edit, begin_admin_zone_drag,
    closed_ring_vertices, commit_admin_zone_drag, commit_admin_zone_ring, delete_admin_zone,
    move_admin_ring_vertex, move_admin_zone_drag, start_admin_zone_drag, AdminZoneDrag, InkSite,
};
pub use district::{
    add_admin_zone, core_ring_for_rect, ring_bounds, stamp_core_rings, tile_in_ring,
    DevelopmentPressure, DevelopmentPressureBook, DistrictBook, DistrictMetrics,
    DistrictMetricsBook, DistrictRecord, DistrictStyleRules,
};
pub use execute::{
    approve_all_growth_proposals_into_pending, approve_growth_proposal_into_pending,
    commit_growth_pending_to_funnel_system, commit_top_district_growth,
    enqueue_approved_growth_proposal, growth_pending_commit_event, growth_site_at_town_center, growth_sites_for_towns,
    enqueue_generated_town_growth_once_system, enqueue_second_town_growth,
    enqueue_second_town_growth_system, enqueue_town_growth_once,
    pending_blueprint_from_growth_proposal, site_archetype_for_growth_proposal,
    TownGrowthEnqueueGate, TownSecondGrowthGate,
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
    building_grammar_seed_for_site, lot_idx_from_site_id, lot_seed, mix_u64, town_seed, CitySeedContext, DEFAULT_TOWN_ID,
    DEFAULT_WORLD_SEED, };
pub use town::{
    access_policy_for_generated_index, found_settlement, generated_town_book,
    generated_town_book_on_land, place_towns_for_full_generation, portland_fixture_town,
    region_name_for_seed,
    seed_settlement_books_if_empty, snap_generated_towns_off_water, town_allows_entry,
    EMPTY_TOWN_BOOK_COUNT,
    town_allows_growth_commit, town_allows_logistics_entry, town_allows_vehicle,
    GeneratedTownDrySnap, TownAccessPolicy, TownBook, TownRecord,
};
pub use zoning::ZoningClass;

pub use district::portland_fixture_district;

use bevy::prelude::*;

use crate::strategic::InfrastructureSiteSet;

pub struct SettlementPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SettlementGrowthSet {
    Decide,
    Present,
}

fn settlement_registry_startup(mut commands: Commands) {
    commands.insert_resource(load_block_recipe_registry());
    commands.insert_resource(load_block_archetype_registry());
}

impl Plugin for SettlementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                settlement_registry_startup,
                load_block_street_furniture_scenes,
                load_block_lod_impostor_scene,
            ),
        )
            .init_resource::<TownBook>()
            .init_resource::<town::GeneratedTownDrySnap>()
            .init_resource::<DistrictBook>()
            .init_resource::<admin_ink::AdminZoneDrag>()
            .init_resource::<faction_step::FactionStanceBook>()
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
            .add_message::<crate::strategic::CommitConstructionSiteEvent>()
            .init_resource::<crate::strategic::settlement::TownGrowthEnqueueGate>()
            .init_resource::<crate::strategic::settlement::TownSecondGrowthGate>()
            .configure_sets(
                Update,
                SettlementGrowthSet::Decide.before(SettlementGrowthSet::Present),
            )
            .add_systems(
                Update,
                town::snap_generated_towns_off_water
                    .before(enqueue_generated_town_growth_once_system),
            )
            .add_systems(
                Update,
                (
                    rollup_district_metrics_system,
                    compute_district_pressure_system,
                    compute_market_saturation_system,
                    growth_proposal_tick_system,
                    faction_step::faction_growth_step_system,
                    sync_growth_proposal_ghosts_system,
                    push_proposal_ghosts_to_visual_requests,
                    sync_block_frames_system,
                    resolve_block_archetypes_system,
                    evaluate_block_recipes_system,
                    advance_block_rollout_staging_system,
                    begin_block_rollout_staging_system,
                    release_staged_block_lot_proposals_system,
                    execute_staged_block_growth_system,
                    enqueue_generated_town_growth_once_system,
                    enqueue_second_town_growth_system,
                    commit_growth_pending_to_funnel_system,
                )
                    .chain()
                    .in_set(SettlementGrowthSet::Decide)
                    .after(InfrastructureSiteSet::Planning),
            )
            .add_systems(
                Update,
                (
                    spawn_block_street_furniture_system,
                    spawn_block_lod_impostors_system,
                    sync_block_lod_impostor_visibility_system,
                    push_block_recipe_visual_requests_system,
                )
                    .chain()
                    .in_set(SettlementGrowthSet::Present)
                    .after(InfrastructureSiteSet::Planning),
            );
    }
}

#[cfg(test)]
mod tests {
    



}
