//! Interaction-frame budget — defer heavy HUD panels when the UI pass overruns.

use bevy::prelude::*;

use super::shell_framework::ProductShellWidgetId;

pub const DEFAULT_UI_BUDGET_MS: f32 = 8.0;
pub const DEFAULT_ASYNC_BUDGET_MS: f32 = 2.0;

#[derive(Resource, Clone, Debug)]
pub struct HudFrameBudget {
    pub ui_budget_ms: f32,
    pub async_budget_ms: f32,
    pub deferred_widgets: Vec<ProductShellWidgetId>,
    pub overruns_frame: u32,
    pub overruns_total: u64,
    pub deferred_widget_count_frame: u32,
    pub worst_offender: Option<ProductShellWidgetId>,
    pub worst_offender_ms: f32,
    pub last_frame_ms: f32,
    pub dynamic_background_hz: f32,
}

impl Default for HudFrameBudget {
    fn default() -> Self {
        Self {
            ui_budget_ms: DEFAULT_UI_BUDGET_MS,
            async_budget_ms: DEFAULT_ASYNC_BUDGET_MS,
            deferred_widgets: Vec::new(),
            overruns_frame: 0,
            overruns_total: 0,
            deferred_widget_count_frame: 0,
            worst_offender: None,
            worst_offender_ms: 0.0,
            last_frame_ms: 0.0,
            dynamic_background_hz: super::shell_update_budget::BACKGROUND_PANEL_HZ,
        }
    }
}

impl HudFrameBudget {
    pub fn begin_frame(&mut self) {
        self.deferred_widgets.clear();
        self.overruns_frame = 0;
        self.deferred_widget_count_frame = 0;
        self.worst_offender = None;
        self.worst_offender_ms = 0.0;
    }

    pub fn record_widget_ms(&mut self, id: ProductShellWidgetId, ms: f32) {
        if ms > self.worst_offender_ms {
            self.worst_offender_ms = ms;
            self.worst_offender = Some(id);
        }
    }

    pub fn should_defer(&self, id: ProductShellWidgetId, frame_ms: f32) -> bool {
        if frame_ms <= self.ui_budget_ms {
            return false;
        }
        matches!(
            id,
            ProductShellWidgetId::Explainability
                | ProductShellWidgetId::IntelTimeline
                | ProductShellWidgetId::CommandShell
                | ProductShellWidgetId::OverlaysPanel
        )
    }

    pub fn note_deferred(&mut self, id: ProductShellWidgetId) {
        if !self.deferred_widgets.contains(&id) {
            self.deferred_widgets.push(id);
        }
        self.deferred_widget_count_frame = self.deferred_widget_count_frame.saturating_add(1);
    }

    pub fn finalize_frame(&mut self, frame_ms: f32) {
        self.last_frame_ms = frame_ms;
        if frame_ms > self.ui_budget_ms {
            self.overruns_frame = self.overruns_frame.saturating_add(1);
            self.overruns_total = self.overruns_total.wrapping_add(1);
            self.dynamic_background_hz = (self.dynamic_background_hz * 0.85).max(2.0);
        } else {
            self.dynamic_background_hz = (self.dynamic_background_hz * 1.02)
                .min(super::shell_update_budget::BACKGROUND_PANEL_HZ);
        }
    }
}

pub fn apply_hud_interaction_frame_budget(
    mut budget: ResMut<HudFrameBudget>,
    frame_diag: Res<super::frame_budget_diagnostics::FrameBudgetDiagnostics>,
) {
    let frame_ms = frame_diag.egui_frame_ms.max(budget.last_frame_ms);
    budget.finalize_frame(frame_ms);
}
