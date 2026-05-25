//! Overlay shell — toggle groups + legend widgets (mock consumers only).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::style::UiPalette;
use crate::gui::world_representation::WorldRepresentationFrame;
use crate::render::AppStage5ReadinessReport;

use super::info_tabs::{
    draw_info_tab_bar, draw_info_tab_body, HudInfoLiveData, HudInfoTabState,
};
use super::layout_store::HudLayoutStore;
use super::overlay_framework::OverlayFrameworkState;
use super::shell_framework::{
    capture_shell_layout, shell_default_window_pos, shell_widget_runs_egui_with_budget,
    show_product_shell_window, HudDockRegistry, HudWidgetId, ShellWindowHost,
};
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::ShellWidgetDiagnostics;
use super::pending_hud_layout_commit::PendingHudLayoutCommit;
use super::retained_widget_cache::RetainedWidgetCache;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlayToggleGroup {
    Threat,
    Logistics,
    Recon,
    Utility,
}

#[derive(Resource, Clone, Debug)]
pub struct OverlayShellState {
    pub groups: [bool; 4],
    pub legend_open: bool,
}

impl Default for OverlayShellState {
    fn default() -> Self {
        Self {
            groups: [false, false, false, false],
            legend_open: false,
        }
    }
}

impl OverlayShellState {
    pub fn group_mut(&mut self, group: OverlayToggleGroup) -> &mut bool {
        match group {
            OverlayToggleGroup::Threat => &mut self.groups[0],
            OverlayToggleGroup::Logistics => &mut self.groups[1],
            OverlayToggleGroup::Recon => &mut self.groups[2],
            OverlayToggleGroup::Utility => &mut self.groups[3],
        }
    }
}

#[must_use]
pub fn mock_overlay_channel_descriptors() -> Vec<crate::strategic::OverlayChannelDescriptor> {
    super::overlay_framework::default_overlay_channel_runtimes()
        .into_iter()
        .map(|row| row.descriptor)
        .collect()
}

pub struct OverlayShellPlugin;

impl Plugin for OverlayShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OverlayFrameworkState>()
            .init_resource::<OverlayShellState>()
            .add_plugins(super::info_tabs::HudInfoTabPlugin);
    }
}

pub fn draw_overlay_shell_egui(
    ctx: &mut egui::Context,
    palette: &UiPalette,
    shell: &mut OverlayShellState,
    framework: &mut OverlayFrameworkState,
    tabs: &mut HudInfoTabState,
    dock: &mut HudDockRegistry,
    layout_store: &mut HudLayoutStore,
    update_budget: &mut ProductShellUpdateBudget,
    now_secs: f32,
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
    minimap_legend: Option<&str>,
    _retained: &mut RetainedWidgetCache,
    _legend_revision: u64,
    pending_layout: &mut PendingHudLayoutCommit,
    world: Option<&WorldRepresentationFrame>,
    readiness: Option<&AppStage5ReadinessReport>,
    live: Option<&HudInfoLiveData>,
) {
    let mut open = dock.slot(HudWidgetId::OverlaysPanel).visible;
    if !shell_widget_runs_egui_with_budget(
        dock,
        HudWidgetId::OverlaysPanel,
        open,
        Some(update_budget),
        now_secs,
    ) {
        return;
    }
    let default_size = [300.0, 280.0];
    if let Some(response) = show_product_shell_window(
        ctx,
        ShellWindowHost {
            id: HudWidgetId::OverlaysPanel,
            title: "Info & overlays",
            default_pos: shell_default_window_pos(ctx, HudWidgetId::OverlaysPanel, default_size),
            default_size,
            min_size: [240.0, 200.0],
        },
        layout_store,
        dock,
        &mut open,
        |ui| {
            draw_info_tab_bar(ui, palette, tabs);
            draw_info_tab_body(
                ui,
                palette,
                tabs.active,
                tabs,
                shell,
                framework,
                world,
                readiness,
                live,
                minimap_legend,
            );
        },
        widget_timing,
    ) {
        capture_shell_layout(layout_store, HudWidgetId::OverlaysPanel, &response, Some(pending_layout));
    }
    dock.slot_mut(HudWidgetId::OverlaysPanel).visible = open;
}
