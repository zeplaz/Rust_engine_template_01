use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManufacturingDomain {
    Concrete,
    Aluminum,
    Power,
    Custom,
}

/// Serializable domain-level blueprint for future modular factories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingBlueprint {
    pub id: String,
    pub domain: ManufacturingDomain,
    pub process_tags: Vec<String>,
    pub throughput_target: f32,
}

/// ECS runtime marker to attach a blueprint id to an entity.
#[derive(Component, Debug, Clone)]
pub struct ManufacturingNode {
    pub blueprint_id: String,
    pub local_efficiency: f32,
}
