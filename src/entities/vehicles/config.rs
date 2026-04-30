//! Serializable road-vehicle definitions (JSON / save data). No ECS types here.

use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entities::types::v_flagz::RoadVehicleType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilitaryCivilian {
    Civilian,
    Military,
}

impl FromStr for MilitaryCivilian {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "civilian" | "civ" => Ok(MilitaryCivilian::Civilian),
            "military" | "mil" => Ok(MilitaryCivilian::Military),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureTile {
    pub path: String,
    pub tiles: u32,
    pub emission_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadVehicleConfig {
    pub name: String,
    pub vtype: RoadVehicleType,
    pub capacity: i32,
    pub mass: f32,
    pub max_speed: f32,
    #[serde(default)]
    pub military_civilian: Option<String>,
    /// Outer key: load state (`full` / `empty`). Inner key: time-of-day (`midday`, `miday`, `night`).
    #[serde(default)]
    pub textures: HashMap<String, HashMap<String, Vec<TextureTile>>>,
}
