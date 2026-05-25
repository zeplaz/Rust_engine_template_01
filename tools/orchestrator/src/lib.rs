//! Architectural intent-preservation orchestrator for the Rust engine template.
//!
//! Parses `cargo --message-format=json` diagnostics, classifies warnings by migration
//! state (not severity alone), traces subsystem ownership, and emits runbooks +
//! agent coordination artifacts. **Does not auto-delete warnings.**

pub mod architectural;
pub mod authority_scan;
pub mod cargo_collect;
pub mod classify;
pub mod drift;
pub mod knowledge;
pub mod main_thread_shift;
pub mod models;
pub mod ownership;
pub mod plan_slice;
pub mod pipeline;
pub mod reports;
pub mod scanner;
pub mod state;
pub mod subsystem;

pub use models::*;
pub use pipeline::run_build_pipeline;
pub use plan_slice::{run_plan_slice, ContinuationTask, PlanSliceReport};
