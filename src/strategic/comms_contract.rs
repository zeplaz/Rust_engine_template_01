//! Stage-7 **planning contracts** — DTOs only; no dispatch solvers or duplicate extract paths.
//!
//! Product locks: `prompts/guides/stage7_behavioral_world_designer_brief_v1.md`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::systems::sim_control::SimStepStamp;

/// Authoritative comms plane (v1 subset — extend when coalition planners land).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommunicationPlane {
    StrategicCommand,
    LogisticsHub,
    SensorRelay,
    TacticalLine,
}

/// Informational vs order-bearing planes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaneAuthority {
    Orders,
    OrdersAndRouting,
    Informational,
    LocalLimited,
}

impl CommunicationPlane {
    #[must_use]
    pub const fn authority(self) -> PlaneAuthority {
        match self {
            Self::StrategicCommand => PlaneAuthority::Orders,
            Self::LogisticsHub => PlaneAuthority::OrdersAndRouting,
            Self::SensorRelay => PlaneAuthority::Informational,
            Self::TacticalLine => PlaneAuthority::LocalLimited,
        }
    }
}

/// Typed command envelope — queue ownership is per-plane later (**BQ-125**).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DispatchMessage {
    pub plane: CommunicationPlane,
    pub issued_at: SimStepStamp,
    pub deliver_after: SimStepStamp,
    pub command_id: u64,
    pub summary: String,
}

/// Wire envelope for delayed / degraded dispatch (**BQ-125**).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DispatchEnvelope {
    pub message: DispatchMessage,
    pub loss_probability: f32,
    pub corruption_hint: f32,
}

/// Registry-driven overlay legend row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayChannelDescriptor {
    pub utility: UtilityChannel,
    pub overlay: StrategicOverlayType,
    pub color_rgb: [u8; 3],
}

/// Serializable belief snapshot without ECS handles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeliefSnapshotDto {
    pub entity_bits: u64,
    pub confidence: IntelConfidence,
    pub last_refresh: SimStepStamp,
    pub summary: String,
}

/// Per-actor belief snapshot (not world truth).
#[derive(Clone, Debug, PartialEq)]
pub struct BeliefRecord {
    pub entity: Entity,
    pub confidence: IntelConfidence,
    pub last_refresh: SimStepStamp,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntelConfidence {
    pub scalar: f32,
    pub half_life_ticks: u32,
}

/// Authoritative utility channel owner contract (**one owner per channel**).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UtilityChannel {
    Threat,
    Logistics,
    Visibility,
    Congestion,
    Instability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategicOverlayType {
    Recon,
    LogisticsStress,
    Congestion,
    Threat,
    EwCoverage,
}

/// Strategic intent stub — local execution resolves later.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissionIntent {
    pub issued_at: SimStepStamp,
    pub label: String,
    pub priority: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_message_ron_roundtrip() {
        let msg = DispatchMessage {
            plane: CommunicationPlane::StrategicCommand,
            issued_at: SimStepStamp::new(4, 1),
            deliver_after: SimStepStamp::new(6, 1),
            command_id: 7,
            summary: "secure corridor".into(),
        };
        let ron = ron::ser::to_string(&msg).expect("serialize");
        let back: DispatchMessage = ron::from_str(&ron).expect("deserialize");
        assert_eq!(msg, back);
    }

    #[test]
    fn dispatch_envelope_ron_roundtrip() {
        let envelope = DispatchEnvelope {
            message: DispatchMessage {
                plane: CommunicationPlane::LogisticsHub,
                issued_at: SimStepStamp::new(1, 0),
                deliver_after: SimStepStamp::new(3, 0),
                command_id: 2,
                summary: "reroute".into(),
            },
            loss_probability: 0.1,
            corruption_hint: 0.0,
        };
        let ron = ron::ser::to_string(&envelope).expect("serialize");
        let back: DispatchEnvelope = ron::from_str(&ron).expect("deserialize");
        assert_eq!(envelope, back);
    }

    #[test]
    fn utility_channel_single_owner_table_is_stable() {
        let channels = [
            UtilityChannel::Threat,
            UtilityChannel::Logistics,
            UtilityChannel::Visibility,
        ];
        assert_eq!(channels.len(), 3);
    }
}
