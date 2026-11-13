//! Growth proposals + queue (PROC-OG-2-001).

use bevy::prelude::*;

use super::actors::{BuildingUsage, GrowthActorLayer, GrowthReasonCode};
use super::district::{DevelopmentPressureBook, DistrictBook};
use super::ids::{ArchetypeId, BlockId, DistrictId};
use super::market::{niche_factor, proposal_rejected_by_saturation, MarketSaturationBook};

#[derive(Clone, Debug, PartialEq)]
pub struct GrowthProposal {
    pub district_id: DistrictId,
    pub block_id: Option<BlockId>,
    pub archetype_id: ArchetypeId,
    pub usage: BuildingUsage,
    pub actor_layer: GrowthActorLayer,
    pub anchor_tile: IVec2,
    pub priority: f32,
    pub seed: u64,
    pub reason_codes: Vec<GrowthReasonCode>,
    pub saturation_at_submit: f32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct GrowthProposalQueue {
    pub proposals: Vec<GrowthProposal>,
}

impl GrowthProposalQueue {
    pub fn enqueue(&mut self, proposal: GrowthProposal) {
        self.proposals.push(proposal);
    }

    pub fn drain_approved(&mut self) -> Vec<GrowthProposal> {
        std::mem::take(&mut self.proposals)
    }
}

#[must_use]
pub fn score_proposal(
    pressure: &super::district::DevelopmentPressure,
    saturation: &super::market::MarketSaturation,
    usage: BuildingUsage,
) -> f32 {
    let base = match usage {
        BuildingUsage::Residential => pressure.residential,
        BuildingUsage::Commercial | BuildingUsage::Office => pressure.commercial,
        BuildingUsage::Industrial | BuildingUsage::Logistics => pressure.industrial,
        _ => 0.25,
    };
    base * niche_factor(saturation, usage)
}

pub fn growth_proposal_tick_system(
    district_book: Res<DistrictBook>,
    pressure_book: Res<DevelopmentPressureBook>,
    saturation_book: Res<MarketSaturationBook>,
    mut queue: ResMut<GrowthProposalQueue>,
) {
    for district in district_book.districts.values() {
        let Some(pressure) = pressure_book.by_district.get(&district.id) else {
            continue;
        };
        let saturation = saturation_book
            .by_district
            .get(&district.id)
            .cloned()
            .unwrap_or_default();
        for archetype in &district.style_rules.allowed_archetypes {
            let usage = if archetype.0.contains("shop") || archetype.0.contains("grocery") {
                BuildingUsage::Commercial
            } else if archetype.0.contains("warehouse") {
                BuildingUsage::Logistics
            } else {
                BuildingUsage::Commercial
            };
            let priority = score_proposal(pressure, &saturation, usage);
            if priority <= 0.01 {
                continue;
            }
            let proposal = GrowthProposal {
                district_id: district.id.clone(),
                block_id: None,
                archetype_id: archetype.clone(),
                usage,
                actor_layer: GrowthActorLayer::Growth,
                anchor_tile: district.tile_rect.min,
                priority,
                seed: 42,
                reason_codes: Vec::new(),
                saturation_at_submit: saturation
                    .by_usage
                    .get(&usage)
                    .copied()
                    .unwrap_or(0.0),
            };
            if proposal_rejected_by_saturation(district, &saturation, &proposal) {
                let _ = super::market::saturation_reason_codes(true);
                continue;
            }
            if queue
                .proposals
                .iter()
                .any(|p| p.archetype_id == proposal.archetype_id && p.district_id == proposal.district_id)
            {
                continue;
            }
            queue.enqueue(proposal);
        }
    }
}

#[cfg(test)]
mod tests {
    
    
    
    

}

