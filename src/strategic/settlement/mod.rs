//! Settlement hierarchy + organic growth (SET-P5 / ECON-OG / PROC-OG).

mod actors;
mod assign;
mod block;
mod block_archetype;
mod block_frame;
mod district;
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
pub use district::{
    DevelopmentPressure, DevelopmentPressureBook, DistrictBook, DistrictMetrics,
    DistrictMetricsBook, DistrictRecord, DistrictStyleRules,
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

impl Plugin for SettlementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TownBook>()
            .init_resource::<DistrictBook>()
            .init_resource::<BlockBook>()
            .init_resource::<DistrictMetricsBook>()
            .init_resource::<DevelopmentPressureBook>()
            .init_resource::<MarketSaturationBook>()
            .init_resource::<GrowthProposalQueue>()
            .init_resource::<AutoBuildPolicyBook>()
            .init_resource::<BlockFrameBook>()
            .init_resource::<GrowthProposalGhostState>()
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
