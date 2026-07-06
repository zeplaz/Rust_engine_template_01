//! Re-export event bus from render-safe module.
pub use crate::dev::diagnostic_events::{
    DiagnosticEvent, MigAAuditEvent, PerfAttributionEvent, RenderScheduleEvent,
    VisualReadinessEvent,
};
