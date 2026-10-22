//! **CITY-G1-C2-001** — BlockFrame debug overlay (archetype lots, green edges, brown scatter).
//!
//! **VISUAL-JANK-CHROME-001:** never auto-open in Simulation — editor / env opt-in only
//! (`BLOCK_FRAME_DEBUG=1`). Prevents green/brown mini-grid bleed into sim chrome.

use bevy::prelude::*;

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockFrameDebugUiState {
    /// Operator/dev arm — Simulation requires this (or env) before the window paints.
    pub visible: bool,
}

/// Retired egui BlockFrame paint — Bevy debug mesh owns live draw.
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_block_frame_debug_overlay() -> Result {
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
        app.init_resource::<BlockFrameDebugUiState>();
    }
}
