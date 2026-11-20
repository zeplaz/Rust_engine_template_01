//! Road tiers (PLAN-ROADS-v1 RD-0): speed, lanes, capacity, weather and flood behaviour per tier.
//! Table: `assets/config/infrastructure/road_tiers.ron`, built-in defaults when it is missing.
//! The transport graph turns a tier's speed into edge travel time; the truck route planner reads
//! the same table, so road choice is by time.

use std::sync::OnceLock;

use serde::Deserialize;

/// Built road quality. `Track` is worn by traffic (not built); `Motorway` is limited access.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize)]
pub enum RoadTier {
    Track,
    Dirt,
    #[default]
    Gravel,
    Paved,
    Concrete,
    Motorway,
}

impl RoadTier {
    pub const ALL: [RoadTier; 6] = [
        RoadTier::Track,
        RoadTier::Dirt,
        RoadTier::Gravel,
        RoadTier::Paved,
        RoadTier::Concrete,
        RoadTier::Motorway,
    ];

    /// Transport-edge profile id for this tier.
    #[must_use]
    pub fn profile_id(self) -> &'static str {
        match self {
            RoadTier::Track => "road_track",
            RoadTier::Dirt => "road_dirt",
            RoadTier::Gravel => "road_gravel",
            RoadTier::Paved => "road_paved",
            RoadTier::Concrete => "road_concrete",
            RoadTier::Motorway => "road_motorway",
        }
    }

    #[must_use]
    pub fn from_profile_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.profile_id() == id)
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            RoadTier::Track => "Track",
            RoadTier::Dirt => "Dirt road",
            RoadTier::Gravel => "Gravel road",
            RoadTier::Paved => "Paved road",
            RoadTier::Concrete => "Concrete road",
            RoadTier::Motorway => "Motorway",
        }
    }

    /// Limited access: joins other roads only at its ends (interchanges); crossings are grade separated.
    #[must_use]
    pub fn limited_access(self) -> bool {
        matches!(self, RoadTier::Motorway)
    }

    #[must_use]
    pub fn spec(self) -> &'static RoadTierSpec {
        road_tier_table().spec(self)
    }
}

/// One tier row.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RoadTierSpec {
    pub tier: RoadTier,
    pub speed_kmh: f32,
    pub lanes_default: u8,
    pub capacity_veh_h_lane: f32,
    /// Speed multiplier when wet (mud on unpaved tiers).
    pub wet_speed_mul: f32,
    /// Water depth (m) that closes the road.
    pub flood_close_m: f32,
}

impl RoadTierSpec {
    #[must_use]
    pub fn speed_mps(&self) -> f32 {
        self.speed_kmh / 3.6
    }
}

/// The tier table plus the off-road speed trucks fall back to.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RoadTierTable {
    pub schema_version: u32,
    /// Cross-country truck speed (km/h) until the off-road speed field (RD-2) lands.
    pub offroad_speed_kmh: f32,
    pub tiers: Vec<RoadTierSpec>,
}

impl RoadTierTable {
    #[must_use]
    pub fn spec(&self, tier: RoadTier) -> &RoadTierSpec {
        self.tiers
            .iter()
            .find(|s| s.tier == tier)
            .unwrap_or_else(|| default_table().tiers.iter().find(|s| s.tier == tier).expect("default has all tiers"))
    }

    #[must_use]
    pub fn offroad_speed_mps(&self) -> f32 {
        self.offroad_speed_kmh / 3.6
    }
}

const TABLE_PATH: &str = "assets/config/infrastructure/road_tiers.ron";

fn row(tier: RoadTier, speed_kmh: f32, lanes: u8, cap: f32, wet: f32, flood: f32) -> RoadTierSpec {
    RoadTierSpec {
        tier,
        speed_kmh,
        lanes_default: lanes,
        capacity_veh_h_lane: cap,
        wet_speed_mul: wet,
        flood_close_m: flood,
    }
}

fn default_table() -> &'static RoadTierTable {
    static DEFAULT: OnceLock<RoadTierTable> = OnceLock::new();
    DEFAULT.get_or_init(|| RoadTierTable {
        schema_version: 1,
        offroad_speed_kmh: 18.0,
        tiers: vec![
            row(RoadTier::Track, 25.0, 1, 40.0, 0.4, 0.3),
            row(RoadTier::Dirt, 35.0, 1, 60.0, 0.5, 0.3),
            row(RoadTier::Gravel, 55.0, 2, 120.0, 0.85, 0.5),
            row(RoadTier::Paved, 80.0, 2, 300.0, 1.0, 0.5),
            row(RoadTier::Concrete, 90.0, 2, 400.0, 1.0, 0.6),
            row(RoadTier::Motorway, 110.0, 4, 900.0, 1.0, 0.8),
        ],
    })
}

/// The loaded table (RON when present and valid, else the built-in defaults).
#[must_use]
pub fn road_tier_table() -> &'static RoadTierTable {
    static TABLE: OnceLock<RoadTierTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::fs::read_to_string(TABLE_PATH)
            .ok()
            .and_then(|text| match ron::from_str::<RoadTierTable>(&text) {
                Ok(t) => Some(t),
                Err(e) => {
                    bevy::log::warn!(target: "construction::roads", "road tier table {TABLE_PATH}: {e}; using defaults");
                    None
                }
            })
            .unwrap_or_else(|| default_table().clone())
    })
}

/// Speed (m/s) of a transport-edge profile when it is a road tier.
#[must_use]
pub fn road_profile_speed_mps(profile: &str) -> Option<f32> {
    RoadTier::from_profile_id(profile).map(|t| t.spec().speed_mps())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_table_parses_and_has_every_tier() {
        let text = std::fs::read_to_string(TABLE_PATH).expect("road tier table ships");
        let table: RoadTierTable = ron::from_str(&text).expect("valid RON");
        for t in RoadTier::ALL {
            assert!(table.tiers.iter().any(|s| s.tier == t), "{t:?} missing");
        }
        let speeds: Vec<f32> = RoadTier::ALL.iter().map(|t| table.spec(*t).speed_kmh).collect();
        assert!(speeds.windows(2).all(|w| w[0] < w[1]), "tiers get faster: {speeds:?}");
        assert!(table.offroad_speed_kmh < table.spec(RoadTier::Track).speed_kmh);
    }
}
