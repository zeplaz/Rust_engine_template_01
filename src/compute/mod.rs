//! GPU **compute** orchestration (logic offload) — dual-graph sibling to [`crate::render::extraction::RenderProjectionGraph`].
//!
//! Frame snapshots are read-only inputs; compute nodes own their outputs. No ECS queries inside dispatch.

mod compute_dispatch_graph;
mod frame_snapshots;
mod heat_diffusion;
mod heat_diffusion_dense;
mod heat_diffusion_gpu;
mod dense_heat_gpu_dispatch;
mod heat_weather;

pub use compute_dispatch_graph::{
    check_parallel_claim, compute_critical_path_len, compute_graph_parallel_width,
    compute_parallel_width_for_cores, compute_stage_levels, run_compute_dispatch_graph,
    ComputeContext, ComputeDispatchCadence, ComputeDispatchGraph, ComputeDispatchPlugin,
    ComputeDispatchSystemSet, ComputeNodeTrait, ComputeStage, FireInfluenceDispatchNode,
    ParallelClaimRejected, COMPUTE_STAGE_COUNT, COMPUTE_STAGE_EDGES,
    COMPUTE_STAGE_EXECUTION_ORDER,
};
pub use frame_snapshots::{AgentFrame, NavFieldFrame, NavHeatCostSample, NavWildlifeSoftSample};
pub use heat_diffusion::{
    advance_heat_diffusion_field, eligible_fire_rows, heat_to_nav_cost, run_heat_diffusion_step,
    sync_nav_field_from_heat_diffusion, HeatDiffusionCell, HeatDiffusionDispatchNode,
    HeatDiffusionFieldBuffers, HeatDiffusionGpuCell,
};
pub use heat_diffusion_dense::{
    dense_roi_from_fire, run_dense_heat_step, DenseHeatGrid, HeatDiffusionConfig, DENSE_DECAY,
    DENSE_SOURCE_BLEND, DENSE_W_CENTER, DENSE_W_SIDE,
};
pub use dense_heat_gpu_dispatch::{
    advance_dense_heat_play_gpu_or_cpu, DenseHeatGpuComputePassSet, DenseHeatGpuHandles,
    DenseHeatGpuJob, DenseHeatGpuPlugin, DenseHeatGpuPrimary, DENSE_HEAT_CPU_ENV,
    DENSE_HEAT_GPU_MAX_F32,
};
pub use heat_diffusion_gpu::{
    advance_dense_heat_play, dense_grid_max_abs_diff, fire_vfx_blend_dense_heat, pack_dense_to_cells,
    sample_dense_heat_at_chunk, wgsl_embeds_dense_constants, DenseHeatPlayField,
    HEAT_DIFFUSION_DENSE_WGSL,
};
pub use heat_weather::{
    apply_climate_to_dense_heat, apply_weather_to_dense_heat, WEATHER_RAIN_DAMP,
    WEATHER_WIND_FRAC_AT_20,
};
