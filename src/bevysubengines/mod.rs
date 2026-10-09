//! **PARKED** legacy world-generation subengine (`legacy_inventory_v1.md` LG-09, cleanup R9).
//!
//! `WorldGeneratorSubenginePlugin` is not added to any app. The designer-facing path is
//! `terrain::generation::WorldGenToolsPlugin` (+ `world_generator_enhanced::WorldGenParams`); this
//! module keeps a separate `WorldGenParams` and a `WorldData` save format that nothing reads.
//! Do not register it. Unblock for removal: capture the save-format notes in a doc/test and retarget
//! the two `include_str!` guard tests that pin its run condition
//! (`gui/editor/world_preview/world_gen_bevy_shell.rs`, `gui/hud/shell_diagnostics.rs`).
mod world_generator_plugin;

// Public exports
pub use world_generator_plugin::*;
