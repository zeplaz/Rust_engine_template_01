//! Power node hover — Bevy satellite lives in `sim_play_bevy_chrome` (`SimPlayPowerHover`).
//!
//! Retired egui draw stub kept for call-site stability until mass stub prune (DEFER).

use bevy::prelude::*;

/// HUD-NAT-009 — keep power hover readable inside the map hole.
pub const HOVER_CARD_MAX_H: f32 = 160.0;

/// Honest wire check: Bevy root + sync exist (empty `draw_*_egui` stub is not a wire).
#[must_use]
pub fn power_node_hover_card_wired() -> bool {
    include_str!("sim_play_bevy_chrome.rs").contains("pub struct SimPlayPowerHover")
        && include_str!("sim_play_bevy_chrome.rs").contains("fn sync_sim_play_power_hover")
        && include_str!("../in_game_hud.rs").contains("SimPlayPowerHover")
}

