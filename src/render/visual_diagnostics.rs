//! Temporary consolidated visual / viewport / render-spine diagnostics.
//!
//! Enable: `VISUAL_DIAG=1`, `STAGE5_VERBOSE=1`, or `--debug-visual-diag`.
//! Filter: `RUST_LOG=visual_diag=info,visual_diag::anomaly=warn,sim_view_sync=info`
//!
//! Logs on **state edges** plus a full snapshot every 120 frames.

use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::engine::{AppState, WorldGenState};
use crate::gui::hud::{HudDockRegistry, HudWidgetId};
use crate::gui::{
    CameraVisualState, MainWorldCamera, MainWorldCameraOrthoTrace, MainWorldCameraViewportLatch,
    MapCameraDesired, MapPresentationDiagnostics, MapViewInstanceId, RepresentationBand,
    RepresentationResult, ResolvedMapViewFrames, SimulationMapViewport, SimulationMapViewportDebug,
    SimulationMapViewportTrace,
};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::frame_perf::{FramePerf, FrameWallClock};
use crate::render::{
    DebugRenderTraceConfig, GpuRepresentationMetrics, ResolvedViewports, SharedOverlayFieldBuffers,
    Stage5ReadinessProfile, TileWorldFallbackRasterDirty, ViewportPresentationMismatch,
    WorldFireParticleFrame,
};

pub const VISUAL_DIAG_TARGET: &str = "visual_diag";

#[inline]
pub fn visual_diag_enabled(cfg: Option<&DebugRenderTraceConfig>) -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let env = *ENV.get_or_init(|| {
        std::env::var_os("VISUAL_DIAG").is_some() || std::env::var_os("STAGE5_VERBOSE").is_some()
    });
    env || cfg.is_some_and(|c| c.visual_diag_trace)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct VisualDiagSignature {
    frame: u64,
    win: (u32, u32),
    sim_valid: bool,
    sim_frozen: bool,
    sim_wh: (u32, u32),
    commit_tag: u8,
    render_hole: bool,
    scissor: Option<(u32, u32, u32, u32)>,
    view_px: (u32, u32),
    resolved_sim_valid: bool,
    resolved_sim_wh: (u32, u32),
    raster_rev: u64,
    fire_particles: u32,
    overlay_chunks: u32,
    repr_band: u8,
    app: u8,
    base: u8,
}

fn pack_vec2(v: Vec2) -> (u32, u32) {
    (v.x.round() as u32, v.y.round() as u32)
}

fn pack_uvec2(v: UVec2) -> (u32, u32) {
    (v.x, v.y)
}

fn pack_viewport(vp: Option<bevy::camera::Viewport>) -> Option<(u32, u32, u32, u32)> {
    vp.map(|v| {
        (
            v.physical_position.x,
            v.physical_position.y,
            v.physical_size.x,
            v.physical_size.y,
        )
    })
}

fn tag_app(s: AppState) -> u8 {
    match s {
        AppState::Setup => 0,
        AppState::WorldGen => 1,
        AppState::InGame => 2,
        AppState::Paused => 3,
        AppState::Shutdown => 4,
    }
}

fn tag_base(s: BaseState) -> u8 {
    match s {
        BaseState::MainMenu => 0,
        BaseState::Editor => 1,
        BaseState::Simulation => 2,
        BaseState::Shutdown => 3,
    }
}

fn tag_flow(s: WorldGenFlowState) -> u8 {
    match s {
        WorldGenFlowState::Idle => 0,
        WorldGenFlowState::NewWorldSetup => 1,
        WorldGenFlowState::PreviewReady => 2,
        WorldGenFlowState::FullReady => 3,
        WorldGenFlowState::LoadingSave => 4,
    }
}

fn tag_wg(s: WorldGenState) -> u8 {
    match s {
        WorldGenState::Idle => 0,
        WorldGenState::Preview => 1,
        WorldGenState::Generating => 2,
        WorldGenState::Ready => 3,
        WorldGenState::FullReady => 4,
        WorldGenState::Dismissed => 5,
    }
}

fn tag_repr_band(b: RepresentationBand) -> u8 {
    match b {
        RepresentationBand::Full => 0,
        RepresentationBand::Tactical => 1,
        RepresentationBand::Strategic => 2,
        RepresentationBand::OverlayOnly => 3,
        RepresentationBand::Dormant => 4,
    }
}

fn tag_commit(s: &str) -> u8 {
    match s {
        "hole_inadequate" => 1,
        "hole_settling" => 2,
        "hole_settled" => 3,
        "hole_hold" => 4,
        // legacy tags (pre hole-latch migration)
        "frozen_inadequate_meas" | "inadequate" => 1,
        "frozen_shrink_ignored" | "settling" => 2,
        "frozen_expand" | "settled_freeze" => 3,
        _ => 0,
    }
}

#[derive(SystemParam)]
pub(crate) struct VisualDiagCtx<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    sim: Res<'w, SimulationMapViewport>,
    sim_trace: Res<'w, SimulationMapViewportTrace>,
    sim_dbg: Res<'w, SimulationMapViewportDebug>,
    latch: Res<'w, MainWorldCameraViewportLatch>,
    ortho: Res<'w, MainWorldCameraOrthoTrace>,
    desired: Res<'w, MapCameraDesired>,
    cam: Query<'w, 's, &'static Camera, With<MainWorldCamera>>,
    resolved: Res<'w, ResolvedViewports>,
    mismatch: Res<'w, ViewportPresentationMismatch>,
    raster: Res<'w, TileWorldFallbackRasterDirty>,
    world: Res<'w, crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    profile: Res<'w, Stage5ReadinessProfile>,
    app: Res<'w, State<AppState>>,
    base: Res<'w, State<BaseState>>,
    flow: Res<'w, State<WorldGenFlowState>>,
    wg: Res<'w, State<WorldGenState>>,
    dock: Res<'w, HudDockRegistry>,
    policy: Option<Res<'w, RepresentationResult>>,
    visual: Option<Res<'w, CameraVisualState>>,
    particles: Option<Res<'w, WorldFireParticleFrame>>,
    overlay: Option<Res<'w, SharedOverlayFieldBuffers>>,
    graph: Option<Res<'w, RenderProjectionGraph>>,
    gpu_metrics: Option<Res<'w, GpuRepresentationMetrics>>,
    perf: Option<Res<'w, FramePerf>>,
    wall: Option<Res<'w, FrameWallClock>>,
    map_pres: Option<Res<'w, MapPresentationDiagnostics>>,
    map_frames: Option<Res<'w, ResolvedMapViewFrames>>,
}

fn log_visual_diag_snapshot(frame: u64, periodic: bool, ctx: &VisualDiagCtx, cam_vp: Option<bevy::camera::Viewport>) {
    let win_logical = ctx
        .windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ONE);
    let win_physical = ctx
        .windows
        .single()
        .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
        .unwrap_or(UVec2::ONE);
    let scale = ctx.windows.single().map(|w| w.scale_factor()).unwrap_or(1.0);

    let sim = ctx.sim.as_ref();
    let sim_trace = ctx.sim_trace.as_ref();
    let sim_dbg = ctx.sim_dbg.as_ref();
    let latch = ctx.latch.as_ref();
    let ortho = ctx.ortho.as_ref();
    let desired = ctx.desired.as_ref();
    let resolved = ctx.resolved.as_ref();
    let mismatch = ctx.mismatch.as_ref();
    let profile = ctx.profile.as_ref();
    let dock = ctx.dock.as_ref();
    let app = *ctx.app.get();
    let base = *ctx.base.get();
    let flow = *ctx.flow.get();
    let wg = *ctx.wg.get();
    let world_wh = (ctx.world.width, ctx.world.height);

    let sim_adequate = sim.is_adequate_for_camera();
    let render_hole = ortho.using_hole;
    let view_px = pack_vec2(ortho.view_pixels);

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        periodic,
        win_logical = ?pack_vec2(win_logical),
        win_physical = ?pack_uvec2(win_physical),
        scale,
        app = tag_app(app),
        base = tag_base(base),
        flow = tag_flow(flow),
        worldgen = tag_wg(wg),
        readiness_profile = ?profile,
        "VISUAL_DIAG window"
    );

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        sim_valid = sim.valid,
        sim_adequate,
        sim_min = ?sim.min,
        sim_max = ?sim.max,
        sim_wh = ?pack_vec2(sim.logical_size()),
        measured_valid = sim_trace.measured_valid,
        measured_wh = ?pack_vec2(sim_trace.measured_size),
        committed_wh = ?pack_vec2(sim_trace.committed_size),
        sim_held = sim_trace.committed_from_stable_hold,
        settle_streak = sim_trace.settle_streak,
        layout_settled = sim_trace.layout_settled,
        frozen = sim_dbg.frozen,
        last_commit = sim_dbg.last_commit,
        pending_wh = ?pack_vec2(sim_dbg.pending_wh),
        pending_min = ?sim_dbg.pending_min,
        pending_max = ?sim_dbg.pending_max,
        "VISUAL_DIAG sim_viewport"
    );

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        cam_desired_x = desired.translation.x,
        cam_desired_y = desired.translation.y,
        cam_zoom = desired.scale.x,
        latch_hole = latch.using_hole,
        render_hole,
        latch_invalid_streak = latch.invalid_streak,
        latch_valid_streak = latch.valid_streak,
        cam_scissor = ?pack_viewport(cam_vp),
        ortho_fixed_w = ortho.fixed_width.round() as u32,
        ortho_fixed_h = ortho.fixed_height.round() as u32,
        map_view_px_w = view_px.0,
        map_view_px_h = view_px.1,
        world_w = world_wh.0,
        world_h = world_wh.1,
        "VISUAL_DIAG camera"
    );

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        resolved_rev = resolved.revision,
        primary_valid = resolved.primary_window.valid,
        primary_wh = ?pack_vec2(resolved.primary_window.logical_size),
        sim_resolved_valid = resolved.simulation_map.valid,
        sim_resolved_wh = ?pack_vec2(resolved.simulation_map.logical_size),
        preview_valid = resolved.world_preview.valid,
        preview_wh = ?pack_vec2(resolved.world_preview.logical_size),
        minimap_valid = resolved.minimap_panel.valid,
        minimap_wh = ?pack_vec2(resolved.minimap_panel.logical_size),
        mismatch_preview = mismatch.world_preview_extent_mismatch,
        mismatch_minimap = mismatch.minimap_panel_extent_mismatch,
        mismatch_sim_map = mismatch.simulation_map_extent_mismatch,
        mismatch_stale_tex = mismatch.stale_texture_binding,
        "VISUAL_DIAG resolved_viewports"
    );

    if let Some(frames) = ctx.map_frames.as_ref() {
        info!(
            target: VISUAL_DIAG_TARGET,
            frame,
            world_preview_proj_rev = frames.world_preview.projection_revision,
            minimap_proj_rev = frames.minimap.projection_revision,
            sim_map_proj_rev = frames.simulation_map.projection_revision,
            "VISUAL_DIAG map_view_frames"
        );
    }

    if let Some(pres) = ctx.map_pres.as_ref() {
        let wp = pres.slot(MapViewInstanceId::WorldPreview);
        let mm = pres.slot(MapViewInstanceId::Minimap);
        info!(
            target: VISUAL_DIAG_TARGET,
            frame,
            wp_fit = ?wp.fit_mode,
            wp_viewport = ?wp.viewport_extent,
            wp_fit_scale = wp.fit_scale,
            wp_expected_fit_scale = wp.expected_fit_scale,
            wp_zoom = wp.camera_zoom,
            mm_fit = ?mm.fit_mode,
            mm_viewport = ?mm.viewport_extent,
            mm_fit_scale = mm.fit_scale,
            "VISUAL_DIAG map_presentation"
        );
    }

    let (repr_band, repr_lod, particle_cap) = ctx
        .policy
        .as_ref()
        .map(|p| (
            tag_repr_band(p.active_band),
            format!("{:?}", p.world_lod_band),
            p.particle_policy.rows_cap,
        ))
        .unwrap_or((255, "none".to_string(), 0));

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        raster_rev = ctx.raster.revision(),
        repr_band,
        repr_lod,
        particle_rows_cap = particle_cap,
        visual_intent = ctx.visual.as_ref().map(|v| format!("{:?}", v.intent)),
        visual_zoom_alpha = ctx.visual.as_ref().map(|v| v.zoom_alpha),
        fire_particle_rows = ctx
            .particles
            .as_ref()
            .map(|p| p.instances.len())
            .unwrap_or(0),
        fire_spark_rows = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.rows)
            .unwrap_or(0),
        fire_spark_phase = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.phase)
            .unwrap_or("none"),
        fire_spark_scatter_slots = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.scatter_slots)
            .unwrap_or(0),
        fire_spark_scatter_max = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.scatter_max)
            .unwrap_or(0),
        fire_spark_zoom_alpha = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.zoom_alpha)
            .unwrap_or(0.0),
        fire_spark_additive_blend = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.additive_blend)
            .unwrap_or(false),
        fire_spark_budget_capped = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.budget_capped)
            .unwrap_or(false),
        fire_spark_compute_enabled = crate::render::fire_spark_compute_enabled(),
        fire_particle_view_culled = ctx
            .particles
            .as_ref()
            .map(|p| p.spark_witness.view_culled)
            .unwrap_or(false),
        fire_particle_stamp = ctx.particles.as_ref().map(|p| p.snapshot_stamp).unwrap_or(0),
        overlay_rev = ctx.overlay.as_ref().map(|o| o.revision).unwrap_or(0),
        overlay_chunk_cells = ctx
            .overlay
            .as_ref()
            .map(|o| o.chunk_fire_heat.len())
            .unwrap_or(0),
        graph_fire_inst = ctx
            .graph
            .as_ref()
            .map(|g| g.fire.instance_buffer.len())
            .unwrap_or(0),
        graph_fire_heat = ctx
            .graph
            .as_ref()
            .map(|g| g.fire.chunk_heat.len())
            .unwrap_or(0),
        gpu_instance_rows = ctx.gpu_metrics.as_ref().map(|m| m.instance_rows).unwrap_or(0),
        gpu_dispatch = ctx.gpu_metrics.as_ref().map(|m| m.dispatch_count).unwrap_or(0),
        gpu_draw = ctx.gpu_metrics.as_ref().map(|m| m.draw_instances).unwrap_or(0),
        "VISUAL_DIAG render_spine"
    );

    if let (Some(perf), Some(wall)) = (ctx.perf.as_ref(), ctx.wall.as_ref()) {
        info!(
            target: VISUAL_DIAG_TARGET,
            frame,
            tile_raster_ms = perf.tile_raster_ms,
            tile_raster_ran = perf.tile_raster_ran,
            world_repr_ms = perf.world_repr_ms,
            projection_graph_ms = perf.projection_graph_ms,
            domain_merge_ms = perf.domain_merge_ms,
            readiness_ms = perf.readiness_ms,
            cpu_pre_egui_ms = wall.cpu_pre_egui_ms,
            cpu_egui_ms = wall.cpu_egui_ms,
            cpu_post_egui_ms = wall.cpu_post_egui_ms,
            gpu_gap_ms = wall.gpu_gap_ms,
            "VISUAL_DIAG perf"
        );
    }

    info!(
        target: VISUAL_DIAG_TARGET,
        frame,
        cmd_shell = dock.slot(HudWidgetId::CommandShell).visible,
        overlay_tray = dock.slot(HudWidgetId::OverlayTray).visible,
        transmission = dock.slot(HudWidgetId::Transmission).visible,
        "VISUAL_DIAG hud_shell"
    );
}

fn log_visual_diag_anomalies(prev: VisualDiagSignature, sig: VisualDiagSignature) {
    if prev.render_hole != sig.render_hole {
        warn!(
            target: "visual_diag::anomaly",
            frame = sig.frame,
            was = prev.render_hole,
            now = sig.render_hole,
            "RENDER_HOLE_FLIP"
        );
    }
    if prev.scissor != sig.scissor {
        warn!(
            target: "visual_diag::anomaly",
            frame = sig.frame,
            was = ?prev.scissor,
            now = ?sig.scissor,
            "CAMERA_SCISSOR_CHANGED"
        );
    }
    if prev.sim_valid != sig.sim_valid {
        warn!(
            target: "visual_diag::anomaly",
            frame = sig.frame,
            was_valid = prev.sim_valid,
            now_valid = sig.sim_valid,
            "SIM_VIEWPORT_VALIDITY_CHANGED"
        );
    }
    if prev.resolved_sim_valid != sig.resolved_sim_valid {
        warn!(
            target: "visual_diag::anomaly",
            frame = sig.frame,
            was = prev.resolved_sim_valid,
            now = sig.resolved_sim_valid,
            "RESOLVED_SIM_MAP_VALIDITY_CHANGED"
        );
    }
    if prev.commit_tag != sig.commit_tag {
        warn!(
            target: "visual_diag::anomaly",
            frame = sig.frame,
            was = prev.commit_tag,
            now = sig.commit_tag,
            "SIM_COMMIT_BRANCH_CHANGED"
        );
    }
    if prev.view_px != sig.view_px && prev.render_hole == sig.render_hole && sig.render_hole {
        let dw = sig.view_px.0.abs_diff(prev.view_px.0);
        let dh = sig.view_px.1.abs_diff(prev.view_px.1);
        if dw > 16 || dh > 8 {
            warn!(
                target: "visual_diag::anomaly",
                frame = sig.frame,
                was = ?prev.view_px,
                now = ?sig.view_px,
                delta_w = dw,
                delta_h = dh,
                "MAP_VIEW_PX_JUMP"
            );
        }
    }
}

fn log_visual_diag_startup(cfg: Res<DebugRenderTraceConfig>) {
    if visual_diag_enabled(Some(cfg.as_ref())) {
        info!(
            target: VISUAL_DIAG_TARGET,
            "VISUAL_DIAG active — RUST_LOG=visual_diag=info,visual_diag::anomaly=warn"
        );
    }
}

/// Runs in `Last` after viewport / camera / raster hooks.
pub(crate) fn trace_visual_diagnostics(
    cfg: Res<DebugRenderTraceConfig>,
    frame: Res<FrameCount>,
    ctx: VisualDiagCtx,
    mut last_sig: Local<Option<VisualDiagSignature>>,
    mut last_periodic: Local<u64>,
) {
    if !visual_diag_enabled(Some(cfg.as_ref())) {
        return;
    }

    let frame_n = u64::from(frame.0);
    let cam_vp = ctx.cam.single().ok().and_then(|c| c.viewport.clone());
    let sig = VisualDiagSignature {
        frame: frame_n,
        win: ctx
            .windows
            .single()
            .map(|w| (w.width().round() as u32, w.height().round() as u32))
            .unwrap_or((0, 0)),
        sim_valid: ctx.sim.valid,
        sim_frozen: ctx.sim_dbg.frozen,
        sim_wh: pack_vec2(ctx.sim.logical_size()),
        commit_tag: tag_commit(ctx.sim_dbg.last_commit),
        render_hole: ctx.ortho.using_hole,
        scissor: pack_viewport(cam_vp.clone()),
        view_px: pack_vec2(ctx.ortho.view_pixels),
        resolved_sim_valid: ctx.resolved.simulation_map.valid,
        resolved_sim_wh: pack_vec2(ctx.resolved.simulation_map.logical_size),
        raster_rev: ctx.raster.revision(),
        fire_particles: ctx
            .particles
            .as_ref()
            .map(|p| p.instances.len() as u32)
            .unwrap_or(0),
        overlay_chunks: ctx
            .overlay
            .as_ref()
            .map(|o| o.chunk_fire_heat.len() as u32)
            .unwrap_or(0),
        repr_band: ctx
            .policy
            .as_ref()
            .map(|p| tag_repr_band(p.active_band))
            .unwrap_or(255),
        app: tag_app(*ctx.app.get()),
        base: tag_base(*ctx.base.get()),
    };

    let edge = last_sig.as_ref() != Some(&sig);
    let periodic = frame_n.saturating_sub(*last_periodic) >= 120;
    if !edge && !periodic {
        return;
    }

    if let Some(prev) = *last_sig {
        if edge {
            log_visual_diag_anomalies(prev, sig);
        }
    }

    log_visual_diag_snapshot(frame_n, periodic, &ctx, cam_vp);

    *last_sig = Some(sig);
    if periodic {
        *last_periodic = frame_n;
    }
}

pub struct VisualDiagnosticsPlugin;

impl Plugin for VisualDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationMapViewportDebug>()
            .add_systems(Startup, log_visual_diag_startup)
            .add_systems(Last, trace_visual_diagnostics);
    }
}
