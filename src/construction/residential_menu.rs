//! Residential submenu — zoning + catalog-backed structures.
//!
//! Returns a row model; the Bevy toolbox / picker paints it (no egui).

use super::build_tool_authority::{BuildingArchetypeId, ZoneTool};
use super::building_catalog::{ApartmentForm, BuildingFamily};
use super::building_definitions::menu_rows::{MenuItem, MenuPick, MenuRow};
use super::building_definitions::{
    intent_from_apartment_form, intent_from_definition, BuildingDefinitionRegistry,
};

/// Rows for the Residential category (zone paint first, then placed structures).
#[must_use]
pub fn draw_residential_submenu(registry: &BuildingDefinitionRegistry) -> Vec<MenuRow> {
    let mut rows = vec![
        MenuRow::Section("Zoning (district paint)".into()),
        MenuRow::Note("Paints a zone overlay - not a building.".into()),
    ];
    for (label, zone) in [
        ("Low density", ZoneTool::ResidentialLow),
        ("Medium density", ZoneTool::ResidentialMedium),
        ("High density", ZoneTool::ResidentialHigh),
        ("Apartments (zone)", ZoneTool::Apartments),
        ("Mixed use", ZoneTool::MixedUse),
    ] {
        rows.push(MenuRow::Item(MenuItem {
            label: label.into(),
            detail: String::new(),
            pick: MenuPick::Zone(zone),
        }));
    }
    rows.push(MenuRow::Section("Structures (placed buildings)".into()));
    for form in [
        ApartmentForm::Duplex,
        ApartmentForm::Quadplex,
        ApartmentForm::ThreeStoryBlock,
        ApartmentForm::FiveStoryBlock,
        ApartmentForm::HighRise,
    ] {
        let preview = intent_from_apartment_form(form, registry);
        rows.push(MenuRow::Item(MenuItem::building(
            BuildingArchetypeId::Housing,
            preview,
            None,
        )));
    }
    push_family(&mut rows, registry, BuildingFamily::Residential, BuildingArchetypeId::Housing);
    rows
}

/// Catalog rows of one family, sorted by display name, skipping `builtin:*` / `mock:*` stand-ins.
pub(super) fn push_family(
    rows: &mut Vec<MenuRow>,
    registry: &BuildingDefinitionRegistry,
    family: BuildingFamily,
    archetype: BuildingArchetypeId,
) {
    let mut defs: Vec<_> = registry
        .ids_by_family(family)
        .into_iter()
        .filter(|id| !id.starts_with("builtin:") && !id.starts_with("mock:"))
        .filter_map(|id| registry.get(id))
        .collect();
    defs.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    for def in defs {
        rows.push(MenuRow::Item(MenuItem::building(
            archetype,
            intent_from_definition(def),
            None,
        )));
    }
}
