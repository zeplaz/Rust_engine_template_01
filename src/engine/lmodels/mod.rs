//! Logic / ML research prototypes (**stub**).
//!
//! The `research_lmodels` Cargo feature enables optional `linfa*` dependencies and this module.
//! There is **no** engine integration yet — the legacy experimental sources were removed as
//! non-compiling placeholders. When implementing:
//! - wire algorithms against `linfa` / `linfa-clustering` / `linfa-nn` / `linfa-ica` / `linfa-kernel`
//! - keep the default game build free of this feature.
//!
//! A build-time `cargo:warning` is emitted from `build.rs` when `research_lmodels` is enabled.

// Reference optional crates so they are not flagged as unused when the feature is on.
#[cfg(feature = "research_lmodels")]
use {
    linfa as _, linfa_clustering as _, linfa_ica as _, linfa_kernel as _, linfa_nn as _,
};

/// Marker type reserved for future `research_lmodels` integration.
#[derive(Debug, Clone, Copy, Default)]
#[deprecated(
    note = "research_lmodels stub only — replace with real ML integration when implemented"
)]
pub struct LmodelsPlaceholder;
