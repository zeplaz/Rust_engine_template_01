//! **INT-PICK** — single screen → tactical-map pick authority (fleet `interaction`, ledger
//! `src/dev/interaction_bug_ledger_v1.md`).
//!
//! Every world pick on the simulation map (building ghost, demolish, road / rail / power-line /
//! zone drawing, hover cards) resolves through [`TacticalMapPickFrame`], so a pick lands on the
//! world point under the cursor **in the frame the player is looking at**: the rendered RTT ortho
//! ([`MainWorldCameraOrthoTrace`]) first, the desired camera only before the first ortho fit.
//!
//! **Pick contract (survives a 3D / perspective main camera — fleet `graphics`):** a pick is
//! "logical cursor px → point on the ground plane in tactical world XY (x = column, y = row,
//! +Y north)". If the main camera stops being a top-down ortho, replace
//! [`TacticalMapPickFrame::screen_to_world_xy`] / [`TacticalMapPickFrame::world_xy_to_screen`]
//! with a camera-ray → ground intersection and its inverse; callers and the
//! `input_interaction_regression` suite stay unchanged.

use bevy::math::{UVec2, Vec2};

use super::tactical::map_camera::{
    orthographic_fixed_world_span, MainWorldCameraOrthoTrace,
};
use super::tactical::map_camera::map_camera_rtt_view_pixels;
use super::SimulationMapViewport;

/// Which camera state produced a [`TacticalMapPickFrame`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapPickSource {
    /// Last ortho actually rendered into the tactical RTT (what is on screen).
    RenderedOrtho,
    /// Desired camera + RTT texture extent (no ortho fit yet).
    DesiredCamera,
}

/// Screen rect of the tactical map hole + the world span it shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TacticalMapPickFrame {
    /// Fill rect min (logical px, egui / window space).
    pub screen_min: Vec2,
    /// Fill rect max (logical px).
    pub screen_max: Vec2,
    /// World XY under the fill rect center.
    pub world_center: Vec2,
    /// World units spanned by the whole fill rect.
    pub world_span: Vec2,
    pub source: MapPickSource,
}

impl TacticalMapPickFrame {
    /// Frame for the ortho last written to the main world camera.
    #[must_use]
    pub fn from_rendered_ortho(
        map_vp: &SimulationMapViewport,
        ortho: &MainWorldCameraOrthoTrace,
    ) -> Option<Self> {
        if !map_vp.is_adequate_for_camera() {
            return None;
        }
        if !(ortho.fixed_width > 0.0 && ortho.fixed_height > 0.0)
            || !ortho.fixed_width.is_finite()
            || !ortho.fixed_height.is_finite()
        {
            return None;
        }
        Some(Self {
            screen_min: map_vp.min,
            screen_max: map_vp.max,
            world_center: ortho.camera_center,
            world_span: Vec2::new(ortho.fixed_width, ortho.fixed_height),
            source: MapPickSource::RenderedOrtho,
        })
    }

    /// Frame derived from a camera pose. `tex_extent` is the RTT texture size; when it differs
    /// from the fill aspect the ortho letterboxes exactly like [`map_camera_rtt_view_pixels`].
    #[must_use]
    pub fn from_camera(
        map_vp: &SimulationMapViewport,
        camera_center: Vec2,
        zoom: f32,
        tex_extent: Vec2,
    ) -> Option<Self> {
        if !map_vp.is_adequate_for_camera() || !zoom.is_finite() {
            return None;
        }
        let view_px = map_camera_rtt_view_pixels(map_vp, tex_extent);
        let (w, h) = orthographic_fixed_world_span(view_px, zoom.abs().max(1e-6), 0.0, 0.0);
        Some(Self {
            screen_min: map_vp.min,
            screen_max: map_vp.max,
            world_center: camera_center,
            world_span: Vec2::new(w, h),
            source: MapPickSource::DesiredCamera,
        })
    }

    /// Rendered ortho first; camera pose only before the first ortho fit.
    #[must_use]
    pub fn resolve(
        map_vp: &SimulationMapViewport,
        ortho: Option<&MainWorldCameraOrthoTrace>,
        camera_center: Vec2,
        zoom: f32,
        tex_extent: Vec2,
    ) -> Option<Self> {
        ortho
            .and_then(|o| Self::from_rendered_ortho(map_vp, o))
            .or_else(|| Self::from_camera(map_vp, camera_center, zoom, tex_extent))
    }

    #[inline]
    #[must_use]
    pub fn screen_center(&self) -> Vec2 {
        (self.screen_min + self.screen_max) * 0.5
    }

    #[inline]
    #[must_use]
    pub fn screen_size(&self) -> Vec2 {
        (self.screen_max - self.screen_min).max(Vec2::ONE)
    }

    /// World units per logical screen pixel on each axis.
    #[inline]
    #[must_use]
    pub fn world_per_px(&self) -> Vec2 {
        self.world_span / self.screen_size()
    }

    #[must_use]
    pub fn contains_screen(&self, cursor: Vec2) -> bool {
        cursor.x >= self.screen_min.x
            && cursor.y >= self.screen_min.y
            && cursor.x <= self.screen_max.x
            && cursor.y <= self.screen_max.y
    }

    /// Logical cursor → world XY. World +Y is north; screen +Y is down.
    #[must_use]
    pub fn screen_to_world_xy(&self, cursor: Vec2) -> Vec2 {
        let d = (cursor - self.screen_center()) * self.world_per_px();
        Vec2::new(self.world_center.x + d.x, self.world_center.y - d.y)
    }

    /// Inverse of [`Self::screen_to_world_xy`].
    #[must_use]
    pub fn world_xy_to_screen(&self, world: Vec2) -> Vec2 {
        let per_px = self.world_per_px().max(Vec2::splat(1e-9));
        let d = world - self.world_center;
        self.screen_center() + Vec2::new(d.x / per_px.x, -d.y / per_px.y)
    }
}

/// World XY → tile, rejecting anything outside `0..world_w × 0..world_h` (INT-002).
///
/// `floor().max(0.0) as u32` silently clamped off-world picks onto row/column 0.
#[must_use]
pub fn world_xy_to_tile_in_world(world: Vec2, world_w: u32, world_h: u32) -> Option<UVec2> {
    if !world.x.is_finite() || !world.y.is_finite() || world.x < 0.0 || world.y < 0.0 {
        return None;
    }
    let x = world.x.floor() as u32;
    let y = world.y.floor() as u32;
    (x < world_w && y < world_h).then_some(UVec2::new(x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vp(min: Vec2, max: Vec2) -> SimulationMapViewport {
        let mut v = SimulationMapViewport::default();
        v.valid = true;
        v.min = min;
        v.max = max;
        v
    }

    #[test]
    fn roundtrip_screen_world_screen() {
        let v = vp(Vec2::new(100.0, 40.0), Vec2::new(1700.0, 940.0));
        for zoom in [0.05_f32, 0.5, 1.0, 4.0, 60.0] {
            let f = TacticalMapPickFrame::from_camera(&v, Vec2::new(300.0, 700.0), zoom, v.logical_size())
                .unwrap();
            for c in [Vec2::new(101.0, 41.0), Vec2::new(900.0, 490.0), Vec2::new(1699.0, 939.0)] {
                let w = f.screen_to_world_xy(c);
                let back = f.world_xy_to_screen(w);
                assert!((back - c).length() < 1e-2, "zoom {zoom}: {c} -> {w} -> {back}");
            }
        }
    }

    #[test]
    fn camera_frame_matches_rendered_ortho_when_texture_letterboxes() {
        // 5120 px wide fill, RTT clamped to 4096 px (RTT_MAX_EXTENT) → ortho letterboxes.
        let v = vp(Vec2::ZERO, Vec2::new(5120.0, 1440.0));
        let tex = Vec2::new(4096.0, 1440.0);
        let zoom = 2.0;
        let center = Vec2::new(1000.0, 1000.0);
        let view_px = map_camera_rtt_view_pixels(&v, tex);
        let (fw, fh) = orthographic_fixed_world_span(view_px, zoom, 0.0, 0.0);
        let ortho = MainWorldCameraOrthoTrace {
            fixed_width: fw,
            fixed_height: fh,
            view_pixels: view_px,
            desired_zoom: zoom,
            authority_zoom: zoom,
            camera_center: center,
        };
        let a = TacticalMapPickFrame::from_rendered_ortho(&v, &ortho).unwrap();
        let b = TacticalMapPickFrame::from_camera(&v, center, zoom, tex).unwrap();
        let cursor = Vec2::new(4800.0, 200.0);
        assert!((a.screen_to_world_xy(cursor) - b.screen_to_world_xy(cursor)).length() < 1e-3);
    }

    #[test]
    fn tile_rejects_off_world() {
        assert_eq!(world_xy_to_tile_in_world(Vec2::new(-0.01, 3.0), 10, 10), None);
        assert_eq!(world_xy_to_tile_in_world(Vec2::new(3.0, -40.0), 10, 10), None);
        assert_eq!(world_xy_to_tile_in_world(Vec2::new(10.0, 3.0), 10, 10), None);
        assert_eq!(world_xy_to_tile_in_world(Vec2::new(3.0, 10.0), 10, 10), None);
        assert_eq!(world_xy_to_tile_in_world(Vec2::new(f32::NAN, 1.0), 10, 10), None);
        assert_eq!(
            world_xy_to_tile_in_world(Vec2::new(9.99, 0.0), 10, 10),
            Some(UVec2::new(9, 0))
        );
    }

    #[test]
    fn degenerate_ortho_falls_back_to_camera() {
        let v = vp(Vec2::ZERO, Vec2::new(800.0, 600.0));
        let bad = MainWorldCameraOrthoTrace::default();
        let f = TacticalMapPickFrame::resolve(&v, Some(&bad), Vec2::new(5.0, 5.0), 1.0, v.logical_size())
            .unwrap();
        assert_eq!(f.source, MapPickSource::DesiredCamera);
    }
}
