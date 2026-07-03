//! Shared [`egui::ScrollArea`] defaults so overflow regions show usable scroll bars.

use bevy_egui::egui::{self, ScrollArea};
use bevy_egui::egui::scroll_area::ScrollBarVisibility;

fn configure_scroll_area(area: ScrollArea) -> ScrollArea {
    area.auto_shrink([false, false])
        .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
}

/// Vertical scroll for a panel body; use [`widget_scroll_vertical_capped`] when height is fixed.
#[must_use]
pub fn widget_scroll_vertical(id_salt: impl std::hash::Hash + std::fmt::Debug) -> ScrollArea {
    configure_scroll_area(ScrollArea::vertical().id_salt(id_salt))
}

/// Vertical scroll with an explicit max height (lists, log panes).
#[must_use]
pub fn widget_scroll_vertical_capped(id_salt: impl std::hash::Hash + std::fmt::Debug, max_height: f32) -> ScrollArea {
    configure_scroll_area(
        ScrollArea::vertical()
            .id_salt(id_salt)
            .max_height(max_height.max(48.0)),
    )
}

/// Vertical scroll sized to the remaining space in the current layout.
#[must_use]
pub fn widget_scroll_vertical_fill(id_salt: impl std::hash::Hash + std::fmt::Debug, available_height: f32) -> ScrollArea {
    configure_scroll_area(
        ScrollArea::vertical()
            .id_salt(id_salt)
            .max_height(available_height.max(96.0)),
    )
}

/// Horizontal + vertical scroll (large canvases, minimap editors).
#[must_use]
pub fn widget_scroll_both(id_salt: impl std::hash::Hash + std::fmt::Debug) -> ScrollArea {
    configure_scroll_area(ScrollArea::both().id_salt(id_salt))
}

/// Apply solid, high-contrast scroll bars for the CMD palette.
pub fn apply_scroll_style(style: &mut egui::Style, palette: &super::UiPalette) {
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.bar_width = 10.0;
    scroll.handle_min_length = 18.0;
    scroll.bar_inner_margin = 4.0;
    scroll.bar_outer_margin = 2.0;
    scroll.foreground_color = true;
    style.spacing.scroll = scroll;
    style.visuals.widgets.inactive.fg_stroke.color = palette.fg_muted;
}
