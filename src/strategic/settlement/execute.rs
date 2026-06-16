//! Growth proposal → construction pending queue (PROC-OG-APPROVE-001).

use bevy::prelude::*;

use crate::construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

use super::actors::{growth_actor_may_enqueue, BuildingUsage};
use super::growth::{GrowthProposal, GrowthProposalQueue};
use super::ids::ArchetypeId;

#[must_use]
pub fn site_archetype_for_growth_proposal(proposal: &GrowthProposal) -> SiteArchetype {
    match proposal.usage {
        BuildingUsage::Residential => SiteArchetype::CivilHousing,
        BuildingUsage::Industrial | BuildingUsage::Logistics => SiteArchetype::Factory,
        BuildingUsage::Office | BuildingUsage::Commercial => SiteArchetype::Factory,
        _ => SiteArchetype::Factory,
    }
}

#[must_use]
pub fn default_footprint_for_archetype(archetype: &ArchetypeId) -> FootprintTiles {
    if archetype.0.contains("warehouse") {
        FootprintTiles {
            width: 4,
            depth: 2,
        }
    } else {
        FootprintTiles {
            width: 1,
            depth: 1,
        }
    }
}

#[must_use]
pub fn pending_blueprint_from_growth_proposal(proposal: &GrowthProposal) -> PendingBuildBlueprint {
    let origin = BuildSiteTile {
        x: proposal.anchor_tile.x.max(0) as u32,
        z: proposal.anchor_tile.y.max(0) as u32,
    };
    PendingBuildBlueprint {
        kind: PendingEntryKind::GrowthProposal,
        label: format!("growth:{}", proposal.archetype_id.0),
        archetype: site_archetype_for_growth_proposal(proposal),
        origin,
        footprint: default_footprint_for_archetype(&proposal.archetype_id),
        layer: LayerType::Surface,
        rotation_quarter_turns: 0,
        mirror_x: false,
        approved: true,
        catalog_id: Some(proposal.archetype_id.0.clone()),
    }
}

pub fn enqueue_approved_growth_proposal(
    proposal: GrowthProposal,
    pending: &mut PendingConstructionQueue,
) -> bool {
    if !growth_actor_may_enqueue(proposal.actor_layer) {
        return false;
    }
    pending.push(pending_blueprint_from_growth_proposal(&proposal));
    true
}

pub fn approve_growth_proposal_into_pending(
    queue: &mut GrowthProposalQueue,
    index: usize,
    pending: &mut PendingConstructionQueue,
) -> bool {
    if index >= queue.proposals.len() {
        return false;
    }
    let proposal = queue.proposals.remove(index);
    enqueue_approved_growth_proposal(proposal, pending)
}

pub fn approve_all_growth_proposals_into_pending(
    queue: &mut GrowthProposalQueue,
    pending: &mut PendingConstructionQueue,
) -> usize {
    let mut count = 0usize;
    while let Some(proposal) = queue.proposals.pop() {
        if enqueue_approved_growth_proposal(proposal, pending) {
            count += 1;
        }
    }
    count
}

#[must_use]
pub fn growth_approve_execute_pipeline_witness_green() -> bool {
    use super::actors::{BuildingUsage, GrowthActorLayer};
    use super::ids::DistrictId;

    let mut queue = GrowthProposalQueue::default();
    queue.enqueue(GrowthProposal {
        district_id: DistrictId("d".into()),
        block_id: None,
        archetype_id: ArchetypeId("corner_shop".into()),
        usage: BuildingUsage::Commercial,
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: IVec2::new(4, 4),
        priority: 0.5,
        seed: 1,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    });
    let mut pending = PendingConstructionQueue::default();
    if !approve_growth_proposal_into_pending(&mut queue, 0, &mut pending) {
        return false;
    }
    queue.proposals.is_empty()
        && pending.entries.len() == 1
        && pending.entries[0].approved
        && pending.entries[0].kind == PendingEntryKind::GrowthProposal
        && pending.entries[0].catalog_id.as_deref() == Some("corner_shop")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_approve_enqueues_pending_blueprint() {
        assert!(growth_approve_execute_pipeline_witness_green());
    }
}
