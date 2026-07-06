//! RGR-P5-001 — render-thread perf-probe bracket modules (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::frame_perf` /
//! `crate::render::stall_watch` locations so existing call sites keep resolving.
//!
//! `visual_perf_budget.rs` was inspected and **not** moved here — it is a production visual
//! perf *policy/feedback* module (`TileRasterSpikeFeedback` zoom-dirty deferral), not a thin
//! timing probe; it stays at `render/visual_perf_budget.rs` (RGR-P5-001 skip, reported).

pub mod frame_perf;
pub mod stall_watch;

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: debug/diagnostic surfaces staying render-side.
pub mod debug_render_trace;
pub mod debug_viewport_overlay;
pub mod visual_diagnostics;
pub mod full_render_diagnostic;
pub mod vfx_capture_hook;

// RTT-SPRITE-TRACE — temporary render-world probe (env-gated `RTT_SPRITE_TRACE`).
pub mod rtt_sprite_trace;
