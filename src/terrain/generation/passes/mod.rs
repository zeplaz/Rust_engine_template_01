//! Multi-pass terrain pipeline (material unification U4).
//!
//! **Chunk origin:** world tile at local `(lx, ly)` is `(chunk_xy.x * size.x + lx, chunk_xy.y * size.y + ly)`
//! (min-corner / grid corner), matching legacy `generate_world` integer tile indices. Chunk-center sampling would
//! offset noise by half a tile unless both code paths change together.

pub mod p1_fields;

pub use p1_fields::fill_fields;
