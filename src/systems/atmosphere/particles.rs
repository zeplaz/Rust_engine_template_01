//! Deferred CPU particle budget types — **not scheduled** (ES-0).
//! Production particles: `crate::render::fire_vfx` → GPU instanced quad spine.
//! ES-2 will register a domain frontend on `AtmospherePipelineSet::Particles`.

use bevy::prelude::*;

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

#[derive(Resource, Debug)]
pub struct AtmosphereParticlePool {
    pub max_alive: usize,
    pub next_write_slot: usize,
}

impl Default for AtmosphereParticlePool {
    fn default() -> Self {
        Self {
            max_alive: 4096,
            next_write_slot: 0,
        }
    }
}

#[derive(Resource, Debug)]
pub struct AtmosphereParticleBudget {
    pub max_instances: usize,
    pub last_desired: usize,
}

impl Default for AtmosphereParticleBudget {
    fn default() -> Self {
        Self {
            max_instances: 2048,
            last_desired: 0,
        }
    }
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
