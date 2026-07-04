//! UX-E01 M1 — dedicated minimap GPU compositor RT (separate from world-preview authority).

mod witness_collectors;
mod composite;
mod diagnostics;
mod gpu_compute;
mod pass;
mod plugin;
mod render_target;

pub use diagnostics::{
    diagnostics_json_snapshot, minimap_gpu_debug_logging_enabled, MinimapGpuCompositorDiagnostics,
    MinimapGpuDispatchReason, MinimapGpuSkipReason,
};

pub use witness_collectors::{
    build_minimap_compositor_proof_payload, build_minimap_compositor_proof_payload_with_tray,
    fixture_ui_oh_m2_001_compositor, fixture_ui_w3_m3_001_compositor, witness_harness_tray,
    ui_oh_m2_001_green, ui_oh_m3_001_green, ui_w3_m3_001_green, ui_w3_m3_001_operational_green,
    ui_w3_m3_001_stage7_operational_green, ui_p3_m2_minimap_acceptance_green,
    ui_p3_m2_tray_opt_green, ui_p3_m3_minimap_acceptance_green, ui_p3_m3_replay_001_green,
    ui_p3_m3_units_001_green, ui_p3_m4_minimap_acceptance_green, ui_p3_001_minimap_acceptance_green,
    ui_w3_m2_001_green,
};
pub use crate::dev::runtime_witness::minimap::{
    commit_minimap_compositor_live_proof,
    refresh_perf_vis_p1b_gpu_default_live_witness, refresh_ui_oh_m2_001_live_witness,
    refresh_ui_w3_m2_001_live_witness, refresh_ui_w3_m3_001_live_witness,
    refresh_ui_w3_m3_001_stage7_operational_witness, write_minimap_compositor_live_proof_system,
    MinimapCompositorLiveProofState,
};
pub use pass::{
    apply_minimap_gpu_resize_request, bootstrap_minimap_gpu_render_target,
    commit_minimap_render_target_bind_system,
    minimap_gpu_compositor_default_on_unset, minimap_gpu_compositor_env_enabled,
    minimap_terrain_source_label,
    perf_vis_p1b_gpu_default_001_green, queue_minimap_render_target_resize,
    run_minimap_compositor_pass, sync_minimap_presentation_source,
    minimap_gpu_compositor_runtime_enabled, MinimapCompositePath,
    MinimapCompositorState,
};
pub use render_target::{
    committed_minimap_render_target_handle, minimap_rgba_image, try_commit_minimap_render_target,
    MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier, MinimapRenderTargetRegistry,
};
pub use plugin::MinimapCompositorPlugin;

#[cfg(test)]
mod tests {
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
    fn minimap_terrain_source_label_matches_gpu_authority() {
        use crate::render::TerrainRenderAuthority;
        assert_eq!(
            super::minimap_terrain_source_label(TerrainRenderAuthority::GpuInstancedAtlas),
            "gpu_atlas"
        );
        assert_eq!(
            super::minimap_terrain_source_label(TerrainRenderAuthority::CpuFallback),
            "cpu_fallback"
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
    fn ui_w3_m2_001_live_witness_refresh() {
        use super::{refresh_ui_w3_m2_001_live_witness, ui_w3_m2_001_green};
        use crate::gui::hud::HudOverlayTrayState;

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
    fn ui_oh_m2_001_live_witness_refresh() {
        use super::{refresh_ui_oh_m2_001_live_witness, ui_oh_m2_001_green};
        use crate::gui::hud::HudOverlayTrayState;

        assert!(refresh_ui_oh_m2_001_live_witness());
        let text = std::fs::read_to_string("debug_runs/minimap_compositor_live.json")
            .expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v["ui_oh_m2_001"]["green"], serde_json::json!(true));
        assert_eq!(v["ui_w3_m2_001"]["green"], serde_json::json!(true));
        assert_eq!(v["logistics_rows"], serde_json::json!(2));
        assert_eq!(v["construction_rows"], serde_json::json!(18));
        assert_eq!(v["logistics_heat_enabled"], serde_json::json!(true));
        assert_eq!(v["construction_heat_enabled"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m2_green"], serde_json::json!(true));
        assert_eq!(v["composite_path"], serde_json::json!("GpuCompute"));
        let tray = super::witness_collectors::witness_harness_tray();
        let compositor = super::witness_collectors::fixture_ui_oh_m2_001_compositor(&tray);
        assert!(ui_oh_m2_001_green(&compositor));
    }

    #[test]
    fn minimap_compositor_live_witness_refresh() {
        use super::diagnostics::MinimapGpuCompositorDiagnostics;
        use crate::dev::runtime_witness::commit_minimap_compositor_live_proof;
        use super::pass::MinimapCompositorState;
        use crate::gui::hud::HudOverlayTrayState;

        let tray = super::witness_collectors::witness_harness_tray();
        let compositor = MinimapCompositorState {
            stamp: 4,
            compositor_revision: 2,
            dual_minimap_present: false,
            extent_match_px: 0.0,
            logistics_rows: 2,
            construction_rows: 18,
            ecology_rows: 100,
            fow_rows: 16,
            ew_rows: 12,
            fire_heat_enabled: tray.fire_heat,
            logistics_heat_enabled: tray.logistics_heat,
            construction_heat_enabled: tray.construction_heat,
            ecology_heat_enabled: tray.ecology_heat,
            fow_heat_enabled: true,
            ew_heat_enabled: true,
            units_heat_enabled: true,
            unit_marker_rows: 6,
            replay_scrub_enabled: true,
            veg_burn_rows: 1,
            burn_overrides_topology: true,
            veg_extract_revision: 1,
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
            Some(&tray),
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
        assert_eq!(v["ui_p3_m2_tray_opt_green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m2_green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m3_green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m4_green"], serde_json::json!(true));
        assert_eq!(v["fow_enabled"], serde_json::json!(true));
        assert_eq!(v["ew_overlay_enabled"], serde_json::json!(true));
        assert_eq!(v["veg_burn_rows"], serde_json::json!(1));
        assert_eq!(v["burn_overrides_topology"], serde_json::json!(true));
        assert_eq!(v["veg_minimap_burn_merge_green"], serde_json::json!(true));
        assert_eq!(v["fow_rows"], serde_json::json!(16));
        assert_eq!(v["ew_rows"], serde_json::json!(12));
        assert_eq!(v["construction_rows"], serde_json::json!(18));
        assert_eq!(v["ecology_rows"], serde_json::json!(100));
        assert_eq!(v["unit_marker_rows"], serde_json::json!(6));
        assert_eq!(v["replay_scrub_enabled"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m3_units_001_green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m3_replay_001_green"], serde_json::json!(true));
    }

    #[test]
    fn ui_p3_m2_tray_opt_green_when_tray_matches_compositor() {
        use super::witness_collectors::ui_p3_m2_tray_opt_green;
        use super::pass::MinimapCompositorState;
        use crate::gui::hud::HudOverlayTrayState;

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
        use super::pass::{MinimapCompositePath, MinimapCompositorState};

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
        use super::pass::MinimapCompositorState;

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
        use super::pass::MinimapCompositorState;

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
        use super::witness_collectors::ui_p3_001_minimap_acceptance_green;
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
        let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
            .lock()
            .expect("MINIMAP_GPU_COMPOSITOR env tests");
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
        let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
            .lock()
            .expect("MINIMAP_GPU_COMPOSITOR env tests");
        let prior = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
        std::env::set_var("MINIMAP_GPU_COMPOSITOR", "0");
        assert!(!super::pass::minimap_gpu_compositor_env_enabled());
        match prior {
            Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
            None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
        }
    }

    /// **PERF-VIS-P1B-GPU-DEFAULT-001** — GPU default + disk witness without `RASTER_*` / `MINIMAP_GPU_COMPOSITOR=1`.
    #[test]
    fn perf_vis_p1b_gpu_default_001_without_raster_env() {
        use bevy::state::app::StatesPlugin;
        use crate::engine::states::BaseState;
        use crate::gui::MinimapPresentationSource;

        let _guard = MINIMAP_GPU_COMPOSITOR_ENV_LOCK
            .lock()
            .expect("MINIMAP_GPU_COMPOSITOR env tests");
        let prior_gpu = std::env::var("MINIMAP_GPU_COMPOSITOR").ok();
        let prior_raster_minimap = std::env::var("RASTER_MINIMAP").ok();
        let prior_raster_chunks = std::env::var("RASTER_CHUNKS_PER_FRAME").ok();
        std::env::remove_var("MINIMAP_GPU_COMPOSITOR");
        std::env::remove_var("RASTER_MINIMAP");
        std::env::remove_var("RASTER_CHUNKS_PER_FRAME");
        assert!(super::pass::minimap_gpu_compositor_env_enabled());
        assert!(super::pass::minimap_gpu_compositor_default_on_unset());

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);
        app.init_resource::<MinimapShellState>();
        app.init_resource::<MinimapRenderTargetRegistry>();
        app.init_resource::<super::MinimapCompositorState>();
        app.add_systems(Update, super::pass::sync_minimap_presentation_source);
        {
            let mut registry = app.world_mut().resource_mut::<MinimapRenderTargetRegistry>();
            let mut images = Assets::<Image>::default();
            registry.committed_size = UVec2::new(64, 64);
            registry.revision = 1;
            registry.committed_image = images.add(super::render_target::minimap_rgba_image(64, 64));
        }
        {
            let mut compositor = app.world_mut().resource_mut::<super::MinimapCompositorState>();
            compositor.stamp = 3;
            compositor.composite_path = super::MinimapCompositePath::GpuCompute;
        }
        app.update();
        let shell = app.world().resource::<MinimapShellState>();
        assert_eq!(
            shell.presentation_source,
            MinimapPresentationSource::SharedRenderTargetImage
        );

        assert!(super::refresh_perf_vis_p1b_gpu_default_live_witness());
        let text = std::fs::read_to_string("debug_runs/minimap_compositor_live.json")
            .expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(
            v["presentation_source"],
            serde_json::json!("SharedRenderTargetImage")
        );
        assert!(
            v["perf_vis_p1b_gpu_default_001"]["green"]
                .as_bool()
                .unwrap_or(false)
        );
        assert!(
            !v["perf_vis_p1b_gpu_default_001"]["raster_env_required"]
                .as_bool()
                .unwrap_or(true)
        );

        match prior_gpu {
            Some(v) => std::env::set_var("MINIMAP_GPU_COMPOSITOR", v),
            None => std::env::remove_var("MINIMAP_GPU_COMPOSITOR"),
        }
        match prior_raster_minimap {
            Some(v) => std::env::set_var("RASTER_MINIMAP", v),
            None => std::env::remove_var("RASTER_MINIMAP"),
        }
        match prior_raster_chunks {
            Some(v) => std::env::set_var("RASTER_CHUNKS_PER_FRAME", v),
            None => std::env::remove_var("RASTER_CHUNKS_PER_FRAME"),
        }
    }
}
