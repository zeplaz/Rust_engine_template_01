//! Minimap compositor + egui bind deep trace ring buffer.

use std::collections::VecDeque;

use bevy::prelude::*;
use serde::Serialize;

use super::latch::deep_debug_active;

const RING_CAP: usize = 256;

#[derive(Clone, Debug, Serialize)]
pub struct MinimapCompositorDecision {
    pub frame: u64,
    pub committed: bool,
    pub skip_reason: Option<String>,
    pub dispatch_reason: Option<String>,
    pub presentation_source: String,
    pub gpu_compositor_env: bool,
    pub shell_visible: bool,
    pub shell_minimized: bool,
    pub rt_size: [u32; 2],
    pub terrain_handle_valid: bool,
    pub fingerprint: u64,
    pub stamp: u64,
    pub overlay_revision: u64,
    pub logistics_rows: u32,
    pub construction_rows: u32,
    pub ecology_rows: u32,
    pub composite_path: String,
    pub fallback_w: u32,
    pub fallback_h: u32,
    pub minimap_image_valid: bool,
    pub panel_viewport: [f32; 2],
}

#[derive(Clone, Debug, Serialize)]
pub struct MinimapEguiBindEvent {
    pub frame: u64,
    pub bound: bool,
    pub reason: String,
    pub uses_gpu_compositor: bool,
}

#[derive(Resource, Default)]
pub struct MinimapDeepTrace {
    pub compositor: VecDeque<MinimapCompositorDecision>,
    pub egui_bind: VecDeque<MinimapEguiBindEvent>,
    pub anomaly_count: u32,
}

impl MinimapDeepTrace {
    fn push_compositor(&mut self, row: MinimapCompositorDecision) {
        if row.committed && row.terrain_handle_valid && row.rt_size == [0, 0] {
            self.anomaly_count = self.anomaly_count.saturating_add(1);
        }
        if self.compositor.len() >= RING_CAP {
            self.compositor.pop_front();
        }
        self.compositor.push_back(row);
    }

    fn push_egui(&mut self, row: MinimapEguiBindEvent) {
        if self.egui_bind.len() >= RING_CAP {
            self.egui_bind.pop_front();
        }
        self.egui_bind.push_back(row);
    }
}

pub fn record_minimap_compositor_decision(
    trace: Option<&mut MinimapDeepTrace>,
    _frame: u64,
    row: MinimapCompositorDecision,
) {
    if !deep_debug_active() {
        return;
    }
    if let Some(t) = trace {
        crate::deep_debug_trace!(
            "engine_deep_debug::minimap",
            "compositor frame={} committed={} skip={:?} dispatch={:?} path={} rt={:?} terrain={}",
            row.frame,
            row.committed,
            row.skip_reason,
            row.dispatch_reason,
            row.composite_path,
            row.rt_size,
            row.terrain_handle_valid
        );
        t.push_compositor(row);
    }
}

pub fn record_minimap_egui_bind(
    trace: Option<&mut MinimapDeepTrace>,
    frame: u64,
    bound: bool,
    reason: &str,
    uses_gpu: bool,
) {
    if !deep_debug_active() {
        return;
    }
    if let Some(t) = trace {
        t.push_egui(MinimapEguiBindEvent {
            frame,
            bound,
            reason: reason.to_string(),
            uses_gpu_compositor: uses_gpu,
        });
    }
}

pub fn snapshot_minimap_after_compositor_pass(
    frame: Res<bevy::diagnostic::FrameCount>,
    cfg: Res<super::latch::DeepDebugConfig>,
    mut trace: ResMut<MinimapDeepTrace>,
    shell: Res<crate::gui::MinimapShellState>,
    fallback: Res<crate::render::TileWorldFallbackState>,
    registry: Res<crate::render::minimap_compositor::MinimapRenderTargetRegistry>,
    compositor: Res<crate::render::minimap_compositor::MinimapCompositorState>,
    diagnostics: Res<crate::render::minimap_compositor::MinimapGpuCompositorDiagnostics>,
) {
    if !cfg.active || !cfg.minimap_trace {
        return;
    }
    let skip = match diagnostics.last_skip {
        crate::render::minimap_compositor::MinimapGpuSkipReason::None => None,
        other => Some(format!("{other:?}")),
    };
    let dispatch = match diagnostics.last_dispatch {
        crate::render::minimap_compositor::MinimapGpuDispatchReason::None => None,
        other => Some(format!("{other:?}")),
    };
    let committed = compositor.stamp > 0
        && diagnostics.last_skip == crate::render::minimap_compositor::MinimapGpuSkipReason::None
        && diagnostics.last_commit_stamp > 0;
    record_minimap_compositor_decision(
        Some(&mut trace),
        frame.0 as u64,
        MinimapCompositorDecision {
            frame: frame.0 as u64,
            committed,
            skip_reason: skip,
            dispatch_reason: dispatch,
            presentation_source: format!("{:?}", shell.presentation_source),
            gpu_compositor_env: crate::render::minimap_compositor::minimap_gpu_compositor_env_enabled(),
            shell_visible: shell.visible,
            shell_minimized: shell.minimized,
            rt_size: [registry.committed_size.x, registry.committed_size.y],
            terrain_handle_valid: fallback.image != Handle::default(),
            fingerprint: diagnostics.last_fingerprint,
            stamp: compositor.stamp,
            overlay_revision: compositor.last_overlay_revision,
            logistics_rows: compositor.logistics_rows,
            construction_rows: compositor.construction_rows,
            ecology_rows: compositor.ecology_rows,
            composite_path: format!("{:?}", compositor.composite_path),
            fallback_w: fallback.last_w,
            fallback_h: fallback.last_h,
            minimap_image_valid: fallback.minimap_image != Handle::default(),
            panel_viewport: [
                shell.panel_viewport_suggestion_logical_size.x,
                shell.panel_viewport_suggestion_logical_size.y,
            ],
        },
    );
}

pub fn minimap_trace_snapshot(trace: &MinimapDeepTrace) -> serde_json::Value {
    serde_json::json!({
        "anomaly_count": trace.anomaly_count,
        "compositor_tail": trace.compositor.iter().rev().take(48).collect::<Vec<_>>(),
        "egui_bind_tail": trace.egui_bind.iter().rev().take(24).collect::<Vec<_>>(),
    })
}
