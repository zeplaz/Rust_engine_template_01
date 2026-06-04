//! Settlement hierarchy + organic growth (SET-P5 / ECON-OG / PROC-OG).

mod actors;
mod assign;
mod block;
mod district;
mod growth;
mod ids;
mod market;
mod policy;
mod pressure;
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
