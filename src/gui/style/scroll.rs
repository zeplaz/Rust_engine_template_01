//! Scroll helpers — egui ScrollArea retired; Bevy UI owns overflow.

/// Placeholder salt type for call-site compatibility (no-op after egui drop).
#[derive(Clone, Debug)]
pub struct RetiredScrollArea;

#[must_use]
pub fn widget_scroll_vertical(_id_salt: impl std::hash::Hash + std::fmt::Debug) -> RetiredScrollArea {
    RetiredScrollArea
}

#[must_use]
pub fn widget_scroll_vertical_capped(
    _id_salt: impl std::hash::Hash + std::fmt::Debug,
    _max_height: f32,
) -> RetiredScrollArea {
    RetiredScrollArea
}

#[must_use]
pub fn widget_scroll_vertical_fill(
    _id_salt: impl std::hash::Hash + std::fmt::Debug,
    _available_height: f32,
) -> RetiredScrollArea {
    RetiredScrollArea
}

#[must_use]
pub fn widget_scroll_both(_id_salt: impl std::hash::Hash + std::fmt::Debug) -> RetiredScrollArea {
    RetiredScrollArea
}

pub fn apply_scroll_style(_palette: &super::UiPalette) {}
