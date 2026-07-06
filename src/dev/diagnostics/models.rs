//! Passive witness **data models** — re-exports for subscribers (**CB-RGR-001**).

pub use super::perf_attribution::{PerfAttributionWitness, PERF_ATTRIBUTION_WINDOW};
pub use super::render_schedule::{
    RenderScheduleHandoffMs, RenderScheduleSpans, RenderScheduleWitness,
};
pub use super::visual_readiness::{visual_readiness_witness_json, VisualReadinessWitness};
