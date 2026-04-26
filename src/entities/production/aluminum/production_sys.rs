// src/production/materials/aluminum/plugin.rs
use super::components::*;
use super::systems::*;
use crate::production::core::resources::ResourceType;
use bevy::prelude::*;

/// Plugin for aluminum production simulation
pub struct AluminumProductionPlugin;

impl Plugin for AluminumProductionPlugin {
    fn build(&self, app: &mut App) {
        // Register aluminum-specific types
        app.register_type::<BauxiteMine>()
            .register_type::<AluminaRefinery>()
            .register_type::<AluminumSmelter>()
            .register_type::<AluminumFabricationPlant>()
            .register_type::<FabricationLineType>()
            // Register aluminum resource types
            .add_systems(Startup, register_aluminum_resource_types)
            // Add aluminum-specific systems
            .add_systems(
                Update,
                (
                    bauxite_mining_system,
                    alumina_refining_system,
                    aluminum_smelting_system,
                    aluminum_fabrication_system,
                ),
            )
            // Add resource transportation system
            .add_systems(PostUpdate, aluminum_production_chain_system)
            // Initialize aluminum settings
            .init_resource::<AluminumProductionSettings>();
    }
}

fn register_aluminum_resource_types() {
    // This would register aluminum-specific resource types in a central registry
    // Implementation depends on how you manage your resource type system
}
