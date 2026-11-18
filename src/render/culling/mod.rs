//! Visibility helpers (MATH-05 / RND-02). Frustum cull feeds GPU-CULL indirect draw.

pub mod frustum;

pub use frustum::{FrustumPlanes, Plane4};
