//! Industrial building submenu — catalog-backed picks grouped by supply chain
//! (row model; Bevy paints it).

use std::collections::BTreeMap;

use super::build_tool_authority::BuildingArchetypeId;
use super::building_catalog::BuildingFamily;
use super::building_definitions::menu_rows::{MenuItem, MenuRow};
use super::building_definitions::{
    intent_from_archetype, intent_from_definition, BuildingDefinition, BuildingDefinitionRegistry,
};

/// Rows for the Industrial category.
#[must_use]
pub fn draw_industrial_submenu(registry: &BuildingDefinitionRegistry) -> Vec<MenuRow> {
    let mut rows = vec![
        MenuRow::Note("Place each supply-chain step separately; power loads sum on the grid.".into()),
        MenuRow::Section("Generic".into()),
    ];
    for archetype in [BuildingArchetypeId::Factory, BuildingArchetypeId::Depot] {
        rows.push(MenuRow::Item(MenuItem::building(
            archetype,
            intent_from_archetype(archetype, registry),
            Some(format!("Generic {archetype:?}")),
        )));
    }

    let mut by_chain: BTreeMap<String, Vec<&BuildingDefinition>> = BTreeMap::new();
    let mut unchained: Vec<&BuildingDefinition> = Vec::new();
    for id in registry.ids_by_family(BuildingFamily::Industry) {
        if id.starts_with("builtin:") || id.starts_with("mock:") {
            continue;
        }
        let Some(def) = registry.get(id) else { continue };
        match def.supply_chain.as_ref() {
            Some(chain) => by_chain.entry(chain.clone()).or_default().push(def),
            None => unchained.push(def),
        }
    }
    for defs in by_chain.values_mut() {
        defs.sort_by(|a, b| {
            a.supply_chain_role
                .map(|r| format!("{r:?}"))
                .cmp(&b.supply_chain_role.map(|r| format!("{r:?}")))
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
    }
    unchained.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    for (chain, defs) in by_chain {
        rows.push(MenuRow::Section(crate::gui::hud::sim_hud_copy::human_chain_label(&chain)));
        for def in defs {
            rows.push(MenuRow::Item(MenuItem::building(
                BuildingArchetypeId::Factory,
                intent_from_definition(def),
                None,
            )));
        }
    }
    if !unchained.is_empty() {
        rows.push(MenuRow::Section("Other industry".into()));
        for def in unchained {
            rows.push(MenuRow::Item(MenuItem::building(
                BuildingArchetypeId::Factory,
                intent_from_definition(def),
                None,
            )));
        }
    }
    rows
}
