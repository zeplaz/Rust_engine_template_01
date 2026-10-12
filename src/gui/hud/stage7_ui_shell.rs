//! Stage-7 UI scaffolding — DTO/mock viewers only (no comm authority).

use bevy::prelude::*;

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
