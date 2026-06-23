//! Per-frame ECS / asset / render inventory for deep debug witnesses.

use bevy::asset::Assets;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::gui::{MapViewInstances, MinimapShellState};
use crate::render::minimap_compositor::{
    diagnostics_json_snapshot, MinimapCompositorState, MinimapGpuCompositorDiagnostics,
    MinimapRenderTargetRegistry,
};
use crate::render::view_runtime::ViewRuntimeTrace;
use crate::render::TileWorldFallbackState;

use super::latch::DeepDebugConfig;
use super::minimap_trace::{minimap_trace_snapshot, MinimapDeepTrace};
use super::subsystem_cache::DeepDebugSubsystemCache;

#[derive(Resource, Default)]
pub struct DeepDebugFrameProbe {
    pub last_flush_frame: u64,
}

pub fn sample_deep_debug_frame(
    frame: &bevy::diagnostic::FrameCount,
    cfg: &DeepDebugConfig,
    probe: &mut DeepDebugFrameProbe,
    trace: &MinimapDeepTrace,
    images: &Assets<Image>,
    meshes: &Assets<Mesh>,
    materials: &Assets<StandardMaterial>,
    shell: &MinimapShellState,
    fallback: &TileWorldFallbackState,
    registry: &MinimapRenderTargetRegistry,
    compositor: &MinimapCompositorState,
    gpu_diag: &MinimapGpuCompositorDiagnostics,
    map_views: &MapViewInstances,
    view_trace: Option<&ViewRuntimeTrace>,
    subsystem_cache: Option<&DeepDebugSubsystemCache>,
) -> Option<Value> {
    if !cfg.active {
        return None;
    }
    let n = frame.0 as u64;
    if n == 0 || n % cfg.flush_every_n_frames as u64 != 0 {
        return None;
    }
    probe.last_flush_frame = n;

    let mut body = json!({
        "frame": n,
        "inventory": {
            "images": images.len(),
            "meshes": meshes.len(),
            "materials": materials.len(),
        },
        "minimap_shell": {
            "visible": shell.visible,
            "minimized": shell.minimized,
            "presentation_source": format!("{:?}", shell.presentation_source),
            "panel_viewport": [shell.panel_viewport_suggestion_logical_size.x, shell.panel_viewport_suggestion_logical_size.y],
        },
        "minimap_fallback": {
            "last_w": fallback.last_w,
            "last_h": fallback.last_h,
            "sprite_entity": fallback.sprite_entity.is_some(),
            "minimap_image": fallback.minimap_image != Handle::default(),
            "main_image": fallback.image != Handle::default(),
        },
        "minimap_registry": {
            "committed_size": [registry.committed_size.x, registry.committed_size.y],
            "revision": registry.revision,
            "has_rt": registry.committed_image != Handle::default(),
        },
        "minimap_compositor": {
            "stamp": compositor.stamp,
            "path": format!("{:?}", compositor.composite_path),
            "display_handle_gpu": compositor.stamp > 0,
            "logistics_rows": compositor.logistics_rows,
            "construction_rows": compositor.construction_rows,
            "ecology_rows": compositor.ecology_rows,
            "dual_minimap": compositor.dual_minimap_present,
        },
        "minimap_gpu_diagnostics": diagnostics_json_snapshot(&gpu_diag),
        "map_view_minimap_revision": map_views.minimap.revision,
        "minimap_trace": minimap_trace_snapshot(&trace),
    });

    if let Some(vt) = view_trace {
        body["view_runtime_violations"] = json!(vt
            .violations
            .iter()
            .map(|v| format!("{v:?}"))
            .collect::<Vec<_>>());
        body["view_runtime_trace_len"] = json!(vt.entries.len());
    }

    if cfg.schedule_trace {
        if let Some(cache) = subsystem_cache {
            body["subsystem_isolation"] = cache.isolation.clone();
            body["visual_memory_queues"] = cache.memory_queues.clone();
        }
    }

    if cfg.gpu_render_trace {
        body["gpu_image_formats_sample"] = json!(sample_image_formats(&images, 8));
    }

    Some(body)
}

fn sample_image_formats(images: &Assets<Image>, limit: usize) -> Value {
    let rows: Vec<Value> = images
        .iter()
        .take(limit)
        .map(|(id, img)| {
            json!({
                "id": format!("{:?}", id),
                "w": img.width(),
                "h": img.height(),
                "format": format!("{:?}", img.texture_descriptor.format),
            })
        })
        .collect();
    json!(rows)
}

pub fn plugin_diagnostics(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
}
