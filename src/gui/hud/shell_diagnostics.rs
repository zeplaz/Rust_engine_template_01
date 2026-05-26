//! Lightweight product-shell diagnostics (consumer-only).

use std::collections::HashMap;

use bevy::prelude::*;

use super::shell_framework::ProductShellWidgetId;
use super::viewport_rect_sanity::{ViewportRectIssueKind, ViewportRectSource};

#[derive(Resource, Clone, Debug, Default)]
pub struct ProductShellDiagnostics {
    /// Lifetime product-shell egui passes (editor sessions can inflate before sim proof).
    pub egui_pass_count: u64,
    /// **UI-P2B-CODER-B** — passes since last `OnEnter(Simulation)`; used for `phase2b_closed`.
    pub egui_pass_count_sim_session: u64,
    pub last_frame_delta_secs: f32,
    pub texture_rebuilds: HashMap<ProductShellWidgetId, u64>,
    pub visible_widgets: HashMap<ProductShellWidgetId, bool>,
    pub viewport_rect_issues: HashMap<ViewportRectSource, u64>,
    pub last_viewport_rect_issue: Option<(ViewportRectSource, ViewportRectIssueKind)>,
}

impl ProductShellDiagnostics {
    /// **UI-P2B-CODER-B** — clear cumulative counter at sim enter (PLAN-UI-SHELL-2B-001).
    pub fn reset_egui_pass_count_for_simulation_session(&mut self) {
        self.egui_pass_count = 0;
        self.egui_pass_count_sim_session = 0;
    }

    pub fn record_egui_pass(&mut self) {
        self.egui_pass_count = self.egui_pass_count.wrapping_add(1);
    }

    pub fn record_egui_pass_in_simulation(&mut self) {
        self.record_egui_pass();
        self.egui_pass_count_sim_session = self.egui_pass_count_sim_session.wrapping_add(1);
    }

    pub fn bump_texture_rebuild(&mut self, id: ProductShellWidgetId) {
        *self.texture_rebuilds.entry(id).or_insert(0) += 1;
    }

    pub fn set_widget_visible(&mut self, id: ProductShellWidgetId, visible: bool) {
        self.visible_widgets.insert(id, visible);
    }

    pub fn record_viewport_rect_issue(
        &mut self,
        source: ViewportRectSource,
        kind: ViewportRectIssueKind,
    ) {
        *self.viewport_rect_issues.entry(source).or_insert(0) += 1;
        self.last_viewport_rect_issue = Some((source, kind));
    }

    #[must_use]
    pub fn texture_rebuild_count(&self, id: ProductShellWidgetId) -> u64 {
        self.texture_rebuilds.get(&id).copied().unwrap_or(0)
    }
}

pub fn product_shell_diagnostics_tick(time: Res<Time>, mut diag: ResMut<ProductShellDiagnostics>) {
    diag.last_frame_delta_secs = time.delta_secs();
}
