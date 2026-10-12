//! Product-shell HUD tick — Bevy dock owns chrome; retired egui root is a no-op.

use bevy::prelude::*;

use crate::gui::construction_growth_inspector::{
    sync_ecology_growth_hint, EcologyGrowthHint, GrowthInspectorUiState,
};

pub struct HudRootTickPlugin;

impl Plugin for HudRootTickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrowthInspectorUiState>()
            .init_resource::<EcologyGrowthHint>()
            .add_systems(Update, sync_ecology_growth_hint);
    }
}

/// Retired egui product-shell pass — Bevy dock / toolbox own editor chrome.
pub fn hud_product_shell_egui_root() -> Result {
    Ok(())
}
