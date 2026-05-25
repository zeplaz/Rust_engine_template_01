//! Industrial building submenu — catalog-backed picks grouped by supply chain.

use bevy_egui::egui;
use std::collections::BTreeMap;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{intent_from_archetype, BuildingDefinition, BuildingDefinitionRegistry};
use super::residential_menu::draw_intent_preview;

fn pick(tool: &mut ActiveBuildTool, preview: super::building_catalog::BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
    tool.building_intent = Some(preview);
}

fn draw_def_button(ui: &mut egui::Ui, tool: &mut ActiveBuildTool, registry: &BuildingDefinitionRegistry, id: &str) {
    if let Some(def) = registry.get(id) {
        let label = format!(
            "{} ({:.0} power)",
            def.display_name, def.power_consumption
        );
        if ui.button(label).clicked() {
            pick(tool, registry.intent_preview(id).unwrap());
        }
    }
}

fn chain_groups(registry: &BuildingDefinitionRegistry) -> (BTreeMap<String, Vec<&BuildingDefinition>>, Vec<&BuildingDefinition>) {
    let mut by_chain: BTreeMap<String, Vec<&BuildingDefinition>> = BTreeMap::new();
    let mut unchained = Vec::new();
    for id in registry.ids_by_family(BuildingFamily::Industry) {
        if id.starts_with("builtin:") {
            continue;
        }
        let Some(def) = registry.get(id) else { continue };
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

pub fn draw_industrial_submenu(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
) {
    ui.label(egui::RichText::new("Industrial").strong());
    ui.label("Place each supply-chain step separately; power loads sum on the grid.");
    for archetype in [BuildingArchetypeId::Factory, BuildingArchetypeId::Depot] {
        let preview = intent_from_archetype(archetype, registry);
        if ui.button(format!("Generic {}", preview.label)).clicked() {
            pick(tool, preview);
        }
    }

    let (chains, unchained) = chain_groups(registry);
    for (chain_id, defs) in chains {
        ui.separator();
        ui.label(egui::RichText::new(chain_id.replace('_', " ")).strong());
        for def in defs {
            draw_def_button(ui, tool, registry, def.id.as_str());
        }
    }
    if !unchained.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Other industry").strong());
        for def in unchained {
            draw_def_button(ui, tool, registry, def.id.as_str());
        }
    }

    if let Some(intent) = tool.building_intent.as_ref() {
        ui.separator();
        draw_intent_preview(ui, intent);
    }
}
