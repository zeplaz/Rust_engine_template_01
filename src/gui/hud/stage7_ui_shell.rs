//! Stage-7 UI scaffolding — DTO/mock viewers only (no comm authority).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::std_floating;
use crate::strategic::{
    BeliefSnapshotDto, CommunicationPlane, DispatchEnvelope, DispatchMessage, IntelConfidence,
    OverlayChannelDescriptor, UtilityChannel,
};

use super::explainability_viewer::{draw_explainability_viewer, ExplainabilityViewerState};
use super::hud_async_task_queue::{HudAsyncTask, HudAsyncTaskQueue};
use super::virtualized_list::draw_virtualized_rows;
use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Debug, Default)]
pub struct Stage7UiShellState {
    pub intel_timeline_open: bool,
    pub comms_panel_open: bool,
    pub dispatch_log_open: bool,
    pub explainability_open: bool,
}

pub struct Stage7UiShellPlugin;

impl Plugin for Stage7UiShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stage7UiShellState>()
            .init_resource::<ExplainabilityViewerState>();
    }
}

pub fn draw_stage7_ui_shell_egui(
    ctx: &mut egui::Context,
    shell: &mut Stage7UiShellState,
    explainability: &mut ExplainabilityViewerState,
    async_queue: &mut HudAsyncTaskQueue,
) {
    if !(shell.intel_timeline_open
        || shell.comms_panel_open
        || shell.dispatch_log_open
        || shell.explainability_open)
    {
        return;
    }
    std_floating(egui::Window::new("Stage-7 UI (mock)"))
        .id(egui::Id::new("stage7_ui_shell"))
        .default_pos(egui::pos2(480.0, 120.0))
        .default_size([320.0, 260.0])
        .show(ctx, |ui| {
            ui.checkbox(&mut shell.intel_timeline_open, "Intel timeline");
            ui.checkbox(&mut shell.comms_panel_open, "Comms panel");
            ui.checkbox(&mut shell.dispatch_log_open, "Dispatch log");
            if ui
                .checkbox(&mut shell.explainability_open, "Explainability replay")
                .clicked()
                && shell.explainability_open
            {
                async_queue.enqueue(HudAsyncTask::ExplainabilityTransform {
                    beliefs: mock_belief_snapshots(),
                });
            }
            ui.separator();
            if shell.intel_timeline_open {
                ui.label(egui::RichText::new("Intel timeline (mock)").strong());
                for row in mock_belief_snapshots() {
                    ui.label(format!(
                        "entity {} · conf {:.0}% · {}",
                        row.entity_bits,
                        row.confidence.scalar * 100.0,
                        row.summary
                    ));
                }
            }
            if shell.comms_panel_open {
                ui.separator();
                ui.label(egui::RichText::new("Comms planes (mock)").strong());
                for plane in [
                    CommunicationPlane::StrategicCommand,
                    CommunicationPlane::LogisticsHub,
                    CommunicationPlane::SensorRelay,
                    CommunicationPlane::TacticalLine,
                ] {
                    ui.label(format!("{:?} · {:?}", plane, plane.authority()));
                }
            }
            if shell.dispatch_log_open {
                ui.separator();
                ui.label(egui::RichText::new("Dispatch log (mock)").strong());
                if async_queue.cache.dispatch_log_lines.is_empty() {
                    async_queue.enqueue(HudAsyncTask::DispatchLogFormat);
                }
                draw_virtualized_rows(
                    ui,
                    "stage7_dispatch_log",
                    18.0,
                    120.0,
                    async_queue.cache.dispatch_log_lines.len(),
                    |ui, row| {
                        ui.label(&async_queue.cache.dispatch_log_lines[row]);
                    },
                );
            }
            if shell.explainability_open {
                ui.separator();
                if async_queue.cache.explainability_lines.is_empty() {
                    draw_explainability_viewer(ui, explainability);
                } else {
                    for line in &async_queue.cache.explainability_lines {
                        ui.label(line);
                    }
                }
            }
        });
}

#[must_use]
pub fn mock_belief_snapshots() -> Vec<BeliefSnapshotDto> {
    vec![
        BeliefSnapshotDto {
            entity_bits: 0xA1,
            confidence: IntelConfidence {
                scalar: 0.72,
                half_life_ticks: 120,
            },
            last_refresh: SimStepStamp::new(12, 0),
            summary: "Corridor pressure rising".into(),
        },
        BeliefSnapshotDto {
            entity_bits: 0xB2,
            confidence: IntelConfidence {
                scalar: 0.41,
                half_life_ticks: 60,
            },
            last_refresh: SimStepStamp::new(10, 0),
            summary: "Logistics hub contested".into(),
        },
    ]
}

#[must_use]
pub fn mock_dispatch_envelopes() -> Vec<DispatchEnvelope> {
    vec![DispatchEnvelope {
        message: DispatchMessage {
            plane: CommunicationPlane::StrategicCommand,
            issued_at: SimStepStamp::new(4, 0),
            deliver_after: SimStepStamp::new(6, 0),
            command_id: 1,
            summary: "Secure corridor".into(),
        },
        loss_probability: 0.08,
        corruption_hint: 0.1,
    }]
}

#[must_use]
pub fn mock_utility_overlay_rows() -> Vec<OverlayChannelDescriptor> {
    vec![OverlayChannelDescriptor {
        utility: UtilityChannel::Threat,
        overlay: crate::strategic::StrategicOverlayType::Threat,
        color_rgb: [220, 96, 96],
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage7_mock_dispatch_envelope_is_stable() {
        assert_eq!(mock_dispatch_envelopes().len(), 1);
    }
}
