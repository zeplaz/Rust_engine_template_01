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
            fill_override: None,
        });
    }
}

#[cfg(test)]
mod tests {
}

