//! Fire VFX frontend — projection → instanced-quad rows, scatter, witness.
//!
//! Backend transport: [`crate::render::gpu_instanced_quad`] + [`crate::render::gpu_particle_draw`].

pub mod emit;
pub mod frame;
pub mod pack;
pub mod witness;

pub use emit::{
    emit_world_fire_particles_from_projection,
    seed_world_fire_particles_from_overlay_heat,
    update_world_fire_particles_from_projection, view_aware_particle_cull_wired,
};
pub use frame::{WorldFireParticleFrame, WorldFireParticleGpuStorage};
pub use pack::{GpuParticleInstance, GpuParticleQuadVertex, ParticleClass};
pub use witness::{
    fire_spark_011_green, fire_spark_compute_enabled, FireSparkWitness,
    FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA, FIRE_SPARK_SCATTER_MAX,
    FIRE_SPARK_STRATEGIC_ZOOM_ALPHA, FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
};
