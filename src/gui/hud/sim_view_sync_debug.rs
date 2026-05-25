//! Simulation map / window / camera viewport sync tracing.
//!
//! Enable with `SIM_VIEW_SYNC_DEBUG=1`, `STAGE5_VERBOSE=1`, or `--debug-sim-view-sync`.
//! Filter: `RUST_LOG=sim_view_sync=info,sim_view_sync::anomaly=warn`
//!
//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner viewport_migration_agent
//! @orchestrator-do-not-cleanup

use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::engine::{AppState, WorldGenState};
use crate::gui::hud::{HudDockRegistry, HudWidgetId, ViewportRectSanity};
use crate::gui::CommandLeftStackState;
use crate::gui::map_camera::{
    MainWorldCamera, MainWorldCameraOrthoTrace, MainWorldCameraViewportLatch, MapCameraDesired,
};
use crate::gui::{
    MinimapShellState, SimulationMapViewport, SimulationMapViewportDebug, SimulationMapViewportTrace,
};
use crate::render::{DebugRenderTraceConfig, ResolvedViewports, TileWorldFallbackRasterDirty};

#[inline]
pub fn sim_view_sync_debug_enabled(cfg: Option<&DebugRenderTraceConfig>) -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let env = *ENV.get_or_init(|| {
        std::env::var_os("SIM_VIEW_SYNC_DEBUG").is_some()
            || std::env::var_os("STAGE5_VERBOSE").is_some()
    });
    env || cfg.is_some_and(|c| c.sim_view_sync_trace)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SyncSignature {
    win_logical: (u32, u32),
    win_physical: (u32, u32),
    sim_valid: bool,
    sim_adequate: bool,
    sim_held: bool,
    commit_tag: u8,
    cam_hole: bool,
    render_hole: bool,
    cam_scissor: Option<(u32, u32, u32, u32)>,
    ortho_wh: (u32, u32),
    view_px: (u32, u32),
    raster_rev: u64,
    resolved_rev: u64,
    app: u8,
    base: u8,
    flow: u8,
    wg: u8,
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

fn tag_commit(s: &str) -> u8 {
    match s {
        "hole_inadequate" => 1,
        "hole_settling" => 2,
        "hole_settled" => 3,
        "hole_hold" => 4,
        "frozen_inadequate_meas" | "inadequate" => 1,
        "frozen_shrink_ignored" | "settling" => 5,
        "frozen_expand" | "settled_freeze" => 4,
        _ => 0,
    }
}

#[derive(SystemParam)]
pub(crate) struct SimViewSyncCtx<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    sim: Res<'w, SimulationMapViewport>,
    trace: Res<'w, SimulationMapViewportTrace>,
    sim_dbg: Res<'w, SimulationMapViewportDebug>,
    latch: Res<'w, MainWorldCameraViewportLatch>,
    cam: Query<'w, 's, &'static Camera, With<MainWorldCamera>>,
    ortho: Res<'w, MainWorldCameraOrthoTrace>,
    raster_dirty: Res<'w, TileWorldFallbackRasterDirty>,
    resolved: Res<'w, ResolvedViewports>,
    app: Res<'w, State<AppState>>,
    base: Res<'w, State<BaseState>>,
    flow: Res<'w, State<WorldGenFlowState>>,
    wg: Res<'w, State<WorldGenState>>,
    dock: Res<'w, HudDockRegistry>,
    viewport_sanity: Res<'w, ViewportRectSanity>,
    left_stack: Res<'w, CommandLeftStackState>,
    map_desired: Res<'w, MapCameraDesired>,
    minimap: Res<'w, MinimapShellState>,
}

fn log_sync_edge(
    frame: u64,
    sig: SyncSignature,
    prev: Option<SyncSignature>,
    sim: &SimulationMapViewport,
    trace: &SimulationMapViewportTrace,
    sim_dbg: &SimulationMapViewportDebug,
    latch: &MainWorldCameraViewportLatch,
    dock: &HudDockRegistry,
    viewport_sanity: &ViewportRectSanity,
    left_stack: &CommandLeftStackState,
    map_desired: &MapCameraDesired,
    minimap: &MinimapShellState,
) {
    info!(
        target: "sim_view_sync",
        frame,
        win_logical = ?sig.win_logical,
        win_physical = ?sig.win_physical,
        sim_valid = sig.sim_valid,
        sim_adequate = sig.sim_adequate,
        sim_min = ?sim.min,
        sim_max = ?sim.max,
        measured_valid = trace.measured_valid,
        measured_wh = ?trace.measured_size,
        committed_wh = ?trace.committed_size,
        sim_wh = ?sim.logical_size(),
        settle_streak = trace.settle_streak,
        layout_settled = trace.layout_settled,
        sim_held = sig.sim_held,
        last_commit = sim_dbg.last_commit,
        frozen = sim_dbg.frozen,
        pending_wh = ?sim_dbg.pending_wh,
        cam_hole = sig.cam_hole,
        render_hole = sig.render_hole,
        cam_invalid_streak = latch.invalid_streak,
        cam_valid_streak = latch.valid_streak,
        cam_scissor = ?sig.cam_scissor,
        ortho_fixed_wh = ?sig.ortho_wh,
        map_view_px = ?sig.view_px,
        raster_rev = sig.raster_rev,
        resolved_rev = sig.resolved_rev,
        app = sig.app,
        base = sig.base,
        flow = sig.flow,
        worldgen = sig.wg,
        cmd_shell = dock.slot(HudWidgetId::CommandShell).visible,
        overlay_tray = dock.slot(HudWidgetId::OverlayTray).visible,
        transmission = dock.slot(HudWidgetId::Transmission).visible,
        rect_sanity_issues = viewport_sanity.issues_total,
        left_stack_collapsed = left_stack.collapsed,
        map_cam_scale = ?map_desired.scale,
        minimap_visible = minimap.visible,
        "SIM_VIEW_SYNC"
    );

    if let Some(p) = prev {
        if p.cam_hole != sig.cam_hole {
            warn!(
                target: "sim_view_sync::anomaly",
                frame,
                was_hole = p.cam_hole,
                now_hole = sig.cam_hole,
                "CAMERA_VIEWPORT_MODE_FLIP (full-window vs map-hole scissor)"
            );
        }
        if p.cam_scissor != sig.cam_scissor {
            warn!(
                target: "sim_view_sync::anomaly",
                frame,
                was = ?p.cam_scissor,
                now = ?sig.cam_scissor,
                "CAMERA_SCISSOR_CHANGED"
            );
        }
        if p.sim_adequate != sig.sim_adequate || p.sim_valid != sig.sim_valid {
            warn!(
                target: "sim_view_sync::anomaly",
                frame,
                was_valid = p.sim_valid,
                now_valid = sig.sim_valid,
                was_adequate = p.sim_adequate,
                now_adequate = sig.sim_adequate,
                "SIM_MAP_VIEWPORT_VALIDITY_CHANGED"
            );
        }
        if sig.render_hole != sig.cam_hole {
            warn!(
                target: "sim_view_sync::anomaly",
                frame,
                latch_hole = sig.cam_hole,
                render_hole = sig.render_hole,
                "VIEWPORT_ORTHO_MISMATCH (latch vs render_hole — should not happen after immediate latch release)"
            );
        }
        if p.render_hole != sig.render_hole {
            warn!(
                target: "sim_view_sync::anomaly",
                frame,
                was_render_hole = p.render_hole,
                now_render_hole = sig.render_hole,
                was_scissor = ?p.cam_scissor,
                now_scissor = ?sig.cam_scissor,
                "RENDER_MODE_FLIP (map-hole scissor vs full-window — primary blink source)"
            );
        }
        if sig.render_hole == p.render_hole {
            let dw = sig.view_px.0.abs_diff(p.view_px.0);
            let dh = sig.view_px.1.abs_diff(p.view_px.1);
            if (dw > 16 || dh > 8) && (p.ortho_wh != sig.ortho_wh || p.view_px != sig.view_px) {
                warn!(
                    target: "sim_view_sync::anomaly",
                    frame,
                    ortho_was = ?p.ortho_wh,
                    ortho_now = ?sig.ortho_wh,
                    view_px_was = ?p.view_px,
                    view_px_now = ?sig.view_px,
                    delta_w = dw,
                    delta_h = dh,
                    "ORTHO_VIEW_PX_DRIFT (same render mode — hole size jumped)"
                );
            }
        }
    }
}

/// Runs in `Last` after map camera + UI viewport commit; logs edges when layout/camera state changes.
pub(crate) fn trace_sim_view_sync_state(
    cfg: Res<DebugRenderTraceConfig>,
    frame: Res<FrameCount>,
    ctx: SimViewSyncCtx,
    mut last: Local<Option<SyncSignature>>,
) {
    if !sim_view_sync_debug_enabled(Some(cfg.as_ref())) {
        return;
    }

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

    let cam_vp = ctx.cam.single().ok().and_then(|c| c.viewport.clone());
    let sig = SyncSignature {
        win_logical: pack_vec2(win_logical),
        win_physical: pack_uvec2(win_physical),
        sim_valid: ctx.sim.valid,
        sim_adequate: ctx.sim.is_adequate_for_camera(),
        sim_held: ctx.trace.committed_from_stable_hold,
        commit_tag: tag_commit(ctx.sim_dbg.last_commit),
        cam_hole: ctx.latch.using_hole,
        render_hole: ctx.ortho.using_hole,
        cam_scissor: pack_viewport(cam_vp),
        ortho_wh: (
            ctx.ortho.fixed_width.round() as u32,
            ctx.ortho.fixed_height.round() as u32,
        ),
        view_px: pack_vec2(ctx.ortho.view_pixels),
        raster_rev: ctx.raster_dirty.revision(),
        resolved_rev: ctx.resolved.revision,
        app: tag_app(*ctx.app.get()),
        base: tag_base(*ctx.base.get()),
        flow: tag_flow(*ctx.flow.get()),
        wg: tag_wg(*ctx.wg.get()),
    };

    if last.as_ref() == Some(&sig) {
        return;
    }

    log_sync_edge(
        u64::from(frame.0),
        sig,
        *last,
        ctx.sim.as_ref(),
        ctx.trace.as_ref(),
        ctx.sim_dbg.as_ref(),
        ctx.latch.as_ref(),
        ctx.dock.as_ref(),
        ctx.viewport_sanity.as_ref(),
        ctx.left_stack.as_ref(),
        ctx.map_desired.as_ref(),
        ctx.minimap.as_ref(),
    );
    *last = Some(sig);
}

fn log_sim_view_sync_startup(cfg: Res<DebugRenderTraceConfig>) {
    if sim_view_sync_debug_enabled(Some(cfg.as_ref())) {
        info!(
            target: "sim_view_sync",
            "SIM_VIEW_SYNC_DEBUG active — filter RUST_LOG=sim_view_sync=info,sim_view_sync::anomaly=warn"
        );
    }
}

pub struct SimViewSyncDebugPlugin;

impl Plugin for SimViewSyncDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, log_sim_view_sync_startup)
            .add_systems(Last, trace_sim_view_sync_state);
    }
}
