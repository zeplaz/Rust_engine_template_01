//! World-gen / preview chrome tracing (`WORLDGEN_CHROME_DEBUG=1` or `STAGE5_VERBOSE=1`).
//!
//! Use when preview or generator UI persists after load-in or map-view flicker is suspected:
//! - `CHROME_STATE` — edge log whenever app/flow/chrome flags change
//! - `CHROME_DISMISS` / `CHROME_OPEN` — who toggled chrome
//! - `CHROME_ANOMALY` — in-game simulation but generator/preview chrome still active
//! - `PREVIEW_EGUI` — texture rebind / placeholder-while-ready (flicker)

use bevy::prelude::*;

use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::gui::hud::HudDockRegistry;
use crate::gui::hud::HudWidgetId;
use crate::engine::{AppState, WorldGenChromeLatch, WorldGenState};
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::{
    PreviewLifecyclePhase, WorldPreviewLifecycle, WorldPreviewUiState,
};
use crate::gui::MapViewTextureCache;

/// Last dismiss call site (for anomaly logs).
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldGenChromeDebugTrace {
    pub last_dismiss_reason: Option<&'static str>,
    pub dismiss_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChromeSnapshot {
    app: u8,
    worldgen: u8,
    base: u8,
    flow: u8,
    latch_dismissed: bool,
    world_gen_visible: bool,
    preview_window_open: bool,
    lifecycle: u8,
}

#[inline]
pub fn worldgen_chrome_debug_enabled() -> bool {
    crate::dev::test_run_instrumentation::diagnostics_operator_trace_enabled(
        false,
        &["WORLDGEN_CHROME_DEBUG", "STAGE5_VERBOSE"],
    )
}

pub fn log_chrome_dismiss(reason: &'static str, latch: bool, wg_visible: bool, preview_open: bool) {
    if !worldgen_chrome_debug_enabled() {
        return;
    }
    info!(
        target: "worldgen_chrome::dismiss",
        reason,
        latch_dismissed = latch,
        world_gen_visible = wg_visible,
        preview_window_open = preview_open,
        "CHROME_DISMISS"
    );
}

pub fn log_chrome_open(reason: &'static str, wg_visible: bool, preview_open: bool) {
    if !worldgen_chrome_debug_enabled() {
        return;
    }
    info!(
        target: "worldgen_chrome::open",
        reason,
        world_gen_visible = wg_visible,
        preview_window_open = preview_open,
        "CHROME_OPEN"
    );
}

pub fn record_chrome_dismiss_trace(trace: Option<&mut WorldGenChromeDebugTrace>, reason: &'static str) {
    if let Some(t) = trace {
        t.last_dismiss_reason = Some(reason);
        t.dismiss_count = t.dismiss_count.saturating_add(1);
    }
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

fn tag_lifecycle(p: PreviewLifecyclePhase) -> u8 {
    match p {
        PreviewLifecyclePhase::Uninitialized => 0,
        PreviewLifecyclePhase::ReadyToRender => 1,
        PreviewLifecyclePhase::GeneratingWorld => 2,
        PreviewLifecyclePhase::Rendering => 3,
        PreviewLifecyclePhase::Updating => 4,
    }
}

pub(crate) fn trace_worldgen_chrome_state_edges(
    app: Res<State<AppState>>,
    worldgen: Res<State<WorldGenState>>,
    base: Res<State<BaseState>>,
    flow: Res<State<WorldGenFlowState>>,
    latch: Res<WorldGenChromeLatch>,
    world_gen: Res<WorldGenUiState>,
    preview_ui: Res<WorldPreviewUiState>,
    lifecycle: Res<WorldPreviewLifecycle>,
    trace: Option<Res<WorldGenChromeDebugTrace>>,
    mut prev: Local<Option<ChromeSnapshot>>,
) {
    if !worldgen_chrome_debug_enabled() {
        return;
    }

    let snap = ChromeSnapshot {
        app: tag_app(*app.get()),
        worldgen: tag_wg(*worldgen.get()),
        base: tag_base(*base.get()),
        flow: tag_flow(*flow.get()),
        latch_dismissed: latch.full_ready_dismissed,
        world_gen_visible: world_gen.visible,
        preview_window_open: preview_ui.window_open,
        lifecycle: tag_lifecycle(lifecycle.phase),
    };

    if prev.as_ref() == Some(&snap) {
        return;
    }

    let last_dismiss = trace
        .as_ref()
        .and_then(|t| t.last_dismiss_reason)
        .unwrap_or("never");

    info!(
        target: "worldgen_chrome::trace",
        ?app,
        ?worldgen,
        ?base,
        ?flow,
        latch_dismissed = snap.latch_dismissed,
        world_gen_visible = snap.world_gen_visible,
        preview_window_open = snap.preview_window_open,
        lifecycle = ?lifecycle.phase,
        last_dismiss,
        "CHROME_STATE"
    );

    let flow_expects_chrome = matches!(
        *flow.get(),
        WorldGenFlowState::NewWorldSetup
            | WorldGenFlowState::PreviewReady
            | WorldGenFlowState::FullReady
    );
    let chrome_hidden_during_gen = flow_expects_chrome
        && *base.get() == BaseState::Editor
        && !snap.latch_dismissed
        && !snap.world_gen_visible
        && !snap.preview_window_open;
    if chrome_hidden_during_gen {
        warn!(
            target: "worldgen_chrome::anomaly",
            ?app,
            ?worldgen,
            ?flow,
            lifecycle = ?lifecycle.phase,
            "CHROME_HIDDEN_DURING_GEN world-gen flow active but both panels closed"
        );
    }

    let in_game = matches!(*app.get(), AppState::InGame | AppState::Paused);
    let in_sim = *base.get() == BaseState::Simulation;
    let chrome_active = snap.world_gen_visible || snap.preview_window_open;
    if in_game && in_sim && chrome_active {
        warn!(
            target: "worldgen_chrome::anomaly",
            ?app,
            ?worldgen,
            ?flow,
            latch_dismissed = snap.latch_dismissed,
            world_gen_visible = snap.world_gen_visible,
            preview_window_open = snap.preview_window_open,
            lifecycle = ?lifecycle.phase,
            last_dismiss,
            "CHROME_ANOMALY in-game simulation but world-gen/preview chrome still active (check Enter world / dismiss latch / F8 toggle)"
        );
    }

    *prev = Some(snap);
}

pub fn trace_preview_egui_chrome(
    window_open: bool,
    preview_ready: bool,
    texture_bound: bool,
    lifecycle: PreviewLifecyclePhase,
    tex_cache: &MapViewTextureCache,
    projection_revision: u64,
    pipeline_would_run: bool,
) {
    if !worldgen_chrome_debug_enabled() {
        return;
    }

    let binding = tex_cache.binding(crate::gui::MapViewInstanceId::WorldPreview);
    if binding.rebinds_frame > 0 {
        warn!(
            target: "worldgen_chrome::preview_egui",
            rebinds_frame = binding.rebinds_frame,
            rebinds_total = binding.rebinds_total,
            projection_revision,
            window_open,
            preview_ready,
            texture_bound,
            lifecycle = ?lifecycle,
            "PREVIEW_EGUI_REBIND (egui texture id churn — flicker)"
        );
    }

    if window_open && preview_ready && !texture_bound {
        warn!(
            target: "worldgen_chrome::preview_egui",
            projection_revision,
            lifecycle = ?lifecycle,
            pipeline_would_run,
            "PREVIEW_EGUI_PLACEHOLDER ready=true but no bound texture (placeholder flash)"
        );
    }
}

pub struct WorldGenChromeDebugPlugin;

fn log_worldgen_chrome_debug_startup() {
    if worldgen_chrome_debug_enabled() {
        info!(
            target: "worldgen_chrome",
            "WORLDGEN_CHROME_DEBUG active — filter RUST_LOG=worldgen_chrome=info,worldgen_chrome::anomaly=warn,worldgen_chrome::preview_egui=warn"
        );
    }
}

pub fn trace_hud_shell_state(
    base: Res<State<BaseState>>,
    dock: Res<HudDockRegistry>,
    minimap: Res<crate::gui::MinimapShellState>,
) {
    if !worldgen_chrome_debug_enabled() {
        return;
    }
    static LAST_BASE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(255);
    let base_tag = tag_base(*base.get());
    if LAST_BASE.swap(base_tag, std::sync::atomic::Ordering::Relaxed) == base_tag {
        return;
    }
    info!(
        target: "worldgen_chrome::hud",
        ?base,
        minimap_visible = minimap.visible,
        overlay_tray = dock.slot(HudWidgetId::OverlayTray).visible,
        command_shell = dock.slot(HudWidgetId::CommandShell).visible,
        transmission = dock.slot(HudWidgetId::Transmission).visible,
        "HUD_SHELL_STATE (Editor/Simulation = HUD egui may run; MainMenu = player shell off)"
    );
}

impl Plugin for WorldGenChromeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenChromeDebugTrace>()
            .add_systems(Startup, log_worldgen_chrome_debug_startup)
            .add_systems(Last, (trace_worldgen_chrome_state_edges, trace_hud_shell_state));
    }
}
