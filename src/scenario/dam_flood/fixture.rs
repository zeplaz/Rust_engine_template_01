//! Authored valley fixture RON (DF-1 / D3).

use bevy::math::IVec2;
use serde::Deserialize;

pub const DAM_VALLEY_FIXTURE_REL: &str = "assets/configs/scenes/dam_valley_v1.ron";

#[derive(Debug, Clone, Deserialize)]
pub struct DamValleyFixture {
    pub schema_version: u32,
    pub id: String,
    #[serde(default = "default_extent")]
    pub extent_tiles: u32,
    #[serde(default = "default_baseflow")]
    pub river_baseflow_m3s: f32,
    pub narrows_tile: (i32, i32),
    #[serde(default = "default_span")]
    pub dam_span_tiles: u32,
    pub gauges: Vec<DamValleyGaugeSpec>,
    pub town_origin: (i32, i32),
    pub nuclear_plant_origin: (i32, i32),
    pub hydro_plant_origin: (i32, i32),
    pub launch_site_origin: (i32, i32),
    pub forest_floodplain: DamValleyAabb,
    /// Woodland test zone (fire + missile strike), on the dry plateau well away from the dam,
    /// the river and the flood, so the fire test and the water test never interfere. A third
    /// zone will bring water and fire together (S9 dead-over-live burn).
    #[serde(default = "default_woodland_test_zone")]
    pub woodland_test_zone: DamValleyAabb,
    pub road_bridge_tile: (i32, i32),
    /// Gravel supply roads the scene's trucks need (PLAN-ROADS-v1 RD-11): HQ -> dam abutment and
    /// down the west bank to the bridge. Each is one committed straight line (head, tail).
    #[serde(default = "default_supply_roads")]
    pub supply_roads: Vec<((i32, i32), (i32, i32))>,
    /// Faction start config for this scene (same schema as the shipped one): HQ depot, stockpile
    /// and fleet a player would bring to build a dam here. Configures the real start path.
    #[serde(default)]
    pub faction_start: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DamValleyGaugeSpec {
    pub id: String,
    pub label: String,
    pub tile: (i32, i32),
    #[serde(default = "default_half")]
    pub cross_section_half_tiles: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DamValleyAabb {
    pub min: (i32, i32),
    pub max: (i32, i32),
}

fn default_extent() -> u32 {
    320
}
fn default_baseflow() -> f32 {
    150.0
}
fn default_span() -> u32 {
    12
}
fn default_half() -> u32 {
    4
}

impl DamValleyFixture {
    #[must_use]
    pub fn narrows_ivec(&self) -> IVec2 {
        IVec2::new(self.narrows_tile.0, self.narrows_tile.1)
    }
}

/// Load fixture from repo-relative path. Falls back to compiled defaults if missing/invalid.
#[must_use]
pub fn load_dam_valley_fixture() -> DamValleyFixture {
    match std::fs::read_to_string(DAM_VALLEY_FIXTURE_REL) {
        Ok(text) => match ron::from_str::<DamValleyFixture>(&text) {
            Ok(f) => f,
            Err(err) => {
                bevy::log::warn!(
                    target: "dam_flood::fixture",
                    "dam valley fixture parse failed ({err}) — using defaults"
                );
                default_fixture()
            }
        },
        Err(err) => {
            bevy::log::warn!(
                target: "dam_flood::fixture",
                "dam valley fixture missing ({err}) — using defaults"
            );
            default_fixture()
        }
    }
}

/// South-west plateau, ~100 tiles from the dam and ~80 from the river (outside the valley walls).
fn default_woodland_test_zone() -> DamValleyAabb {
    DamValleyAabb { min: (24, 24), max: (84, 84) }
}

fn default_supply_roads() -> Vec<((i32, i32), (i32, i32))> {
    vec![((140, 168), (146, 157)), ((146, 157), (153, 156)), ((146, 157), (152, 110))]
}

fn default_fixture() -> DamValleyFixture {
    DamValleyFixture {
        schema_version: 1,
        id: "dam_valley_v1_default".into(),
        extent_tiles: 320,
        river_baseflow_m3s: 220.0,
        narrows_tile: (160, 160),
        dam_span_tiles: 12,
        gauges: vec![
            DamValleyGaugeSpec {
                id: "G1".into(),
                label: "upstream".into(),
                tile: (160, 220),
                cross_section_half_tiles: 4,
            },
            DamValleyGaugeSpec {
                id: "G2".into(),
                label: "dam_site".into(),
                tile: (160, 160),
                cross_section_half_tiles: 4,
            },
            DamValleyGaugeSpec {
                id: "G3".into(),
                label: "downstream".into(),
                tile: (160, 100),
                cross_section_half_tiles: 4,
            },
        ],
        town_origin: (212, 196),
        nuclear_plant_origin: (180, 140),
        hydro_plant_origin: (160, 152),
        launch_site_origin: (100, 180),
        forest_floodplain: DamValleyAabb {
            min: (120, 120),
            max: (200, 200),
        },
        woodland_test_zone: default_woodland_test_zone(),
        road_bridge_tile: (160, 110),
        supply_roads: default_supply_roads(),
        faction_start: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dam_valley_fixture_loads_or_defaults() {
        let f = load_dam_valley_fixture();
        assert!(f.schema_version >= 1);
        assert_eq!(f.gauges.len(), 3);
        assert!(f.river_baseflow_m3s > 0.0);
    }
}
