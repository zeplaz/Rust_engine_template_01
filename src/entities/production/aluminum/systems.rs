use bevy::prelude::*;

use crate::entities::production::aluminum::components::{
    AluminaRefineryRuntime, AluminumFabricationPlantRuntime, AluminumProductionConfig,
    AluminumSmelterRuntime, BauxiteMineRuntime,
};

pub struct AluminumRuntimePlugin;

impl Plugin for AluminumRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AluminumProductionConfig>().add_systems(
            Update,
            (
                bauxite_mining_runtime_system,
                alumina_refining_runtime_system,
                aluminum_smelting_runtime_system,
                aluminum_fabrication_runtime_system,
            ),
        );
    }
}

fn bauxite_mining_runtime_system(mut query: Query<&mut BauxiteMineRuntime>) {
    for mut mine in &mut query {
        mine.ore_richness = mine.ore_richness.clamp(0.0, 1.0);
        mine.mine_depth = mine.mine_depth.clamp(0.0, mine.max_depth.max(0.0));
        mine.depletion_rate = mine.depletion_rate.clamp(0.0, 1.0);
        mine.environmental_impact = mine.environmental_impact.clamp(0.0, 1.0);
    }
}

fn alumina_refining_runtime_system(mut query: Query<&mut AluminaRefineryRuntime>) {
    for mut refinery in &mut query {
        refinery.digestion_temperature = refinery.digestion_temperature.max(0.0);
        refinery.pressure = refinery.pressure.max(0.0);
        refinery.red_mud_storage = refinery
            .red_mud_storage
            .clamp(0.0, refinery.max_red_mud_storage.max(0.0));
        refinery.caustic_soda_efficiency = refinery.caustic_soda_efficiency.clamp(0.0, 1.0);
    }
}

fn aluminum_smelting_runtime_system(
    config: Res<AluminumProductionConfig>,
    mut query: Query<&mut AluminumSmelterRuntime>,
) {
    for mut smelter in &mut query {
        smelter.current_efficiency = smelter.current_efficiency.clamp(0.0, 1.0);
        smelter.anode_degradation = smelter.anode_degradation.clamp(0.0, 1.0);
        smelter.cryolite_level = smelter.cryolite_level.max(config.aluminum_per_alumina * 0.1);
        smelter.fluoride_emissions = smelter.fluoride_emissions.max(0.0);
    }
}

fn aluminum_fabrication_runtime_system(mut query: Query<&mut AluminumFabricationPlantRuntime>) {
    for mut plant in &mut query {
        plant.alloy_mixing_capacity = plant.alloy_mixing_capacity.max(0.0);
        plant.product_quality = plant.product_quality.clamp(0.0, 1.0);
        plant.scrap_rate = plant.scrap_rate.clamp(0.0, 1.0);
    }
}
