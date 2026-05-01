//! Multi-pass terrain pipeline (material unification U4).
//!
//! **Chunk origin:** world tile at local `(lx, ly)` is `(chunk_xy.x * size.x + lx, chunk_xy.y * size.y + ly)`
//! (min-corner / grid corner), matching legacy `generate_world` integer tile indices. Chunk-center sampling would
//! offset noise by half a tile unless both code paths change together.

pub mod p1_fields;
pub mod p2_threshold_tags;
pub mod p3_classify;
pub mod p4_hydrology;
pub mod p5_agent_overlay;
pub mod p6_materialize;

pub use p1_fields::fill_fields;
pub use p2_threshold_tags::apply_threshold_tags;
pub use p3_classify::classify_cells;
pub use p4_hydrology::{
    apply_hydrology, apply_hydrology_chunk, apply_hydrology_with_params,
};
pub use p5_agent_overlay::apply_agent_overlay;
pub use p6_materialize::{materialize, MaterializedChunkData};
#[cfg(feature = "dev_tools")]
pub use p6_materialize::materialize_traced;
