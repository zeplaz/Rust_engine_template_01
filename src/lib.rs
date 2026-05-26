// Processor Alpha Dine Game Engine
// Main library file
//
// Package id `proc_A_dine01` is intentionally non–snake_case; renaming would churn imports/Cargo.
#![allow(non_snake_case)]
#![recursion_limit = "256"]

// Core modules
pub mod ai;
pub mod core;
pub mod events;
pub mod engine;
pub mod entities;
pub mod gui;
pub mod io;
pub mod compute;
pub mod render;
pub mod systems;
pub mod strategic;
pub mod terrain;
pub mod traits;
pub mod utils;
pub mod dev;
pub mod construction;
pub mod economy;
pub mod bevysubengines;
pub mod idgen;
pub mod scenario;

// Re-export commonly used items
pub use ai::construction::{ConstructionAiConfig, ConstructionAiOwner, ConstructionAiPlugin};
pub use engine::EnginePlugin;
pub use bevysubengines::WorldGeneratorSubenginePlugin;
pub use strategic::{LogisticsGraph, StrategicFieldsAndAiPlugin, StrategicFieldsPlugin};