//! Corridor profile registry (INFRA-E0-001).
//!
//! RON-backed road/rail vocabulary — replaces long-term string `profile.contains("rail")` hacks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::strategic::CorridorType;
use crate::systems::transport::CorridorClass;

/// Default example bundle (ship / tests).
pub const DEFAULT_CORRIDOR_PROFILES_RON: &str =
    "assets/config/infrastructure/corridor_profiles.example.ron";

#[derive(Clone, Debug, Deserialize)]
struct CorridorProfilesFile {
    schema_version: u32,
    #[serde(default)]
    roads: Vec<RoadProfileFile>,
    #[serde(default)]
    rails: Vec<RailProfileFile>,
}

#[derive(Clone, Debug, Deserialize)]
struct RoadProfileFile {
    id: String,
    road_type: String,
    lanes: u8,
    speed_limit_kmh: u16,
    #[serde(default)]
    surface_tags: Vec<String>,
    turn_radius_m: f32,
    #[serde(default = "default_one")]
    base_cost: f32,
    #[serde(default)]
    allowed_agents: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct RailProfileFile {
    id: String,
    gauge: String,
    electrification: String,
    tracks: u8,
    max_speed_kmh: u16,
    turn_radius_m: f32,
    #[serde(default = "default_one")]
    base_cost: f32,
    #[serde(default)]
    allowed_agents: Vec<String>,
}

fn default_one() -> f32 {
    1.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoadProfile {
    pub id: String,
    pub road_type: String,
    pub lanes: u8,
    pub speed_limit_kmh: u16,
    pub surface_tags: Vec<String>,
    pub turn_radius_m: f32,
    pub base_cost: f32,
    pub allowed_agents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RailProfile {
    pub id: String,
    pub gauge: String,
    pub electrification: String,
    pub tracks: u8,
    pub max_speed_kmh: u16,
    pub turn_radius_m: f32,
    pub base_cost: f32,
    pub allowed_agents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CorridorProfileKind {
    Road(RoadProfile),
    Rail(RailProfile),
}

impl CorridorProfileKind {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Road(p) => p.id.as_str(),
            Self::Rail(p) => p.id.as_str(),
        }
    }

    #[must_use]
    pub fn corridor_class(&self) -> CorridorClass {
        match self {
            Self::Road(_) => CorridorClass::Road,
            Self::Rail(_) => CorridorClass::Rail,
        }
    }

    #[must_use]
    pub fn strategic_corridor_type(&self) -> CorridorType {
        match self {
            Self::Road(p) => {
                let t = p.road_type.to_ascii_lowercase();
                if t.contains("highway") || t.contains("express") || t.contains("arterial") {
                    CorridorType::Highway
                } else {
                    CorridorType::Logistics
                }
            }
            Self::Rail(_) => CorridorType::Rail,
        }
    }

    #[must_use]
    pub fn turn_radius_m(&self) -> f32 {
        match self {
            Self::Road(p) => p.turn_radius_m,
            Self::Rail(p) => p.turn_radius_m,
        }
    }
}

/// Loaded corridor profiles — authoritative lookup for edge `profile` ids.
#[derive(Resource, Clone, Debug, Default)]
pub struct ProfileRegistry {
    pub schema_version: u32,
    roads: HashMap<String, RoadProfile>,
    rails: HashMap<String, RailProfile>,
}

impl ProfileRegistry {
    #[must_use]
    pub fn resolve(&self, profile_id: &str) -> Option<CorridorProfileKind> {
        if let Some(p) = self.roads.get(profile_id) {
            return Some(CorridorProfileKind::Road(p.clone()));
        }
        if let Some(p) = self.rails.get(profile_id) {
            return Some(CorridorProfileKind::Rail(p.clone()));
        }
        None
    }

    #[must_use]
    pub fn corridor_class(&self, profile_id: &str) -> CorridorClass {
        self.resolve(profile_id)
            .map(|k| k.corridor_class())
            .unwrap_or_else(|| crate::systems::transport::corridor_class_from_profile(profile_id))
    }

    #[must_use]
    pub fn strategic_corridor_type(&self, profile_id: &str) -> CorridorType {
        if let Some(k) = self.resolve(profile_id) {
            return k.strategic_corridor_type();
        }
        strategic_corridor_type_heuristic(profile_id)
    }

    pub fn load_from_ron(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path.as_ref())?;
        let file: CorridorProfilesFile = ron::de::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("RON: {e}"))
        })?;
        Ok(Self::from_file(file))
    }

    pub fn load_default_example() -> std::io::Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self::load_from_ron(root.join(DEFAULT_CORRIDOR_PROFILES_RON))
    }

    fn from_file(file: CorridorProfilesFile) -> Self {
        let roads = file
            .roads
            .into_iter()
            .map(|p| {
                let profile = RoadProfile {
                    id: p.id.clone(),
                    road_type: p.road_type,
                    lanes: p.lanes,
                    speed_limit_kmh: p.speed_limit_kmh,
                    surface_tags: p.surface_tags,
                    turn_radius_m: p.turn_radius_m,
                    base_cost: p.base_cost,
                    allowed_agents: p.allowed_agents,
                };
                (p.id, profile)
            })
            .collect();
        let rails = file
            .rails
            .into_iter()
            .map(|p| {
                let profile = RailProfile {
                    id: p.id.clone(),
                    gauge: p.gauge,
                    electrification: p.electrification,
                    tracks: p.tracks,
                    max_speed_kmh: p.max_speed_kmh,
                    turn_radius_m: p.turn_radius_m,
                    base_cost: p.base_cost,
                    allowed_agents: p.allowed_agents,
                };
                (p.id, profile)
            })
            .collect();
        Self {
            schema_version: file.schema_version,
            roads,
            rails,
        }
    }
}

fn strategic_corridor_type_heuristic(profile: &str) -> CorridorType {
    let p = profile.to_ascii_lowercase();
    if p.contains("rail") {
        CorridorType::Rail
    } else if p.contains("pipe") {
        CorridorType::Pipeline
    } else if p.contains("power") || p.contains("grid") {
        CorridorType::PowerTransmission
    } else if p.contains("military") || p.contains("supply") {
        CorridorType::MilitarySupply
    } else if p.contains("highway") || p.contains("road") || p == "default_road" {
        CorridorType::Highway
    } else {
        CorridorType::Logistics
    }
}

pub struct InfrastructureProfilesPlugin;

impl Plugin for InfrastructureProfilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::infrastructure::UtilityAuthoringTool>();
        match ProfileRegistry::load_default_example() {
            Ok(reg) => {
                app.insert_resource(reg);
            }
            Err(e) => {
                warn!(
                    target: "infrastructure::profiles",
                    "ProfileRegistry default load failed ({e}); using empty registry"
                );
                app.insert_resource(ProfileRegistry::default());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_example_ron_profiles() {
        let reg = ProfileRegistry::load_default_example().expect("example RON");
        assert_eq!(reg.schema_version, 1);
        assert!(reg.resolve("default_road").is_some());
        assert!(reg.resolve("standard_gauge_freight").is_some());
        assert!(reg.resolve("missing_profile").is_none());
    }

    #[test]
    fn registry_corridor_class_beats_string_heuristic() {
        let reg = ProfileRegistry::load_default_example().expect("example RON");
        assert_eq!(
            reg.corridor_class("standard_gauge_freight"),
            CorridorClass::Rail
        );
        assert_eq!(reg.corridor_class("default_road"), CorridorClass::Road);
    }

    #[test]
    fn road_profile_turn_radius_present() {
        let reg = ProfileRegistry::load_default_example().expect("example RON");
        let kind = reg.resolve("highway_arterial").expect("highway");
        assert!(kind.turn_radius_m() >= 30.0);
    }
}
