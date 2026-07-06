//! FX-FIRE-SPARK witness gates and env toggles (frontend authority).

/// Live witness for FX-FIRE-SPARK-003 (stage5 / diagnostic JSON).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireSparkWitness {
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
/// px-per-tile (tiles are 1 world unit). Hard cull only below this — heat blobs only, no spark flood.
pub const FIRE_SPARK_MIN_PX_PER_TILE: f32 = 1.5;
/// Designer operational play anchor — sparse sparks must read here ([`design_zoom_fire_read_v1.md`]).
/// Re-keyed to px-per-tile alongside [`FIRE_SPARK_MIN_PX_PER_TILE`] (was `zoom_alpha` 0.42).
pub const FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE: f32 = 2.5;
/// Full scatter density by operational play (not tactical-only cinematic zoom). Re-keyed to px-per-tile.
pub const FIRE_SPARK_FULL_SCATTER_PX_PER_TILE: f32 = 4.0;
/// P2-FIRE-SPARK-011 / `--test visual` proof band — still expressed on the `zoom_alpha` axis
/// (drives [`FireSparkWitness::zoom_alpha`] / camera proof-lock harnesses, not the px-per-tile cull).
/// Matches [`crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA`].
pub const FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA: f32 = 0.85;
pub(crate) const FIRE_SPARK_BUDGET_PRESSURE: f32 = 0.85;

/// Phase B compute advection gate (`FIRE_SPARK_COMPUTE=0|false|off` disables).
#[inline]
#[must_use]
pub fn fire_spark_compute_enabled() -> bool {
    !matches!(
        std::env::var("FIRE_SPARK_COMPUTE").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

/// P2-FIRE-SPARK-011 — tactical shower read @ proof zoom (D-F07 / F-T03).
#[must_use]
pub fn fire_spark_011_green(w: &FireSparkWitness) -> bool {
    w.rows > 0
        && w.scatter_slots >= 3
        && w.zoom_alpha >= FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA * 0.75
        && !w.view_culled
        && w.additive_blend
}

#[inline]
pub(crate) fn fire_spark_witness_phase() -> &'static str {
    if fire_spark_compute_enabled() {
        "A+B"
    } else {
        "A"
    }
}

/// FIRE-VIS-001: scatter ramp on **px-per-tile** (camera `zoom_level`), not `zoom_alpha` — see
/// [`FIRE_SPARK_MIN_PX_PER_TILE`]. Continuous ramp 0..1 between min and full-scatter px-per-tile.
#[inline]
pub(crate) fn fire_spark_zoom_scatter_gate(px_per_tile: f32) -> f32 {
    let span = (FIRE_SPARK_FULL_SCATTER_PX_PER_TILE - FIRE_SPARK_MIN_PX_PER_TILE).max(1e-4);
    ((px_per_tile - FIRE_SPARK_MIN_PX_PER_TILE) / span).clamp(0.0, 1.0)
}
