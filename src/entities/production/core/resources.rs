//! Resource storage types for production core — canonical `ResourceType` is `entities::types::p_enumz`.

use std::collections::HashMap;

use bevy::prelude::Component;

pub use crate::entities::types::p_enumz::ResourceType;

#[derive(Component, Debug, Clone, Default)]
pub struct ResourceStorage {
    pub amounts: HashMap<ResourceType, f32>,
}
