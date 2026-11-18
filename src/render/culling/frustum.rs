//! MATH-05 / RND-02 — frustum primitives from GEA 4e Vol I §5.6.3–6 + Vol II §11.5.1.2.
//!
//! Plane as `[n, d]` (`h = n·P + d`). Sphere vs six planes: inside when `h_i >= -radius`
//! for all planes (normals point inward). AABB uses the “positive vertex” per plane.
//!
//! Production caller: [`crate::render::pipelines::gpu_frustum_cull::sync_fire_frustum_cull_frame`]
//! (BOOK-S2 / GPU-CULL.1).

use bevy::math::{Mat4, Vec3, Vec4};

/// Plane `n·P + d = 0` with unit normal pointing toward the **inside** of the frustum.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane4 {
    pub n: Vec3,
    pub d: f32,
}

impl Plane4 {
    #[must_use]
    pub fn from_normal_distance(n: Vec3, d: f32) -> Self {
        let len = n.length();
        if len > 1e-8 {
            Self {
                n: n / len,
                d: d / len,
            }
        } else {
            Self { n: Vec3::Y, d: 0.0 }
        }
    }

    /// Signed distance `h = n·P + d` (positive = inside / front for inward normals).
    #[inline]
    #[must_use]
    pub fn signed_distance(self, p: Vec3) -> f32 {
        self.n.dot(p) + self.d
    }
}

/// Six inward-facing frustum planes (order: left, right, bottom, top, near, far).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrustumPlanes {
    pub planes: [Plane4; 6],
}

impl FrustumPlanes {
    /// Extract clip-space frustum from a view-projection matrix (column-major Bevy `Mat4`).
    ///
    /// Uses Gribb/Hartmann-style row combinations so tests work in world space.
    #[must_use]
    pub fn from_view_proj(vp: Mat4) -> Self {
        let r0 = vp.row(0);
        let r1 = vp.row(1);
        let r2 = vp.row(2);
        let r3 = vp.row(3);
        let mk = |v: Vec4| Plane4::from_normal_distance(v.truncate(), v.w);
        Self {
            planes: [
                mk(r3 + r0), // left
                mk(r3 - r0), // right
                mk(r3 + r1), // bottom
                mk(r3 - r1), // top
                mk(r3 + r2), // near
                mk(r3 - r2), // far
            ],
        }
    }

    /// Book sphere cull: center must satisfy `h_i >= -radius` for all planes.
    #[must_use]
    pub fn contains_sphere(self, center: Vec3, radius: f32) -> bool {
        let r = radius.max(0.0);
        self.planes
            .iter()
            .all(|p| p.signed_distance(center) >= -r)
    }

    /// AABB cull via positive-vertex (p-vertex) test — conservative for frustum.
    #[must_use]
    pub fn contains_aabb(self, min: Vec3, max: Vec3) -> bool {
        for p in &self.planes {
            let px = if p.n.x >= 0.0 { max.x } else { min.x };
            let py = if p.n.y >= 0.0 { max.y } else { min.y };
            let pz = if p.n.z >= 0.0 { max.z } else { min.z };
            if p.signed_distance(Vec3::new(px, py, pz)) < 0.0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math05_unit_plane_distance() {
        let p = Plane4::from_normal_distance(Vec3::Y * 2.0, -4.0);
        assert!((p.n - Vec3::Y).length() < 1e-5);
        assert!((p.d - (-2.0)).abs() < 1e-5);
        assert!((p.signed_distance(Vec3::new(0.0, 3.0, 0.0)) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn math05_sphere_outside_single_plane() {
        let fr = FrustumPlanes {
            planes: [
                Plane4::from_normal_distance(Vec3::X, 0.0),
                Plane4::from_normal_distance(-Vec3::X, 10.0),
                Plane4::from_normal_distance(Vec3::Y, 0.0),
                Plane4::from_normal_distance(-Vec3::Y, 10.0),
                Plane4::from_normal_distance(Vec3::Z, 0.0),
                Plane4::from_normal_distance(-Vec3::Z, 10.0),
            ],
        };
        assert!(fr.contains_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0));
        assert!(!fr.contains_sphere(Vec3::new(-2.0, 5.0, 5.0), 1.0));
        assert!(fr.contains_sphere(Vec3::new(-0.5, 5.0, 5.0), 1.0));
    }

    #[test]
    fn math05_aabb_rejected_when_fully_outside() {
        let fr = FrustumPlanes {
            planes: [
                Plane4::from_normal_distance(Vec3::X, 0.0),
                Plane4::from_normal_distance(-Vec3::X, 10.0),
                Plane4::from_normal_distance(Vec3::Y, 0.0),
                Plane4::from_normal_distance(-Vec3::Y, 10.0),
                Plane4::from_normal_distance(Vec3::Z, 0.0),
                Plane4::from_normal_distance(-Vec3::Z, 10.0),
            ],
        };
        assert!(fr.contains_aabb(Vec3::splat(1.0), Vec3::splat(2.0)));
        assert!(!fr.contains_aabb(Vec3::splat(11.0), Vec3::splat(12.0)));
    }
}
