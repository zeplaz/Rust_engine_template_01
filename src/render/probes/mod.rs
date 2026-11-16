//! Render-thread probes — perf brackets + GPU capture / inspection (DBG-04).
//!
//! **V4-DBG04** ADOPT: **DBG-04** (gea4v2:12.9.4) · LEGACY **`render_debug_tools`** (gea_v2_e4 p248).
//! Capture / inspection are env-gated (`VFX_CAPTURE` / `RTT_SPRITE_TRACE`) via [`vfx_capture_env`];
//! never ship as play panels (AGENTS.md). Do not dual-write RND-03 overlay order or RND-04
//! transparent / cloud Z keys from this tree.
//!
//! RGR-P5-001 — render-thread perf-probe bracket modules (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::probes::frame_perf` /
//! `crate::render::stall_watch` locations so existing call sites keep resolving.
//!
//! `visual_perf_budget.rs` was inspected and **not** moved here — it is a production visual
//! perf *policy/feedback* module (`TileRasterSpikeFeedback` zoom-dirty deferral), not a thin
//! timing probe; it stays at `render/visual_perf_budget.rs` (RGR-P5-001 skip, reported).

pub mod frame_hitch;
pub mod frame_perf;
pub mod stall_watch;

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: debug/diagnostic surfaces staying render-side.
pub mod debug_render_trace;
pub mod debug_viewport_overlay;
pub mod visual_diagnostics;
pub mod vfx_capture_env;
pub mod vfx_capture_hook;

// RTT-SPRITE-TRACE — DBG-04 inspection probe (env-gated `RTT_SPRITE_TRACE`).
pub mod rtt_sprite_trace;
