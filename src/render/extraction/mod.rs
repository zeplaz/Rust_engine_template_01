//! Sim → render **extraction** passes (ephemeral buffers, messages, no gameplay ownership).

mod fire_extract_scan;
mod fire_emission_profile;
mod frame_snapshot;
mod fire_visual_extract;
mod render_projection_graph;
mod procedural_build_extract;
mod procedural_module_extract;
mod smoke_visual_extract;

mod vegetation_visual_extract;

pub use vegetation_visual_extract::{
    build_vegetation_extract_frame, build_harness_topo_extract_frame,
    extract_glyph_deterministic, harness_topo_extract_witness_green,
    refresh_landscape_extract_sprite_witness, refresh_vegetation_extract_witness,
    vegetation_extract_witness_green, VegetationExtractFrame, VegetationExtractFrameSet,
    VegetationVisualExtractPlugin, VegExtractRow, VegExtractModifiers, LANDSCAPE_GRAMMAR_EXTRACT_LIVE_JSON,
};
pub use frame_snapshot::ExtractFrameSnapshot;
pub use fire_emission_profile::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, FireEmissionProfile, FireVisualProxy,
};
pub use crate::render::sim_visual_extract::FireVisualGpuInstance;
pub use fire_visual_extract::{
    extract_fire_simulation_snapshot, sync_shared_overlay_from_simulation, FireAtmosphereAggregate,
    FireVisualFramePlugin, FireVisualFrameSet,
};
pub use fire_extract_scan::{
    build_fire_extract_scan_set, expand_moore_rim_one, fire_extract_glow_domain,
};
pub use smoke_visual_extract::{build_smoke_visual_extract, SmokeVisualBridgeWitness};
pub use crate::render::sim_visual_extract::FireVisualFrame;
pub use procedural_build_extract::{
    assemble_procedural_build_instances, extract_procedural_build_assembly,
    ProceduralBuildExtract, ProceduralBuildInstance,
};
pub use procedural_module_extract::{
    load_procedural_module_scenes, scene_for_module, sync_procedural_module_visual_policy,
    ProceduralModuleSceneCatalog, ProceduralModuleVisualPolicy,
};
pub use render_projection_graph::{
    f2_tactical_fire_projection_fixture, fire_projection_stamp_aligned,
    projection_graph_build_signature, projection_graph_runtime_order_snapshot,
    run_render_projection_graph, spatial_distribution_stats, FireProjectionNode, ProjectionNodeTrait,
    ProjectionGraphFrameCoherence, RenderProjectionContext, RenderProjectionGraph,
    CLUSTERED_FIRE_INSTANCE_CAP,
};
