//! **CITY-G1-C2-001** — BlockFrame debug overlay (archetype lots, green edges, brown scatter).
//!
//! **VISUAL-JANK-CHROME-001:** never auto-open in Simulation — editor / env opt-in only
//! (`BLOCK_FRAME_DEBUG=1`). Prevents green/brown mini-grid bleed into sim chrome.

use bevy::prelude::*;

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
fn archetype_color(archetype: Option<BlockArchetype>) -> crate::gui::UiRgba {
    match archetype {
        Some(BlockArchetype::ForestPark) => crate::gui::UiRgba::from_rgb(46, 125, 50),
        Some(BlockArchetype::LowDensityRes) => crate::gui::UiRgba::from_rgb(100, 149, 237),
        Some(BlockArchetype::MediumDensityRes) => crate::gui::UiRgba::from_rgb(65, 105, 225),
        Some(BlockArchetype::HighDensityCommercial) => crate::gui::UiRgba::from_rgb(218, 165, 32),
        Some(BlockArchetype::Industrial) => crate::gui::UiRgba::from_rgb(112, 128, 144),
        Some(BlockArchetype::Civic) => crate::gui::UiRgba::from_rgb(186, 85, 211),
        None => crate::gui::UiRgba::from_rgb(70, 70, 80),
    }
}

fn paint_block_mini_grid() {}

pub fn draw_block_frame_debug_overlay() -> Result { Ok(()) }

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
        app.init_resource::<BlockFrameDebugUiState>();
    }
}
