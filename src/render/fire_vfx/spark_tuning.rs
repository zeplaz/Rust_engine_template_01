//! FX-FIRE-SPARK tuning constants, env toggles and per-frame spark stats (frontend authority).

/// Per-frame spark emission stats (rows, scatter slots, zoom) for FX-FIRE-SPARK.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireSparkStats {
    pub phase: &'static str,
    pub rows: usize,
    pub scatter_max: usize,
    pub scatter_slots: usize,
    pub zoom_alpha: f32,
    pub additive_blend: bool,
    pub budget_capped: bool,
    pub view_culled: bool,
    pub projection_view: &'static str,
}

pub const FIRE_SPARK_SCATTER_MAX: usize = 14;
/// FIRE-VIS-001 (2026-07-06): re-keyed from `zoom_alpha` (0.10) to **px-per-tile** — `zoom_alpha`
/// is normalized against per-world zoom limits ([`map_zoom_limits_for_world`](crate::gui::map_zoom_limits_for_world))
/// so alpha 0.10 corresponded to zoom ≈ 9 on a 320-world, unreachable in normal play. This axis is
/// the camera's raw scale (`ExtractedCameraMetrics::zoom_level`), which equals px-per-world-unit ==
/// Operator debug: zoom hard-cull disabled (`0.0`). Scatter density still soft-ramps toward
/// [`FIRE_SPARK_FULL_SCATTER_PX_PER_TILE`]. Re-enable a floor later via designer gate.
pub const FIRE_SPARK_MIN_PX_PER_TILE: f32 = 0.0;
/// Designer operational play anchor — sparse sparks must read here ([`design_zoom_fire_read_v1.md`]).
/// Re-keyed to px-per-tile alongside [`FIRE_SPARK_MIN_PX_PER_TILE`] (was `zoom_alpha` 0.42).
pub const FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE: f32 =
    crate::terrain::world_scale_contract::OPERATIONAL_PX_PER_TILE;
/// Full scatter density by operational play (not tactical-only cinematic zoom). Re-keyed to px-per-tile.
pub const FIRE_SPARK_FULL_SCATTER_PX_PER_TILE: f32 =
    crate::terrain::world_scale_contract::TACTICAL_PX_PER_TILE;
/// P2-FIRE-SPARK-011 / `--test visual` proof band — still expressed on the `zoom_alpha` axis
/// (drives [`FireSparkStats::zoom_alpha`] / camera proof-lock harnesses, not the px-per-tile cull).
/// Matches [`crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA`].
pub const FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA: f32 =
    crate::terrain::world_scale_contract::TACTICAL_PROOF_ZOOM_ALPHA;
pub(crate) const FIRE_SPARK_BUDGET_PRESSURE: f32 = 0.85;

/// Phase B compute advection — **opt-in** (`FIRE_SPARK_COMPUTE=1`). Default off so
/// sparks stay on instance origins (legacy 3D advection was flying them off-map).
#[inline]
#[must_use]
pub fn fire_spark_compute_enabled() -> bool {
    matches!(
        std::env::var("FIRE_SPARK_COMPUTE").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}



#[inline]
pub(crate) fn fire_spark_phase() -> &'static str {
    if fire_spark_compute_enabled() {
        "A+B"
    } else {
        "A"
    }
}

/// Camera scale axis for spark cull / heat boost (px-per-world-unit == px-per-tile).
#[inline]
#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn fire_spark_px_per_tile(scale_x: f32) -> f32 {
    scale_x.abs()
}

/// True when GPU sparks and CPU heat boost should both be active at this zoom.
#[inline]
#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn fire_spark_enabled_at_px_per_tile(px_per_tile: f32) -> bool {
    fire_spark_px_per_tile(px_per_tile) >= FIRE_SPARK_MIN_PX_PER_TILE
}

/// FIRE-VIS-001: scatter ramp on **px-per-tile** (camera `zoom_level`), not `zoom_alpha` — see
/// [`FIRE_SPARK_MIN_PX_PER_TILE`]. Continuous ramp 0..1 between min and full-scatter px-per-tile.
#[inline]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub(crate) fn fire_spark_zoom_scatter_gate(px_per_tile: f32) -> f32 {
    let span = (FIRE_SPARK_FULL_SCATTER_PX_PER_TILE - FIRE_SPARK_MIN_PX_PER_TILE).max(1e-4);
    ((px_per_tile - FIRE_SPARK_MIN_PX_PER_TILE) / span).clamp(0.0, 1.0)
}
