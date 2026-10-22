//! Retired egui map overlay controls.
//!
//! Simulation presentation toggle lives in [`crate::gui::world_presentation_mode`]
//! (`WorldPresentationModePlugin` — **View: Tactical / World** Bevy UI button).
//! Ops-strip height/slope remains [`crate::gui::map_tile_raster::PlayMapPixelMode`].

/// No-op stub retained for callers that still import the old egui entry.
/// Prefer [`crate::gui::WorldPresentationMode`] + sim shell toggle.
pub fn map_overlay_controls_ui() {}
