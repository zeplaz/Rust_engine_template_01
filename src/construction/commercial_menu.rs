//! Commercial building submenu — catalog-backed picks.

use bevy_egui::egui;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{intent_from_archetype, BuildingDefinitionRegistry};
use super::residential_menu::draw_intent_preview;

fn pick(tool: &mut ActiveBuildTool, preview: super::building_catalog::BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Office);
    tool.building_intent = Some(preview);
}

pub fn draw_commercial_submenu(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
) {
    ui.label(egui::RichText::new("Commercial").strong());
    for archetype in [BuildingArchetypeId::Office, BuildingArchetypeId::Retail] {
        let preview = intent_from_archetype(archetype, registry);
        if ui.button(preview.label.clone()).clicked() {
            pick(tool, preview);
        }
    }
    for id in registry.ids_by_family(BuildingFamily::Retail) {
        if id.starts_with("builtin:") {
            continue;
        }
        if let Some(def) = registry.get(id) {
            if ui.button(def.display_name.clone()).clicked() {
                pick(tool, registry.intent_preview(id).unwrap());
            }
        }
    }
    if let Some(intent) = tool.building_intent.as_ref() {
        ui.separator();
        draw_intent_preview(ui, intent);
    }
}
