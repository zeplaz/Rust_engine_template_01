//! **CITY-G1-C2-001** — BlockFrame debug overlay (archetype lots, green edges, brown scatter).
//!
//! **VISUAL-JANK-CHROME-001:** never auto-open in Simulation — editor / env opt-in only
//! (`BLOCK_FRAME_DEBUG=1`). Prevents green/brown mini-grid bleed into sim chrome.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::states::BaseState;
use crate::strategic::settlement::{
    scatter_interior_tiles, street_edge_tiles, BlockArchetype, BlockBook, BlockFrame,
    BlockFrameBook,
};

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockFrameDebugUiState {
    /// Operator/dev arm — Simulation requires this (or env) before the window paints.
    pub visible: bool,
}

#[must_use]
fn block_frame_debug_env_armed() -> bool {
    std::env::var("BLOCK_FRAME_DEBUG")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "on" | "TRUE" | "ON"))
}

#[must_use]
fn block_frame_debug_may_draw(base: BaseState, ui_state: &BlockFrameDebugUiState) -> bool {
    match base {
        BaseState::Editor => ui_state.visible || block_frame_debug_env_armed(),
        BaseState::Simulation => ui_state.visible && block_frame_debug_env_armed(),
        _ => false,
    }
}

#[must_use]
fn archetype_color(archetype: Option<BlockArchetype>) -> egui::Color32 {
    match archetype {
        Some(BlockArchetype::ForestPark) => egui::Color32::from_rgb(46, 125, 50),
        Some(BlockArchetype::LowDensityRes) => egui::Color32::from_rgb(100, 149, 237),
        Some(BlockArchetype::MediumDensityRes) => egui::Color32::from_rgb(65, 105, 225),
        Some(BlockArchetype::HighDensityCommercial) => egui::Color32::from_rgb(218, 165, 32),
        Some(BlockArchetype::Industrial) => egui::Color32::from_rgb(112, 128, 144),
        Some(BlockArchetype::Civic) => egui::Color32::from_rgb(186, 85, 211),
        None => egui::Color32::from_rgb(70, 70, 80),
    }
}

fn paint_block_mini_grid(
    ui: &mut egui::Ui,
    frame: &BlockFrame,
    block_tiles: &std::collections::HashSet<IVec2>,
    archetype: Option<BlockArchetype>,
) {
    let edge = street_edge_tiles(frame);
    let scatter = scatter_interior_tiles(frame, block_tiles);
    let cell = 10.0;
    let size = egui::vec2(
        frame.extent.x as f32 * cell,
        frame.extent.y as f32 * cell,
    );
    let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    for z in 0..frame.extent.y {
        for x in 0..frame.extent.x {
            let tile = IVec2::new(frame.anchor.x + x as i32, frame.anchor.y + z as i32);
            let crect = egui::Rect::from_min_size(
                rect.min + egui::vec2(x as f32 * cell, z as f32 * cell),
                egui::vec2(cell - 1.0, cell - 1.0),
            );
            let color = if edge.contains(&tile) {
                egui::Color32::from_rgb(60, 180, 75)
            } else if scatter.contains(&tile) {
                egui::Color32::from_rgb(139, 90, 43)
            } else {
                archetype_color(archetype)
            };
            painter.rect_filled(crect, 1.0, color);
        }
    }
}

pub fn draw_block_frame_debug_overlay(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<BlockFrameDebugUiState>,
    base: Res<State<BaseState>>,
    frames: Option<Res<BlockFrameBook>>,
    blocks: Option<Res<BlockBook>>,
) -> Result {
    let Ok(ctx) = contexts.ctx_mut() else {
        return Ok(());
    };
    let Some(frames) = frames else {
        ui_state.visible = false;
        return Ok(());
    };
    if frames.frames.is_empty() {
        ui_state.visible = false;
        return Ok(());
    }
    // Editor: env or prior arm may open. Simulation: never auto — env ∧ visible only.
    if matches!(*base.get(), BaseState::Editor) && block_frame_debug_env_armed() {
        ui_state.visible = true;
    }
    if !block_frame_debug_may_draw(*base.get(), ui_state.as_ref()) {
        return Ok(());
    }

    egui::Window::new("Block frames (CITY-G1-C2)")
        .id(egui::Id::new("settlement_block_frame_debug"))
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.label("Lots by archetype · street edges green · interior scatter brown");
            ui.separator();
            let mut rows: Vec<_> = frames.frames.values().collect();
            rows.sort_by(|a, b| a.block_id.0.cmp(&b.block_id.0));
            for frame in rows {
                let archetype = blocks
                    .as_ref()
                    .and_then(|b| b.blocks.get(&frame.block_id))
                    .and_then(|r| r.archetype);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "{} · {}×{} · street {}",
                            frame.block_id.0,
                            frame.extent.x,
                            frame.extent.y,
                            frame.street_side.as_str(),
                        ));
                        ui.label(format!(
                            "anchor ({}, {}) · orient {} · junction {:?}",
                            frame.anchor.x,
                            frame.anchor.y,
                            frame.orientation_quarter_turns,
                            frame.junction_tile,
                        ));
                    });
                    if let Some(block) = blocks
                        .as_ref()
                        .and_then(|b| b.blocks.get(&frame.block_id))
                    {
                        paint_block_mini_grid(ui, frame, &block.tiles, archetype);
                    }
                });
                ui.separator();
            }
        });
    Ok(())
}

#[must_use]
pub fn settlement_block_frame_debug_overlay_wired_witness_green() -> bool {
    use crate::strategic::settlement::block_frame_debug_overlay_wired_witness_green;

    // Honest: structural frame fixture (not Path::exists). Plugin wires
    // `draw_block_frame_debug_overlay` via [`SettlementBlockFrameDebugPlugin`].
    block_frame_debug_overlay_wired_witness_green()
}

pub struct SettlementBlockFrameDebugPlugin;

impl Plugin for SettlementBlockFrameDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockFrameDebugUiState>()
            .add_systems(EguiPrimaryContextPass, draw_block_frame_debug_overlay);
    }
}
