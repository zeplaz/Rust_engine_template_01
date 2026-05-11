//! **GPU bridge (stub)** — numeric acceleration boundary: AI batch scoring (Path A) vs world fields (Path B).
//!
//! No gameplay authority here; CPU ECS remains source of truth for missions, fracture policy, and tooling.

use bevy::prelude::*;

/// Packed agent row for a future GPU evaluate pass (Path A).
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct AgentGpuPacket {
    pub traits: [f32; 8],
    pub emotion: [f32; 4],
    pub pressures: [f32; 4],
}

/// Result row from GPU scoring (Path A); CPU chooses discrete actions.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct AgentGpuResult {
    pub action_scores: [f32; 8],
}

/// Which offload lane is active (staging only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GpuSimLane {
    #[default]
    CpuOnly,
    /// Trait/fuzzy batch evaluation on GPU (future).
    AgentBehaviorBatch,
    /// Noise / diffusion / preview fields (future; aligns with worldgen preview runbook).
    WorldFieldBatch,
}

#[derive(Resource, Debug, Default)]
pub struct GpuBridgeState {
    pub lane: GpuSimLane,
    pub agent_packet_upload_cursor: usize,
}

pub struct GpuBridgePlugin;

impl Plugin for GpuBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuBridgeState>();
    }
}
