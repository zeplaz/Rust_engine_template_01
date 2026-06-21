//! UX orchestration plugin — drives legacy engine states from [`super::ux_states`].

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

use crate::engine::states::{
    BaseState, InGameMenuState, WorldGenFlowState,
};
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::hud::frame_budget_diagnostics::FrameBudgetDiagnostics;
use crate::gui::PauseMenuPendingAction;
use crate::terrain::generation::world_generator_enhanced::WorldGenJobSlot;

use super::test_harness::{DebugQuickWorldGenPending, TestWorldHarness};
use super::ux_states::{
    AppState, PauseState, UxFrameSpikeGuard, WorldGenChromeLatch, WorldGenState,
};

static UX_SPIKE_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Process-wide spike latch for cheap checks from hot systems (no extra ECS params).
#[must_use]
pub fn ux_spike_active() -> bool {
    UX_SPIKE_ACTIVE.load(Ordering::Relaxed)
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UxBridgeSet;

pub struct UxOrchestrationPlugin;

impl Plugin for UxOrchestrationPlugin {
    fn build(&self, app: &mut App) {
        // Resources must exist before `init_state` — default `AppState::Setup` runs OnEnter immediately.
        app.init_resource::<WorldGenChromeLatch>()
            .init_resource::<UxFrameSpikeGuard>()
            .init_state::<AppState>()
            .init_state::<WorldGenState>()
            .init_state::<PauseState>()
            .configure_sets(Update, UxBridgeSet)
            .add_systems(Startup, ux_bootstrap_from_legacy)
            .add_systems(
                Update,
                (
                    sync_legacy_to_ux,
                    bridge_ux_to_legacy.in_set(UxBridgeSet),
                    ux_frame_spike_watchdog,
                )
                    .chain()
                    .after(crate::gui::toggle_pause_menu_on_escape),
            )
            .add_systems(OnEnter(AppState::Setup), ux_on_enter_setup)
            .add_systems(OnEnter(AppState::WorldGen), ux_on_enter_world_gen)
            .add_systems(OnEnter(AppState::InGame), ux_on_enter_in_game)
            .add_systems(OnEnter(AppState::Paused), ux_on_enter_paused)
            .add_systems(OnEnter(AppState::Shutdown), ux_on_enter_shutdown)
            .add_systems(OnEnter(WorldGenState::FullReady), ux_on_enter_worldgen_full_ready)
            .add_systems(OnEnter(WorldGenState::Dismissed), ux_on_enter_worldgen_dismissed);
    }
}

/// One-shot: align UX states with whatever legacy boot / test harness already set.
fn ux_bootstrap_from_legacy(
    base: Res<State<BaseState>>,
    flow: Res<State<WorldGenFlowState>>,
    menu: Res<State<InGameMenuState>>,
    job: Res<WorldGenJobSlot>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    mut next_pause: ResMut<NextState<PauseState>>,
) {
    let app = legacy_to_app_state(*base.get(), *menu.get());
    let wg = legacy_to_worldgen_state(*flow.get(), job.is_busy());
    let pause = legacy_to_pause_state(*menu.get());
    next_app.set(app);
    next_wg.set(wg);
    next_pause.set(pause);
}

/// Mirror legacy transitions (world gen plugin, test harness, pause menu) into UX states.
fn sync_legacy_to_ux(
    base: Res<State<BaseState>>,
    flow: Res<State<WorldGenFlowState>>,
    menu: Res<State<InGameMenuState>>,
    pending: Res<PauseMenuPendingAction>,
    debug_quick: Option<Res<DebugQuickWorldGenPending>>,
    job: Res<WorldGenJobSlot>,
    latch: Res<WorldGenChromeLatch>,
    app: Res<State<AppState>>,
    worldgen: Res<State<WorldGenState>>,
    pause: Res<State<PauseState>>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    mut next_pause: ResMut<NextState<PauseState>>,
) {
    // Menu debug bootstrap: block stale legacy flow from yanking WG off the ladder — but still
    // mirror forward progress (Preview → Generating → Ready) while preview/full jobs run.
    if debug_quick.is_some_and(|p| p.active) {
        let mirrored_app = legacy_to_app_state(*base.get(), *menu.get());
        let in_game_after_dismiss = latch.full_ready_dismissed
            && matches!(*app.get(), AppState::InGame | AppState::Paused);
        let mut mirrored_wg = legacy_to_worldgen_state(*flow.get(), job.is_busy());
        if latch.full_ready_dismissed && mirrored_wg == WorldGenState::FullReady {
            mirrored_wg = WorldGenState::Dismissed;
        }
        let mirrored_pause = if pending.is_pending() {
            PauseState::ConfirmExit
        } else {
            legacy_to_pause_state(*menu.get())
        };

        let block_stale_wg_pull = matches!(
            (*worldgen.get(), mirrored_wg, *flow.get()),
            (
                WorldGenState::Preview,
                WorldGenState::FullReady | WorldGenState::Idle,
                WorldGenFlowState::FullReady | WorldGenFlowState::Idle
            )
        );
        let wg_sync_blocked = matches!(*worldgen.get(), WorldGenState::Dismissed)
            && latch.full_ready_dismissed
            && !matches!(
                mirrored_wg,
                WorldGenState::Preview | WorldGenState::Generating | WorldGenState::Ready
            );
        if !in_game_after_dismiss
            && *worldgen.get() != mirrored_wg
            && !wg_sync_blocked
            && !block_stale_wg_pull
        {
            next_wg.set(mirrored_wg);
        }

        if *app.get() != mirrored_app {
            let block_world_gen_pull = mirrored_app == AppState::WorldGen
                && matches!(*worldgen.get(), WorldGenState::Dismissed);
            let block_menu_pull = latch.full_ready_dismissed
                && matches!(*app.get(), AppState::InGame | AppState::Paused)
                && mirrored_app == AppState::Setup;
            let block_world_gen_from_main_menu = matches!(*base.get(), BaseState::MainMenu)
                && mirrored_app == AppState::WorldGen
                && matches!(*app.get(), AppState::Setup);
            let block_setup_pull_during_worldgen = mirrored_app == AppState::Setup
                && matches!(*app.get(), AppState::WorldGen)
                && super::worldgen_lifecycle_active(worldgen.get());
            if !block_world_gen_pull
                && !block_menu_pull
                && !block_world_gen_from_main_menu
                && !block_setup_pull_during_worldgen
            {
                next_app.set(mirrored_app);
            }
        }
        if *pause.get() != mirrored_pause {
            next_pause.set(mirrored_pause);
        }
        return;
    }

    let mirrored_app = legacy_to_app_state(*base.get(), *menu.get());
    let in_game_after_dismiss = latch.full_ready_dismissed
        && matches!(*app.get(), AppState::InGame | AppState::Paused);
    let mut mirrored_wg = legacy_to_worldgen_state(*flow.get(), job.is_busy());
    if latch.full_ready_dismissed && mirrored_wg == WorldGenState::FullReady {
        mirrored_wg = WorldGenState::Dismissed;
    }
    let mirrored_pause = if pending.is_pending() {
        PauseState::ConfirmExit
    } else {
        legacy_to_pause_state(*menu.get())
    };

    if *app.get() != mirrored_app {
        // Editor base while chrome dismissed must not yank AppState back to WorldGen (panel flicker).
        let block_world_gen_pull = mirrored_app == AppState::WorldGen
            && matches!(*worldgen.get(), WorldGenState::Dismissed);
        // Legacy MainMenu must not yank UX back to Setup after FullReady dismiss (menu blink loop).
        let block_menu_pull = latch.full_ready_dismissed
            && matches!(*app.get(), AppState::InGame | AppState::Paused)
            && mirrored_app == AppState::Setup;
        // Title screen: stale `BaseState::Editor` must not pull App into WorldGen (chrome/flicker loop).
        let block_world_gen_from_main_menu = matches!(*base.get(), BaseState::MainMenu)
            && mirrored_app == AppState::WorldGen
            && matches!(*app.get(), AppState::Setup);
        // Brief Setup flash while `AppState::WorldGen` + active world-gen must not yank back to title.
        let block_setup_pull_during_worldgen = mirrored_app == AppState::Setup
            && matches!(*app.get(), AppState::WorldGen)
            && super::worldgen_lifecycle_active(worldgen.get());
        if !block_world_gen_pull
            && !block_menu_pull
            && !block_world_gen_from_main_menu
            && !block_setup_pull_during_worldgen
        {
            next_app.set(mirrored_app);
        }
    }
    let wg_sync_blocked = matches!(*worldgen.get(), WorldGenState::Dismissed)
        && latch.full_ready_dismissed
        && !matches!(
            mirrored_wg,
            WorldGenState::Preview | WorldGenState::Generating | WorldGenState::Ready
        );
    // Post-dismiss in-game: freeze WG at Dismissed — do not ping-pong Ready↔Dismissed (flow PreviewReady/FullReady flicker).
    if !in_game_after_dismiss
        && *worldgen.get() != mirrored_wg
        && !wg_sync_blocked
    {
        next_wg.set(mirrored_wg);
    }
    if *pause.get() != mirrored_pause {
        next_pause.set(mirrored_pause);
    }
}

/// Push authoritative UX states into legacy Bevy states (single writer for downstream systems).
fn bridge_ux_to_legacy(
    app: Res<State<AppState>>,
    worldgen: Res<State<WorldGenState>>,
    flow: Res<State<WorldGenFlowState>>,
    pause: Res<State<PauseState>>,
    latch: Res<WorldGenChromeLatch>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_menu: ResMut<NextState<InGameMenuState>>,
    mut pending: ResMut<PauseMenuPendingAction>,
) {
    let base = match app.get() {
        // Stale Setup while in-game must not resurrect the Bevy main-menu shell.
        AppState::Setup if latch.full_ready_dismissed => BaseState::Simulation,
        AppState::Setup => BaseState::MainMenu,
        AppState::WorldGen => BaseState::Editor,
        AppState::InGame | AppState::Paused => BaseState::Simulation,
        AppState::Shutdown => BaseState::Shutdown,
    };
    NextState::set_if_neq(&mut *next_base, base);

    let current_flow = *flow.get();
    let in_game_after_dismiss = latch.full_ready_dismissed
        && matches!(*app.get(), AppState::InGame | AppState::Paused);
    let flow = match worldgen.get() {
        WorldGenState::Idle => WorldGenFlowState::Idle,
        // Keep legacy flow at FullReady after dismiss so Idle↔FullReady does not re-enter OnEnter.
        WorldGenState::Dismissed if latch.full_ready_dismissed => WorldGenFlowState::FullReady,
        WorldGenState::Dismissed => WorldGenFlowState::Idle,
        WorldGenState::Preview => legacy_flow_for_worldgen_preview(*flow.get()),
        WorldGenState::Generating => legacy_flow_for_worldgen_generating(*flow.get()),
        WorldGenState::Ready => {
            if in_game_after_dismiss {
                WorldGenFlowState::FullReady
            } else {
                WorldGenFlowState::PreviewReady
            }
        }
        WorldGenState::FullReady => WorldGenFlowState::FullReady,
    };
    if in_game_after_dismiss {
        if current_flow != WorldGenFlowState::FullReady {
            next_flow.set(WorldGenFlowState::FullReady);
        }
    } else {
        NextState::set_if_neq(&mut *next_flow, flow);
    }

    match pause.get() {
        PauseState::Off => {
            pending.clear();
            if *app.get() != AppState::Paused {
                NextState::set_if_neq(&mut *next_menu, InGameMenuState::Normal);
            }
        }
        PauseState::Menu => {
            pending.clear();
            NextState::set_if_neq(&mut *next_menu, InGameMenuState::Pause);
        }
        PauseState::ConfirmExit => {
            NextState::set_if_neq(&mut *next_menu, InGameMenuState::Pause);
        }
    }
}

fn ux_on_enter_setup(
    flow: Res<State<WorldGenFlowState>>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    mut world_gen_ui: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<crate::gui::editor::world_preview::WorldPreviewUiState>,
    mut lifecycle: ResMut<crate::gui::editor::world_preview::WorldPreviewLifecycle>,
) {
    // Only tear down chrome on the real title screen — not transient `App=Setup` during UX/legacy fights.
    if !matches!(
        *flow.get(),
        WorldGenFlowState::Idle | WorldGenFlowState::LoadingSave
    ) {
        return;
    }
    next_wg.set(WorldGenState::Idle);
    world_gen_ui.visible = false;
    preview_ui.window_open = false;
    lifecycle.park_uninitialized();
}

fn ux_on_enter_world_gen(
    worldgen: Res<State<WorldGenState>>,
    latch: Res<WorldGenChromeLatch>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    _world_gen_ui: ResMut<WorldGenUiState>,
    _preview_ui: ResMut<WorldPreviewUiState>,
) {
    if latch.full_ready_dismissed {
        return;
    }
    if matches!(*worldgen.get(), WorldGenState::Idle) {
        next_wg.set(WorldGenState::Preview);
    }
    // Chrome opens from `ux_begin_world_gen_from_menu` + `open_world_gen_chrome_on_new_world_setup` only.
    // Do not force panels here — re-entrant OnEnter(WorldGen) during UX/legacy fights caused flicker.
}

fn ux_on_enter_in_game(
    mut next_pause: ResMut<NextState<PauseState>>,
    mut next_base: ResMut<NextState<BaseState>>,
    worldgen: Res<State<WorldGenState>>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    mut world_gen_ui: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
    mut lifecycle: ResMut<crate::gui::editor::world_preview::WorldPreviewLifecycle>,
    mut latch: ResMut<WorldGenChromeLatch>,
) {
    next_pause.set(PauseState::Off);
    NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
    if matches!(*worldgen.get(), WorldGenState::FullReady) {
        next_wg.set(WorldGenState::Dismissed);
    }
    crate::gui::editor::world_preview::dismiss_world_gen_preview_chrome(
        &mut world_gen_ui,
        &mut preview_ui,
        &mut lifecycle,
        &mut latch,
        "ux_on_enter_in_game",
    );
}

fn ux_on_enter_paused(mut next_pause: ResMut<NextState<PauseState>>) {
    next_pause.set(PauseState::Menu);
}

fn ux_on_enter_shutdown(mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

/// Operator confirmed **Enter world** or auto FullReady — dismiss chrome and enter in-game UX.
pub fn ux_enter_world_from_world_gen(
    latch: &mut WorldGenChromeLatch,
    next_app: &mut NextState<AppState>,
    next_wg: &mut NextState<WorldGenState>,
    next_pause: &mut NextState<PauseState>,
    next_base: &mut NextState<BaseState>,
    world_gen_ui: &mut WorldGenUiState,
    preview_ui: &mut crate::gui::editor::world_preview::WorldPreviewUiState,
    lifecycle: &mut crate::gui::editor::world_preview::WorldPreviewLifecycle,
) {
    crate::gui::editor::world_preview::dismiss_world_gen_preview_chrome(
        world_gen_ui,
        preview_ui,
        lifecycle,
        latch,
        "ux_enter_world",
    );
    next_pause.set(PauseState::Off);
    next_app.set(AppState::InGame);
    next_wg.set(WorldGenState::Dismissed);
    NextState::set_if_neq(next_base, BaseState::Simulation);
}

/// FINISH-UX-03: FullReady → enter game + dismiss chrome (via Dismissed + existing OnEnter handlers).
fn ux_on_enter_worldgen_full_ready(
    pending: Option<Res<DebugQuickWorldGenPending>>,
    harness: Option<Res<TestWorldHarness>>,
    mut latch: ResMut<WorldGenChromeLatch>,
    mut next_app: ResMut<NextState<AppState>>,
    mut next_wg: ResMut<NextState<WorldGenState>>,
    mut next_pause: ResMut<NextState<PauseState>>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut world_gen_ui: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<crate::gui::editor::world_preview::WorldPreviewUiState>,
    mut lifecycle: ResMut<crate::gui::editor::world_preview::WorldPreviewLifecycle>,
) {
    if pending.is_some_and(|p| p.active) {
        info!(
            target: "app_shell",
            "FullReady: defer auto-enter — menu/CLI debug bootstrap owns sim entry"
        );
        return;
    }
    if harness.is_some_and(|h| h.active && !h.finished) {
        info!(
            target: "test_harness",
            "FullReady: defer auto-enter — test harness bootstrap owns sim entry"
        );
        return;
    }
    ux_enter_world_from_world_gen(
        &mut latch,
        &mut next_app,
        &mut next_wg,
        &mut next_pause,
        &mut next_base,
        &mut world_gen_ui,
        &mut preview_ui,
        &mut lifecycle,
    );
    info!(
        target: "ux::worldgen",
        "FullReady: UX → InGame + Dismissed (auto-close gen/preview chrome)"
    );
}

/// FINISH-UX-07: park preview lifecycle when dismissed.
fn ux_on_enter_worldgen_dismissed(
    mut world_gen_ui: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<crate::gui::editor::world_preview::WorldPreviewUiState>,
    mut lifecycle: ResMut<crate::gui::editor::world_preview::WorldPreviewLifecycle>,
) {
    world_gen_ui.visible = false;
    preview_ui.window_open = false;
    lifecycle.park_uninitialized();
}

/// FINISH-UX-06: spike detection for throttling preview / heavy paths.
fn ux_frame_spike_watchdog(
    budget: Option<Res<FrameBudgetDiagnostics>>,
    mut guard: ResMut<UxFrameSpikeGuard>,
) {
    let frame_ms = budget.map(|b| b.frame_time_ms).unwrap_or(0.0);
    let was_spike = guard.spike_active;
    guard.last_frame_ms = frame_ms;
    guard.suppress_preview_this_frame = false;
    guard.suppress_optional_diagnostics = false;
    if frame_ms > guard.max_ms {
        guard.spike_over_budget_streak = guard.spike_over_budget_streak.saturating_add(1);
    } else {
        guard.spike_over_budget_streak = 0;
    }
    guard.spike_active =
        guard.spike_over_budget_streak >= guard.spike_enter_frames.max(1);
    UX_SPIKE_ACTIVE.store(guard.spike_active, Ordering::Relaxed);
    if guard.spike_active {
        guard.suppress_preview_this_frame = true;
        guard.suppress_optional_diagnostics = true;
        if !was_spike {
            warn!(
                target: "ux::perf",
                "frame spike {:.1}ms > {:.1}ms for {} frames — suppress_preview",
                frame_ms,
                guard.max_ms,
                guard.spike_over_budget_streak
            );
        }
    }
}

fn legacy_to_app_state(base: BaseState, menu: InGameMenuState) -> AppState {
    if menu == InGameMenuState::Pause {
        return AppState::Paused;
    }
    match base {
        BaseState::MainMenu => AppState::Setup,
        BaseState::Editor => AppState::WorldGen,
        BaseState::Simulation => AppState::InGame,
        BaseState::Shutdown => AppState::Shutdown,
    }
}

fn legacy_to_worldgen_state(flow: WorldGenFlowState, job_busy: bool) -> WorldGenState {
    if job_busy && matches!(flow, WorldGenFlowState::NewWorldSetup) {
        return WorldGenState::Generating;
    }
    match flow {
        WorldGenFlowState::Idle | WorldGenFlowState::LoadingSave => WorldGenState::Idle,
        WorldGenFlowState::NewWorldSetup => WorldGenState::Preview,
        WorldGenFlowState::PreviewReady => WorldGenState::Ready,
        WorldGenFlowState::FullReady => WorldGenState::FullReady,
    }
}

fn legacy_to_pause_state(menu: InGameMenuState) -> PauseState {
    if menu == InGameMenuState::Pause {
        PauseState::Menu
    } else {
        PauseState::Off
    }
}

/// Public hook: app shell / menus set UX when starting world gen from main menu.
pub fn ux_begin_world_gen_from_menu(
    next_app: &mut NextState<AppState>,
    next_wg: &mut NextState<WorldGenState>,
    next_base: &mut NextState<BaseState>,
    next_flow: &mut NextState<WorldGenFlowState>,
    latch: &mut WorldGenChromeLatch,
    world_gen_ui: &mut WorldGenUiState,
    preview_ui: &mut crate::gui::editor::world_preview::WorldPreviewUiState,
) {
    latch.reset_for_new_flow();
    next_app.set(AppState::WorldGen);
    next_wg.set(WorldGenState::Preview);
    NextState::set_if_neq(next_base, BaseState::Editor);
    next_flow.set(WorldGenFlowState::NewWorldSetup);
    world_gen_ui.visible = true;
    preview_ui.window_open = true;
    crate::engine::worldgen_chrome_debug::log_chrome_open(
        "ux_begin_world_gen_from_menu",
        world_gen_ui.visible,
        preview_ui.window_open,
    );
}

/// Pause / shell hook: leave simulation and show the Bevy main menu again.
pub fn ux_return_to_main_menu(
    next_app: &mut NextState<AppState>,
    next_wg: &mut NextState<WorldGenState>,
    next_pause: &mut NextState<PauseState>,
    latch: &mut WorldGenChromeLatch,
) {
    latch.reset_for_new_flow();
    next_pause.set(PauseState::Off);
    next_app.set(AppState::Setup);
    next_wg.set(WorldGenState::Idle);
}

/// Public hook: pause menu confirm exit → shutdown or resume.
pub fn ux_pause_confirm_exit_to_shutdown(mut next_app: ResMut<NextState<AppState>>) {
    next_app.set(AppState::Shutdown);
}

pub fn ux_pause_resume(mut next_app: ResMut<NextState<AppState>>, mut next_pause: ResMut<NextState<PauseState>>) {
    next_app.set(AppState::InGame);
    next_pause.set(PauseState::Off);
}

/// Legacy flow while UX is in preview setup — re-arm stale FullReady only; keep PreviewReady for full gen.
#[must_use]
pub const fn legacy_flow_for_worldgen_preview(current_flow: WorldGenFlowState) -> WorldGenFlowState {
    match current_flow {
        WorldGenFlowState::PreviewReady => WorldGenFlowState::PreviewReady,
        WorldGenFlowState::FullReady => WorldGenFlowState::NewWorldSetup,
        _ => WorldGenFlowState::NewWorldSetup,
    }
}

/// Legacy flow while a preview/full job is running — preserve preview-ready only.
#[must_use]
pub const fn legacy_flow_for_worldgen_generating(current_flow: WorldGenFlowState) -> WorldGenFlowState {
    match current_flow {
        WorldGenFlowState::PreviewReady => current_flow,
        // Stale FullReady from a prior world must not block a fresh preview job.
        WorldGenFlowState::FullReady => WorldGenFlowState::NewWorldSetup,
        _ => WorldGenFlowState::NewWorldSetup,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::states::WorldGenFlowState;

    #[test]
    fn preview_rearms_stale_full_ready_flow() {
        assert_eq!(
            legacy_flow_for_worldgen_preview(WorldGenFlowState::FullReady),
            WorldGenFlowState::NewWorldSetup
        );
    }

    #[test]
    fn preview_preserves_preview_ready_for_full_gen_handoff() {
        assert_eq!(
            legacy_flow_for_worldgen_preview(WorldGenFlowState::PreviewReady),
            WorldGenFlowState::PreviewReady
        );
    }

    #[test]
    fn generating_preserves_preview_ready() {
        assert_eq!(
            legacy_flow_for_worldgen_generating(WorldGenFlowState::PreviewReady),
            WorldGenFlowState::PreviewReady
        );
    }

    #[test]
    fn generating_rearms_stale_full_ready() {
        assert_eq!(
            legacy_flow_for_worldgen_generating(WorldGenFlowState::FullReady),
            WorldGenFlowState::NewWorldSetup
        );
    }
}
