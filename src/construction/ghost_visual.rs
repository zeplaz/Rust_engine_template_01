//! Shared ghost color language (valid / invalid / pending / committed).
//!
//! Bevy mesh path (`world_map_frame` / `world_map_network_mesh`) needs higher fill alpha
//! than the retired egui painter — tokens punch valid teal / invalid red for one-glance read.
//!
//! Under **TacticalFlat2D**, ghosts stay abstract occupation/phase tokens (not APS meshes).
//! PixelWorld may later layer VoxelMassing presentation — see
//! `src/dev/plan_dual_view_pixel_world_v1.md`.

use crate::gui::ui_types::{UiRgba, UiStroke};

#[must_use]
pub fn road_segment_color(valid: bool) -> UiRgba {
    if valid {
        // Was 80,220,180 @140 — washed on mid-tone biome under FoW; raise alpha + chroma.
        UiRgba::from_rgba_unmultiplied(56, 236, 188, 210)
    } else {
        UiRgba::from_rgba_unmultiplied(255, 72, 72, 230)
    }
}

#[must_use]
pub fn road_committed_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(42, 44, 52, 230)
}

#[must_use]
pub fn road_control_point_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(255, 230, 140, 220)
}

#[must_use]
pub fn footprint_valid_color() -> UiRgba {
    // Brighter operational green — dark 48/140/72 blended into grass.
    UiRgba::from_rgba_unmultiplied(64, 210, 110, 235)
}

#[must_use]
pub fn footprint_risky_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(230, 168, 40, 240)
}

#[must_use]
pub fn footprint_invalid_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(235, 52, 52, 250)
}

/// Dark rim behind footprint fill quads (Bevy mesh outline, not a second validity pass).
#[must_use]
pub fn footprint_rim_color(valid_family: bool) -> UiRgba {
    if valid_family {
        UiRgba::from_rgba_unmultiplied(18, 72, 36, 255)
    } else {
        UiRgba::from_rgba_unmultiplied(96, 16, 16, 255)
    }
}

/// Ebbtide-style edge discipline — ~1 logical px outline weight (structure, not hue).
#[must_use]
pub fn footprint_outline_stroke_px() -> f32 {
    1.0
}

/// Massing stack rim — slightly lighter than base rim so stacked floors read as edges.
#[must_use]
pub fn massing_stack_rim_color(valid_family: bool) -> UiRgba {
    if valid_family {
        UiRgba::from_rgba_unmultiplied(28, 96, 48, 255)
    } else {
        UiRgba::from_rgba_unmultiplied(120, 28, 28, 255)
    }
}

/// Map-plane offset per stacked floor (world units) — stable under pan/zoom (camera owns scale).
#[must_use]
pub fn massing_floor_offset_world() -> f32 {
    0.16
}

/// Fill quad shrink per stacked floor (fraction of tile) — silhouette taper.
#[must_use]
pub fn massing_floor_shrink() -> f32 {
    0.06
}

#[must_use]
pub fn footprint_place_settle_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(100, 144, 220, 200)
}

#[must_use]
pub fn place_feedback_duration_secs() -> f32 {
    0.28
}

#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn footprint_lock_ring_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(232, 196, 96, 230)
}

#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn footprint_lock_ring_stroke_px() -> f32 {
    2.0
}

#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn footprint_lock_ring_stroke(locked: bool) -> Option<UiStroke> {
    if !locked {
        return None;
    }
    Some(UiStroke::new(
        footprint_lock_ring_stroke_px(),
        footprint_lock_ring_color(),
    ))
}

#[must_use]
pub fn corridor_planned_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(232, 176, 64, 180)
}

#[must_use]
pub fn corridor_in_progress_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(80, 160, 232, 200)
}

#[must_use]
pub fn corridor_phase_tokens_wired_green() -> bool {
    corridor_planned_color() == UiRgba::from_rgba_unmultiplied(232, 176, 64, 180)
        && corridor_in_progress_color() == UiRgba::from_rgba_unmultiplied(80, 160, 232, 200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_invalid_ghost_tokens_differ_in_hue_and_alpha() {
        let v = footprint_valid_color();
        let bad = footprint_invalid_color();
        assert_ne!((v.r, v.g, v.b), (bad.r, bad.g, bad.b));
        assert!(v.a >= 220);
        assert!(bad.a >= 230);
        let road_ok = road_segment_color(true);
        let road_bad = road_segment_color(false);
        assert!(road_ok.a >= 200);
        assert!(road_bad.a >= 220);
        assert_ne!(
            (road_ok.r, road_ok.g, road_ok.b),
            (road_bad.r, road_bad.g, road_bad.b)
        );
        let rim_ok = footprint_rim_color(true);
        let rim_bad = footprint_rim_color(false);
        assert_ne!((rim_ok.r, rim_ok.g, rim_ok.b), (rim_bad.r, rim_bad.g, rim_bad.b));
        assert!((footprint_outline_stroke_px() - 1.0).abs() < f32::EPSILON);
        let stack_ok = massing_stack_rim_color(true);
        assert_ne!((stack_ok.r, stack_ok.g, stack_ok.b), (rim_ok.r, rim_ok.g, rim_ok.b));
        assert!(massing_floor_offset_world() > 0.0);
        assert!(massing_floor_shrink() > 0.0);
    }
}
