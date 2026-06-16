//! **TRIAGE-MAP-ZOOM-SMOOTH-001** / **BUILD-READ-REWIRE-003** — lib witnesses for map zoom coherence.
//!
//! Option A: ortho zoom commits immediately; pan snaps on zoom-axis change (no translation lerp ghost).
//! Tile raster marks dirty on zoom-band crossing even during spike defer (Option B partial).

use bevy::prelude::*;

use super::{
    map_camera_viewport_pixels, map_zoom_alpha_with_limits, map_zoom_limits_for_world,
    MapCameraDesired, SimulationMapViewport, MAP_ZOOM_AXIS_SNAP_EPS,
};
use super::{sim_map_screen_to_world_xy, sim_map_world_xy_to_egui};

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

/// ⟨TRIAGE-MAP-PICK-CLOSURE-001⟩ lib math — sim-map projection roundtrip at multiple zoom levels.
#[must_use]
pub fn map_pick_closure_math_witness_green() -> bool {
    map_pick_closure_math_self_check().is_ok()
}

fn map_pick_closure_math_self_check() -> Result<(), &'static str> {
    let mut vp = SimulationMapViewport::default();
    vp.valid = true;
    vp.min = Vec2::new(100.0, 50.0);
    vp.max = Vec2::new(900.0, 550.0);
    let world_w = 4096.0;
    let world_h = 4096.0;
    let viewport = map_camera_viewport_pixels(Vec2::new(1280.0, 720.0), Some(&vp));
    let (zoom_lo, zoom_hi) = map_zoom_limits_for_world(world_w, world_h, viewport);

    let base = MapCameraDesired {
        translation: Vec3::new(2048.0, 2048.0, 999.0),
        scale: Vec3::splat(2.0),
        ..Default::default()
    };

    for zoom in [zoom_lo, (zoom_lo + zoom_hi) * 0.5, zoom_hi] {
        let mut desired = base.clone();
        desired.scale = Vec3::splat(zoom);
        let alpha = map_zoom_alpha_with_limits(zoom, zoom_lo, zoom_hi);
        if !(0.0..=1.0).contains(&alpha) {
            return Err("zoom_alpha_range");
        }
        for world in [
            Vec2::new(100.0, 200.0),
            Vec2::new(2048.0, 2048.0),
            Vec2::new(3800.0, 900.0),
        ] {
            let screen = sim_map_world_xy_to_egui(world, &desired, &vp, world_w, world_h)
                .ok_or("world_to_screen")?;
            let back = sim_map_screen_to_world_xy(
                Vec2::new(screen.x, screen.y),
                &desired,
                &vp,
                world_w,
                world_h,
            )
            .ok_or("screen_to_world")?;
            if (back - world).length() > MAP_ZOOM_PICK_DELTA_WORLD_MAX {
                return Err("pick_delta_world");
            }
        }
    }
    Ok(())
}

#[must_use]
pub fn map_zoom_coherence_001_witness_green() -> bool {
    map_zoom_coherence_self_check().is_ok()
}

fn map_zoom_coherence_self_check() -> Result<(), &'static str> {
    if !map_pick_closure_math_witness_green() {
        return Err("map_pick_math");
    }
    if !map_zoom_axis_snap_applies(2.0, 2.0 + MAP_ZOOM_AXIS_SNAP_EPS * 4.0) {
        return Err("snap_trigger");
    }
    if map_zoom_axis_snap_applies(2.0, 2.0 + MAP_ZOOM_AXIS_SNAP_EPS * 0.25) {
        return Err("snap_noise_floor");
    }
    if !crate::render::tile_raster_dirty_on_zoom_band_change_enabled() {
        return Err("band_dirty_policy");
    }
    Ok(())
}

#[must_use]
pub fn map_zoom_coherence_001_witness_json() -> serde_json::Value {
    let green = map_zoom_coherence_001_witness_green();
    serde_json::json!({
        "gate": "MAP-ZOOM-COHERENCE-001",
        "green": green,
        "map_zoom_coherence_001": {
            "green": green,
            "double_world_frames_max": MAP_ZOOM_DOUBLE_WORLD_FRAMES_MAX,
            "ghost_screen_delta_px_max": MAP_ZOOM_GHOST_SCREEN_DELTA_PX_MAX,
            "pick_delta_world_max": MAP_ZOOM_PICK_DELTA_WORLD_MAX,
            "band_snap_same_frame": crate::render::tile_raster_dirty_on_zoom_band_change_enabled(),
        },
        "option": "A_snap_pan_on_zoom_axis",
        "zoom_axis_snap_eps": MAP_ZOOM_AXIS_SNAP_EPS,
        "map_pick_closure_math_ok": map_pick_closure_math_witness_green(),
        "design_ref": "src/dev/design_map_zoom_read_v1.md",
        "plan_ref": "src/dev/plan_map_zoom_smooth_exec_001_v1.md",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_zoom_coherence_001_witness_green_lib() {
        assert!(map_zoom_coherence_001_witness_green());
    }

    #[test]
    fn map_pick_closure_math_witness_green_lib() {
        assert!(map_pick_closure_math_witness_green());
    }
}
