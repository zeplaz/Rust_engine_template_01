// Rendering systems
mod base_cam;
pub mod extraction;
mod gpu_buffer_registry;
mod gpu_bind_group_registry;
mod gpu_packed_formats;
mod fire_smoke_shader_handles;
mod fx_burst_request;
pub mod gpu_weather_fire_field;
mod atmosphere_partial_gpu;
pub mod lighting;
mod light;
pub mod shaders;
pub mod sim_visual_extract;
mod tile_world_fallback;
mod overlay_field_buffers;
mod domain_overlay_gpu;
mod gpu_particles;
mod gpu_particle_draw;
mod gpu_representation_metrics;
mod visual_agreement;
mod visual_snapshot_commit;
mod vt_spatial_invariants;
mod logistics_visual_snapshot;
mod ecology_visual_snapshot;
mod visual_domain_snapshots;
mod phase_f_lod_proof;
mod stage6_virtualization;
mod stage5_readiness;
mod vt_app_integration;
mod vt_ci_matrix;
mod spine_governance_matrix;
mod domain_projection_frame;
mod gpu_indirect_draw;

#[cfg(feature = "bevy_tilemap_adapter")]
pub mod tilemap_adapter;

pub use tile_world_fallback::{
    SimMinimapUiState, TileWorldFallbackAfterFireExtract, TileWorldFallbackPlugin,
    TileWorldFallbackRasterDirty, TileWorldFallbackSprite,
};

// Public exports
pub use gpu_bind_group_registry::{
    buffer_binding_for, BindGroupBufferBinding, BindGroupId, GPUBindGroupEntry, GPUBindGroupRegistry,
    WEATHER_FIRE_FIELD_FIRE_BIND_GROUP, WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP,
    WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP,
};
pub use gpu_representation_metrics::GpuRepresentationMetrics;
pub use gpu_particle_draw::{
    register_world_fire_particle_draw, sync_particle_draw_dispatch_from_policy,
    WorldFireParticleDrawDispatch,
};
pub use domain_overlay_gpu::{
    emit_domain_overlay_frame_from_projection, DomainOverlayGpuFrame, EcologyOverlayGpuRow,
    LogisticsOverlayGpuRow,
};
pub use gpu_particles::{
    emit_world_fire_particles_from_projection, GpuParticleInstance, GpuParticleQuadVertex,
    ParticleClass, WorldFireParticleFrame, WorldFireParticleGpuStorage,
};
pub use gpu_packed_formats::{
    ecology_overlay_row_format, fire_particle_expanded_vertex_format, fire_visual_instance_format,
    fire_particle_instance_format, heat_diffusion_cell_format, logistics_overlay_row_format,
    packed_byte_size, LodBandBufferPolicy, PackedBufferFormat, PackedFormatId,
    ECOLOGY_OVERLAY_ROW_FORMAT, FIRE_PARTICLE_EXPANDED_VERTEX_FORMAT, FIRE_PARTICLE_INSTANCE_FORMAT,
    FIRE_VISUAL_INSTANCE_FORMAT, HEAT_DIFFUSION_CELL_FORMAT, HEAT_DIFFUSION_CELL_STRIDE,
    LOGISTICS_OVERLAY_ROW_FORMAT,
};
pub use gpu_buffer_registry::{
    row_capacity_bytes, BufferId, BufferVisibility, ECOLOGY_OVERLAY_BUFFER,
    FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER, FIRE_PARTICLE_INSTANCES_BUFFER,
    FIRE_VISUAL_INSTANCES_BUFFER, GpuSlice, GPUBufferEntry, GPUBufferRegistry,
    HEAT_DIFFUSION_FIELD_BUFFER, LOGISTICS_OVERLAY_BUFFER, RegisteredBufferDescriptor,
    RegistryBufferAllocation, RegistryError, RegistryUploadStats,
};
pub use fire_smoke_shader_handles::{
    FireSmokeShaderHandles, FIRE_PARTICLE_WGSL, SMOKE_VOLUME_WGSL,
};
pub use fx_burst_request::{collect_burst_hints_from_fire_visual, FxParticleBurstRequest};
pub use visual_snapshot_commit::{commit_fire_visual_snapshot, CommittedVisualSnapshotFence};
pub use vt_spatial_invariants::{
    passes_vt5_spatial_invariants, sample_fire_row, VT5_MIN_MEAN_DISTANCE, VT5_MIN_OCCUPIED_CHUNKS,
    VT5_MIN_VARIANCE,
};
pub use logistics_visual_snapshot::LogisticsVisualSnapshot;
pub use ecology_visual_snapshot::EcologyVisualSnapshot;
pub use visual_domain_snapshots::{
    publish_ecology_visual_snapshot, publish_logistics_visual_snapshot,
};
pub use phase_f_lod_proof::{PhaseFLodProofPlugin, PhaseFLodProofReport};
pub use stage6_virtualization::{
    gather_stage6_readiness, stage6_readiness_passes, AsyncDomainApplyQueue, PagedAtlasResidency,
    RuntimeAtlasSlot, Stage6ReadinessReport, Stage6VirtualizationFrame, Stage6VirtualizationPlugin,
};
pub use stage5_readiness::{
    evaluate_app_stage5_readiness, evaluate_stage5_spine_checklist, stage5_readiness_passes,
    AppStage5ReadinessReport, Stage5ReadinessPlugin, Stage5ReadinessProfile, Stage5SpineChecklist,
};
pub use gpu_weather_fire_field::{
    FireVisualGpuInstanceStorage, GpuWeatherFireFieldPlugin, WeatherFireFieldDebugOverlay,
    WeatherFireFieldUniforms,
};
pub use sim_visual_extract::{
    ChunkFireHeat, ChunkSmokeGpu, ClimateVisualAggregate, FireEmitterGpu, FireVisualFrame,
    FireVisualGpuInstance, SimChunkSmokeVisualExtract, SimFireEmitterVisualExtract,
};
pub use lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as FireLightEmissionSample,
    FireLightType, CLUSTER_MERGE_RADIUS,
};
pub use extraction::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, ExtractFrameSnapshot, FireAtmosphereAggregate,
    CLUSTERED_FIRE_INSTANCE_CAP, FireEmissionProfile, FireProjectionNode, FireVisualFramePlugin,
    FireVisualFrameSet, FireVisualProxy, ProjectionNodeTrait, RenderProjectionContext,
    RenderProjectionGraph, run_render_projection_graph,
};
pub use overlay_field_buffers::{
    SharedOverlayFieldBuffers, SharedOverlayFieldBuffersPlugin,
};
pub use visual_agreement::{
    hash_chunk_fire_heat, hash_shared_overlay_heat, record_visual_agreement_frame,
    assert_snapshot_stamp, OverlayAgreementDebug, VisualAgreementError, VisualAgreementFrame,
    WorldPreviewVt4Probe,
};
pub use vt_ci_matrix::{
    apply_vt4_ci_report_to_overlay_debug, apply_vt4_ci_surface_checks, build_deterministic_ci_scenario,
    build_live_vt4_scenario, record_vt_ci_matrix_live, run_vt4_ci_matrix, run_vt5_ci_spatial_matrix,
    Vt4CiReport, Vt4CiScenario, Vt4SurfaceId, VtCiMatrixLiveReport, VtCiMatrixPlugin,
};
pub use domain_projection_frame::{
    build_domain_projection_frame, merge_domain_projection_into_representation,
    publish_domain_projection_frame, DomainProjectionFrame, DomainProjectionFramePlugin,
    DomainProjectionId, DomainProjectionSlice,
};
pub use gpu_indirect_draw::{
    compact_world_fire_indirect_draw, sync_world_fire_indirect_draw, GpuIndirectDrawSpine,
    GpuIndirectDrawSpinePlugin, WorldFireIndirectDrawArgs, WORLD_FIRE_VERTICES_PER_INSTANCE,
};
pub use light::*;

#[cfg(feature = "bevy_tilemap_adapter")]
pub use tilemap_adapter::{
    ChunkTilemaps, TilemapAdapterPlugin, TilemapLayerVisibility,
};