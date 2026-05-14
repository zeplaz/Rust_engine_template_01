//! Minimal **snapshot extraction** contract (`base_visual_dev01_plan_status.md` § `arch-generic-snapshot-layer`).
//! Registry / scheduling graph stays out until `FireVisualFrame` stamp semantics align with phase E.

/// One logical render snapshot producer (sim → owned buffer, no gameplay writes).
///
/// Implementations will attach to concrete extract resources in a follow-up; the trait anchors naming
/// and `Output` typing for interpolation / replay later.
pub trait ExtractFrameSnapshot: Send + Sync + 'static {
    type Output: Send + Sync + 'static;
}
