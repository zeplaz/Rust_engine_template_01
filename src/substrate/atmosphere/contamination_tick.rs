//! AC-003 — fold resident slab contamination into clipmap L0 (witness path).

use bevy::prelude::*;

use crate::substrate::registry::WorldSubstrateRegistry;

use super::{AtmosphereClipmapStack, AtmosphereClipmapWitness};

pub fn contamination_tick_system(
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut witness: ResMut<AtmosphereClipmapWitness>,
    registry: Option<Res<WorldSubstrateRegistry>>,
) {
    if let Some(registry) = registry.as_deref() {
        for chunk in registry.chunks.chunks.values() {
            let seed = chunk
                .contamination
                .airborne
                .iter()
                .copied()
                .fold(0.0_f32, f32::max);
            if seed <= 0.0 {
                continue;
            }
            if let Some(level0) = stack.levels.first_mut() {
                if let Some(cell) = level0.smoke_density.first_mut() {
                    *cell = cell.max(seed);
                }
            }
        }
    }
    let _ = &mut witness;
}
