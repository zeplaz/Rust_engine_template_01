//! Transmission media provider abstraction — static/fake/ticker only (no decode authority).
//!
//! UX-E03 / S7B-DESIGN-003: [`crate::dev::ux_e03_transmission_shell_note_v1`](../../dev/ux_e03_transmission_shell_note_v1.md).

use bevy::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransmissionMediaProviderKind {
    StaticText,
    StaticImage {
        asset_label: String,
    },
    FakeVideoFrames {
        asset_label: String,
        frame_count: u32,
    },
    TextTicker {
        label: String,
    },
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TransmissionMediaProviderRegistry {
    pub active: Option<TransmissionMediaProviderKind>,
}

impl TransmissionMediaProviderRegistry {
    pub fn set_active(&mut self, kind: TransmissionMediaProviderKind) {
        self.active = Some(kind);
    }
}

/// UX-E03-CODER-A — narrative media registry active; transmission shell does not enqueue orders.
#[must_use]
pub fn ux_e03_coder_a_green(registry: &TransmissionMediaProviderRegistry) -> bool {
    registry.active.is_some()
}

/// Lib / sim seed — static text provider (no decode, no StrategicCommand from UI).
pub fn seed_ux_e03_transmission_media_registry(registry: &mut TransmissionMediaProviderRegistry) {
    registry.set_active(TransmissionMediaProviderKind::StaticText);
}
