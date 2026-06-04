//! Auto-build policy + proposal ghost hook (PROC-OG-3-001).

use std::collections::HashMap;

use bevy::prelude::*;

use super::growth::{GrowthProposal, GrowthProposalQueue};
use super::ids::DistrictId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AutoBuildPolicy {
    #[default]
    Manual,
    AutoCommercial,
    AutoAll,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct AutoBuildPolicyBook {
    pub by_district: HashMap<DistrictId, AutoBuildPolicy>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct GrowthProposalGhostState {
    pub active: Vec<GrowthProposal>,
}

pub fn sync_growth_proposal_ghosts_system(
    queue: Res<GrowthProposalQueue>,
    mut ghosts: ResMut<GrowthProposalGhostState>,
) {
    ghosts.active = queue.proposals.clone();
}

pub fn push_proposal_ghosts_to_visual_requests(
    ghosts: Res<GrowthProposalGhostState>,
    mut requests: Option<ResMut<crate::construction::ConstructionVisualRequests>>,
) {
    let Some(requests) = requests.as_mut() else {
        return;
    };
    for proposal in &ghosts.active {
        requests.footprint_tiles.push(crate::construction::FootprintTileRequest {
            tile: proposal.anchor_tile,
            color_kind: crate::construction::FootprintTileColorKind::Risky,
            weight: 0.45,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::settlement::actors::{BuildingUsage, GrowthActorLayer};
    use crate::strategic::settlement::ids::ArchetypeId;

    #[test]
    fn proposal_ghosts_sync_from_queue() {
        assert!(proposal_ghost_witness_green());
    }
}

#[must_use]
pub fn proposal_ghost_witness_green() -> bool {
    use crate::strategic::settlement::actors::{BuildingUsage, GrowthActorLayer};
    use crate::strategic::settlement::ids::ArchetypeId;

    let mut app = App::new();
    app.init_resource::<GrowthProposalQueue>()
        .init_resource::<GrowthProposalGhostState>()
        .add_systems(Update, sync_growth_proposal_ghosts_system);
    {
        let mut queue = app.world_mut().resource_mut::<GrowthProposalQueue>();
        queue.enqueue(GrowthProposal {
            district_id: DistrictId("d".into()),
            block_id: None,
            archetype_id: ArchetypeId("shop".into()),
            usage: BuildingUsage::Commercial,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(4, 4),
            priority: 0.5,
            seed: 1,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
    }
    app.update();
    app.world().resource::<GrowthProposalGhostState>().active.len() == 1
}
