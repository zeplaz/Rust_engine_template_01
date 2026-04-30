use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Serializable policy/config for concrete manufacturing equations.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ConcreteProductionConfig {
    pub limestone_to_cement_ratio: f32,
    pub water_to_cement_ratio: f32,
    pub aggregate_to_cement_ratio: f32,
    pub electricity_per_ton: f32,
}

impl Default for ConcreteProductionConfig {
    fn default() -> Self {
        Self {
            limestone_to_cement_ratio: 1.5,
            water_to_cement_ratio: 0.4,
            aggregate_to_cement_ratio: 3.0,
            electricity_per_ton: 5.0,
        }
    }
}

/// ECS runtime state for mutable per-entity kiln variables.
#[derive(Component, Debug, Clone)]
pub struct CementKilnRuntime {
    pub temperature: f32,
    pub capacity: f32,
    pub efficiency: f32,
}

#[derive(Component, Debug, Clone)]
pub struct AggregateMineRuntime {
    pub deposit_quality: f32,
    pub extraction_rate: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ConcreteMixerRuntime {
    pub capacity: f32,
    pub mixing_efficiency: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcreteType {
    Limecrete,
    Portland,
    Geopolymer,
    Gypsum,
    RapidSet,
    HighStrength,
    Lightweight,
}
