//! HUD / shell developer overlay — read-only diagnostics.

use bevy::prelude::*;

use crate::gui::{
    ActiveMapViewInput, MapFitValidationLog, MapPresentationDiagnostics, MapViewInstanceId,
    MapViewTextureCache, MinimapShellState, PerViewLodHints, ViewManager,
};
use crate::render::DebugRenderTraceConfig;
use crate::systems::weather::WeatherVisualSettings;
use crate::gui::style::{
    native_ui_pixels_per_point, resolve_ui_scale, HudDensityProfile, UiPalette,
    DEFAULT_UI_GLOBAL_SCALE, UI_GLOBAL_SCALE_STEP,
};
use super::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics, FRAME_HISTORY_LEN};
use super::hud_async_task_queue::HudAsyncTaskQueue;
use super::hud_interaction_budget::HudFrameBudget;
use super::interaction_latency::InteractionLatencyMetrics;
use super::retained_widget_cache::RetainedWidgetCache;
use super::layout_store::HudLayoutStore;
use super::player_intent_panel::PlayerIntentPanelState;
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::ShellWidgetDiagnostics;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::shell_diagnostics::ProductShellDiagnostics;
use super::shell_framework::{HudDockRegistry, HudWidgetId};
use super::viewport_rect_sanity::ViewportRectSanity;
use super::world_interaction_diagnostics::WorldInteractionDiagnostics;

#[derive(Resource, Clone, Debug)]
pub struct HudDevOverlayState {
    pub visible: bool,
    pub show_ui_timing: bool,
    pub show_viewport_mismatch: bool,
    pub show_texture_refresh: bool,
    pub show_map_transform: bool,
}

impl Default for HudDevOverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            show_ui_timing: true,
            show_viewport_mismatch: true,
            show_texture_refresh: true,
            show_map_transform: true,
        }
    }
}

pub struct HudDevOverlayPlugin;

impl Plugin for HudDevOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudDevOverlayState>()
            .add_systems(Update, (hud_dev_overlay_keyboard_toggle, hud_ui_scale_keyboard));
    }
}

fn hud_dev_overlay_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HudDevOverlayState>,
) {
    if keys.just_pressed(KeyCode::F4) {
        state.visible = !state.visible;
    }
}

fn hud_ui_scale_keyboard(keys: Res<ButtonInput<KeyCode>>, mut density: ResMut<HudDensityProfile>) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        density.adjust_global_scale(UI_GLOBAL_SCALE_STEP);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        density.adjust_global_scale(-UI_GLOBAL_SCALE_STEP);
    }
}

fn draw_frame_time_graph() {}

