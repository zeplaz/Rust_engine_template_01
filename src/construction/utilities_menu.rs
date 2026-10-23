//! Utilities building submenu — power lines + catalog nodes (row model; Bevy paints it).

use crate::gui::hud::power_hud_icon_atlas::{PowerHudIconAtlasManifest, PowerHudIconId};
use crate::infrastructure::VoltageClass;

use super::build_tool_authority::BuildingArchetypeId;
use super::building_catalog::BuildingFamily;
use super::building_definitions::menu_rows::{MenuItem, MenuPick, MenuRow};
use super::building_definitions::{
    intent_from_archetype, intent_from_definition, BuildingDefinitionRegistry,
};

/// Optional power HUD atlas for utilities picker rows (COD-ART-HUD-ICON-ATLAS-001).
pub struct UtilitiesSubmenuIconUi<'a> {
    pub texture_id: (),
    pub manifest: &'a PowerHudIconAtlasManifest,
    pub idle_tint: crate::gui::UiRgba,
    pub selected_tint: crate::gui::UiRgba,
}

/// Rows for the Utilities category: power lines, then plants / catalog nodes.
#[must_use]
pub fn draw_utilities_submenu(registry: &BuildingDefinitionRegistry) -> Vec<MenuRow> {
    let mut rows = vec![MenuRow::Section("Lines".into())];
    for (label, v) in [
        ("Power line - distribution", VoltageClass::Low),
        ("Power line - medium voltage", VoltageClass::Medium),
        ("Power line - high voltage", VoltageClass::High),
    ] {
        rows.push(MenuRow::Item(MenuItem {
            label: label.into(),
            detail: String::new(),
            pick: MenuPick::PowerLine(v),
        }));
    }
    rows.push(MenuRow::Section("Nodes".into()));
    for archetype in [BuildingArchetypeId::PowerPlant, BuildingArchetypeId::WaterPlant] {
        rows.push(MenuRow::Item(MenuItem::building(
            archetype,
            intent_from_archetype(archetype, registry),
            Some(format!("Generic {archetype:?}")),
        )));
    }
    let mut defs: Vec<_> = registry
        .ids_by_family(BuildingFamily::Power)
        .into_iter()
        .filter(|id| !id.starts_with("builtin:") && !id.starts_with("mock:"))
        .filter_map(|id| registry.get(id))
        .collect();
    defs.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    for def in defs {
        let archetype = match def.site_archetype {
            crate::strategic::SiteArchetype::PowerPlant => BuildingArchetypeId::PowerPlant,
            _ => BuildingArchetypeId::WaterPlant,
        };
        rows.push(MenuRow::Item(MenuItem::building(
            archetype,
            intent_from_definition(def),
            None,
        )));
    }
    rows
}

#[must_use]
pub fn utilities_submenu_power_icons_wired() -> bool {
    PowerHudIconId::inventory().len() >= 13
}
