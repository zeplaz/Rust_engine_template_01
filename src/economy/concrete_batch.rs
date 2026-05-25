//! Concrete batching stub (I4-02) — local mix + cure timer on mixer sites.

use bevy::prelude::*;

/// Local batch state on concrete mixer / integrated plants.
#[derive(Component, Clone, Debug)]
pub struct ConcreteBatchState {
    pub volume_m3: f32,
    pub cure_ticks_remaining: u32,
}

impl Default for ConcreteBatchState {
    fn default() -> Self {
        Self {
            volume_m3: 0.0,
            cure_ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ConcreteBatchRegistered;

pub fn register_concrete_batch_on_activation_system(
    mut commands: Commands,
    q: Query<
        (Entity, &crate::economy::activation::BuildingDefinitionRef),
        (
            With<crate::economy::activation::IndustrialFacilityActivated>,
            Without<ConcreteBatchRegistered>,
        ),
    >,
) {
    for (entity, def_ref) in &q {
        let id = def_ref.catalog_id.as_str();
        if id.contains("mixer") || id.contains("integrated") || id.contains("concrete_basic") {
            commands.entity(entity).insert((
                ConcreteBatchState {
                    volume_m3: 8.0,
                    cure_ticks_remaining: 30,
                },
                ConcreteBatchRegistered,
            ));
        }
    }
}

pub fn tick_concrete_batch_cure_system(mut q: Query<&mut ConcreteBatchState>) {
    for mut batch in &mut q {
        if batch.cure_ticks_remaining > 0 {
            batch.cure_ticks_remaining -= 1;
        }
    }
}
