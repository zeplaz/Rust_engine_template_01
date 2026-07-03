//! Minimap compositor plugin — schedule wiring only.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::{on_visual_cadence_minimap, ViewRepresentationSystemSet};
use crate::render::publish_minimap_operational_unit_markers_system;
use crate::render::extraction::{FireVisualFrameSet, VegetationExtractFrameSet};
use crate::render::ViewportPipelineSet;

use super::composite::{MinimapCompositeDispatch, MinimapCompositeHeatTextures};
use super::gpu_compute::register_minimap_composite_gpu;
use super::pass::{
    apply_minimap_gpu_resize_request, bootstrap_minimap_gpu_render_target,
    commit_minimap_render_target_bind_system, queue_minimap_render_target_resize,
    run_minimap_compositor_pass, sync_minimap_presentation_source,
};
use super::render_target::{
    MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier, MinimapRenderTargetRegistry,
};
use super::diagnostics::MinimapGpuCompositorDiagnostics;
use super::pass::MinimapCompositorState;
use crate::dev::runtime_witness::minimap::{
    write_minimap_compositor_live_proof_system, MinimapCompositorLiveProofState,
};

pub struct MinimapCompositorPlugin;

impl Plugin for MinimapCompositorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapGpuResizeQueue>()
            .init_resource::<MinimapRenderTargetRegistry>()
            .init_resource::<MinimapRenderTargetBindBarrier>()
            .init_resource::<MinimapCompositorState>()
            .init_resource::<MinimapGpuCompositorDiagnostics>()
            .init_resource::<MinimapCompositorLiveProofState>()
            .init_resource::<MinimapCompositeHeatTextures>()
            .init_resource::<MinimapCompositeDispatch>()
            .init_resource::<crate::render::MinimapOperationalSnapshot>();
        register_minimap_composite_gpu(app);
        app.add_systems(
            OnEnter(BaseState::Simulation),
            bootstrap_minimap_gpu_render_target
                .after(crate::gui::hud::simulation_session::apply_simulation_map_presentation_defaults),
        )
        .add_systems(
            Update,
            (
                bootstrap_minimap_gpu_render_target,
                queue_minimap_render_target_resize,
                apply_minimap_gpu_resize_request,
                commit_minimap_render_target_bind_system,
            )
                .chain()
                .in_set(ViewRepresentationSystemSet::RenderTargets)
                .after(ViewportPipelineSet::Resolve),
        )
        .add_systems(
            Update,
            publish_minimap_operational_unit_markers_system
                .in_set(ViewRepresentationSystemSet::SyncOverlayField)
                .run_if(in_state(BaseState::Simulation)),
        )
        .add_systems(
            Update,
            (
                sync_minimap_presentation_source,
                run_minimap_compositor_pass,
            )
                .chain()
                .after(publish_minimap_operational_unit_markers_system)
                .after(ViewRepresentationSystemSet::SyncOverlayField)
                .after(FireVisualFrameSet::BuildProfiles)
                .after(VegetationExtractFrameSet::BuildProfiles)
                .after(crate::render::TileWorldFallbackAfterFireExtract)
                .in_set(ViewRepresentationSystemSet::WorldRender)
                .run_if(on_visual_cadence_minimap),
        )
        .add_systems(
            PostUpdate,
            write_minimap_compositor_live_proof_system.run_if(in_state(BaseState::Simulation)),
        );
    }
}
