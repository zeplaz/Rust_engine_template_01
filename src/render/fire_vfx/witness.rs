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
/// Hard cull only at far strategic zoom (heat blobs only — no spark flood on whole map).
pub const FIRE_SPARK_MIN_ZOOM_ALPHA: f32 = 0.10;
/// Designer operational play anchor — sparse sparks must read here ([`design_zoom_fire_read_v1.md`]).
pub const FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA: f32 = 0.42;
/// Full scatter density by operational play (not tactical-only cinematic zoom).
pub const FIRE_SPARK_FULL_SCATTER_ZOOM_ALPHA: f32 = FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA;
/// P2-FIRE-SPARK-011 / `--test visual` proof band (matches [`crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA`]).
pub const FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA: f32 = 0.85;
/// Legacy alias — tile fallback uses this for CPU heat boost cutoff.
pub const FIRE_SPARK_STRATEGIC_ZOOM_ALPHA: f32 = FIRE_SPARK_MIN_ZOOM_ALPHA;
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

#[inline]
pub(crate) fn fire_spark_zoom_scatter_gate(zoom_alpha: f32) -> f32 {
    let za = zoom_alpha.clamp(0.0, 1.0);
    let span = (FIRE_SPARK_FULL_SCATTER_ZOOM_ALPHA - FIRE_SPARK_MIN_ZOOM_ALPHA).max(1e-4);
    ((za - FIRE_SPARK_MIN_ZOOM_ALPHA) / span).clamp(0.0, 1.0)
}
