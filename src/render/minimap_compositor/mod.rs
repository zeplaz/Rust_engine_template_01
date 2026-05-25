//! UX-E01 M1 — dedicated minimap GPU compositor RT (separate from world-preview authority).

mod composite;
mod diagnostics;
mod gpu_compute;
mod live_proof;
mod pass;
mod render_target;

pub use diagnostics::{
    diagnostics_json_snapshot, minimap_gpu_debug_logging_enabled, MinimapGpuCompositorDiagnostics,
};

pub use live_proof::{
    build_minimap_compositor_proof_payload, commit_minimap_compositor_live_proof,
    write_minimap_compositor_live_proof_system, MinimapCompositorLiveProofState,
};
pub use pass::{
    apply_minimap_gpu_resize_request, commit_minimap_render_target_bind_system,
    minimap_gpu_compositor_env_enabled, queue_minimap_render_target_resize,
    run_minimap_compositor_pass, sync_minimap_presentation_source, MinimapCompositePath,
    MinimapCompositorState,
};
pub use render_target::{
    committed_minimap_render_target_handle, try_commit_minimap_render_target,
    MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier, MinimapRenderTargetRegistry,
};

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::{on_visual_cadence_minimap, ViewRepresentationSystemSet};
use crate::render::extraction::FireVisualFrameSet;
use crate::render::ViewportPipelineSet;

use composite::{MinimapCompositeDispatch, MinimapCompositeHeatTextures};
use gpu_compute::register_minimap_composite_gpu;

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
            .init_resource::<MinimapCompositeDispatch>();
        register_minimap_composite_gpu(app);
        app.add_systems(
            Update,
            (
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
            (
                sync_minimap_presentation_source,
                run_minimap_compositor_pass,
            )
                .chain()
                .after(ViewRepresentationSystemSet::SyncOverlayField)
                .after(FireVisualFrameSet::BuildProfiles)
                .in_set(ViewRepresentationSystemSet::WorldRender)
                .run_if(on_visual_cadence_minimap),
        )
        .add_systems(
            PostUpdate,
            write_minimap_compositor_live_proof_system.run_if(in_state(BaseState::Simulation)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::render_target::minimap_rgba_image;
    use crate::gui::editor::world_preview::{
        PreviewPathAuthority, WorldPreviewRenderTargetRegistry, WorldPreviewTexture,
    };
    use crate::gui::{
        resolve_minimap_texture_source, resolve_world_preview_texture_source,
        MinimapPresentationSource, MinimapShellState,
    };
    use crate::render::{MinimapRenderTargetRegistry, TileWorldFallbackState};
    use bevy::prelude::*;

    #[test]
    fn minimap_and_preview_handles_differ_when_both_allocated() {
        let mut preview = WorldPreviewRenderTargetRegistry::default();
        let mut minimap = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        preview.committed_image = images.add(minimap_rgba_image(128, 128));
        preview.revision = 1;
        minimap.committed_image = images.add(minimap_rgba_image(64, 64));
        minimap.revision = 1;

        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let fallback = TileWorldFallbackState::default();
        let preview_src = resolve_world_preview_texture_source(
            &PreviewPathAuthority::default(),
            &preview,
            &WorldPreviewTexture::default(),
        );
        let minimap_src = resolve_minimap_texture_source(&shell, &fallback, &minimap);
        assert_ne!(preview_src.handle(), minimap_src.handle());
    }

    #[test]
    fn minimap_compositor_proof_payload_fields() {
        use super::diagnostics::MinimapGpuCompositorDiagnostics;
        use super::live_proof::build_minimap_compositor_proof_payload;
        use super::pass::MinimapCompositorState;

        let compositor = MinimapCompositorState {
            stamp: 3,
            compositor_revision: 2,
            dual_minimap_present: false,
            extent_match_px: 0.0,
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        registry.committed_size = UVec2::new(128, 128);
        registry.revision = 2;
        registry.committed_image = Handle::default();
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let diagnostics = MinimapGpuCompositorDiagnostics::default();
        let payload =
            build_minimap_compositor_proof_payload(&compositor, &registry, &shell, 7, false, &diagnostics);
        assert_eq!(payload["stamp"], serde_json::json!(3));
        assert_eq!(payload["compositor_revision"], serde_json::json!(2));
        assert_eq!(payload["dual_minimap_present"], serde_json::json!(false));
        assert_eq!(payload["overlay_revision"], serde_json::json!(7));
    }

    #[test]
    fn minimap_compositor_live_witness_refresh() {
        use super::diagnostics::MinimapGpuCompositorDiagnostics;
        use super::live_proof::commit_minimap_compositor_live_proof;
        use super::pass::MinimapCompositorState;

        let compositor = MinimapCompositorState {
            stamp: 4,
            compositor_revision: 2,
            dual_minimap_present: false,
            extent_match_px: 0.0,
            logistics_rows: 2,
            fire_heat_enabled: true,
            logistics_heat_enabled: true,
            composite_path: super::pass::MinimapCompositePath::GpuCompute,
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        registry.committed_size = UVec2::new(128, 128);
        registry.revision = 2;
        registry.committed_image = images.add(super::render_target::minimap_rgba_image(128, 128));
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let diagnostics = MinimapGpuCompositorDiagnostics::default();
        assert!(commit_minimap_compositor_live_proof(
            &compositor,
            &registry,
            &shell,
            7,
            false,
            &diagnostics,
        ));
        let text = std::fs::read_to_string("debug_runs/minimap_compositor_live.json")
            .expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v["composite_ok"], serde_json::json!(true));
        assert_eq!(v["dual_minimap_present"], serde_json::json!(false));
        assert_eq!(
            v["presentation_source"],
            serde_json::json!("SharedRenderTargetImage")
        );
        assert_eq!(
            v["gpu_compositor_env"],
            serde_json::json!(super::pass::minimap_gpu_compositor_env_enabled())
        );
        assert_eq!(v["logistics_rows"], serde_json::json!(2));
        assert_eq!(v["logistics_heat_enabled"], serde_json::json!(true));
        assert_eq!(v["composite_path"], serde_json::json!("GpuCompute"));
        assert_eq!(v["ui_p3_001_green"], serde_json::json!(true));
    }

    #[test]
    fn composite_fingerprint_skips_identical_inputs() {
        use super::diagnostics::{composite_fingerprint, MinimapGpuCompositorDiagnostics};
        use bevy::prelude::*;

        let terrain = Handle::<Image>::default();
        let a = composite_fingerprint(&terrain, 1, 2, 3, 4, 5, 6, true, true, true, true);
        let b = composite_fingerprint(&terrain, 1, 2, 3, 4, 5, 6, true, true, true, true);
        assert_eq!(a, b);
        let c = composite_fingerprint(&terrain, 2, 2, 3, 4, 5, 6, true, true, true, true);
        assert_ne!(a, c);

        let mut diag = MinimapGpuCompositorDiagnostics::default();
        diag.record_skip(super::diagnostics::MinimapGpuSkipReason::NoChange);
        assert_eq!(diag.skips_no_change, 1);
    }

    #[test]
    fn dispatch_has_commit_tracks_stamp() {
        use super::composite::MinimapCompositeDispatch;
        let mut dispatch = MinimapCompositeDispatch::default();
        assert!(!dispatch.has_commit());
        dispatch.commit_stamp = 5;
        assert!(dispatch.has_commit());
    }

    #[test]
    fn ui_p3_m3_acceptance_green_when_construction_or_ecology_rows() {
        use super::live_proof::ui_p3_m3_minimap_acceptance_green;
        use super::pass::MinimapCompositorState;

        let compositor = MinimapCompositorState {
            construction_heat_enabled: true,
            ecology_heat_enabled: true,
            construction_rows: 1,
            ecology_rows: 0,
            ..Default::default()
        };
        assert!(ui_p3_m3_minimap_acceptance_green(&compositor));

        let ecology_only = MinimapCompositorState {
            construction_rows: 0,
            ecology_rows: 3,
            ..compositor
        };
        assert!(ui_p3_m3_minimap_acceptance_green(&ecology_only));

        let off = MinimapCompositorState {
            construction_heat_enabled: false,
            ..compositor
        };
        assert!(!ui_p3_m3_minimap_acceptance_green(&off));
    }

    #[test]
    fn ui_p3_001_acceptance_green_when_gpu_composite_stable() {
        use super::live_proof::ui_p3_001_minimap_acceptance_green;
        use super::pass::{MinimapCompositePath, MinimapCompositorState};

        let compositor = MinimapCompositorState {
            stamp: 4,
            dual_minimap_present: false,
            extent_match_px: 0.0,
            composite_path: MinimapCompositePath::GpuCompute,
            logistics_rows: 2,
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        registry.committed_image = images.add(super::render_target::minimap_rgba_image(64, 64));
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        assert!(ui_p3_001_minimap_acceptance_green(
            &compositor, &registry, &shell
        ));
    }

    #[test]
    fn gpu_compositor_env_default_on_when_unset() {
        let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
        std::env::remove_var("MINIMAP_GPU_COMPOSITOR");
        assert!(super::pass::minimap_gpu_compositor_env_enabled());
        match prior {
            Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
            None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
        }
    }

    #[test]
    fn gpu_compositor_env_cpu_opt_out() {
        let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
        std::env::set_var("MINIMAP_GPU_COMPOSITOR", "0");
        assert!(!super::pass::minimap_gpu_compositor_env_enabled());
        match prior {
            Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
            None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
        }
    }
}
