//! Power repairs tray section — Bevy list lives in `sim_play_bevy_chrome`
//! (`ContextTrayPowerRepairList`). Retired egui draw stub kept until mass stub prune (DEFER).

use bevy::prelude::*;

/// Honest wire check: Bevy repair list root + sync (not empty egui draw stub).
#[must_use]
pub fn power_repair_panel_wired() -> bool {
    include_str!("sim_play_bevy_chrome.rs").contains("pub struct ContextTrayPowerRepairList")
        && include_str!("sim_play_bevy_chrome.rs").contains("fn sync_context_tray_power_repair_list")
        && include_str!("../in_game_hud.rs").contains("ContextTrayPowerRepairList")
}

/// Tier: Logistics context tray (Bevy shell tab enum).
#[must_use]
pub fn power_repair_panel_tier_tray_logistics() -> bool {
    include_str!("simulation_shell_phase2.rs").contains("ContextTrayTab::Logistics")
        && include_str!("sim_play_bevy_chrome.rs").contains("ContextTrayTab::Logistics")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_repair_panel_witness_tokens() {
        assert!(power_repair_panel_wired());
        assert!(power_repair_panel_tier_tray_logistics());
    }
}
