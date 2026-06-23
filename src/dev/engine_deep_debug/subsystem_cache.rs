//! Cached subsystem + memory queue snapshots (keeps witness system param count low).

use bevy::prelude::*;
use serde_json::Value;

use crate::engine::launch_args::EngineLaunchArgs;
use crate::engine::states::BaseState;
use crate::engine::UxFrameSpikeGuard;
use crate::gui::hud::frame_budget_diagnostics::FrameBudgetDiagnostics;
use crate::gui::hud::shell_update_budget::ProductShellUpdateBudget;
use crate::gui::{MapViewInstances, VisualBudgetSettings, VisualCadence};
use crate::render::minimap_compositor::{
    MinimapGpuCompositorDiagnostics, MinimapRenderTargetRegistry,
};
use crate::render::{TileRasterBudget, TileRasterSpikeFeedback};

use super::latch::DeepDebugConfig;
use super::subsystem_probe::{subsystem_isolation_snapshot, visual_memory_queue_snapshot};

#[derive(Resource, Default, Clone)]
pub struct DeepDebugSubsystemCache {
    pub isolation: Value,
    pub memory_queues: Value,
}

pub fn refresh_deep_debug_subsystem_cache(
    cfg: Res<DeepDebugConfig>,
    mut cache: ResMut<DeepDebugSubsystemCache>,
    images: Res<Assets<Image>>,
    map_views: Res<MapViewInstances>,
    registry: Res<MinimapRenderTargetRegistry>,
    gpu_diag: Res<MinimapGpuCompositorDiagnostics>,
    base_state: Option<Res<State<BaseState>>>,
    launch: Option<Res<EngineLaunchArgs>>,
    budgets: Option<Res<VisualBudgetSettings>>,
    cadence: Option<Res<VisualCadence>>,
    raster_budget: Option<Res<TileRasterBudget>>,
    raster_feedback: Option<Res<TileRasterSpikeFeedback>>,
    spike: Option<Res<UxFrameSpikeGuard>>,
    frame_budget: Option<Res<FrameBudgetDiagnostics>>,
    shell_budget: Option<Res<ProductShellUpdateBudget>>,
) {
    if !cfg.active || !cfg.schedule_trace {
        return;
    }
    let shell_queue = shell_budget.as_deref().map(ProductShellUpdateBudget::debug_queue_snapshot);
    cache.isolation = subsystem_isolation_snapshot(
        base_state.as_deref(),
        launch.as_deref(),
        budgets.as_deref(),
        cadence.as_deref(),
        raster_budget.as_deref(),
        raster_feedback.as_deref(),
        spike.as_deref(),
        map_views.as_ref(),
        shell_queue,
    );
    cache.memory_queues = visual_memory_queue_snapshot(
        images.len(),
        frame_budget.as_deref(),
        gpu_diag.as_ref(),
        registry.as_ref(),
    );
}
