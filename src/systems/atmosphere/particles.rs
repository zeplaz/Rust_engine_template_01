//! CPU-side particle **kinds** + strict budget before GPU instancing (`base_fire2_smoke.md` §5–6).
//!
//! GPU hook: load [`super::gpu_paths::ATMOSPHERE_PARTICLE_INSTANCING_WGSL`] when wiring instanced quads
//! or swap in **Hanabi** / custom indirect draws.

use bevy::prelude::*;

use super::diagnostics::AtmosphereDiagnostics;
use super::field::AtmosphereField;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtmosphereParticleKind {
    Smoke,
    Ash,
    Ember,
    Spark,
    Dust,
    ToxicGas,
    Steam,
}

/// Lightweight instance payload for future GPU upload.
#[derive(Clone, Copy, Debug)]
pub struct AtmosphereParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub density: f32,
    pub temperature: f32,
    pub kind: AtmosphereParticleKind,
}

/// Reserved **GPU pool** sizing for instanced particles / Hanabi bridge (`fx-pool-1`).
#[derive(Resource, Debug, Clone, Copy)]
pub struct AtmosphereParticlePool {
    pub max_alive: u32,
    pub next_write_slot: u32,
}

impl Default for AtmosphereParticlePool {
    fn default() -> Self {
        Self {
            max_alive: 8192,
            next_write_slot: 0,
        }
    }
}

/// Stub controller: derives desired instance count from field smoke mean (capped).
#[derive(Resource, Debug, Clone)]
pub struct AtmosphereParticleBudget {
    pub max_instances: usize,
    pub last_desired: usize,
}

impl Default for AtmosphereParticleBudget {
    fn default() -> Self {
        Self {
            max_instances: 4096,
            last_desired: 0,
        }
    }
}

pub fn atmosphere_particle_controller(
    field: Res<AtmosphereField>,
    mut budget: ResMut<AtmosphereParticleBudget>,
    mut pool: ResMut<AtmosphereParticlePool>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    diag.particle_controller_runs = diag.particle_controller_runs.wrapping_add(1);
    pool.next_write_slot = pool.next_write_slot.wrapping_add(1) % pool.max_alive.max(1);
    let n = field.cells.len().max(1) as f32;
    let mean_smoke: f32 = field.cells.iter().map(|c| c.smoke_density).sum::<f32>() / n;
    let desired = ((mean_smoke * 8000.0) as usize).clamp(0, budget.max_instances);
    budget.last_desired = desired;
}

#[cfg(test)]
mod tests {
    #[test]
    fn desired_particles_clamped_by_max() {
        let max = 100;
        let mean_smoke = 1.0;
        let desired = ((mean_smoke * 8000.0) as usize).clamp(0, max);
        assert_eq!(desired, 100);
    }
}
