//! Unit tests for minimap compositor (extracted from mod.rs — CLN-MOD-002).

static MINIMAP_GPU_COMPOSITOR_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use super::render_target::minimap_rgba_image;
use crate::gui::editor::world_preview::{
    PreviewPathAuthority, WorldPreviewRenderTargetRegistry, WorldPreviewTexture,
};
use crate::gui::{
    resolve_minimap_texture_source, resolve_world_preview_texture_source, MapViewInstances,
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
fn minimap_compositor_proof_payload_fields() {
    use super::diagnostics::MinimapGpuCompositorDiagnostics;
    use super::witness_collectors::build_minimap_compositor_proof_payload;
    use super::state::MinimapCompositorState;

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
fn ui_w3_m2_001_live_witness_refresh() {
    use super::{refresh_ui_w3_m2_001_live_witness, ui_w3_m2_001_green};

    assert!(refresh_ui_w3_m2_001_live_witness());
    let text = std::fs::read_to_string("debug_runs/minimap_compositor_live.json")
        .expect("witness json");
    let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
    assert_eq!(v["ui_w3_m2_001"]["green"], serde_json::json!(true));
    assert_eq!(v["logistics_rows"], serde_json::json!(2));
    assert_eq!(v["construction_rows"], serde_json::json!(18));
    assert_eq!(v["logistics_heat_enabled"], serde_json::json!(true));
    assert_eq!(v["construction_heat_enabled"], serde_json::json!(true));
    let tray = super::witness_collectors::witness_harness_tray();
    let compositor = super::witness_collectors::fixture_ui_oh_m2_001_compositor(&tray);
    assert!(ui_w3_m2_001_green(&compositor));
}



#[test]
fn ui_p3_m2_tray_opt_green_when_tray_matches_compositor() {
    use super::witness_collectors::ui_p3_m2_tray_opt_green;
    use super::state::MinimapCompositorState;

    let compositor = MinimapCompositorState {
        fire_heat_enabled: true,
        logistics_heat_enabled: true,
        construction_heat_enabled: true,
        ecology_heat_enabled: false,
        ..Default::default()
    };
    let mut tray = super::witness_collectors::witness_harness_tray();
    tray.fire_heat = true;
    tray.ecology_heat = false;
    assert!(ui_p3_m2_tray_opt_green(&compositor, &tray));
    tray.logistics_heat = false;
    assert!(!ui_p3_m2_tray_opt_green(&compositor, &tray));
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
fn ui_p3_m2_acceptance_green_when_m2_channels_populated() {
    use super::witness_collectors::ui_p3_m2_minimap_acceptance_green;
    use super::state::{MinimapCompositePath, MinimapCompositorState};

    let compositor = MinimapCompositorState {
        stamp: 2,
        logistics_rows: 2,
        construction_rows: 18,
        ecology_rows: 100,
        construction_heat_enabled: true,
        ecology_heat_enabled: true,
        logistics_heat_enabled: true,
        composite_path: MinimapCompositePath::GpuCompute,
        ..Default::default()
    };
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = Assets::<Image>::default();
    registry.committed_image = images.add(super::render_target::minimap_rgba_image(64, 64));
    let shell = MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    assert!(ui_p3_m2_minimap_acceptance_green(
        &compositor, &registry, &shell, None
    ));
}

#[test]
fn fow_ew_heat_upload_when_operational_snapshot_seeded() {
    use super::composite::{upload_minimap_heat_textures, MinimapCompositeHeatTextures};
    use crate::render::{seed_minimap_m3_fow_ew_witness, MinimapOperationalSnapshot};

    let mut images = Assets::<Image>::default();
    let mut heat = MinimapCompositeHeatTextures::default();
    let mut map_views = MapViewInstances::default();
    map_views.minimap.overlays = crate::gui::minimap_overlay_witness_harness();
    let fallback = TileWorldFallbackState {
        last_w: 64,
        last_h: 64,
        ..Default::default()
    };
    let mut operational = MinimapOperationalSnapshot::default();
    seed_minimap_m3_fow_ew_witness(&mut operational);
    use crate::render::seed_minimap_m3_units_replay_witness;
    use crate::systems::sim_frame_delta::CommittedSimReplayRing;

    seed_minimap_m3_units_replay_witness(&mut operational);
    let mut replay = CommittedSimReplayRing::with_capacity(8);
    replay.record_commit(crate::systems::sim_control::SimStepStamp::new(1, 0));
    replay.record_commit(crate::systems::sim_control::SimStepStamp::new(2, 0));
    let (ok, _, _, _, fow_rows, ew_rows, unit_rows, replay_on, _) = upload_minimap_heat_textures(
        &mut images,
        &mut heat,
        None,
        None,
        None,
        None,
        Some(&operational),
        None,
        Some(&replay),
        None,
        &map_views,
        &fallback,
        UVec2::new(32, 32),
    );
    assert!(ok);
    assert!(fow_rows > 0);
    assert!(ew_rows > 0);
    assert!(unit_rows > 0);
    assert!(replay_on);
}

#[test]
fn ui_p3_m4_001_fow_ew_green_when_enabled_and_rows() {
    use super::witness_collectors::ui_p3_m4_minimap_acceptance_green;
    use super::state::MinimapCompositorState;

    let off = MinimapCompositorState {
        fow_heat_enabled: true,
        ew_heat_enabled: true,
        fow_rows: 0,
        ew_rows: 0,
        ..Default::default()
    };
    assert!(!ui_p3_m4_minimap_acceptance_green(&off));

    let on = MinimapCompositorState {
        fow_heat_enabled: true,
        ew_heat_enabled: true,
        fow_rows: 16,
        ew_rows: 12,
        ..Default::default()
    };
    assert!(ui_p3_m4_minimap_acceptance_green(&on));
}

/// **UI-P3-M3-001** — ecology (and construction) heat enabled with snapshot rows.
#[test]
fn ui_p3_m3_001_ecology_heat_green_when_enabled_and_rows() {
    use super::witness_collectors::ui_p3_m3_minimap_acceptance_green;
    use super::state::MinimapCompositorState;

    let ecology_only = MinimapCompositorState {
        construction_heat_enabled: false,
        ecology_heat_enabled: true,
        ecology_rows: 42,
        ..Default::default()
    };
    assert!(!ui_p3_m3_minimap_acceptance_green(&ecology_only));

    let ecology_on = MinimapCompositorState {
        construction_heat_enabled: true,
        ecology_heat_enabled: true,
        ecology_rows: 100,
        construction_rows: 0,
        ..Default::default()
    };
    assert!(ui_p3_m3_minimap_acceptance_green(&ecology_on));
}

#[test]
fn ui_p3_m3_acceptance_green_when_construction_or_ecology_rows() {
    use super::witness_collectors::ui_p3_m3_minimap_acceptance_green;
    use super::state::MinimapCompositorState;

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
    use super::witness_collectors::ui_p3_001_minimap_acceptance_green;
    use super::state::{MinimapCompositePath, MinimapCompositorState};

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
