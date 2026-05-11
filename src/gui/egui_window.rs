//! Shared egui [`Window`](bevy_egui::egui::Window) options for dev/floating panels.

use bevy_egui::egui;

/// Resizable panels that are not clamped to the parent viewport (avoids awkward top/bottom clipping).
#[inline]
#[must_use]
pub fn std_floating(w: egui::Window<'_>) -> egui::Window<'_> {
    w.resizable(true).constrain(false)
}
