//! Mock 0/1 footprint shapes from `assets/configs/buildings/_mock_shapes.ron`
//! (PLAY-BUILD / BUILD-UX-01). Row model; Bevy paints it.

use super::build_tool_authority::BuildingArchetypeId;
use super::building_definitions::menu_rows::{MenuItem, MenuRow};
use super::building_definitions::{intent_from_definition, BuildingDefinitionRegistry};

/// Rows for the mock-shape category.
#[must_use]
pub fn draw_mock_shapes_submenu(registry: &BuildingDefinitionRegistry) -> Vec<MenuRow> {
    let mut ids: Vec<&str> = registry
        .by_id
        .keys()
        .map(String::as_str)
        .filter(|id| id.starts_with("mock:"))
        .collect();
    ids.sort_unstable();
    let mut rows = vec![MenuRow::Note(
        "Syx-style test footprints loaded from _mock_shapes.ron.".into(),
    )];
    if ids.is_empty() {
        rows.push(MenuRow::Note("No mock shapes loaded.".into()));
        return rows;
    }
    for id in ids {
        let Some(def) = registry.get(id) else { continue };
        rows.push(MenuRow::Item(MenuItem::building(
            BuildingArchetypeId::Factory,
            intent_from_definition(def),
            Some(format!(
                "{} ({}x{})",
                def.display_name, def.footprint.width, def.footprint.depth
            )),
        )));
    }
    rows
}
