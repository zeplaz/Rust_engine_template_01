//! Commercial building submenu — catalog-backed picks.


use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{intent_from_archetype, BuildingDefinitionRegistry};
use super::residential_menu::draw_intent_preview;

fn pick(tool: &mut ActiveBuildTool, preview: super::building_catalog::BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Office);
    tool.building_intent = Some(preview);
}

pub fn draw_commercial_submenu() {}
