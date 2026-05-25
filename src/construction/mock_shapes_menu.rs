//! Mock 0/1 footprint shapes from `assets/configs/buildings/_mock_shapes.ron` (PLAY-BUILD / BUILD-UX-01).

use bevy_egui::egui;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_definitions::BuildingDefinitionRegistry;
use super::residential_menu::draw_intent_preview;

pub fn draw_mock_shapes_submenu(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
) {
    ui.label(egui::RichText::new("Mock footprints (0/1)").strong());
    ui.label(
        egui::RichText::new("Syx-style test shapes — loaded from _mock_shapes.ron")
            .small()
            .weak(),
    );
    let mut mock_ids: Vec<&str> = registry
        .by_id
        .keys()
        .map(|s| s.as_str())
        .filter(|id| id.starts_with("mock:"))
        .collect();
    mock_ids.sort_unstable();
    if mock_ids.is_empty() {
        ui.label(egui::RichText::new("No mock shapes loaded.").weak());
        return;
    }
    for id in mock_ids {
        let Some(def) = registry.get(id) else {
            continue;
        };
        let label = format!(
            "{} ({}×{})",
            def.display_name, def.footprint.width, def.footprint.depth
        );
        if ui.button(label).clicked() {
            tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
            tool.building_intent = registry.intent_preview(id);
        }
    }
    if let Some(intent) = tool.building_intent.as_ref() {
        if intent.catalog_id.as_deref().is_some_and(|id| id.starts_with("mock:")) {
            ui.separator();
            draw_intent_preview(ui, intent);
        }
    }
}
