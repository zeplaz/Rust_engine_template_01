//! RENDER-DIR-RESTRUCTURE-v1 — proof/witness/CI-matrix surfaces (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::stage5_*` /
//! `crate::render::hanabi_witness` / `crate::render::visual_agreement` /
//! `crate::render::spine_governance_matrix` / `crate::render::phase_f_lod_proof` /
//! `crate::render::vt_*` locations so existing call sites keep resolving.

pub mod stage5_closure_witnesses;
pub mod stage5_readiness;
pub mod hanabi_witness;
pub mod visual_agreement;
pub mod spine_governance_matrix;
pub mod phase_f_lod_proof;
pub mod vt_app_integration;
pub mod vt_ci_matrix;
pub mod vt_spatial_invariants;
