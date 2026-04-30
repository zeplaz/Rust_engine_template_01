// LEGACY MODULE (not actively wired):
// kept for migration reference while concrete runtime is split into
// `concrete/components.rs` and `concrete/systems.rs`.
// src/production/materials/concrete/plugin.rs
use super::components::*;
use super::systems::*;
use crate::production::core::resources::ResourceType;
use bevy::prelude::*;

/// Plugin for concrete production simulation
pub struct ConcreteProductionPlugin;

impl Plugin for ConcreteProductionPlugin {
    fn build(&self, app: &mut App) {
        // Register concrete-specific types
        app.register_type::<CementKiln>()
            .register_type::<AggregateMine>()
            .register_type::<ConcreteMixer>()
            .register_type::<ConcreteType>()
            // Register concrete resource types
            .add_systems(Startup, register_concrete_resource_types)
            // Add concrete-specific systems
            .add_systems(
                Update,
                (
                    cement_production_system,
                    aggregate_mining_system,
                    concrete_mixing_system,
                ),
            )
            // Add resource transportation system
            .add_systems(PostUpdate, concrete_production_chain_system)
            // Initialize concrete settings
            .init_resource::<ConcreteProductionSettings>();
    }
}

// src/production/materials/concrete/components.rs
use crate::production::core::resources::ResourceType;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct ConcreteProductionSettings {
    pub limestone_to_cement_ratio: f32,
    pub water_to_cement_ratio: f32,
    pub aggregate_to_cement_ratio: f32,
    pub electricity_per_ton: f32,
}

impl Default for ConcreteProductionSettings {
    fn default() -> Self {
        Self {
            limestone_to_cement_ratio: 1.5,
            water_to_cement_ratio: 0.4,
            aggregate_to_cement_ratio: 3.0,
            electricity_per_ton: 5.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CementKiln {
    pub temperature: f32,
    pub capacity: f32,
    pub efficiency: f32,
    pub co2_emissions: f32,
    pub fuel_consumption: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AggregateMine {
    pub deposit_quality: f32,
    pub extraction_rate: f32,
    pub size_distribution: AggregateSize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ConcreteMixer {
    pub capacity: f32,
    pub mixing_efficiency: f32,
    pub concrete_type: ConcreteType,
    pub additive_usage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum ConcreteType {
    Limecrete,
    Portland,
    Geopolymer,
    Gypsum,
    RapidSet,
    HighStrength,
    Lightweight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum AggregateSize {
    Fine,
    Medium,
    Coarse,
    Mixed,
}
