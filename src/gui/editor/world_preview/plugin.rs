//! WorldPreview plugin — resource init + raster / GPU lifecycle schedule glue.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::gui::ViewRepresentationSystemSet;
use crate::render::{
    attrib_preview_cpu_raster_after, attrib_preview_cpu_raster_before,
    attrib_preview_gpu_present_after, attrib_preview_gpu_present_before,
};

use super::cache::WorldPreviewChunkCaches;
use super::chrome::{
    world_preview_chrome_active, world_preview_pipeline_enabled, WorldPreviewUiState,
};
use super::composite_preview_graph::{self, CompositePreviewGraphResource};
use super::gpu_preview::{self, WorldPreviewGpuRuntime};
use super::preview_lifecycle::{
    self, advance_world_preview_lifecycle_system, WorldPreviewLifecycle, WorldPreviewLifecycleSignals,
};
use super::preview_readiness::{
    prime_world_preview_editor_camera, sync_world_preview_ready, WorldPreviewReady,
    WorldPreviewReadinessDiagnostics,
};
use super::preview_render_contract;
use super::preview_render_state::PreviewRenderState;
use super::preview_vt4;
use super::render_raster;
use super::render_target_barrier::{
    self, WorldPreviewGpuResizeQueue, WorldPreviewRenderTargetBindBarrier,
    WorldPreviewRenderTargetRegistry, WorldPreviewRenderViewportContract, WorldPreviewViewportEvent,
};
use super::texture_cache::{
    self, init_world_preview_texture, sync_world_preview_texture_size, WorldPreviewTexture,
};
use super::viewport_authority::{self, WorldPreviewViewportAuthority};
use super::wave_p_witness;
use super::window::display_world_preview;

/// Deferred GPU resize → commit → camera bind (`render_target_barrier` lifecycle).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum WorldPreviewLifecycleSet {
    ApplyGpuResize,
    CommitRenderTarget,
    BindGpuCamera,
    CameraTransform,
    ChunkQuads,
}

/// Ordering: resize → CPU raster → present swap (`base_visual_dev01_plan_status` § phase-d D-3).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum WorldPreviewRasterOrder {
    SyncTextureSize,
    RasterTiles,
    /// Runs after [`WorldPreviewRasterOrder::RasterTiles`] (CPU wrote `SwapImageBuffers::back`).
    PresentSwap,
}

/// Registers world preview resources + raster + chrome systems.
pub struct WorldPreviewPlugin;

fn seed_full_app_gpu_preview_authority_for_readiness(
    profile: Res<crate::render::Stage5ReadinessProfile>,
    mut gpu: ResMut<WorldPreviewGpuRuntime>,
) {
    if *profile == crate::render::Stage5ReadinessProfile::FULL_APP {
        gpu.offscreen_renderer_ready = true;
    }
}

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::render::Stage5ReadinessProfile::FULL_APP);
        app.init_resource::<CompositePreviewGraphResource>();
        preview_render_contract::init_preview_render_contract_resources(app);
        app.init_resource::<crate::gui::SwapImageBuffers>()
            .init_resource::<WorldPreviewTexture>()
            .init_resource::<gpu_preview::WorldPreviewGpuRuntime>()
            .init_resource::<crate::render::WorldPreviewVt4Probe>()
            .init_resource::<WorldPreviewUiState>()
            .add_message::<WorldPreviewViewportEvent>()
            .init_resource::<WorldPreviewViewportAuthority>()
            .init_resource::<WorldPreviewGpuResizeQueue>()
            .init_resource::<WorldPreviewRenderTargetRegistry>()
            .init_resource::<WorldPreviewRenderViewportContract>()
            .init_resource::<WorldPreviewRenderTargetBindBarrier>()
            .init_resource::<WorldPreviewChunkCaches>()
            .init_resource::<WorldPreviewLifecycle>()
            .init_resource::<WorldPreviewLifecycleSignals>()
            .init_resource::<WorldPreviewReady>()
            .init_resource::<WorldPreviewReadinessDiagnostics>()
            .init_resource::<PreviewRenderState>()
            .init_resource::<wave_p_witness::WavePLiveProofState>()
            .init_resource::<crate::gui::hud::ViewportRectSanity>()
            .configure_sets(
                Update,
                (
                    WorldPreviewRasterOrder::RasterTiles.after(WorldPreviewRasterOrder::SyncTextureSize),
                    WorldPreviewRasterOrder::PresentSwap.after(WorldPreviewRasterOrder::RasterTiles),
                    WorldPreviewLifecycleSet::ApplyGpuResize
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    WorldPreviewLifecycleSet::CommitRenderTarget
                        .after(WorldPreviewLifecycleSet::ApplyGpuResize),
                    WorldPreviewLifecycleSet::BindGpuCamera
                        .after(WorldPreviewLifecycleSet::CommitRenderTarget),
                    WorldPreviewLifecycleSet::CameraTransform
                        .after(WorldPreviewLifecycleSet::BindGpuCamera),
                    WorldPreviewLifecycleSet::ChunkQuads
                        .after(WorldPreviewLifecycleSet::CameraTransform),
                ),
            )
            .add_systems(Startup, init_world_preview_texture)
            .add_systems(
                Startup,
                gpu_preview::prefer_gpu_preview_mode_when_renderer_ready,
            )
            .add_systems(
                Startup,
                seed_full_app_gpu_preview_authority_for_readiness,
            )
            .add_systems(
                Update,
                (
                    viewport_authority::sync_world_preview_viewport_authority
                        .in_set(ViewRepresentationSystemSet::ResolveViewport)
                        .after(crate::render::ViewportPipelineSet::Resolve),
                    viewport_authority::queue_world_preview_gpu_resize_request
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    preview_render_contract::sync_preview_render_contract_after_egui
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    preview_render_contract::sync_preview_render_contract_system
                        .before(WorldPreviewRasterOrder::SyncTextureSize),
                    composite_preview_graph::sync_composite_preview_graph_resource
                        .after(preview_render_contract::sync_preview_render_contract_system),
                    preview_render_contract::sync_preview_path_authority
                        .after(composite_preview_graph::sync_composite_preview_graph_resource),
                    prime_world_preview_editor_camera
                        .in_set(ViewRepresentationSystemSet::ResolveViewport)
                        .after(crate::render::ViewportPipelineSet::Resolve),
                    sync_world_preview_texture_size.in_set(WorldPreviewRasterOrder::SyncTextureSize),
                    sync_world_preview_ready
                        .after(prime_world_preview_editor_camera)
                        .after(sync_world_preview_texture_size),
                ),
            )
            .add_systems(
                Update,
                (
                    attrib_preview_cpu_raster_before
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .after(sync_world_preview_ready)
                        .before(render_raster::update_world_preview_texture)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    render_raster::update_world_preview_texture
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    attrib_preview_cpu_raster_after
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .after(render_raster::update_world_preview_texture)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    texture_cache::present_world_preview_swap_after_raster
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    attrib_preview_gpu_present_before
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .before(texture_cache::present_world_preview_gpu_swap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    texture_cache::present_world_preview_gpu_swap
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    attrib_preview_gpu_present_after
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .after(texture_cache::present_world_preview_gpu_swap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    preview_render_contract::sync_preview_render_target_from_presentation
                        .after(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    advance_world_preview_lifecycle_system
                        .after(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(world_preview_pipeline_enabled),
                    preview_lifecycle::park_preview_lifecycle_when_chrome_dismissed
                        .after(advance_world_preview_lifecycle_system),
                    texture_cache::apply_world_preview_gpu_resize_request
                        .in_set(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::enforce_gpu_preview_pooled_swap
                        .before(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::seed_world_preview_render_target_registry
                        .after(gpu_preview::enforce_gpu_preview_pooled_swap)
                        .before(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::commit_world_preview_render_target
                        .in_set(WorldPreviewLifecycleSet::CommitRenderTarget)
                        .run_if(world_preview_pipeline_enabled),
                ),
            )
            .add_systems(
                Update,
                (
                    gpu_preview::sync_world_preview_offscreen_camera
                        .in_set(WorldPreviewLifecycleSet::BindGpuCamera)
                        .run_if(world_preview_pipeline_enabled),
                    render_target_barrier::sync_world_preview_render_viewport_contract
                        .in_set(ViewRepresentationSystemSet::RenderTargets)
                        .after(WorldPreviewLifecycleSet::BindGpuCamera)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::sync_world_preview_offscreen_camera_transform
                        .in_set(WorldPreviewLifecycleSet::CameraTransform)
                        .after(crate::gui::ViewRepresentationSystemSet::CameraSync)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::sync_world_preview_gpu_chunk_quads
                        .in_set(WorldPreviewLifecycleSet::ChunkQuads)
                        .run_if(world_preview_pipeline_enabled),
                    preview_vt4::capture_world_preview_vt4_probe
                        .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
                    viewport_authority::debug_trace_world_preview_viewport_authority
                        .in_set(ViewRepresentationSystemSet::PostFX),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                display_world_preview
                    .after(crate::gui::sync_shell_layout_drag_gate)
                    .in_set(ViewRepresentationSystemSet::UiCollect)
                    .run_if(world_preview_chrome_active),
            )
            .add_systems(
                Update,
                wave_p_witness::write_wave_p_witness_system
                    .run_if(crate::dev::runtime_witness::wave_p_live_proof_due)
                    .run_if(in_state(crate::engine::states::BaseState::Simulation)),
            );
    }
}
