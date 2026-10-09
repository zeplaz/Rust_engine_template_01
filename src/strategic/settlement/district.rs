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
    /// Closed ring of tile vertices. Empty means membership stays on `tile_rect`.
    #[serde(default)]
    pub ring: Vec<IVec2>,
    #[serde(default)]
    pub style_rules: DistrictStyleRules,
    /// Building sites that belong to this zone. A site id appears in at most one zone.
    #[serde(default)]
    pub site_ids: Vec<u64>,
}

impl DistrictRecord {
    /// Ring interior when the ring has at least three vertices. Boundary counts.
    /// A shorter ring stays on the inclusive rectangle.
    pub fn contains_tile(&self, tile: IVec2) -> bool {
        if self.ring.len() >= 3 {
            tile_in_ring(&self.ring, tile)
        } else {
            self.tile_rect.contains(tile)
        }
    }
}

/// Inclusive corners of a half-open [`IRect`], one vertex per corner.
#[must_use]
pub fn core_ring_for_rect(rect: IRect) -> Vec<IVec2> {
    let min = rect.min;
    let far = IVec2::new(rect.max.x - 1, rect.max.y - 1);
    if far.x < min.x || far.y < min.y {
        return Vec::new();
    }
    vec![
        IVec2::new(min.x, min.y),
        IVec2::new(far.x, min.y),
        IVec2::new(far.x, far.y),
        IVec2::new(min.x, far.y),
    ]
}

/// Half-open bounds of a ring. Matches [`IRect::contains`] for a rectangular ring.
#[must_use]
pub fn ring_bounds(ring: &[IVec2]) -> Option<IRect> {
    let mut iter = ring.iter().copied();
    let first = iter.next()?;
    let mut min = first;
    let mut max_inclusive = first;
    for tile in iter {
        min = min.min(tile);
        max_inclusive = max_inclusive.max(tile);
    }
    Some(IRect {
        min,
        max: max_inclusive.saturating_add(IVec2::ONE),
    })
}

/// Writes the four-corner ring of each district's current rectangle.
pub fn stamp_core_rings(districts: &mut DistrictBook) {
    for district in districts.districts.values_mut() {
        district.ring = core_ring_for_rect(district.tile_rect);
    }
}

/// Even-odd interior, boundary included. The last vertex closes to the first.
#[must_use]
pub fn tile_in_ring(ring: &[IVec2], tile: IVec2) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let n = ring.len();
    let last_closes = n >= 2 && ring[0] == ring[n - 1];
    let edge_count = if last_closes { n - 1 } else { n };
    if edge_count < 3 {
        return false;
    }
    for i in 0..edge_count {
        let a = ring[i];
        let b = ring[(i + 1) % edge_count];
        if point_on_segment(tile, a, b) {
            return true;
        }
    }
    let mut inside = false;
    for i in 0..edge_count {
        let a = ring[i];
        let b = ring[(i + 1) % edge_count];
        let ay = a.y > tile.y;
        let by = b.y > tile.y;
        if ay == by {
            continue;
        }
        let dy = i64::from(b.y) - i64::from(a.y);
        let dx = i64::from(b.x) - i64::from(a.x);
        let px = i64::from(tile.x) - i64::from(a.x);
        let py = i64::from(tile.y) - i64::from(a.y);
        let left = px * dy;
        let right = py * dx;
        let crosses = if dy > 0 { left < right } else { left > right };
        if crosses {
            inside = !inside;
        }
    }
    inside
}

fn point_on_segment(p: IVec2, a: IVec2, b: IVec2) -> bool {
    if a == b {
        return p == a;
    }
    let abx = i64::from(b.x) - i64::from(a.x);
    let aby = i64::from(b.y) - i64::from(a.y);
    let apx = i64::from(p.x) - i64::from(a.x);
    let apy = i64::from(p.y) - i64::from(a.y);
    if abx * apy - aby * apx != 0 {
        return false;
    }
    let dot = apx * abx + apy * aby;
    if dot < 0 {
        return false;
    }
    let len2 = abx * abx + aby * aby;
    dot <= len2
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
            ring: Vec::new(),
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
            site_ids: Vec::new(),
        },
    );
    DistrictBook {
        districts,
        default_district: Some(id),
    }
}

/// One district per town, centered on that town. Order follows town id.
#[must_use]
pub fn generated_districts_for_towns(towns: &super::town::TownBook) -> DistrictBook {
    let mut ids: Vec<_> = towns.towns.keys().cloned().collect();
    ids.sort_by(|a, b| a.0.cmp(&b.0));
    let mut districts = HashMap::new();
    let mut default_district = None;
    for town_id in ids {
        let Some(town) = towns.towns.get(&town_id) else {
            continue;
        };
        let id = DistrictId(format!("dist-{}", town.id.0));
        if towns.default_town.as_ref() == Some(&town_id) {
            default_district = Some(id.clone());
        }
        let center = town.center_tile;
        districts.insert(
            id.clone(),
            DistrictRecord {
                id,
                town_id: town.id.clone(),
                name: format!("{} core", town.name),
                tile_rect: IRect::from_corners(center - IVec2::new(8, 8), center + IVec2::new(8, 8)),
                zoning_default: ZoningClass::Residential,
                ring: Vec::new(),
                style_rules: DistrictStyleRules {
                    allowed_archetypes: vec![ArchetypeId("corner_shop".into())],
                    ..Default::default()
                },
                site_ids: Vec::new(),
            },
        );
    }
    if default_district.is_none() {
        let mut keys: Vec<_> = districts.keys().cloned().collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0));
        default_district = keys.into_iter().next();
    }
    DistrictBook {
        districts,
        default_district,
    }
}

/// Another administrative zone on a town that already has one. Same town id.
#[must_use]
pub fn add_admin_zone(
    districts: &mut DistrictBook,
    town_id: &super::ids::TownId,
    name: &str,
    tile_rect: IRect,
) -> DistrictId {
    let n = districts
        .districts
        .values()
        .filter(|district| &district.town_id == town_id)
        .count();
    let id = DistrictId(format!("zone-{}-{n}", town_id.0));
    districts.districts.insert(
        id.clone(),
        DistrictRecord {
            id: id.clone(),
            town_id: town_id.clone(),
            name: name.to_string(),
            tile_rect,
            zoning_default: ZoningClass::MixedUse,
            ring: Vec::new(),
            style_rules: DistrictStyleRules::default(),
            site_ids: Vec::new(),
        },
    );
    id
}

/// Puts a site in a zone. Refuses a site that already sits in a different zone.
#[must_use]
pub fn place_site_in_zone(districts: &mut DistrictBook, site_id: u64, zone: &DistrictId) -> bool {
    if !districts.districts.contains_key(zone) {
        return false;
    }
    if districts
        .districts
        .values()
        .any(|district| district.id != *zone && district.site_ids.contains(&site_id))
    {
        return false;
    }
    let Some(district) = districts.districts.get_mut(zone) else {
        return false;
    };
    if !district.site_ids.contains(&site_id) {
        district.site_ids.push(site_id);
    }
    true
}

/// Moves a site to another zone of the same town. Other towns are rejected.
/// The site stays a single row: the old zone loses it.
#[must_use]
pub fn move_building_zone(districts: &mut DistrictBook, site_id: u64, dest: &DistrictId) -> bool {
    let Some(dest_town) = districts
        .districts
        .get(dest)
        .map(|district| district.town_id.clone())
    else {
        return false;
    };
    let Some(current) = districts
        .districts
        .values()
        .find(|district| district.site_ids.contains(&site_id))
        .map(|district| district.id.clone())
    else {
        return false;
    };
    let Some(current_town) = districts
        .districts
        .get(&current)
        .map(|district| district.town_id.clone())
    else {
        return false;
    };
    if current_town != dest_town {
        return false;
    }
    if current == *dest {
        return true;
    }
    if let Some(from) = districts.districts.get_mut(&current) {
        from.site_ids.retain(|id| *id != site_id);
    }
    if let Some(to) = districts.districts.get_mut(dest) {
        if !to.site_ids.contains(&site_id) {
            to.site_ids.push(site_id);
        }
    }
    let copies = districts
        .districts
        .values()
        .filter(|district| district.site_ids.contains(&site_id))
        .count();
    copies == 1
}
