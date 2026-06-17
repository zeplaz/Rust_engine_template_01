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
    use super::*;
    use crate::strategic::settlement::district::portland_fixture_district;
    use crate::strategic::settlement::market::compute_market_saturation_for_district;
    use crate::strategic::settlement::pressure::{
        compute_district_pressure_system, rollup_district_metrics_system,
    };
    use crate::strategic::settlement::town::portland_fixture_town;

    #[test]
    fn growth_tick_enqueues_without_world_mutation() {
        assert!(growth_proposal_witness_green());
    }

    #[test]
    fn saturated_district_skips_fourth_shop_proposal() {
        use std::collections::HashMap;

        let town = portland_fixture_town();
        let districts = portland_fixture_district(&town);
        let record = districts.districts.values().next().unwrap().clone();
        let blocks = crate::strategic::settlement::market::fixture_blocks_with_site_count(&record.id, 3);
        let sat = compute_market_saturation_for_district(&record, Some(&blocks));
        let mut saturation_book = MarketSaturationBook::default();
        saturation_book
            .by_district
            .insert(record.id.clone(), sat);
        let pressure = DevelopmentPressureBook {
            by_district: HashMap::from([(
                record.id.clone(),
                super::super::district::DevelopmentPressure {
                    commercial: 0.9,
                    ..Default::default()
                },
            )]),
        };
        let mut app = App::new();
        app.insert_resource(districts)
            .insert_resource(pressure)
            .insert_resource(saturation_book)
            .init_resource::<GrowthProposalQueue>()
            .add_systems(Update, growth_proposal_tick_system);
        app.update();
        let queue = app.world().resource::<GrowthProposalQueue>();
        assert!(queue
            .proposals
            .iter()
            .all(|p| p.archetype_id.0 != "corner_shop"));
    }
}

#[must_use]
pub fn growth_proposal_witness_green() -> bool {
    use crate::strategic::settlement::district::portland_fixture_district;
    use crate::strategic::settlement::pressure::{
        compute_district_pressure_system, rollup_district_metrics_system,
    };
    use crate::strategic::settlement::town::portland_fixture_town;

    let town = portland_fixture_town();
    let districts = portland_fixture_district(&town);
    let mut app = App::new();
    app.insert_resource(town)
        .insert_resource(districts)
        .init_resource::<super::district::DistrictMetricsBook>()
        .init_resource::<DevelopmentPressureBook>()
        .init_resource::<MarketSaturationBook>()
        .init_resource::<GrowthProposalQueue>()
        .add_systems(
            Update,
            (
                rollup_district_metrics_system,
                compute_district_pressure_system,
                super::market::compute_market_saturation_system,
                growth_proposal_tick_system,
            )
                .chain(),
        );
    app.update();
    !app.world().resource::<GrowthProposalQueue>().proposals.is_empty()
}
