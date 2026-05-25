//! Residential submenu — zoning + catalog-backed structures.

use bevy_egui::egui;

use super::building_catalog::{ApartmentForm, BuildingIntentPreview};
use super::building_definitions::{intent_from_apartment_form, BuildingDefinitionRegistry};
use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId, ZoneTool};

fn pick_structure(tool: &mut ActiveBuildTool, preview: BuildingIntentPreview) {
    tool.tool = BuildTool::Building(BuildingArchetypeId::Housing);
    tool.building_intent = Some(preview);
}

pub fn draw_residential_submenu(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
) {
    ui.label(egui::RichText::new("Residential").strong());

    ui.collapsing("Zoning (district paint)", |ui| {
        ui.label(egui::RichText::new("Paints zone overlay — not a building.").small().weak());
        if ui.button("Low Density").clicked() {
            tool.clear_building_intent();
            tool.tool = BuildTool::Zone(ZoneTool::ResidentialLow);
        }
        if ui.button("Medium Density").clicked() {
            tool.clear_building_intent();
            tool.tool = BuildTool::Zone(ZoneTool::ResidentialMedium);
        }
        if ui.button("High Density").clicked() {
            tool.clear_building_intent();
            tool.tool = BuildTool::Zone(ZoneTool::ResidentialHigh);
        }
        if ui.button("Apartments (zone)").clicked() {
            tool.clear_building_intent();
            tool.tool = BuildTool::Zone(ZoneTool::Apartments);
        }
        if ui.button("Mixed Use").clicked() {
            tool.clear_building_intent();
            tool.tool = BuildTool::Zone(ZoneTool::MixedUse);
        }
    });

    ui.collapsing("Structures (placed buildings)", |ui| {
        for form in [
            ApartmentForm::Duplex,
            ApartmentForm::Quadplex,
            ApartmentForm::ThreeStoryBlock,
            ApartmentForm::FiveStoryBlock,
            ApartmentForm::HighRise,
        ] {
            let preview = intent_from_apartment_form(form, registry);
            let label = preview.label.clone();
            if ui.button(label).clicked() {
                pick_structure(tool, preview);
            }
        }
        for id in registry.ids_by_family(super::building_catalog::BuildingFamily::Residential) {
            if id.starts_with("builtin:") {
                continue;
            }
            if let Some(def) = registry.get(id) {
                if ui.button(def.display_name.clone()).clicked() {
                    pick_structure(tool, registry.intent_preview(id).unwrap());
                }
            }
        }
    });

    if let Some(intent) = tool.building_intent.as_ref() {
        ui.separator();
        draw_intent_preview(ui, intent);
    }
}

pub fn draw_intent_preview(ui: &mut egui::Ui, intent: &BuildingIntentPreview) {
    ui.label(egui::RichText::new(&intent.label).strong());
    if let Some(id) = &intent.catalog_id {
        ui.label(egui::RichText::new(format!("Catalog: {id}")).small().weak());
    }
    ui.label(format!(
        "Footprint: {}×{} tiles",
        intent.footprint.width, intent.footprint.depth
    ));
    if !intent.unit_kinds.is_empty() {
        let units: Vec<_> = intent
            .unit_kinds
            .iter()
            .map(|u| format!("{u:?}"))
            .collect();
        ui.label(format!("Units: {}", units.join(" + ")));
    }
    ui.label(format!("Construction cost: {}", intent.construction_cost));
    ui.label(format!("Build time: {} ticks", intent.construction_time_ticks));
    ui.label(format!("Power: {:.0}", intent.power_consumption));
    ui.label(format!("Workers: {}", intent.workers_required));
}
