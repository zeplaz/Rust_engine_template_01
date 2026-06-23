//! **ENGINE-DEEP-DEBUG-001** — intrusive debug build: minimap/GPU/schedule/memory witnesses.
//!
//! Enable at compile time: `--features engine_deep_debug --profile dev-deep-debug`
//! Enable at runtime (any build): `RUST_ENGINE_DEEP_DEBUG=1` or CLI `--deep-debug`
//!
//! Runbook: [`engine_deep_debug_runbook_v1.md`](engine_deep_debug_runbook_v1.md)

mod frame_probe;
mod latch;
mod minimap_trace;
mod plugin;
mod subsystem_cache;
mod subsystem_probe;
mod witness;

pub use latch::{arm_deep_debug_from_cli, deep_debug_active, deep_debug_feature_compiled, DeepDebugConfig};
pub use minimap_trace::{
    record_minimap_compositor_decision, record_minimap_egui_bind, snapshot_minimap_after_compositor_pass,
    MinimapCompositorDecision,
};
pub use plugin::EngineDeepDebugPlugin;
pub use witness::DEEP_DEBUG_WITNESS_REL;

/// Log when deep debug is active (compile-time or runtime latch).
#[macro_export]
macro_rules! deep_debug_log {
    ($target:expr, $($arg:tt)*) => {
        if $crate::dev::engine_deep_debug::deep_debug_active() {
            bevy::log::debug!(target: $target, $($arg)*);
        }
    };
}

/// Trace-level log — only in `engine_deep_debug` feature builds.
#[macro_export]
macro_rules! deep_debug_trace {
    ($target:expr, $($arg:tt)*) => {
        if $crate::dev::engine_deep_debug::deep_debug_active() {
            bevy::log::trace!(target: $target, $($arg)*);
        }
    };
}
