//! District book + sim metrics (OG-1 / ECON-OG-1-A).

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::actors::BuildingUsage;
use super::ids::{ArchetypeId, DistrictId, TownId};
use super::zoning::ZoningClass;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DistrictStyleRules {
    #[serde(default)]
    pub allowed_archetypes: Vec<ArchetypeId>,
    #[serde(default)]
    pub archetype_caps: HashMap<ArchetypeId, u32>,
    #[serde(default)]
    pub usage_caps: HashMap<BuildingUsage, u32>,
}

impl DistrictStyleRules {
    pub fn cap_for_archetype(&self, id: &ArchetypeId) -> u32 {
        self.archetype_caps.get(id).copied().unwrap_or(u32::MAX)
    }

    pub fn cap_for_usage(&self, usage: BuildingUsage) -> u32 {
        self.usage_caps.get(&usage).copied().unwrap_or(match usage {
            BuildingUsage::Commercial => 3,
            BuildingUsage::Residential => 8,
            _ => u32::MAX,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DistrictRecord {
    pub id: DistrictId,
    pub town_id: TownId,
    pub name: String,
    pub tile_rect: IRect,
    pub zoning_default: ZoningClass,
    #[serde(default)]
    pub style_rules: DistrictStyleRules,
}

impl DistrictRecord {
    pub fn contains_tile(&self, tile: IVec2) -> bool {
        self.tile_rect.contains(tile)
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DistrictBook {
    pub districts: HashMap<DistrictId, DistrictRecord>,
    pub default_district: Option<DistrictId>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DistrictMetrics {
    pub population_density: f32,
    pub employment_density: f32,
    pub wealth: f32,
    pub desirability: f32,
    pub transport_access: f32,
    pub services: f32,
    pub pollution: f32,
    pub crime: f32,
    pub employment_demand: f32,
    pub housing_deficit: f32,
    pub freight_access: f32,
    pub utility_service: f32,
    pub civic_pressure: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DevelopmentPressure {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DistrictMetricsBook {
    pub by_district: HashMap<DistrictId, DistrictMetrics>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct DevelopmentPressureBook {
    pub by_district: HashMap<DistrictId, DevelopmentPressure>,
}

pub fn portland_fixture_district(town: &super::town::TownBook) -> DistrictBook {
    let town_id = town.default_town.clone().expect("fixture town");
    let id = DistrictId("north_industrial".into());
    let mut districts = HashMap::new();
    districts.insert(
        id.clone(),
        DistrictRecord {
            id: id.clone(),
            town_id,
            name: "North Industrial".into(),
            tile_rect: IRect::from_corners(IVec2::new(0, 0), IVec2::new(256, 256)),
            zoning_default: ZoningClass::Industrial,
            style_rules: DistrictStyleRules {
                allowed_archetypes: vec![
                    ArchetypeId("corner_shop".into()),
                    ArchetypeId("grocery".into()),
                    ArchetypeId("warehouse".into()),
                ],
                archetype_caps: HashMap::from([
                    (ArchetypeId("corner_shop".into()), 2),
                    (ArchetypeId("grocery".into()), 1),
                ]),
                usage_caps: HashMap::from([(BuildingUsage::Commercial, 4)]),
            },
        },
    );
    DistrictBook {
        districts,
        default_district: Some(id),
    }
}
