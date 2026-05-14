//! Sim → render **extraction** passes (ephemeral buffers, messages, no gameplay ownership).

mod fire_emission_profile;
mod frame_snapshot;
mod fire_visual_extract;
mod render_projection_graph;

pub use frame_snapshot::ExtractFrameSnapshot;
pub use fire_emission_profile::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, FireEmissionProfile, FireVisualProxy,
};
pub use crate::render::sim_visual_extract::FireVisualGpuInstance;
pub use fire_visual_extract::{
    FireAtmosphereAggregate, FireVisualFramePlugin, FireVisualFrameSet,
};
pub use crate::render::sim_visual_extract::FireVisualFrame;
pub use render_projection_graph::{
    run_render_projection_graph, spatial_distribution_stats, FireProjectionNode,
    ProjectionNodeTrait, RenderProjectionContext, RenderProjectionGraph,
    CLUSTERED_FIRE_INSTANCE_CAP,
};
