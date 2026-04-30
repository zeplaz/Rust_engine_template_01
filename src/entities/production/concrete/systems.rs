use bevy::prelude::*;

use crate::entities::production::concrete::components::{
    AggregateMineRuntime, CementKilnRuntime, ConcreteMixerRuntime, ConcreteProductionConfig,
};

pub struct ConcreteRuntimePlugin;

impl Plugin for ConcreteRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConcreteProductionConfig>().add_systems(
            Update,
            (
                cement_kiln_runtime_system,
                aggregate_mine_runtime_system,
                concrete_mixer_runtime_system,
            ),
        );
    }
}

fn cement_kiln_runtime_system(
    config: Res<ConcreteProductionConfig>,
    mut query: Query<&mut CementKilnRuntime>,
) {
    for mut kiln in &mut query {
        kiln.efficiency = kiln.efficiency.clamp(0.0, 1.0);
        kiln.temperature = kiln.temperature.max(0.0);
        kiln.capacity = kiln.capacity.max(config.electricity_per_ton * 0.1);
    }
}

fn aggregate_mine_runtime_system(mut query: Query<&mut AggregateMineRuntime>) {
    for mut mine in &mut query {
        mine.deposit_quality = mine.deposit_quality.clamp(0.0, 1.0);
        mine.extraction_rate = mine.extraction_rate.max(0.0);
    }
}

fn concrete_mixer_runtime_system(mut query: Query<&mut ConcreteMixerRuntime>) {
    for mut mixer in &mut query {
        mixer.capacity = mixer.capacity.max(0.0);
        mixer.mixing_efficiency = mixer.mixing_efficiency.clamp(0.0, 1.0);
    }
}
