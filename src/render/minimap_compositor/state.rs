//! Minimap compositor state — resources, enums, env predicates (no schedule systems).

use bevy::prelude::*;

use crate::gui::{MinimapPresentationSource, MinimapShellState};

use super::render_target::MinimapRenderTargetRegistry;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapCompositePath {
    #[default]
    CpuBridge,
    GpuCompute,
}

#[derive(Resource, Debug, Clone)]
pub struct MinimapCompositorState {
    pub stamp: u64,
    pub compositor_revision: u64,
    pub last_overlay_revision: u64,
    pub dual_minimap_present: bool,
    pub extent_match_px: f32,
    pub composite_path: MinimapCompositePath,
    pub logistics_rows: u32,
    pub construction_rows: u32,
    pub ecology_rows: u32,
    pub fow_rows: u32,
    pub ew_rows: u32,
    pub fire_heat_enabled: bool,
    pub logistics_heat_enabled: bool,
    pub construction_heat_enabled: bool,
    pub ecology_heat_enabled: bool,
    pub fow_heat_enabled: bool,
    pub ew_heat_enabled: bool,
    pub units_heat_enabled: bool,
    pub unit_marker_rows: u32,
    pub replay_scrub_enabled: bool,
    /// **VEG-MINIMAP-BURN-MERGE-001** — rows merged from `VegetationExtractFrame`.
    pub veg_burn_rows: u32,
    pub burn_overrides_topology: bool,
    pub veg_extract_revision: u64,
    /// Witness: `world_raster` | `gpu_bake` | `none`.
    pub terrain_source_label: &'static str,
}

/// Which world-space texture actually feeds the minimap compositor this commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimapTerrainSourceBind {
    None,
    /// CPU dirty-bake world image (`TileWorldFallbackState`).
    WorldRaster,
    /// Flagged spike bake Image (`TERRAIN_GPU_BAKE_SPIKE` + host atlas→world write).
    GpuBake,
}

/// P0-E / RPC-1-005 witness helper — terrain source label from the *actual* bound input.
///
/// The label reports what texture really feeds the compositor, not `TerrainRenderAuthority`.
/// `gpu_bake` is honest **only** when [`MinimapTerrainSourceBind::GpuBake`] (spike bake Image
/// with pixels written). Default-off / CPU world image remains [`WorldRaster`] → `world_raster`.
#[must_use]
pub fn minimap_terrain_source_label(bind: MinimapTerrainSourceBind) -> &'static str {
    match bind {
        MinimapTerrainSourceBind::None => "none",
        MinimapTerrainSourceBind::WorldRaster => "world_raster",
        MinimapTerrainSourceBind::GpuBake => "gpu_bake",
    }
}

/// Resolve bind kind from handles (gpu bake wins only when that handle is the selected input).
#[must_use]
pub fn minimap_terrain_source_bind(
    terrain: &Handle<Image>,
    bake_handle: Option<&Handle<Image>>,
) -> MinimapTerrainSourceBind {
    if *terrain == Handle::default() {
        return MinimapTerrainSourceBind::None;
    }
    if let Some(bake) = bake_handle {
        if *bake != Handle::default() && terrain == bake {
            return MinimapTerrainSourceBind::GpuBake;
        }
    }
    MinimapTerrainSourceBind::WorldRaster
}

impl Default for MinimapCompositorState {
    fn default() -> Self {
        Self {
            stamp: 0,
            compositor_revision: 0,
            last_overlay_revision: 0,
            dual_minimap_present: false,
            extent_match_px: 0.0,
            composite_path: MinimapCompositePath::default(),
            logistics_rows: 0,
            construction_rows: 0,
            ecology_rows: 0,
            fow_rows: 0,
            ew_rows: 0,
            fire_heat_enabled: false,
            logistics_heat_enabled: false,
            construction_heat_enabled: false,
            ecology_heat_enabled: false,
            fow_heat_enabled: false,
            ew_heat_enabled: false,
            units_heat_enabled: false,
            unit_marker_rows: 0,
            replay_scrub_enabled: false,
            veg_burn_rows: 0,
            burn_overrides_topology: false,
            veg_extract_revision: 0,
            terrain_source_label: "none",
        }
    }
}

#[must_use]
pub fn minimap_gpu_compositor_env_enabled() -> bool {
    match std::env::var("MINIMAP_GPU_COMPOSITOR").ok().as_deref() {
        None => true,
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => false,
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => true,
        _ => true,
    }
}

#[must_use]
pub fn minimap_gpu_compositor_default_on_unset() -> bool {
    std::env::var("MINIMAP_GPU_COMPOSITOR").is_err()
}

/// **PERF-VIS-P1B-GPU-DEFAULT-001** — Simulation GPU minimap path without `RASTER_*` / explicit env.
#[must_use]
pub fn perf_vis_p1b_gpu_default_001_green(
    shell: &MinimapShellState,
    registry: &MinimapRenderTargetRegistry,
    compositor: &MinimapCompositorState,
) -> bool {
    minimap_gpu_compositor_env_enabled()
        && shell.presentation_source == MinimapPresentationSource::SharedRenderTargetImage
        && registry.committed_image != Handle::default()
        && compositor.stamp > 0
        && compositor.composite_path == MinimapCompositePath::GpuCompute
}

#[must_use]
pub(crate) fn perf_vis_p1b_witness_json(
    shell: &MinimapShellState,
    registry: &MinimapRenderTargetRegistry,
    compositor: &MinimapCompositorState,
) -> serde_json::Value {
    serde_json::json!({
        "gate": "PERF-VIS-P1B-GPU-DEFAULT-001",
        "green": perf_vis_p1b_gpu_default_001_green(shell, registry, compositor),
        "gpu_compositor_default_on": minimap_gpu_compositor_default_on_unset(),
        "gpu_compositor_env": minimap_gpu_compositor_env_enabled(),
        "presentation_source": match shell.presentation_source {
            MinimapPresentationSource::SharedCpuRaster => "SharedCpuRaster",
            MinimapPresentationSource::SharedRenderTargetImage => "SharedRenderTargetImage",
        },
        "raster_env_required": false,
    })
}

/// GPU compositor env on and shader pipeline healthy (no runtime fallback).
#[must_use]
pub fn minimap_gpu_compositor_runtime_enabled() -> bool {
    minimap_gpu_compositor_env_enabled()
        && !super::diagnostics::MINIMAP_GPU_SHADER_FAILED.load(std::sync::atomic::Ordering::Relaxed)
}
