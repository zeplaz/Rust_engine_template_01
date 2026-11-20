//! Read-only accessors on [`SurfaceWaterField`] for the gameplay water query ([`super::WaterTileProbe`])
//! and the incremental flood publisher (WM-4). Kept apart from `surface_water.rs` (solver owner):
//! nothing here writes the field.

use bevy::math::{IVec2, Vec2};

use super::surface_water::SurfaceWaterField;

/// Depth-averaged speed cap (m/s), same clamp as `cell_speed_mps`.
const MAX_SPEED_MPS: f32 = 50.0;

impl SurfaceWaterField {
    /// Water surface height (world metres) over world tile `(wx, wz)`; `None` outside a wired field.
    #[inline]
    #[must_use]
    pub fn surface_at_world(&self, wx: i32, wz: i32) -> Option<f32> {
        if !self.wired || !self.contains_world(wx, wz) {
            return None;
        }
        let i = self.idx((wx - self.origin_x) as u32, (wz - self.origin_z) as u32);
        Some(self.surface_at(i))
    }

    /// Depth-averaged flow velocity (m/s, x / z of the map) over world tile `(wx, wz)`.
    /// Zero on a dry cell; `None` outside a wired field.
    #[inline]
    #[must_use]
    pub fn velocity_at_world(&self, wx: i32, wz: i32) -> Option<Vec2> {
        if !self.wired || !self.contains_world(wx, wz) {
            return None;
        }
        let i = self.idx((wx - self.origin_x) as u32, (wz - self.origin_z) as u32);
        let depth = self.depth_m.get(i).copied().unwrap_or(0.0);
        if depth < 1.0e-3 {
            return Some(Vec2::ZERO);
        }
        // q is the flux across a face ~ speed * depth * dx.
        let k = 1.0 / (depth * self.dx_m.max(1.0));
        let v = Vec2::new(
            self.qx_m3s.get(i).copied().unwrap_or(0.0),
            self.qy_m3s.get(i).copied().unwrap_or(0.0),
        ) * k;
        Some(v.clamp_length_max(MAX_SPEED_MPS))
    }

    /// Number of activity tiles the field is cut into.
    #[inline]
    #[must_use]
    pub fn water_tile_count(&self) -> usize {
        self.tiles.tile_count()
    }

    /// Activity tiles the last solver step touched; water can only have moved inside these (and the
    /// tiles a barrier / obstacle / bed edit woke, which the next step lists).
    #[inline]
    #[must_use]
    pub fn stepped_water_tiles(&self) -> &[u32] {
        &self.tiles.active
    }

    /// Local cell rectangle `[x0, x1) x [z0, z1)` of activity tile `t`.
    #[inline]
    #[must_use]
    pub fn water_tile_rect(&self, t: usize) -> (usize, usize, usize, usize) {
        self.tiles.tile_rect(t)
    }

    /// World tile of local cell index `i`.
    #[inline]
    #[must_use]
    pub fn world_tile_of_index(&self, i: usize) -> IVec2 {
        let w = self.width.max(1) as usize;
        IVec2::new(self.origin_x + (i % w) as i32, self.origin_z + (i / w) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_and_velocity_follow_the_cell() {
        let mut f = SurfaceWaterField::default();
        f.allocate(10, 20, 4, 4, vec![5.0; 16], 0.0);
        let i = f.idx(1, 2);
        f.depth_m[i] = 2.0;
        f.qx_m3s[i] = 2.0 * 3.0 * f.dx_m.max(1.0); // 3 m/s along x at 2 m deep
        assert_eq!(f.surface_at_world(11, 22), Some(7.0));
        let v = f.velocity_at_world(11, 22).expect("inside");
        assert!((v.x - 3.0).abs() < 1.0e-4 && v.y.abs() < 1.0e-6, "{v:?}");
        assert_eq!(f.velocity_at_world(10, 20), Some(Vec2::ZERO));
        assert_eq!(f.surface_at_world(0, 0), None, "outside the field");
    }
}
