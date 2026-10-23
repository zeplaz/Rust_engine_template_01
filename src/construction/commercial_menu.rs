//! Commercial building submenu — catalog-backed picks (row model; Bevy paints it).

use super::build_tool_authority::BuildingArchetypeId;
use super::building_catalog::BuildingFamily;
use super::building_definitions::menu_rows::{MenuItem, MenuRow};
use super::building_definitions::{intent_from_archetype, BuildingDefinitionRegistry};
use super::residential_menu::push_family;

/// Rows for the Commercial category.
#[must_use]
pub fn draw_commercial_submenu(registry: &BuildingDefinitionRegistry) -> Vec<MenuRow> {
    let mut rows = vec![MenuRow::Section("Generic".into())];
    for archetype in [BuildingArchetypeId::Office, BuildingArchetypeId::Retail] {
        rows.push(MenuRow::Item(MenuItem::building(
            archetype,
            intent_from_archetype(archetype, registry),
            Some(format!("Generic {archetype:?}")),
        )));
    }
    let section_at = rows.len();
    rows.push(MenuRow::Section("Catalog".into()));
    let after_header = rows.len();
    push_family(&mut rows, registry, BuildingFamily::Retail, BuildingArchetypeId::Retail);
    if rows.len() == after_header {
        rows.truncate(section_at);
    }
    rows
}
