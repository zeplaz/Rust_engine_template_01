// src/production/materials/aluminum/components.rs
use crate::production::core::resources::ResourceType;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct AluminumProductionSettings {
    pub bauxite_per_electricity: f32,
    pub alumina_per_bauxite: f32,
    pub aluminum_per_alumina: f32,
    pub electricity_per_aluminum: f32,
}

impl Default for AluminumProductionSettings {
    fn default() -> Self {
        Self {
            bauxite_per_electricity: 0.5,
            alumina_per_bauxite: 0.25,
            aluminum_per_alumina: 0.2,
            electricity_per_aluminum: 15.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BauxiteMine {
    pub ore_richness: f32,
    pub mine_depth: f32,
    pub max_depth: f32,
    pub depletion_rate: f32,
    pub environmental_impact: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AluminaRefinery {
    pub digestion_temperature: f32,
    pub pressure: f32,
    pub red_mud_storage: f32,
    pub max_red_mud_storage: f32,
    pub caustic_soda_efficiency: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AluminumSmelter {
    pub pot_count: u32,
    pub current_efficiency: f32,
    pub anode_degradation: f32,
    pub cryolite_level: f32,
    pub fluoride_emissions: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AluminumFabricationPlant {
    pub production_line_type: FabricationLineType,
    pub alloy_mixing_capacity: f32,
    pub product_quality: f32,
    pub scrap_rate: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum FabricationLineType {
    Extrusion,
    Rolling,
    Casting,
    Forging,
}
