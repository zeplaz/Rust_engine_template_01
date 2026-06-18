//! Utilities building submenu — catalog-backed picks.

use bevy_egui::egui;

use crate::gui::hud::power_hud_icon_atlas::{
    draw_power_hud_icon_labeled, PowerHudIconAtlasManifest, PowerHudIconId,
};
use crate::infrastructure::VoltageClass;

use super::build_tool_authority::{ActiveBuildTool, BuildTool, BuildingArchetypeId};
use super::building_catalog::BuildingFamily;
use super::building_definitions::{intent_from_archetype, BuildingDefinitionRegistry};
use super::residential_menu::draw_intent_preview;

/// Optional power HUD atlas for utilities picker rows (COD-ART-HUD-ICON-ATLAS-001).
pub struct UtilitiesSubmenuIconUi<'a> {
    pub texture_id: egui::TextureId,
    pub manifest: &'a PowerHudIconAtlasManifest,
    pub idle_tint: egui::Color32,
    pub selected_tint: egui::Color32,
}

fn pick(
    tool: &mut ActiveBuildTool,
    archetype: BuildingArchetypeId,
    preview: super::building_catalog::BuildingIntentPreview,
) {
    tool.tool = BuildTool::Building(archetype);
    tool.building_intent = Some(preview);
}

fn pick_power_line(tool: &mut ActiveBuildTool, voltage: VoltageClass) {
    tool.tool = BuildTool::PowerLine(voltage);
    tool.building_intent = None;
}

fn power_line_row(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    icons: Option<&UtilitiesSubmenuIconUi<'_>>,
    voltage: VoltageClass,
    icon: PowerHudIconId,
    label: &str,
) -> bool {
    let selected = matches!(tool.tool, BuildTool::PowerLine(v) if v == voltage);
    let clicked = if let Some(icons) = icons {
        let tint = if selected {
            icons.selected_tint
        } else {
            icons.idle_tint
        };
        draw_power_hud_icon_labeled(
            ui,
            icons.texture_id,
            icons.manifest,
            icon,
            16.0,
            tint,
            label,
            selected,
        )
        .clicked()
    } else if ui.button(label).clicked() {
        true
    } else {
        false
    };
    if clicked {
        pick_power_line(tool, voltage);
    }
    clicked
}

pub fn draw_utilities_submenu(
    ui: &mut egui::Ui,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    icons: Option<&UtilitiesSubmenuIconUi<'_>>,
) {
    ui.label(egui::RichText::new("Utilities").strong());
    ui.label(egui::RichText::new("Lines").small().weak());
    power_line_row(
        ui,
        tool,
        icons,
        VoltageClass::Medium,
        PowerHudIconId::PowerLineTool,
        "Draw power line (MV)",
    );
    power_line_row(
        ui,
        tool,
        icons,
        VoltageClass::High,
        PowerHudIconId::VoltageHigh,
        "Draw power line (HV)",
    );
    ui.separator();
    ui.label(egui::RichText::new("Nodes").small().weak());
    for archetype in [BuildingArchetypeId::PowerPlant, BuildingArchetypeId::WaterPlant] {
        let preview = intent_from_archetype(archetype, registry);
        let place_icon = match archetype {
            BuildingArchetypeId::PowerPlant => PowerHudIconId::SubstationPlace,
            _ => PowerHudIconId::TransformerPlace,
        };
        let selected = matches!(tool.tool, BuildTool::Building(a) if a == archetype);
        let clicked = if let Some(icons) = icons {
            let tint = if selected {
                icons.selected_tint
            } else {
                icons.idle_tint
            };
            draw_power_hud_icon_labeled(
                ui,
                icons.texture_id,
                icons.manifest,
                place_icon,
                16.0,
                tint,
                &preview.label,
                selected,
            )
            .clicked()
        } else if ui.button(preview.label.clone()).clicked() {
            true
        } else {
            false
        };
        if clicked {
            pick(tool, archetype, preview);
        }
    }
    for id in registry.ids_by_family(BuildingFamily::Power) {
        if id.starts_with("builtin:") {
            continue;
        }
        if let Some(def) = registry.get(id) {
            if ui.button(def.display_name.clone()).clicked() {
                let archetype = match def.site_archetype {
                    crate::strategic::SiteArchetype::PowerPlant => BuildingArchetypeId::PowerPlant,
                    _ => BuildingArchetypeId::WaterPlant,
                };
                pick(tool, archetype, registry.intent_preview(id).unwrap());
            }
        }
    }
    if let Some(intent) = tool.building_intent.as_ref() {
        ui.separator();
        draw_intent_preview(ui, intent);
    }
}

#[must_use]
pub fn utilities_submenu_power_icons_wired() -> bool {
    PowerHudIconId::inventory().len() >= 13
}
