//! Resource storage types for production core — canonical `ResourceType` is `entities::types::p_enumz`.

use std::collections::HashMap;

use bevy::prelude::Component;

pub use crate::entities::types::p_enumz::ResourceType;

#[derive(Component, Debug, Clone, Default)]
pub struct ResourceStorage {
    pub amounts: HashMap<ResourceType, f32>,
}

/// Optional per-type **capacity** (silos, warehouses, wagons). HUD bars use **stock / cap**
/// when `max_amounts[type] > 0`; otherwise bars scale relative to the largest stock in the panel.
#[derive(Component, Debug, Clone, Default)]
pub struct ResourceStorageCapacity {
    pub max_amounts: HashMap<ResourceType, f32>,
}
