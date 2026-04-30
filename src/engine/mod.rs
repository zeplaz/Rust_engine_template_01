// Core engine functionality
mod engine;
mod engine_with_worldgen;  // New engine implementation with world generation
pub mod states;
mod transitions;
mod sets;

// Logic models — heavy optional deps; see `research_lmodels` feature in Cargo.toml.
#[cfg(feature = "research_lmodels")]
pub mod lmodels;

// Public exports
pub use engine_with_worldgen::*;  // Use the world generation version
pub use states::*;
pub use transitions::*;
pub use sets::*;