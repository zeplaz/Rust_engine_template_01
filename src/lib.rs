// Processor Alpha Dine Game Engine
// Main library file

// Core modules
pub mod core;
pub mod engine;
pub mod entities;
pub mod gui;
pub mod io;
pub mod render;
pub mod systems;
pub mod terrain;
pub mod traits;
pub mod utils;
pub mod bevysubengines;
pub mod idgen;

// Re-export commonly used items
pub use engine::EnginePlugin;
pub use bevysubengines::WorldGeneratorSubenginePlugin;