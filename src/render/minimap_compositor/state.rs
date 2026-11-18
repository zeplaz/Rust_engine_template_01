//! Minimap compositor state — resources, enums, env predicates (no schedule systems).

use bevy::prelude::*;



#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapCompositePath {
    /// Opt-out / degrade only: `MINIMAP_GPU_COMPOSITOR=0`, headless CI, or shader-failed.
    CpuBridge,
    /// GS-09 play default — zero-copy VRAM composite.
    #[default]
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
    /// GPU RTT world bake Image (`TERRAIN_GPU_BAKE_SPIKE` + control→`terrain_bake_rtt`).
    GpuBake,
}

/// P0-E / RPC-1-005 / GS-09 witness helper — terrain source label from the *actual* bound input.
///
/// The label reports what texture really feeds the compositor, not `TerrainRenderAuthority`.
/// `gpu_bake` is honest **only** when [`MinimapTerrainSourceBind::GpuBake`] (bake Image ready).
/// Fallback CPU world image remains [`WorldRaster`] → `world_raster` (CI / spike-off only).
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

/// GS-09: GPU compositor is **default ON** when unset.
/// Set `MINIMAP_GPU_COMPOSITOR=0` for CI/headless opt-out (CpuBridge + SharedCpuRaster).
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



/// GPU compositor env on and shader pipeline healthy (no runtime fallback).
#[must_use]
pub fn minimap_gpu_compositor_runtime_enabled() -> bool {
    minimap_gpu_compositor_env_enabled()
        && !super::diagnostics::MINIMAP_GPU_SHADER_FAILED.load(std::sync::atomic::Ordering::Relaxed)
}
