//! Disk witnesses for deep debug sessions.

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde_json::{json, Value};

use super::frame_probe::sample_deep_debug_frame;
use super::latch::{deep_debug_active, DeepDebugConfig};
use super::subsystem_cache::DeepDebugSubsystemCache;

pub const DEEP_DEBUG_WITNESS_REL: &str = "debug_runs/deep_debug/engine_deep_debug_live.json";
const JSONL_REL: &str = "debug_runs/deep_debug/engine_deep_debug_frames.jsonl";

fn repo_relative(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn write_live_witness(body: &Value) {
    let path = repo_relative(DEEP_DEBUG_WITNESS_REL);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let wrapped = json!({
        "schema": "engine_deep_debug_live_v1",
        "task_id": "ENGINE-DEEP-DEBUG-001",
        "ok": true,
        "written_at_epoch_secs": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "glyph_chain": "⛔⚠🔴⚡🧊",
        "body": body,
        "_agent_meta": {
            "profile": "ENGINE-DEEP-DEBUG",
            "source_system": "engine_deep_debug",
            "relative_path": DEEP_DEBUG_WITNESS_REL,
        },
    });
    if let Ok(text) = serde_json::to_string_pretty(&wrapped) {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(tmp, path);
        }
    }
}

fn append_jsonl_line(body: &Value) {
    let path = repo_relative(JSONL_REL);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(line) = serde_json::to_string(body) {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{line}");
        }
    }
}

pub fn deep_debug_post_update(
    frame: Res<bevy::diagnostic::FrameCount>,
    cfg: Res<DeepDebugConfig>,
    mut probe: ResMut<super::frame_probe::DeepDebugFrameProbe>,
    trace: Res<super::minimap_trace::MinimapDeepTrace>,
    images: Res<Assets<Image>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    shell: Res<crate::gui::MinimapShellState>,
    fallback: Res<crate::render::TileWorldFallbackState>,
    registry: Res<crate::render::minimap_compositor::MinimapRenderTargetRegistry>,
    compositor: Res<crate::render::minimap_compositor::MinimapCompositorState>,
    gpu_diag: Res<crate::render::minimap_compositor::MinimapGpuCompositorDiagnostics>,
    map_views: Res<crate::gui::MapViewInstances>,
    view_trace: Option<Res<crate::render::view_runtime::ViewRuntimeTrace>>,
    subsystem_cache: Res<DeepDebugSubsystemCache>,
) {
    if !cfg.active {
        return;
    }
    let jsonl = cfg.jsonl_frames;
    if let Some(body) = sample_deep_debug_frame(
        frame.as_ref(),
        cfg.as_ref(),
        probe.as_mut(),
        trace.as_ref(),
        images.as_ref(),
        meshes.as_ref(),
        materials.as_ref(),
        shell.as_ref(),
        fallback.as_ref(),
        registry.as_ref(),
        compositor.as_ref(),
        gpu_diag.as_ref(),
        map_views.as_ref(),
        view_trace.as_deref(),
        Some(subsystem_cache.as_ref()),
    ) {
        write_live_witness(&body);
        if jsonl {
            append_jsonl_line(&body);
        }
    }
}

pub fn log_deep_debug_banner() {
    if deep_debug_active() {
        info!(
            target: "engine_deep_debug",
            witness = DEEP_DEBUG_WITNESS_REL,
            jsonl = JSONL_REL,
            "deep debug witnesses flushing — see src/dev/engine_deep_debug_runbook_v1.md"
        );
    }
}
