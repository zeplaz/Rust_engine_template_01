//! Sim → render **extraction** passes (ephemeral buffers, messages, no gameplay ownership).

mod fire_emission_profile;
mod fire_visual_extract;

pub use fire_emission_profile::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, FireEmissionProfile, FireVisualProxy,
};
pub use crate::render::sim_visual_extract::FireVisualGpuInstance;
pub use fire_visual_extract::{
    FireAtmosphereAggregate, FireVisualFrame, FireVisualFramePlugin, FireVisualFrameSet,
};
