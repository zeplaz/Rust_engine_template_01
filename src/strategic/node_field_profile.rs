//! **Functional node** tagging — buildings / facilities as graph nodes that declare how they bias fields.

use bevy::prelude::{Component, Entity};

/// High-level economic / military / city role (tag layer; simulation fills [`super::world_field_layers::ChunkFieldCell`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Component)]
pub enum NodeRole {
    #[default]
    Production,
    Storage,
    Consumption,
    LogisticsHub,
    TransportLink,
    Defense,
    Sensor,
    Command,
    Environmental,
}

/// Scalar injection toward [`super::world_field_layers::ChunkFieldCell`] / overlays (design-time or runtime).
#[derive(Clone, Copy, Debug, Default, Component)]
pub struct FieldContribution {
    pub supply: f32,
    pub demand: f32,
    pub control: f32,
    pub threat: f32,
    pub visibility: f32,
    pub morale: f32,
}

/// Optional back-link for build pipeline bookkeeping (structure spawn, ledger rows).
#[derive(Clone, Copy, Debug, Component)]
pub struct FieldEmitterParent {
    pub entity: Entity,
}
