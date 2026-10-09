use bevy::prelude::*;

use crate::engine::{AppState, WorldGenFlowState};
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::editor::world_preview::{open_world_gen_workspace, world_preview_unified_workspace};

// Re-export for `tilemap_adapter` and other call sites.
pub use crate::gui::editor::world_preview::PreviewLayers;

#[derive(Resource)]
pub struct WorldGenUiState {
    pub visible: bool,
    /// D-01 / D-04: generator slide sheet inside unified workspace (body deferred to LAYOUT-002).
    pub generator_sheet_open: bool,
    pub preview_layers: PreviewLayers,
    /// Index into [`MobilityProfileRegistry::profiles`](crate::terrain::mobility::MobilityProfileRegistry).
    pub mobility_profile_index: usize,
    /// Close panel while generation is running — confirm before dismiss + cleanup.
    pub confirm_close_pending: bool,
}

impl Default for WorldGenUiState {
    fn default() -> Self {
        Self {
            visible: false,
            generator_sheet_open: false,
            preview_layers: PreviewLayers::default(),
            mobility_profile_index: 0,
            confirm_close_pending: false,
        }
    }
}

// Toggle event for the world gen UI
#[derive(Message)]
pub struct ToggleWorldGenUiEvent;

/// Operator confirmed close while generation was active — cancel job + release preview chrome.
#[derive(Message)]
pub struct CancelActiveWorldGenEvent;

// System to toggle the world gen UI
pub fn toggle_world_gen_ui_system(
    base: Res<State<crate::engine::states::BaseState>>,
    mut events: MessageReader<ToggleWorldGenUiEvent>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
    flow: Res<State<WorldGenFlowState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
) {
    if matches!(*base.get(), crate::engine::states::BaseState::Simulation) {
        return;
    }
    for _ in events.read() {
        let will_show = !(world_gen_ui_state.visible && preview_ui.window_open);
        if will_show && matches!(*flow.get(), WorldGenFlowState::Idle) {
            NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::NewWorldSetup);
        }
        if will_show {
            open_world_gen_workspace(&mut world_gen_ui_state, &mut preview_ui);
        } else {
            world_gen_ui_state.visible = false;
            world_gen_ui_state.generator_sheet_open = false;
            preview_ui.window_open = false;
        }
        if will_show {
            crate::engine::worldgen_chrome_debug::log_chrome_open(
                "toggle_world_gen_f8",
                world_gen_ui_state.visible,
                preview_ui.window_open,
            );
        }
    }
}

/// After full generation completes, dismiss tooling chrome until Escape/pause reopens it.
fn dismiss_world_gen_chrome_after_full_ready(
    mut latch: ResMut<crate::engine::WorldGenChromeLatch>,
    mut world_gen_ui: ResMut<WorldGenUiState>,
    mut world_preview_ui: ResMut<WorldPreviewUiState>,
    mut lifecycle: ResMut<crate::gui::editor::world_preview::WorldPreviewLifecycle>,
) {
    if latch.full_ready_dismissed {
        return;
    }
    crate::gui::editor::world_preview::dismiss_world_gen_preview_chrome(
        &mut world_gen_ui,
        &mut world_preview_ui,
        &mut lifecycle,
        &mut latch,
        "flow_full_ready",
    );
    info!(
        target: "world_gen::flow",
        "FullReady: auto-dismissed World Generator panel and World Preview window (reopen via Escape → pause or F8)"
    );
}

/// PLAY-02c: skip egui pass when hidden or in active simulation (sim enter dismisses chrome).
#[must_use]
pub fn world_gen_ui_chrome_visible(
    app: Res<State<AppState>>,
    state: Res<WorldGenUiState>,
    preview: Res<WorldPreviewUiState>,
) -> bool {
    crate::gui::editor::world_preview::world_gen_ux_app_active(app)
        && state.visible
        && !world_preview_unified_workspace(&preview)
}

// Plugin to register all world generation UI components
pub struct WorldGenUiPlugin;

impl Plugin for WorldGenUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenUiState>()
            .init_resource::<crate::gui::editor::world_preview::world_gen_bevy_shell::WorldGenTuningNote>()
            .add_message::<ToggleWorldGenUiEvent>()
            .add_systems(
                OnEnter(WorldGenFlowState::FullReady),
                dismiss_world_gen_chrome_after_full_ready,
            )
            .add_systems(Update, toggle_world_gen_ui_system)
            .add_systems(
                OnEnter(WorldGenFlowState::NewWorldSetup),
                crate::gui::editor::world_preview::open_world_gen_chrome_on_new_world_setup,
            )
            .add_systems(
                OnEnter(AppState::WorldGen),
                crate::gui::editor::world_preview::world_gen_bevy_shell::spawn_world_gen_bevy_shell,
            )
            .add_systems(
                OnExit(AppState::WorldGen),
                crate::gui::editor::world_preview::world_gen_bevy_shell::despawn_world_gen_bevy_shell,
            )
            .add_systems(
                Update,
                (
                    crate::gui::editor::world_preview::world_gen_bevy_shell::sync_world_gen_bevy_shell,
                    crate::gui::editor::world_preview::world_gen_bevy_shell::world_gen_bevy_shell_actions,
                )
                    .chain()
                    .run_if(in_state(AppState::WorldGen))
                    .in_set(crate::gui::ViewRepresentationSystemSet::UiCollect),
            );
    }
}