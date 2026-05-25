//! Transmission media provider abstraction — static/fake/ticker only (no decode authority).

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
