//! In-memory registry built from embedded JSON (always available at compile time).

use bevy::prelude::*;
use std::collections::HashMap;

use crate::entities::production::power::plant_definition::{PlantDefinition, PlantDefinitionFile};

#[derive(Resource, Debug, Clone)]
pub struct PlantDefinitionRegistry {
    pub by_id: HashMap<String, PlantDefinition>,
}

impl Default for PlantDefinitionRegistry {
    fn default() -> Self {
        Self::from_embedded_json()
    }
}

impl PlantDefinitionRegistry {
    /// Bytes shipped with the binary; edit `assets/config/power/plant_definitions.json` and rebuild.
    pub fn from_embedded_json() -> Self {
        const JSON: &str = include_str!("../../../../assets/config/power/plant_definitions.json");
        Self::from_json_str(JSON).unwrap_or_else(|e| {
            bevy::log::error!("plant_definitions.json parse failed: {e}");
            Self {
                by_id: HashMap::new(),
            }
        })
    }

    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        let file: PlantDefinitionFile = serde_json::from_str(s)?;
        let mut by_id = HashMap::with_capacity(file.plants.len());
        for p in file.plants {
            by_id.insert(p.id.clone(), p);
        }
        Ok(Self { by_id })
    }

    pub fn get(&self, id: &str) -> Option<&PlantDefinition> {
        self.by_id.get(id)
    }
}

#[cfg(test)]
mod plant_registry_tests {
    use super::*;

    #[test]
    fn plant_definitions_embedded_json_round_trips_ron_format() {
        let json = include_str!("../../../../assets/config/power/plant_definitions.json");
        let file: PlantDefinitionFile = serde_json::from_str(json).expect("embedded json");
        let cfg = ron::ser::PrettyConfig::new().depth_limit(8).indentor("    ".into());
        let ron_s = ron::ser::to_string_pretty(&file, cfg).unwrap();
        let file2: PlantDefinitionFile = ron::de::from_str(&ron_s).unwrap();
        assert_eq!(file.schema_version, file2.schema_version);
        assert_eq!(file.plants.len(), file2.plants.len());
    }
}
