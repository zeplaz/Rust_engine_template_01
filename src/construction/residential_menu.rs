//! Residential submenu — zoning + catalog-backed structures.


use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId, ZoneTool};
use super::building_catalog::{ApartmentForm, BuildingIntentPreview};
use super::building_definitions::{intent_from_apartment_form, BuildingDefinitionRegistry};

fn pick_structure(tool: &mut ActiveBuildTool, preview: BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Housing);
    tool.building_intent = Some(preview);
}

pub fn draw_residential_submenu() {}

pub fn draw_intent_preview() {}
