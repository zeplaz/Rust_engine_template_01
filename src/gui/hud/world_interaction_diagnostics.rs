//! Consumer-only map and construction interaction diagnostics.

use bevy::prelude::*;

use crate::construction::ConstructionQueuePanelView;
use crate::gui::view_projection_authority::camera_translation;
use crate::gui::{MapCameraDesired, MapViewInstances, MinimapShellState, ViewId, ViewManager};

#[derive(Resource, Clone, Debug, Default)]
pub struct WorldInteractionDiagnostics {
    pub construction_throughput_hint: f32,
    pub construction_queue_latency_ms: f32,
    pub hover_diagnostics_active: bool,
    pub map_interaction_latency_ms: f32,
    pub pending_queue_depth: usize,
    pub optimistic_hover_active: bool,
    pub hover_highlight_strength: f32,
    pub tooltip_pending: bool,
    pub deferred_detail_ready: bool,
}

pub fn refresh_world_interaction_diagnostics(
    construction: Res<ConstructionQueuePanelView>,
    minimap: Res<MinimapShellState>,
    map_views: Res<MapViewInstances>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesired>,
    time: Res<Time>,
    mut diag: ResMut<WorldInteractionDiagnostics>,
) {
    diag.construction_throughput_hint = construction.logistics_score;
    diag.pending_queue_depth = construction.pending_count;
    diag.construction_queue_latency_ms = if construction.pending_count > 0 {
        (construction.pending_count as f32 * 4.0).min(250.0)
    } else {
        0.0
    };
    let minimap_follow = map_views.minimap.follow_mode;
    let map_active = minimap.diagnostic_ui_wrote_camera || minimap_follow as u8 > 0;
    diag.hover_diagnostics_active = map_active;
    diag.optimistic_hover_active = map_active || construction.pending_count > 0;
    diag.hover_highlight_strength = if diag.optimistic_hover_active {
        1.0
    } else {
        diag.hover_highlight_strength * 0.85
    };
    diag.tooltip_pending = construction.pending_count > 0 && !diag.deferred_detail_ready;
    diag.deferred_detail_ready = construction.pending_count == 0
        || diag.construction_queue_latency_ms < 120.0;
    let camera_anchor = camera_translation(&view_manager, ViewId::WorldMain)
        .unwrap_or_else(|| desired.translation.truncate());
    let camera_delta = camera_anchor.length_squared();
    diag.map_interaction_latency_ms = if camera_delta > 0.0 {
        time.delta_secs() * 1000.0
    } else {
        diag.map_interaction_latency_ms * 0.9
    };
}
