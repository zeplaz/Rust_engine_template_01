//! Developmental HUD — L0/L1 context and validation copy (`developmental_ux_runbook_v1.md` § UX-1…UX-4 scaffolding).

pub mod cause_chain;
pub mod contextual_tip;
pub mod strategic_preview;
pub mod tool_help;
pub mod validation_feedback;

pub use cause_chain::{
    update_developmental_cause_strip_system, DevelopmentalCauseStripLine, DevelopmentalCauseStripRoot,
};
pub use contextual_tip::{update_developmental_context_strip_system, DevelopmentalContextStripLine};
pub use validation_feedback::{ValidationDiagnostic, ValidationSeverity};
