//! Industrial building submenu — catalog-backed picks grouped by supply chain.

use std::collections::BTreeMap;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{
    intent_from_archetype, BuildingDefinition, BuildingDefinitionRegistry,
};
use super::residential_menu::draw_intent_preview;

fn pick(tool: &mut ActiveBuildTool, preview: super::building_catalog::BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
    tool.building_intent = Some(preview);
}

fn draw_def_button() {}

fn chain_groups(
    registry: &BuildingDefinitionRegistry,
) -> (
    BTreeMap<String, Vec<&BuildingDefinition>>,
    Vec<&BuildingDefinition>,
) {
    let mut by_chain: BTreeMap<String, Vec<&BuildingDefinition>> = BTreeMap::new();
    let mut unchained = Vec::new();
    for id in registry.ids_by_family(BuildingFamily::Industry) {
        if id.starts_with("builtin:") {
            continue;
        }
        let Some(def) = registry.get(id) else {
            continue;
        };
        if let Some(chain) = def.supply_chain.as_ref() {
            by_chain.entry(chain.clone()).or_default().push(def);
        } else {
            unchained.push(def);
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
    (by_chain, unchained)
}

pub fn draw_industrial_submenu() {}
