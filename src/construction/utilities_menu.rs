//! Utilities building submenu — catalog-backed picks.


use crate::gui::hud::power_hud_icon_atlas::{
    draw_power_hud_icon_labeled, PowerHudIconAtlasManifest, PowerHudIconId,
};
use crate::infrastructure::VoltageClass;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{intent_from_archetype, BuildingDefinitionRegistry};
use super::residential_menu::draw_intent_preview;

/// Optional power HUD atlas for utilities picker rows (COD-ART-HUD-ICON-ATLAS-001).
pub struct UtilitiesSubmenuIconUi<'a> {
    pub texture_id: (),
    pub manifest: &'a PowerHudIconAtlasManifest,
    pub idle_tint: crate::gui::UiRgba,
    pub selected_tint: crate::gui::UiRgba,
}

fn pick(
    tool: &mut ActiveBuildTool,
    archetype: BuildingArchetypeId,
    preview: super::building_catalog::BuildingIntentPreview,
) {
    tool.tool = BuildTool::Building(archetype);
    tool.building_intent = Some(preview);
}

fn pick_power_line(tool: &mut ActiveBuildTool, voltage: VoltageClass) {
    tool.tool = BuildTool::PowerLine(voltage);
    tool.building_intent = None;
}

fn power_line_row() {}

pub fn draw_utilities_submenu() {}

#[must_use]
pub fn utilities_submenu_power_icons_wired() -> bool {
    PowerHudIconId::inventory().len() >= 13
}
