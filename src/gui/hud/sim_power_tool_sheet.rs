//! **COD-ART-HUD-ICON-ATLAS-001** — rail-anchored power line tool sheet with HUD icons.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::construction::{
    commit_power_line_to_utility_graph, ActivePowerLinePlacement, ActiveBuildTool, BuildStripState,
    BuildTool, PowerLineRoutingMode, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{
    draw_power_hud_icon_labeled, PowerHudEguiTextureCache, PowerHudIconAtlasManifest,
    PowerHudIconAtlasUi, PowerHudIconId,
};
use crate::gui::UiPalette;
use crate::infrastructure::utility::graph::{UtilityGraph, UtilityNetworkSnapshotResource};
use crate::infrastructure::VoltageClass;

use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const POWER_TOOL_SHEET_W_PX: f32 = 300.0;
/// Logical height of the Bevy power sheet, used by the pointer gate (not a second window).
pub const POWER_TOOL_SHEET_BLOCK_H_PX: f32 = 260.0;

#[derive(Resource, Debug, Clone, Default)]
pub struct SimPowerToolSheetState {
    pub open: bool,
}

impl SimPowerToolSheetState {
    pub fn sync_from_tool(&mut self, tool: &ActiveBuildTool) {
        self.open = matches!(tool.tool, BuildTool::PowerLine(_));
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[derive(SystemParam)]
pub struct SimPowerToolSheetDrawParams<'w> {
    pub strip: Res<'w, BuildStripState>,
    pub palette: Res<'w, UiPalette>,
    pub tool: ResMut<'w, ActiveBuildTool>,
    pub power_sheet: ResMut<'w, SimPowerToolSheetState>,
    pub placement: ResMut<'w, ActivePowerLinePlacement>,
    pub snap_res: ResMut<'w, UtilityNetworkSnapshotResource>,
    pub graph: ResMut<'w, UtilityGraph>,
    pub atlas_ui: Option<Res<'w, PowerHudIconAtlasUi>>,
    pub manifests: Res<'w, Assets<PowerHudIconAtlasManifest>>,
    pub tex_cache: ResMut<'w, PowerHudEguiTextureCache>,
}

#[must_use]
pub fn power_tool_sheet_anchor(strip: &BuildStripState, left_stack_collapsed: bool) -> bevy::prelude::Vec2 {
    let _ = strip;
    super::simulation_shell_phase2::build_rail_slot_anchor_xy(
        ToolContext::Utilities,
        left_stack_collapsed,
    )
}

fn voltage_label(v: VoltageClass) -> &'static str {
    match v {
        VoltageClass::Low => "Distribution",
        VoltageClass::Medium => "Medium",
        VoltageClass::High => "Transmission",
    }
}


#[must_use]
pub fn sim_power_tool_sheet_icons_wired() -> bool {
    crate::gui::hud::power_hud_icon_atlas::power_hud_atlas_assets_on_disk()
        && PowerHudIconId::for_voltage(VoltageClass::Medium) == PowerHudIconId::VoltageMedium
        && PowerHudIconId::for_routing_mode(PowerLineRoutingMode::Curved) == PowerHudIconId::RouteCurved
}
