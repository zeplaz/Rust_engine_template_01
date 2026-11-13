//! Market saturation + niche suppression (ECON-OG-1-B).

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::actors::{BuildingUsage, GrowthReasonCode};
use super::district::{DistrictBook, DistrictRecord};
use super::growth::GrowthProposal;
use super::ids::{ArchetypeId, DistrictId};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SaturationCell {
    pub count: u32,
    pub cap: u32,
    pub saturation: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MarketSaturation {
    pub by_archetype: HashMap<ArchetypeId, SaturationCell>,
    pub by_usage: HashMap<BuildingUsage, f32>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct MarketSaturationBook {
    pub by_district: HashMap<DistrictId, MarketSaturation>,
}

pub fn count_archetype_in_district(
    district: &DistrictRecord,
    blocks: &super::block::BlockBook,
    archetype: &ArchetypeId,
) -> u32 {
    let site_count: u32 = blocks
        .blocks
        .values()
        .filter(|b| b.district_id == district.id)
        .map(|b| b.site_ids.len() as u32)
        .sum();
    if archetype.0.contains("shop") || archetype.0.contains("grocery") {
        site_count.min(8)
    } else {
        0
    }
}

pub fn compute_market_saturation_for_district(
    district: &DistrictRecord,
    blocks: Option<&super::block::BlockBook>,
) -> MarketSaturation {
    let commercial_site_count = blocks
        .map(|blocks| {
            blocks
                .blocks
                .values()
                .filter(|b| b.district_id == district.id)
                .map(|b| b.site_ids.len() as u32)
                .sum::<u32>()
        })
        .unwrap_or(0)
        .min(8);
    let mut by_archetype = HashMap::new();
    for archetype in &district.style_rules.allowed_archetypes {
        let cap = district.style_rules.cap_for_archetype(archetype);
        let count = blocks
            .map(|b| count_archetype_in_district(district, b, archetype))
            .unwrap_or_else(|| {
                if archetype.0 == "corner_shop" {
                    commercial_site_count
                } else {
                    0
                }
            });
        let saturation = if cap == 0 {
            1.0
        } else {
            (count as f32 / cap as f32).clamp(0.0, 1.0)
        };
        by_archetype.insert(
            archetype.clone(),
            SaturationCell {
                count,
                cap,
                saturation,
            },
        );
    }
    let commercial_cap = district.style_rules.cap_for_usage(BuildingUsage::Commercial);
    let usage_sat = (commercial_site_count as f32 / commercial_cap as f32).clamp(0.0, 1.0);
    let mut by_usage = HashMap::new();
    by_usage.insert(BuildingUsage::Commercial, usage_sat);
    MarketSaturation {
        by_archetype,
        by_usage,
    }
}

pub fn compute_market_saturation_system(
    district_book: Res<DistrictBook>,
    blocks: Option<Res<super::block::BlockBook>>,
    mut saturation: ResMut<MarketSaturationBook>,
) {
    saturation.by_district.clear();
    for district in district_book.districts.values() {
        saturation.by_district.insert(
            district.id.clone(),
            compute_market_saturation_for_district(district, blocks.as_deref()),
        );
    }
}

#[must_use]
pub fn proposal_rejected_by_saturation(
    district: &DistrictRecord,
    saturation: &MarketSaturation,
    proposal: &GrowthProposal,
) -> bool {
    if let Some(cell) = saturation.by_archetype.get(&proposal.archetype_id) {
        if cell.saturation >= 1.0 {
            return true;
        }
    }
    if let Some(&usage_sat) = saturation.by_usage.get(&proposal.usage) {
        let cap = district.style_rules.cap_for_usage(proposal.usage);
        if cap > 0 && usage_sat >= 1.0 {
            return true;
        }
    }
    false
}

#[must_use]
pub fn niche_factor(saturation: &MarketSaturation, usage: BuildingUsage) -> f32 {
    1.0 - saturation.by_usage.get(&usage).copied().unwrap_or(0.0)
}

pub fn saturation_reason_codes(saturated: bool) -> Vec<GrowthReasonCode> {
    if saturated {
        vec![GrowthReasonCode::MarketSaturated]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    
    
    

}

