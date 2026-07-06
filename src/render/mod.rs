// Rendering systems
pub mod extraction;
pub mod lighting;
pub mod shaders;
mod tile_world_fallback;
mod visual_perf_budget;
mod fire_vfx;
mod stage6_virtualization;
pub(crate) mod stage5_full_app_harness;
mod viewport_pipeline;
pub mod view_runtime;
mod mig_a_static;
pub mod minimap_compositor;

#[cfg(feature = "bevy_tilemap_adapter")]
pub mod tilemap_adapter;

// RGR-P5-001 — mechanical move: GPU raster/draw pipelines + render-thread perf probes.
mod pipelines;
pub(crate) mod probes;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: registries, formats, GPU lifetime.
mod core;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: fire/water VFX spine (renamed from plan's
// "fire_vfx" to "fx_spine" to avoid collision with the pre-existing render::fire_vfx frontend).
mod fx_spine;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: proof/witness/CI-matrix surfaces.
mod witness;

/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_fire_particle_raster`].
pub(crate) mod gpu_fire_particle_raster {
    pub use crate::render::pipelines::gpu_fire_particle_raster::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_indirect_draw`].
pub(crate) mod gpu_indirect_draw {
    pub use crate::render::pipelines::gpu_indirect_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_particle_draw`].
pub(crate) mod gpu_particle_draw {
    pub use crate::render::pipelines::gpu_particle_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_tile_debug_draw`].
pub(crate) mod gpu_tile_debug_draw {
    pub use crate::render::pipelines::gpu_tile_debug_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_water_particle_draw`].
pub(crate) mod gpu_water_particle_draw {
    pub use crate::render::pipelines::gpu_water_particle_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_water_particle_raster`].
pub(crate) mod gpu_water_particle_raster {
    pub use crate::render::pipelines::gpu_water_particle_raster::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_water_surface_draw`].
pub(crate) mod gpu_water_surface_draw {
    pub use crate::render::pipelines::gpu_water_surface_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::terrain_instanced_draw`].
pub(crate) mod terrain_instanced_draw {
    pub use crate::render::pipelines::terrain_instanced_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::frame_perf`].
pub(crate) mod frame_perf {
    pub use crate::render::probes::frame_perf::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::stall_watch`].
pub(crate) mod stall_watch {
    pub use crate::render::probes::stall_watch::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: GPU passes, draw/raster/compute, overlay draws → pipelines/.
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_spark_compute`].
pub(crate) mod gpu_spark_compute {
    pub use crate::render::pipelines::gpu_spark_compute::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_instanced_quad`].
pub(crate) mod gpu_instanced_quad {
    pub use crate::render::pipelines::gpu_instanced_quad::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_tile_debug_buffer`].
pub(crate) mod gpu_tile_debug_buffer {
    pub use crate::render::pipelines::gpu_tile_debug_buffer::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_weather_fire_field`].
pub mod gpu_weather_fire_field {
    pub use crate::render::pipelines::gpu_weather_fire_field::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::overlay_field_buffers`].
pub(crate) mod overlay_field_buffers {
    pub use crate::render::pipelines::overlay_field_buffers::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::power_map_overlay_draw`].
pub(crate) mod power_map_overlay_draw {
    pub use crate::render::pipelines::power_map_overlay_draw::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::domain_overlay_gpu`].
pub(crate) mod domain_overlay_gpu {
    pub use crate::render::pipelines::domain_overlay_gpu::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::atmosphere_partial_gpu`].
pub(crate) mod atmosphere_partial_gpu {
    pub use crate::render::pipelines::atmosphere_partial_gpu::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_particles`].
pub(crate) mod gpu_particles {
    pub use crate::render::pipelines::gpu_particles::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::gpu_water_particles`].
pub(crate) mod gpu_water_particles {
    pub use crate::render::pipelines::gpu_water_particles::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::tactical_vector_overlay`].
pub(crate) mod tactical_vector_overlay {
    pub use crate::render::pipelines::tactical_vector_overlay::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::infrastructure_overlay`].
pub(crate) mod infrastructure_overlay {
    pub use crate::render::pipelines::infrastructure_overlay::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::light`].
pub(crate) mod light {
    pub use crate::render::pipelines::light::*;
}
/// Path-preserving shim — contents now live at [`crate::render::pipelines::core2d_overlay_order`].
pub(crate) mod core2d_overlay_order {
    pub use crate::render::pipelines::core2d_overlay_order::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: registries, formats, GPU lifetime → core/.
/// Path-preserving shim — contents now live at [`crate::render::core::gpu_buffer_registry`].
pub(crate) mod gpu_buffer_registry {
    pub use crate::render::core::gpu_buffer_registry::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::gpu_bind_group_registry`].
pub(crate) mod gpu_bind_group_registry {
    pub use crate::render::core::gpu_bind_group_registry::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::gpu_packed_formats`].
pub(crate) mod gpu_packed_formats {
    pub use crate::render::core::gpu_packed_formats::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::gpu_surface_teardown`].
pub(crate) mod gpu_surface_teardown {
    pub use crate::render::core::gpu_surface_teardown::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::gpu_representation_metrics`].
pub(crate) mod gpu_representation_metrics {
    pub use crate::render::core::gpu_representation_metrics::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::terrain_material_atlas`].
pub(crate) mod terrain_material_atlas {
    pub use crate::render::core::terrain_material_atlas::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::terrain_render_authority`].
pub(crate) mod terrain_render_authority {
    pub use crate::render::core::terrain_render_authority::*;
}
/// Path-preserving shim — contents now live at [`crate::render::core::per_view_residency`].
pub(crate) mod per_view_residency {
    pub use crate::render::core::per_view_residency::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: extract-side snapshots/metrics → extraction/.
/// Path-preserving shim — contents now live at [`crate::render::extraction::sim_visual_extract`].
pub mod sim_visual_extract {
    pub use crate::render::extraction::sim_visual_extract::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::fire_view_extract`].
pub(crate) mod fire_view_extract {
    pub use crate::render::extraction::fire_view_extract::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::extracted_camera_metrics`].
pub(crate) mod extracted_camera_metrics {
    pub use crate::render::extraction::extracted_camera_metrics::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::ecology_visual_snapshot`].
pub(crate) mod ecology_visual_snapshot {
    pub use crate::render::extraction::ecology_visual_snapshot::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::logistics_visual_snapshot`].
pub(crate) mod logistics_visual_snapshot {
    pub use crate::render::extraction::logistics_visual_snapshot::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::visual_domain_snapshots`].
pub(crate) mod visual_domain_snapshots {
    pub use crate::render::extraction::visual_domain_snapshots::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::visual_snapshot_commit`].
pub(crate) mod visual_snapshot_commit {
    pub use crate::render::extraction::visual_snapshot_commit::*;
}
/// Path-preserving shim — contents now live at [`crate::render::extraction::domain_projection_frame`].
pub(crate) mod domain_projection_frame {
    pub use crate::render::extraction::domain_projection_frame::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: fire/water VFX spine → fx_spine/.
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fire_chunk_entity_index`].
pub(crate) mod fire_chunk_entity_index {
    pub use crate::render::fx_spine::fire_chunk_entity_index::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fire_chunk_runtime`].
pub(crate) mod fire_chunk_runtime {
    pub use crate::render::fx_spine::fire_chunk_runtime::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fire_streaming`].
pub(crate) mod fire_streaming {
    pub use crate::render::fx_spine::fire_streaming::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fire_smoke_shader_handles`].
pub(crate) mod fire_smoke_shader_handles {
    pub use crate::render::fx_spine::fire_smoke_shader_handles::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fire7_f7_a_exit`].
pub(crate) mod fire7_f7_a_exit {
    pub use crate::render::fx_spine::fire7_f7_a_exit::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::view_fire_projection`].
pub(crate) mod view_fire_projection {
    pub use crate::render::fx_spine::view_fire_projection::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::fx_burst_request`].
pub(crate) mod fx_burst_request {
    pub use crate::render::fx_spine::fx_burst_request::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::hanabi_embellishment`].
pub mod hanabi_embellishment {
    pub use crate::render::fx_spine::hanabi_embellishment::*;
}
/// Path-preserving shim — contents now live at [`crate::render::fx_spine::water_surface_visual`].
pub(crate) mod water_surface_visual {
    pub use crate::render::fx_spine::water_surface_visual::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: debug/diagnostic surfaces → probes/.
/// Path-preserving shim — contents now live at [`crate::render::probes::debug_render_trace`].
pub(crate) mod debug_render_trace {
    pub use crate::render::probes::debug_render_trace::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::debug_viewport_overlay`].
pub(crate) mod debug_viewport_overlay {
    pub use crate::render::probes::debug_viewport_overlay::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::visual_diagnostics`].
pub(crate) mod visual_diagnostics {
    pub use crate::render::probes::visual_diagnostics::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::full_render_diagnostic`].
pub(crate) mod full_render_diagnostic {
    pub use crate::render::probes::full_render_diagnostic::*;
}
/// Path-preserving shim — contents now live at [`crate::render::probes::vfx_capture_hook`].
pub(crate) mod vfx_capture_hook {
    pub use crate::render::probes::vfx_capture_hook::*;
}

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: proof/witness/CI-matrix surfaces → witness/.
/// Path-preserving shim — contents now live at [`crate::render::witness::stage5_closure_witnesses`].
pub(crate) mod stage5_closure_witnesses {
    pub use crate::render::witness::stage5_closure_witnesses::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::stage5_readiness`].
pub(crate) mod stage5_readiness {
    pub use crate::render::witness::stage5_readiness::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::hanabi_witness`].
pub mod hanabi_witness {
    pub use crate::render::witness::hanabi_witness::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::visual_agreement`].
pub(crate) mod visual_agreement {
    pub use crate::render::witness::visual_agreement::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::phase_f_lod_proof`].
pub(crate) mod phase_f_lod_proof {
    pub use crate::render::witness::phase_f_lod_proof::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::vt_ci_matrix`].
pub(crate) mod vt_ci_matrix {
    pub use crate::render::witness::vt_ci_matrix::*;
}
/// Path-preserving shim — contents now live at [`crate::render::witness::vt_spatial_invariants`].
pub(crate) mod vt_spatial_invariants {
    pub use crate::render::witness::vt_spatial_invariants::*;
}

pub mod api;
pub use api::*;
