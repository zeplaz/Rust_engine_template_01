//! Generic GPU instanced-quad transport + fire VFX facade.
//!
//! Domain logic lives in [`crate::render::fire_vfx`]; see `src/dev/plan_gpu_particle_backend_split_v1.md`.

pub use crate::render::extracted_camera_metrics::FireParticleCameraScale;
pub use crate::render::fire_vfx::*;
pub use crate::render::gpu_instanced_quad::GpuInstancedQuadInstance;
pub use crate::render::fire_vfx::pack::{GpuParticleInstance, GpuParticleQuadVertex};

const _: () = assert!(
    std::mem::size_of::<GpuInstancedQuadInstance>() == std::mem::size_of::<GpuParticleInstance>()
);
