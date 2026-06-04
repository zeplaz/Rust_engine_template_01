//! **INFRA-E1-001** — Bevy plugin: authoritative [`TransportGraph`] resource.

use bevy::prelude::*;

use super::graph::TransportGraph;
use crate::infrastructure::authoring::CorridorAuthoringSession;

pub struct InfrastructureTransportPlugin;

impl Plugin for InfrastructureTransportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransportGraph>()
            .init_resource::<CorridorAuthoringSession>();
    }
}
