//! **TRIAGE-MAP-ZOOM-SMOOTH-001** / **BUILD-READ-REWIRE-003** — lib witnesses for map zoom coherence.
//!
//! Option A: ortho zoom commits immediately; pan snaps on zoom-axis change (no translation lerp ghost).
//! Tile raster marks dirty on zoom-band crossing even during spike defer (Option B partial).


use super::MAP_ZOOM_AXIS_SNAP_EPS;

/// Green threshold — design charter (`design_map_zoom_read_v1.md`).
pub const MAP_ZOOM_PICK_DELTA_WORLD_MAX: f32 = 1.0;
/// Green threshold — ghost screen delta after settle (px).
pub const MAP_ZOOM_GHOST_SCREEN_DELTA_PX_MAX: f32 = 4.0;
/// Option A — max acceptable double-world frames per scroll step (lib contract).
pub const MAP_ZOOM_DOUBLE_WORLD_FRAMES_MAX: u32 = 1;

/// Returns true when zoom-axis change should snap pan (skip translation lerp).
#[must_use]
pub fn map_zoom_axis_snap_applies(prev_scale: f32, next_scale: f32) -> bool {
    (next_scale - prev_scale).abs() > MAP_ZOOM_AXIS_SNAP_EPS
}

#[cfg(test)]
mod tests {
    

}
