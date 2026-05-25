//! Windowed row rendering for large HUD tables.

use bevy_egui::egui;

pub fn draw_virtualized_rows(
    ui: &mut egui::Ui,
    id: &str,
    row_height: f32,
    viewport_height: f32,
    row_count: usize,
    mut draw_row: impl FnMut(&mut egui::Ui, usize),
) {
    if row_count == 0 {
        return;
    }
    let row_height = row_height.max(16.0);
    let viewport_height = viewport_height.max(row_height);
    egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(viewport_height)
        .show(ui, |ui| {
            let scroll = ui.clip_rect().top() - ui.min_rect().top();
            let first = (scroll / row_height).floor().max(0.0) as usize;
            let visible = (viewport_height / row_height).ceil() as usize + 2;
            let last = (first + visible).min(row_count);
            ui.allocate_space(egui::vec2(ui.available_width(), first as f32 * row_height));
            for row in first..last {
                draw_row(ui, row);
            }
            let remaining = row_count.saturating_sub(last);
            ui.allocate_space(egui::vec2(ui.available_width(), remaining as f32 * row_height));
        });
}
