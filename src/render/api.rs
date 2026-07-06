//! RGR-M1 explicit public render API — plugins, GPU spine, stage5, diagnostics.
//! `crate::render::*` re-exports this module from `mod.rs`.

pub use super::infrastructure_overlay::{
    collect_infrastructure_overlay_edges_system, collect_transport_overlay_edges_system,
    compute_island_partition, infra_overlay_polish_green, infrastructure_overlay_hud_legend_wired,
    infrastructure_overlay_legend_rows, infrastructure_overlay_polish_witness_fields,
    stroke_for_layer, InfrastructureEdgeOverlay, InfrastructureNetworkLayer,
    InfrastructureOverlayDrawRequests, InfrastructureOverlayLegendRow,
    InfrastructureOverlayPlugin, InfrastructureOverlaySettings, InfrastructureOverlayStroke,
    PowerLineOverlayState, PowerMapOverlayPresentation, power_map_overlay_green,
    power_map_overlay_witness_fields, power_overlay_extended_legend_rows,
    stroke_for_power_line_state, stroke_for_voltage_class, voltage_for_link,
    sync_power_overlay_auto_on_system, refresh_power_island_from_damage_system,
};
pub use super::power_map_overlay_draw::{
    draw_power_map_overlay_egui, paint_stroke_line, power_map_overlay_draw_witness_green,
};
pub use super::terrain_render_authority::{
    resolve_sim_default_authority, terrain_cpu_fallback_env_forced, TerrainRenderAuthority,
    TerrainRenderAuthorityPlugin,
};
pub use super::terrain_material_atlas::{
    atlas_uv_for_material, rebuild_terrain_material_atlas, sync_terrain_material_atlas_from_registry,
    TerrainMaterialAtlasGpu, TerrainMaterialAtlasPlugin,
};
pub use super::terrain_instanced_draw::{
    register_terrain_instanced_draw, TerrainInstancedDrawGlobals, TerrainInstancedDrawPlugin,
    TerrainInstancedRenderHost, TerrainInstanceMap, TerrainTileInstance, TERRAIN_INSTANCED_WGSL,
};
pub use super::tile_world_fallback::{
    draw_simulation_minimap_egui, simulation_minimap_egui_texture, SimMinimapUiState,
    tile_raster_dirty_on_zoom_band_change_enabled, TileFallbackRasterPolicy,
    TileWorldFallbackAfterFireExtract, TileWorldFallbackChunkGrid, TileWorldFallbackPlugin,
    TileWorldFallbackRasterCtrl, TileWorldFallbackRasterDirty, TileWorldFallbackSprite,
    TileWorldFallbackState, RASTER_CHUNK_TILES,
};
pub use super::tactical_vector_overlay::{
    sync_tactical_vector_overlay_from_projection, tactical_vector_overlay_witness_json,
    TacticalVectorOverlayPlugin, TacticalVectorOverlayState,
};
pub use super::visual_perf_budget::{
    sync_tile_raster_spike_feedback_system, fire_extract_cadence_due, FireExtractCadence,
    FireExtractClock,
    FireExtractDirtyQueue, FireExtractInputFingerprint,
    FireExtractDiagnostics, FireExtractFrameReport,
    TileRasterBudget, TileRasterSpikeFeedback, RASTER_SPIKE_EMA_MS, RASTER_SPIKE_FRAME_MS,
};
pub use super::fire_chunk_entity_index::{
    bootstrap_chunk_fire_entity_index_if_empty, sync_chunk_fire_entity_index_added,
    sync_chunk_fire_entity_index_removed, ChunkFireEntityIndex,
};
pub use super::gpu_bind_group_registry::{
    buffer_binding_for, BindGroupBufferBinding, BindGroupId, GPUBindGroupEntry, GPUBindGroupRegistry,
    WEATHER_FIRE_FIELD_FIRE_BIND_GROUP, WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP,
    WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, WORLD_FIRE_PARTICLE_SPARK_BIND_GROUP,
};
pub use super::gpu_representation_metrics::GpuRepresentationMetrics;
pub use super::gpu_particle_draw::{
    register_world_fire_particle_draw, sync_particle_draw_dispatch_from_policy,
    FireParticlePipelineConfig, InstancedParticleExpandPipeline, WorldFireParticleDrawDispatch,
};
pub use super::gpu_spark_compute::{
    build_fire_spark_attractors, register_fire_spark_compute,
    FireSparkAttractors, SparkSimState, WorldFireSparkComputeDispatch,
};
pub use super::gpu_fire_particle_raster::{register_fire_particle_raster_draw, FireParticleDrawGlobals};
pub use super::gpu_water_surface_draw::{register_water_surface_draw, WaterOverlayDrawGlobals};
pub use super::gpu_water_particle_draw::{
    register_world_water_particle_draw, WorldWaterParticleDrawDispatch, WorldWaterParticleGpuStorage,
    WATER_PARTICLE_WGSL,
};
pub use super::gpu_water_particle_raster::{register_world_water_particle_raster, WaterParticleDrawGlobals};
pub use super::gpu_water_particles::{
    emit_world_water_particles_from_catalog, evaluate_water_vfx_witness_bands,
    update_world_water_particles_from_catalog, water_strategic_001_green,
    water_vfx_witness_json, water_w2_foam_001_green, water_witness_001_green,
    water_witness_foam_or_ocean_green, catalog_has_coast_ocean, catalog_has_river_bend,
    GpuWaterParticleInstance, GpuWaterParticleQuadVertex, GpuWaterParticlesPlugin,
    WaterParticleDensityScale, WaterParticleProfile, WaterParticleWitness,
    WaterVfxWitnessBands, WorldWaterParticleFrame, WATER_LAKE_GLINTS_PER_CHUNK,
    WATER_OCEAN_FOAM_PER_CHUNK, WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA,
    WATER_RIVER_FOAM_PER_CHUNK, WATER_RIVER_STREAKS_PER_CHUNK,
    WATER_TACTICAL_WITNESS_ZOOM_ALPHA,
};
pub use super::water_surface_visual::{
    apply_water_surface_overlay_subregion, sync_water_overlay_draw_frame, RiverPolylineSegment,
    water_strategic_001_shader_motion_green, water_strategic_witness_zoom_alpha,
    WaterMotionAnchor, WaterOverlayDrawFrame, WaterSurfaceKind, WaterSurfaceVisualCatalog,
    WaterSurfaceVisualPlugin, WaterSurfaceVisualSet, WATER_SURFACE_OVERLAY_WGSL,
    WATER_STRATEGIC_ZOOM_ALPHA,
};
pub use super::domain_overlay_gpu::{
    emit_domain_overlay_frame_from_projection, DomainOverlayGpuFrame, EcologyOverlayGpuRow,
    LogisticsOverlayGpuRow,
};
pub use super::extracted_camera_metrics::{
    particle_view_globals_from_metrics, sync_extracted_camera_metrics,
    sync_fire_particle_camera_scale, ExtractedCameraMetrics, ExtractedCameraMetricsPlugin,
    ExtractedCameraMetricsSet, FireParticleCameraScale,
};
pub use super::fire_vfx::{
    fire_spark_011_green, fire_spark_compute_enabled, FireSparkWitness,
    FIRE_SPARK_FULL_SCATTER_PX_PER_TILE, FIRE_SPARK_MIN_PX_PER_TILE,
    FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE, FIRE_SPARK_SCATTER_MAX,
    FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
};
pub use super::gpu_instanced_quad::{
    GpuInstancedQuadInstance, GpuInstancedQuadVertex, ParticleSystemUniforms, ParticleViewGlobals,
};
pub use super::gpu_particles::{
    emit_world_fire_particles_from_projection, seed_world_fire_particles_from_overlay_heat,
    update_world_fire_particles_from_projection,
    view_aware_particle_cull_wired, GpuParticleInstance,
    GpuParticleQuadVertex, ParticleClass, WorldFireParticleFrame, WorldFireParticleGpuStorage,
};
pub use super::stage5_full_app_harness::{
    LogE01CaptureLane, STAGE5_FULL_APP_LIVE_JSON,
};
pub use super::gpu_packed_formats::{
    ecology_overlay_row_format, fire_particle_expanded_vertex_format, fire_visual_instance_format,
    fire_particle_instance_format, heat_diffusion_cell_format, logistics_overlay_row_format,
    water_particle_expanded_vertex_format, water_particle_instance_format,
    packed_byte_size, LodBandBufferPolicy, PackedBufferFormat, PackedFormatId,
    ECOLOGY_OVERLAY_ROW_FORMAT, FIRE_PARTICLE_EXPANDED_VERTEX_FORMAT, FIRE_PARTICLE_INSTANCE_FORMAT,
    FIRE_VISUAL_INSTANCE_FORMAT, HEAT_DIFFUSION_CELL_FORMAT, HEAT_DIFFUSION_CELL_STRIDE,
    LOGISTICS_OVERLAY_ROW_FORMAT, WATER_PARTICLE_EXPANDED_VERTEX_FORMAT,
    WATER_PARTICLE_INSTANCE_FORMAT,
};
pub use super::gpu_buffer_registry::{
    row_capacity_bytes, BufferId, BufferVisibility, ECOLOGY_OVERLAY_BUFFER,
    FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER, FIRE_PARTICLE_INSTANCES_BUFFER,
    FIRE_SPARK_ATTRACTORS_BUFFER, FIRE_SPARK_STATE_BUFFER,
    WATER_PARTICLE_EXPANDED_VERTICES_BUFFER, WATER_PARTICLE_INSTANCES_BUFFER,
    FIRE_VISUAL_INSTANCES_BUFFER, GpuSlice, GPUBufferEntry, GPUBufferRegistry,
    HEAT_DIFFUSION_FIELD_BUFFER, LOGISTICS_OVERLAY_BUFFER, RegisteredBufferDescriptor,
    RegistryBufferAllocation, RegistryError, RegistryUploadStats,     TILE_DEBUG_INSTANCES_BUFFER, TERRAIN_INSTANCES_BUFFER,
};
pub use super::fire_smoke_shader_handles::{
    FireSmokeShaderHandles, FIRE_PARTICLE_WGSL, FIRE_SPARK_COMPUTE_WGSL, SMOKE_VOLUME_WGSL,
};
pub use super::fx_burst_request::{collect_burst_hints_from_fire_visual, FxParticleBurstRequest};
pub use super::visual_snapshot_commit::{commit_fire_visual_snapshot, CommittedVisualSnapshotFence};
pub use super::vt_spatial_invariants::{
    passes_vt5_spatial_invariants, sample_fire_row, vt5_spatial_eval_deferred,
    VT5_MIN_EVAL_FIRE_INSTANCES, VT5_MIN_MEAN_DISTANCE, VT5_MIN_OCCUPIED_CHUNKS,
    VT5_MIN_VARIANCE,
};
pub use super::logistics_visual_snapshot::LogisticsVisualSnapshot;
pub use super::ecology_visual_snapshot::EcologyVisualSnapshot;
pub use super::visual_domain_snapshots::{
    fill_logistics_snapshot, fill_minimap_unit_markers_from_logistics,
    publish_ecology_visual_snapshot, publish_logistics_visual_snapshot,
    publish_minimap_operational_unit_markers_system, seed_minimap_m2_logistics_construction_witness,
    seed_minimap_m2_overlay_witness, seed_minimap_m3_fow_ew_witness,
    seed_minimap_m3_units_replay_witness, unit_markers_real_reader_witness_green,
    MinimapOperationalSnapshot, MINIMAP_UNIT_MARKER_SNAPSHOT_CAP,
};
pub use super::phase_f_lod_proof::{PhaseFLodProofPlugin, PhaseFLodProofReport};
pub use super::gpu_surface_teardown::{GpuSurfaceTeardownPlugin, VisualTestGracefulExit};
pub use super::per_view_residency::{
    per_view_residency_contains, residency_coords_for_view_instance, PerViewResidencyConsumerWindow,
    RESIDENCY_VIEW_CHUNK_SPACING_WORLD,
};
pub use crate::dev::runtime_witness::stage6::{
    build_stage6_proof_payload, write_ops_f01_perf_attribution_section,
    Stage6LiveProofState, Stage6VirtualizationWitness, STAGE6_VIRTUALIZATION_JSON,
};
pub use super::view_fire_projection::{
    fire_frame_for_projection_graph, projection_fire_source_view,
};
pub use super::stage6_virtualization::{
    atlas_slots_from_gpu_path, chunk_in_residency_consumer_window, chunk_in_residency_table,
    clear_async_domain_apply_labels_after_stream_apply, gather_stage6_readiness,
    intersect_visible_chunks_with_residency_window, publish_stage6_virtualization_frame,
    stage6_readiness_passes, AsyncDomainApplyQueue, PagedAtlasResidency,
    ResidencyDrivenConsumerWindow, RuntimeAtlasSlot, Stage6ReadinessReport,
    Stage6VirtualizationFrame, Stage6VirtualizationPlugin,
};
pub use super::stage5_full_app_harness::probe_full_app_stage5_readiness;
pub use super::stage5_closure_witnesses::{
    FirePlaybackStabilityWitness, Stage5FireViewChunkWitness, Stage5LodBandLogWitness,
    Stage5MapCameraBridgeWitness,
};
pub use super::stage5_readiness::{
    evaluate_app_stage5_readiness, evaluate_stage5_spine_checklist, stage5_readiness_passes,
    AppStage5ReadinessReport, Stage5ReadinessEvalInvocation, Stage5ReadinessPlugin,
    Stage5ReadinessProfile, Stage5SpineChecklist, PERF_PLAY_READINESS_GREEN_LOG_INTERVAL,
};
pub use super::debug_render_trace::{
    trace_camera_sync, trace_particle_routing, trace_render_target, trace_viewport,
    DebugRenderTraceConfig, TRACE_CAMERA, TRACE_PARTICLES, TRACE_RENDER_TARGET, TRACE_VIEWPORT,
};
pub use super::visual_diagnostics::{
    visual_diag_enabled, visual_diag_plugin_latch, VisualDiagnosticsPlugin, VISUAL_DIAG_TARGET,
};
pub use super::debug_viewport_overlay::{debug_viewport_overlay_enabled, DebugViewportOverlayPlugin};
pub use super::viewport_pipeline::{
    primary_window_logical_presentable, resolved_particle_half_extents, ResolvedViewport,
    ResolvedViewports, ViewportPipelinePlugin, ViewportPipelineSet, ViewportPresentationMismatch,
    PRIMARY_WINDOW_MIN_LOGICAL_PX,
};
pub use super::view_runtime::ViewRuntimePlugin;
pub use super::full_render_diagnostic::{
    arm_full_render_diagnostic_for_full_app,
    full_render_diagnostic_has_critical_anomaly, full_render_diagnostic_plugin_latch,
    note_full_render_camera_drove_ui_follow,
    note_full_render_ui_wrote_map_camera, record_full_render_diagnostic_ui_rect,
    FullRenderDiagnosticFeedback, FullRenderDiagnosticGate, FullRenderDiagnosticPlugin,
    FullRenderDiagnosticSet, FullRenderDiagnosticSummary, FullRenderDiagnosticUiHook, FullRenderUiLayoutProbe,
    FullRenderUiSlot,
};
pub use super::gpu_weather_fire_field::{
    FireVisualGpuInstanceStorage, GpuWeatherFireFieldPlugin, WeatherFireFieldDebugOverlay,
    WeatherFireFieldUniforms,
};
pub use super::fire_chunk_runtime::{
    chunk_coords_above_visual_eps, fire_chunk_lod_state_from_simulation, fire_lod_band_for_visual_heat, sync_active_fire_chunk_set,
    sync_fire_chunk_lod_from_snapshot, ActiveFireChunkSet, ChunkCoord, FireChunk, FireChunkLodState,
    FireChunkRuntime, FireLodBand, FireSimulationSnapshot, VisibleFireChunkSet, FIRE_SIM_CHUNK_ACTIVE_EPS,
};
pub use super::fire7_f7_a_exit::{
    fire7_f7_a_exit_001_criteria, fire7_f7_a_exit_001_green, minimap_compositor_queries_fire_ecs,
    Fire7F7AExitCriteria,
};
pub use super::fire_streaming::{
    fire_streaming_b_green, FireStreamingLiveProofState,
    FireStreamingPlugin, FireStreamingWitness, FIRE_STREAMING_LIVE_JSON, FIRE_STREAMING_SLEEP_RADIUS,
};
pub use super::fire_view_extract::{
    build_fire_visual_frames_by_view, clamp_fire_lod_for_world_band, fire7_f7_c_001_green,
    fire_cap_for_world_band, fire_lod_designer_table_wired, sync_visible_fire_chunks_from_views,
    tactical_fire_visual, FireVisualFramesByView, FIRE_LOD_CAP_STRATEGIC, FIRE_LOD_CAP_TACTICAL,
    FIRE_VIEW_CHUNK_SPACING_WORLD,
};
pub use super::gpu_surface_teardown::visual_teardown_vr02_wired;
pub use super::vfx_capture_hook::{VfxCaptureHookPlugin, VfxCaptureHookState};
pub use super::sim_visual_extract::{
    ChunkFireHeat, ChunkSmokeGpu, ClimateVisualAggregate, FireEmitterGpu, FireVisualFrame,
    FireVisualGpuInstance, SimChunkSmokeVisualExtract, SimFireEmitterVisualExtract,
    FIRE_VISUAL_ACTIVE_HEAT_EPS,
};
pub use super::lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as FireLightEmissionSample,
    FireLightType, CLUSTER_MERGE_RADIUS,
};
pub use super::extraction::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, ExtractFrameSnapshot, FireAtmosphereAggregate,
    CLUSTERED_FIRE_INSTANCE_CAP, FireEmissionProfile, FireProjectionNode, FireVisualFramePlugin,
    FireVisualFrameSet, FireVisualProxy, ProjectionNodeTrait, RenderProjectionContext,
    RenderProjectionGraph, run_render_projection_graph,
};
pub use super::overlay_field_buffers::{
    chunk_fire_heat_maps_differ, chunk_fire_heat_maps_differ_eps, CHUNK_FIRE_HEAT_OVERLAY_EPS,
    CHUNK_FIRE_OVERLAY_DISPLAY_MIN, SharedOverlayFieldBuffers, SharedOverlayFieldBuffersPlugin,
};
pub use super::visual_agreement::{
    hash_chunk_fire_heat, hash_shared_overlay_heat, hash_simulation_chunk_heat_for_overlay,
    record_visual_agreement_frame,
    assert_snapshot_stamp, OverlayAgreementDebug, VisualAgreementError, VisualAgreementFrame,
    WorldPreviewVt4Probe,
};
pub use super::vt_ci_matrix::{
    apply_vt4_ci_report_to_overlay_debug, apply_vt4_ci_surface_checks, build_deterministic_ci_scenario,
    build_live_vt4_scenario, record_vt_ci_matrix_live, run_vt4_ci_matrix, run_vt5_ci_spatial_matrix,
    Vt4CiReport, Vt4CiScenario, Vt4SurfaceId, VtCiMatrixLiveReport, VtCiMatrixPlugin,
    full_app_vt_ci_fixture_passes, stage5_vt_deep_001_green, stage5_vt_flicker_visual_001_witness,
};
pub use super::domain_projection_frame::{
    build_domain_projection_frame, merge_domain_projection_into_representation,
    publish_domain_projection_frame, DomainProjectionFrame, DomainProjectionFramePlugin,
    DomainProjectionId, DomainProjectionSlice,
};
pub use super::frame_perf::{
    attrib_fire_build_view_after, attrib_fire_build_view_before, attrib_fire_particles_after,
    attrib_fire_particles_before, attrib_fire_pipeline_after, attrib_fire_pipeline_before,
    attrib_fire_project_after, attrib_fire_project_before, attrib_preview_cpu_raster_after,
    attrib_preview_cpu_raster_before, attrib_preview_gpu_present_after,
    attrib_preview_gpu_present_before, attrib_streaming_reconstruct_after,
    attrib_streaming_reconstruct_before, emit_frame_perf_summary, frame_perf_verbose,
    intra_update_stall_log, log_perf_phase, minimap_size_trace_enabled, record_frame_perf_ms,
    record_map_fit_sync_ms, record_tile_storage_apply_ms, record_viewport_sync_ms,
    reset_frame_perf_counters, scoped_ms, trace_minimap_size_writer,
    timed, timed_opt, FramePerf, FramePerfPlugin, FramePerfSlot, FrameUpdateAttrib, FrameWallClock,
    PerfScope,
};
pub use super::mig_a_static::{
    mig_a_static_bulk_bundle, mig_a_tactical_visibility_range,
    mig_a1_static_transform_optimizations_enabled, mig_a10_spine_dispatch_authority, MigAStaticBulk,
};
pub use super::stall_watch::{
    stall_checkpoint_before_world_repr, stall_checkpoint_post_fire_project, stall_checkpoint_post_streaming_spine,
    stall_checkpoint_post_world_repr, stall_watch_enabled, stall_watch_plugin_latch, FrameScheduleSpans, FrameStallWatch,
    FrameSubstageSpans, StallWatchPlugin, STALL_THRESHOLD_MS,
    stall_substage_fire_build_view, stall_substage_fire_commit, stall_substage_fire_emitter_sync,
    stall_substage_fire_pre_extract, stall_substage_fire_sim_snapshot, stall_substage_fire_sync_active, stall_substage_fire_sync_lod,
    stall_substage_fire_sync_overlay, stall_substage_fire_sync_visible, stall_substage_map_apply_input,
    stall_substage_map_derive, stall_substage_map_smooth, stall_substage_minimap_intent,
    stall_substage_repr_apply_result, stall_substage_repr_compute_frame, stall_substage_repr_decay_lod,
    stall_substage_repr_proc_extract, stall_substage_repr_refresh_lod, stall_substage_view_sync,
    stall_pre_egui,
};
pub use crate::dev::diagnostics::{
    perf_attribution_witness_json, perf_attribution_witness_lib_fixture,
    percentile_from_slice, reset_perf_attribution_witness_on_enter_simulation,
    sync_perf_attribution_witness_system, PerfAttributionWitness, PERF_ATTRIBUTION_WINDOW,
    reset_visual_readiness_witness_on_enter_simulation, sync_visual_readiness_witness_system,
    visual_readiness_witness_json, visual_readiness_witness_lib_fixture, VisualReadinessWitness,
    VisualReadinessWitnessPlugin, RenderScheduleHandoffMs, RenderScheduleSpans,
    RenderScheduleWitness,
};
pub use super::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlayOrderPlugin, CORE2D_OVERLAY_SDR_FORMAT,
};
pub use super::gpu_tile_debug_buffer::register_tile_debug_instance_storage_upload;
pub use super::gpu_tile_debug_draw::register_tile_debug_instanced_draw;
pub use super::gpu_indirect_draw::{
    apply_gpu_indirect_spine_dispatch_authority, compact_world_fire_indirect_draw,
    sync_world_fire_indirect_draw, GpuIndirectDrawSpine, GpuIndirectDrawSpinePlugin,
    GPU_INDIRECT_DISPATCH_WORKGROUP, WorldFireIndirectDrawArgs, WORLD_FIRE_VERTICES_PER_INSTANCE,
};
pub use super::minimap_compositor::{
    build_minimap_compositor_proof_payload, build_minimap_compositor_proof_payload_with_tray,
    minimap_gpu_compositor_env_enabled, ui_p3_m2_minimap_acceptance_green,
    ui_p3_m3_minimap_acceptance_green, MinimapGpuCompositorDiagnostics, MinimapCompositorPlugin,
    MinimapCompositorState, MinimapRenderTargetRegistry,
};
pub use super::light::*;
#[cfg(feature = "bevy_tilemap_adapter")]
pub use super::tilemap_adapter::{
    ChunkTilemaps, TilemapAdapterPlugin, TilemapLayerVisibility,
};
/// Deprecated GUI re-exports — import from `crate::gui` (RGR-M1-002/003).
pub mod deprecated_gui_shims {
    #[deprecated(
        since = "2026.07.04",
        note = "import from crate::gui::{MinimapShellState, MinimapOverlayMask, MinimapPresentationMode}"
    )]
    pub use crate::gui::{MinimapOverlayMask, MinimapPresentationMode, MinimapShellState};
}
