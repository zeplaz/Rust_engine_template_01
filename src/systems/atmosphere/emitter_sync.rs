//! Low-count [`FireEmitter`] on chunk entities (`base_fire2_smoke.md` §4).
//! Rates scale with heat and aggregated [`FuelLayer`](crate::terrain::fire::FuelLayer) when present.

use bevy::prelude::*;

use crate::systems::fire::{ChunkFuelProfile, ChunkSurfaceFire};
use crate::systems::sim_control::SimControlState;
use crate::terrain::fire::FuelLayer;
use crate::terrain::generation::Chunk;

use super::diagnostics::AtmosphereDiagnostics;

#[derive(Component, Clone, Copy, Debug)]
pub struct FireEmitter {
    pub intensity: f32,
    pub smoke_rate: f32,
    pub ember_rate: f32,
}

/// CPU emitter row from heat + optional unified fuel (used by sync + tests).
#[inline]
pub fn fire_emitter_from_heat_fuel(heat: f32, fuel: Option<&FuelLayer>) -> FireEmitter {
    let h = heat.clamp(0.0, 1.0);
    let f = fuel.copied().unwrap_or_default();
    let smoke = h
        * 45.0
        * (0.48 + f.toxic_smoke * 0.5 + f.volatility * 0.38 + f.surface_fuel * 0.18).clamp(0.08, 2.2);
    let ember = h * 8.0 * (0.28 + f.ember_generation * 0.95).clamp(0.05, 2.5);
    FireEmitter {
        intensity: h,
        smoke_rate: smoke,
        ember_rate: ember,
    }
}

pub(crate) fn sync_fire_emitters(
    ctrl: Res<SimControlState>,
    mut diag: ResMut<AtmosphereDiagnostics>,
    mut commands: Commands,
    q: Query<
        (Entity, &Chunk, &ChunkSurfaceFire, Option<&ChunkFuelProfile>),
        Without<FireEmitter>,
    >,
) {
    if !ctrl.should_tick() {
        return;
    }
    diag.emitter_sync_runs = diag.emitter_sync_runs.wrapping_add(1);
    for (e, _, fire, prof) in &q {
        let fuel = prof.map(|p| p.to_fuel_layer());
        commands.entity(e).insert(fire_emitter_from_heat_fuel(
            fire.heat,
            fuel.as_ref(),
        ));
    }
}

pub(crate) fn update_fire_emitters_from_heat(
    ctrl: Res<SimControlState>,
    mut q: Query<(&ChunkSurfaceFire, &mut FireEmitter, Option<&ChunkFuelProfile>)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    for (fire, mut em, prof) in &mut q {
        let fuel = prof.map(|p| p.to_fuel_layer());
        *em = fire_emitter_from_heat_fuel(fire.heat, fuel.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_scale_with_intensity() {
        let em = fire_emitter_from_heat_fuel(0.5, None);
        assert!(em.smoke_rate > em.ember_rate);
        assert!((em.intensity - 0.5).abs() < 1e-5);
    }

    #[test]
    fn fuel_dump_boosts_smoke_over_neutral() {
        let neutral = fire_emitter_from_heat_fuel(0.6, None);
        let dump = fire_emitter_from_heat_fuel(0.6, Some(&FuelLayer::fuel_dump()));
        assert!(dump.smoke_rate > neutral.smoke_rate);
    }
}
