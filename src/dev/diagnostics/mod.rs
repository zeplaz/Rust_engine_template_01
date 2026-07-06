//! # Dev diagnostics — refactor-by-extraction home for witness telemetry
//!
//! Witness JSON under `debug_runs/` is **telemetry**, not render pipeline logic.
//! This module owns passive models, event subscribers, and disk I/O delegation
//! to [`crate::dev::runtime_witness`].
//!
//! ## Architecture
//!
//! ```text
//! RenderApp probes (thin, timed) ──channel/event──► MainWorld subscribers (this module)
//!                                                          │
//!                                                          ▼
//!                                              runtime_witness::write_enveloped_witness
//! ```
//!
//! ## Migration checklist (batch order)
//!
//! 1. **Extract data** — move struct defs to [`models`] (started; render still owns canonical copies until batch 1 lands).
//! 2. **Redirect senders** — render probes call `MessageWriter<DiagnosticEvent>` via [`OffloadDiagnostics`] instead of `ResMut<Witness>`.
//! 3. **Strip render re-exports** — remove witness plugins from `render/mod.rs` pub use block.
//! 4. **Wire receiver** — register [`DevDiagnosticsPlugin`] in `engine_with_worldgen.rs` (gated by [`dev_diagnostics_enabled`]).
//! 5. **Render schedule** — passive models in [`render_schedule`]; RenderApp probes removed (Great Unhook P0).
//! 6. **Split `mig_a_adoption.rs`** — runtime adoption (`MigAStaticBulk`, spine flags) in `render/mig_a_static.rs`; plugin + audit JSON in `dev/mig_a_adoption.rs`.
//!
//! Aligns with cleanup **D3** (gate unconditional diagnostic plugins) and **RN-PERF → DV** routing in `codebase_index_v1.md`.
//!
//! ## Do NOT move here
//!
//! - `frame_perf` — always-on perf spine (cleanup D3 keeps one spine in render).
//! - Domain witness collectors already under `src/dev/runtime_witness/` or `*_witness_collectors.rs`.

pub mod events;
pub mod models;
pub mod offload;
pub mod perf_attribution;
pub mod plugin;
pub mod render_schedule;
pub mod subscribers;
pub mod view_authority_sample;
pub mod visual_capture_probe;
pub mod visual_readiness;

pub use events::{
    DiagnosticEvent, MigAAuditEvent, PerfAttributionEvent, RenderScheduleEvent,
    VisualReadinessEvent,
};
pub use models::{
    visual_readiness_witness_json, PerfAttributionWitness, RenderScheduleHandoffMs,
    RenderScheduleSpans, RenderScheduleWitness, VisualReadinessWitness, PERF_ATTRIBUTION_WINDOW,
};
pub use perf_attribution::{
    perf_attribution_witness_json, perf_attribution_witness_lib_fixture,
    percentile_from_slice, reset_perf_attribution_witness_on_enter_simulation,
    sync_perf_attribution_witness_system,
};
pub use visual_readiness::{
    reset_visual_readiness_witness_on_enter_simulation, sync_visual_readiness_witness_system,
    visual_readiness_witness_lib_fixture, VisualReadinessWitnessPlugin,
};
pub use offload::OffloadDiagnostics;
pub use plugin::{dev_diagnostics_enabled, DevDiagnosticsPlugin};
pub use view_authority_sample::view_authority_sample_json;
pub use visual_capture_probe::{
    visual_capture_probe_enabled, VisualCaptureProbePlugin, VisualCaptureProbeState,
};
