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
