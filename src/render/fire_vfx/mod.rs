//! Fire VFX frontend — projection → instanced-quad rows, scatter, spark tuning.
//!
//! Backend transport: [`crate::render::pipelines::gpu_instanced_quad`] + [`crate::render::pipelines::gpu_particle_draw`].

pub mod consumable;
pub mod emit;
pub mod frame;
pub mod pack;
pub mod particle_anim;
pub mod spark_tuning;

pub use consumable::{
    effect_spawn_hook_resolver_wired, resolve_effect_spawn, spawn_effect_consumable,
    EffectConsumable, EffectConsumableDef, EffectConsumableRegistry,
    EffectConsumableRegistryPlugin, EffectFieldId, EffectLane, EffectResolvedSpawn,
    EffectSpawnResolveContext, SpawnHookKind, EFFECT_REGISTRY_ROOT,
};
pub use emit::{
    emit_world_fire_particles_from_projection, seed_world_fire_particles_from_overlay_heat,
    update_world_fire_particles_from_projection, view_aware_particle_cull_wired,
};
pub use frame::{WorldFireParticleFrame, WorldFireParticleGpuStorage};
pub use pack::{GpuParticleInstance, GpuParticleQuadVertex, ParticleClass};
pub use spark_tuning::{
    fire_spark_compute_enabled, FireSparkStats, FIRE_SPARK_FULL_SCATTER_PX_PER_TILE,
    FIRE_SPARK_MIN_PX_PER_TILE, FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE, FIRE_SPARK_SCATTER_MAX,
    FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
};

