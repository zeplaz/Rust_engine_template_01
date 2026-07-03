//! Root [`egui::Ui`] for top-level panels during `begin_pass` / `end_pass` (egui 0.35+).

use bevy_egui::egui::{self, Id, LayerId, Ui, UiBuilder};

/// Build the per-pass root UI that side/top/bottom panels attach to.
#[must_use]
pub fn new_root_ui(ctx: &egui::Context) -> Ui {
    let viewport_rect = ctx.input(|input| input.viewport_rect());
    Ui::new(
        ctx.clone(),
        Id::new((ctx.viewport_id(), "__top_ui")),
        UiBuilder::new()
            .layer_id(LayerId::background())
            .max_rect(viewport_rect),
    )
}
