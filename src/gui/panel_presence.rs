//! Runtime panel presence — honest witness markers for Bevy UI roots.
//!
//! Sync systems set flags when a root entity exists. Lib proofs may also check
//! Bevy component markers in source; empty `draw_*_egui` stubs never count as wired.

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct PanelPresence {
    pub diagnostics_root: bool,
    pub keybindings_root: bool,
    pub logistics_orders_root: bool,
    pub logistics_targets_root: bool,
    pub power_hover_root: bool,
    pub power_repair_list_root: bool,
    pub editor_dock_root: bool,
    pub build_toolbox_root: bool,
    pub map_editor_shell_root: bool,
    pub road_popup_root: bool,
    pub staged_placements_root: bool,
    pub faction_tools_root: bool,
    pub ai_explainability_root: bool,
    pub pressure_composer_root: bool,
    pub assembly_snapshot_qc_root: bool,
}

pub struct PanelPresencePlugin;

impl Plugin for PanelPresencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PanelPresence>();
    }
}
