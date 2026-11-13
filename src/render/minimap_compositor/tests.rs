//! Unit tests for minimap compositor (extracted from mod.rs — CLN-MOD-002).

static MINIMAP_GPU_COMPOSITOR_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
fn minimap_terrain_source_label_reflects_bound_world_texture() {
    use super::MinimapTerrainSourceBind;
    assert_eq!(
        super::minimap_terrain_source_label(MinimapTerrainSourceBind::WorldRaster),
        "world_raster"
    );
    assert_eq!(
        super::minimap_terrain_source_label(MinimapTerrainSourceBind::None),
        "none"
    );
    assert_eq!(
        super::minimap_terrain_source_label(MinimapTerrainSourceBind::GpuBake),
        "gpu_bake"
    );
}

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
fn gpu_compositor_env_default_on_when_unset() {
    let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
        .lock()
        .expect("MINIMAP_GPU_COMPOSITOR env tests");
    let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
    std::env::remove_var("MINIMAP_GPU_COMPOSITOR");
    assert!(super::state::minimap_gpu_compositor_env_enabled());
    match prior {
        Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
        None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
    }
}

#[test]
fn gpu_compositor_env_cpu_opt_out() {
    let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
        .lock()
        .expect("MINIMAP_GPU_COMPOSITOR env tests");
    let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
    std::env::set_var("MINIMAP_GPU_COMPOSITOR", "0");
    assert!(!super::state::minimap_gpu_compositor_env_enabled());
    match prior {
        Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
        None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
    }
}


/// **RPC-0-001** — runtime predicate must go false when the shader-failed atomic is set,
/// independent of the env toggle, so a lying `composite_path` witness cannot occur.
#[test]
fn gpu_compositor_runtime_enabled_false_on_shader_failure() {
    let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
        .lock()
        .expect("MINIMAP_GPU_COMPOSITOR env tests");
    let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
    std::env::remove_var("MINIMAP_GPU_COMPOSITOR");

    let was_failed = super::diagnostics::MINIMAP_GPU_SHADER_FAILED
        .swap(false, std::sync::atomic::Ordering::Relaxed);
    assert!(super::state::minimap_gpu_compositor_runtime_enabled());

    super::diagnostics::MINIMAP_GPU_SHADER_FAILED
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(!super::state::minimap_gpu_compositor_runtime_enabled());
    // env-only predicate stays true — this is exactly the mismatch RPC-0-001 fixes.
    assert!(super::state::minimap_gpu_compositor_env_enabled());

    super::diagnostics::MINIMAP_GPU_SHADER_FAILED
        .store(was_failed, std::sync::atomic::Ordering::Relaxed);
    match prior {
        Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
        None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
    }
}

/// **RPC-0-001** — `run_minimap_compositor_pass` must flip `composite_path` to `CpuBridge`
/// the same frame the shader-failed atomic is observed, matching what
/// `sync_minimap_presentation_source` already does for `MinimapShellState`.
#[test]
fn composite_path_flips_to_cpu_bridge_on_shader_failure() {
    let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
        .lock()
        .expect("MINIMAP_GPU_COMPOSITOR env tests");
    let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
    std::env::remove_var("MINIMAP_GPU_COMPOSITOR");
    let was_failed = super::diagnostics::MINIMAP_GPU_SHADER_FAILED
        .swap(true, std::sync::atomic::Ordering::Relaxed);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<MinimapShellState>();
    app.init_resource::<MinimapRenderTargetRegistry>();
    app.init_resource::<super::MinimapCompositorState>();
    app.init_resource::<super::diagnostics::MinimapGpuCompositorDiagnostics>();
    app.init_resource::<crate::render::TileWorldFallbackState>();
    app.init_resource::<super::composite::MinimapCompositeDispatch>();
    app.init_resource::<super::composite::MinimapCompositeHeatTextures>();
    app.init_resource::<crate::gui::MapViewInstances>();
    app.init_resource::<crate::render::ResolvedViewports>();
    app.init_resource::<crate::gui::VisualCadence>();
    app.init_resource::<Assets<Image>>();
    {
        let mut shell = app.world_mut().resource_mut::<MinimapShellState>();
        shell.visible = true;
        shell.minimized = false;
    }
    {
        let mut compositor = app.world_mut().resource_mut::<super::MinimapCompositorState>();
        compositor.stamp = 7;
        compositor.composite_path = super::MinimapCompositePath::GpuCompute;
    }
    app.add_systems(Update, super::pass::run_minimap_compositor_pass);
    app.update();

    let compositor = app.world().resource::<super::MinimapCompositorState>();
    assert_eq!(compositor.composite_path, super::MinimapCompositePath::CpuBridge);

    super::diagnostics::MINIMAP_GPU_SHADER_FAILED
        .store(was_failed, std::sync::atomic::Ordering::Relaxed);
    match prior {
        Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
        None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
    }
}
