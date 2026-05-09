// Core engine functionality
mod engine;
mod engine_with_worldgen; // New engine implementation with world generation
pub mod launch_args;
pub mod states;
pub mod test_harness;
mod transitions;
mod sets;

// Logic models — heavy optional deps; see `research_lmodels` feature in Cargo.toml.
#[cfg(feature = "research_lmodels")]
pub mod lmodels;

// Public exports
pub use engine_with_worldgen::*; // Use the world generation version
pub use launch_args::{EngineLaunchArgs, TestScene};
pub use states::*;
pub use test_harness::{ActiveTestScene, TestHarnessPlugin, TestWorldHarness};
pub use transitions::*;
pub use sets::*;