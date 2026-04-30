use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

/// Resource taxonomy for production / logistics (serializable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Hash, PartialEq)]
pub enum ResourceType {
    Wood,
    Coal,
    Oil,
    RareEarth,
    Metal,
    Steel,
    Concrete,
    Fertilizer,
    Chemicals,
    Electronics,
    Energy,
    Fuel,
    Ammunition,
    WarSupply,
    Knowledge,
    Labour,
    Food,
    Water,
    Paper,
    Electricity,
}

impl ResourceType {
    /// Used by production storage decay (expand when perishability is data-driven).
    pub fn is_perishable(self) -> bool {
        matches!(self, Self::Food | Self::Water)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceFilter {
    All,
}
