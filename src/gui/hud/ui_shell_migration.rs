//! Phase 2A UI shell migration — **transitional re-export shim** (cleanup-intelligence **B**).
//!
//! **Classification (Sprint 3.3 — 2026-05-24):**
//! - **Category:** B — transitional migration bridge
//! - **Authority:** [`simulation_shell_phase2`](super::simulation_shell_phase2.rs) (authoritative)
//! - **Readers:** `gui/hud/mod.rs` re-export; legacy `use crate::gui::hud::ui_shell_migration::*` call sites
//! - **Action:** **Preserve** until import paths converge on `simulation_shell_phase2` directly
//! - **Delete when:** zero external `ui_shell_migration` imports outside `hud/mod.rs` (grep gate)
//!
//! Do **not** add new systems here — extend `SimulationShellPhase2Plugin` only.

pub use super::simulation_shell_phase2::*;
