//! JSON loader for road vehicle configs. Legacy Drez helpers: [`super::legacy_drez`].

use crate::entities::RoadVehicleConfig;
use std::fs;
use std::path::Path;

pub fn deserialize_road_vehicle_configs<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<RoadVehicleConfig>, Box<dyn std::error::Error + Send + Sync>> {
    let file_contents = fs::read_to_string(path)?;
    let road_vehicle_configs: Vec<RoadVehicleConfig> = serde_json::from_str(&file_contents)?;
    Ok(road_vehicle_configs)
}
