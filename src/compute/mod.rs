//! GPU **compute** orchestration (logic offload) — dual-graph sibling to [`crate::render::extraction::RenderProjectionGraph`].
//!
//! Frame snapshots are read-only inputs; compute nodes own their outputs. No ECS queries inside dispatch.

mod compute_dispatch_graph;
mod frame_snapshots;
mod heat_diffusion;

pub use compute_dispatch_graph::{
    run_compute_dispatch_graph, ComputeContext, ComputeDispatchCadence, ComputeDispatchGraph,
    ComputeDispatchPlugin, ComputeDispatchSystemSet, ComputeNodeTrait, FireInfluenceDispatchNode,
};
pub use frame_snapshots::{AgentFrame, NavFieldFrame};
pub use heat_diffusion::{
    advance_heat_diffusion_field, eligible_fire_rows, run_heat_diffusion_step, HeatDiffusionCell,
    HeatDiffusionDispatchNode, HeatDiffusionFieldBuffers, HeatDiffusionGpuCell,
};
