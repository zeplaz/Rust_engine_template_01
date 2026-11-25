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
    /// Materials per metre of road (cargo, units) — the corridor works bill.
    #[serde(default)]
    pub materials_per_m: Vec<(String, f32)>,
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
    /// Cross-country truck speed (km/h) on open dry ground; the off-road speed field (RD-2)
    /// scales it per tile.
    pub offroad_speed_kmh: f32,
    /// RD-2 off-road field tuning (fords, rubble, mud, snow).
    #[serde(default)]
    pub offroad: OffroadTuning,
    /// RD-5 worn tracks (wear, fade, forming).
    #[serde(default)]
    pub worn_tracks: WornTrackTuning,
    /// RD-7 rail freight (trains, yards, intermodal).
    #[serde(default)]
    pub rail: RailTuning,
    pub tiers: Vec<RoadTierSpec>,
    /// Materials per metre of railway (ballast, rails, sleepers).
    #[serde(default = "default_rail_materials")]
    pub rail_materials_per_m: Vec<(String, f32)>,
}

/// RD-2: how water, rubble and wetness slow a wheeled vehicle off the road network
/// (land and slope come from the `wheeled_logistics` mobility profile).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct OffroadTuning {
    /// Water deeper than this (m) is a wall off-road.
    pub ford_max_m: f32,
    /// Water shallower than this (m) is just wet ground.
    pub wet_m: f32,
    /// Speed multiplier fording shallow water.
    pub ford_mul: f32,
    /// Speed multiplier over rubble / crater tiles.
    pub rubble_mul: f32,
    /// Soil moisture (0..1) where mud starts; full mud at 1.
    pub mud_from_moisture: f32,
    /// Speed lost in full mud (0..1).
    pub mud_slow: f32,
    /// Speed lost under full snow cover (0..1).
    pub snow_slow: f32,
}

impl Default for OffroadTuning {
    fn default() -> Self {
        Self { ford_max_m: 0.5, wet_m: 0.05, ford_mul: 0.35, rubble_mul: 0.3, mud_from_moisture: 0.6, mud_slow: 0.6, snow_slow: 0.5 }
    }
}

/// RD-7: freight trains, yards and when a haul goes by rail.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct RailTuning {
    /// Line speed of a freight train (km/h).
    pub speed_kmh: f32,
    /// Speed on the yard siding between the yard and the line (km/h).
    pub siding_kmh: f32,
    /// Units one wagon carries (t, m3 for tank wagons).
    pub wagon_capacity: u32,
    /// A yard joins the nearest rail node within this many tiles.
    pub yard_reach_tiles: f32,
    /// Hauls shorter than this (tiles, straight line) stay on the road.
    pub intermodal_min_tiles: f32,
    /// Wagons in a train a yard builds.
    pub build_wagons: u8,
    /// Steel per wagon and Machinery for the engine when a yard builds a train.
    pub steel_per_wagon: u32,
    pub engine_machinery: u32,
    /// Sim seconds between two trains one faction builds automatically.
    pub build_interval_s: u64,
    /// Level crossing: seconds a truck waits at the barrier, plus this per train running the line.
    pub crossing_base_s: f32,
    pub crossing_per_train_s: f32,
}

impl Default for RailTuning {
    fn default() -> Self {
        Self {
            speed_kmh: 80.0,
            siding_kmh: 15.0,
            wagon_capacity: 50,
            yard_reach_tiles: 10.0,
            intermodal_min_tiles: 60.0,
            build_wagons: 12,
            steel_per_wagon: 10,
            engine_machinery: 30,
            build_interval_s: 600,
            crossing_base_s: 12.0,
            crossing_per_train_s: 25.0,
        }
    }
}

impl RailTuning {
    #[must_use]
    pub fn speed_mps(&self) -> f32 {
        self.speed_kmh / 3.6
    }

    #[must_use]
    pub fn siding_mps(&self) -> f32 {
        self.siding_kmh / 3.6
    }

    /// Materials one built train costs (cargo, units).
    #[must_use]
    pub fn train_cost(&self) -> Vec<(&'static str, u32)> {
        vec![("Steel", self.steel_per_wagon * u32::from(self.build_wagons)), ("Machinery", self.engine_machinery)]
    }
}

/// RD-5: how traffic wears tracks into the ground.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct WornTrackTuning {
    /// Passes a tile needs before it counts as worn.
    pub passes_to_form: f32,
    /// Wear halves after this many sim seconds without traffic.
    pub half_life_s: f32,
    /// Shortest connected worn run (tiles) that becomes a track.
    pub min_run_tiles: usize,
    /// Wear a moving formation leaves on its tile per second, by class.
    pub infantry_per_s: f32,
    pub light_per_s: f32,
    pub wheeled_per_s: f32,
    pub tracked_per_s: f32,
}

impl Default for WornTrackTuning {
    fn default() -> Self {
        Self {
            passes_to_form: 30.0,
            half_life_s: 3600.0,
            min_run_tiles: 6,
            infantry_per_s: 0.05,
            light_per_s: 0.2,
            wheeled_per_s: 0.5,
            tracked_per_s: 1.5,
        }
    }
}

fn default_rail_materials() -> Vec<(String, f32)> {
    vec![("Gravel".into(), 1.0), ("Steel".into(), 0.25), ("Timber".into(), 0.3)]
}

/// Default bill per metre of each tier (cargo, units).
fn default_tier_materials(tier: RoadTier) -> Vec<(String, f32)> {
    let m: &[(&str, f32)] = match tier {
        RoadTier::Track => &[],
        RoadTier::Dirt => &[("Fuel", 0.02)],
        RoadTier::Gravel => &[("Gravel", 0.6), ("Fuel", 0.03)],
        RoadTier::Paved => &[("Gravel", 0.8), ("Cement", 0.1), ("Fuel", 0.05)],
        RoadTier::Concrete => &[("Gravel", 0.6), ("Concrete", 1.2), ("Steel", 0.05)],
        RoadTier::Motorway => &[("Gravel", 1.5), ("Concrete", 2.5), ("Steel", 0.15), ("Fuel", 0.1)],
    };
    m.iter().map(|(c, q)| ((*c).to_owned(), *q)).collect()
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
        materials_per_m: default_tier_materials(tier),
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
        offroad: OffroadTuning::default(),
        worn_tracks: WornTrackTuning::default(),
        rail: RailTuning::default(),
        rail_materials_per_m: default_rail_materials(),
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
