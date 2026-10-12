//! **COD-SIM-HUD-BUILD-PICKER-001** — rail-anchored build picker sheet (sim only).

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::construction::{
    draw_commercial_submenu, draw_mock_shapes_submenu, draw_residential_submenu,
    draw_utilities_submenu, ActiveBuildTool, BuildStripState, BuildTool, BuildingArchetypeId,
    BuildingDefinition, BuildingDefinitionRegistry, BuildingFamily, DefenseKind, RailType, RoadType,
    ToolContext, UtilitiesSubmenuIconUi,
};
use crate::construction::building_definitions::intent_from_archetype;
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{
    PowerHudEguiTextureCache, PowerHudIconAtlasManifest, PowerHudIconAtlasUi,
};
use crate::gui::UiPalette;

use super::sim_hud_copy::{
    human_chain_label, power_tier_compact, PICKER_DEFENSE_FOOTER_HINT, PICKER_DEFENSE_LEAD,
    PICKER_EMPTY_CATEGORY, PICKER_GENERIC_DEPOT, PICKER_GENERIC_FACTORY, PICKER_INDUSTRY_LEAD,
    PICKER_INDUSTRY_OTHER, PICKER_ROW_BUNKER, PICKER_ROW_CAPTION_DRAGON_TEETH,
    PICKER_ROW_CAPTION_MINEFIELD, PICKER_ROW_DEFENSIVE_WALL, PICKER_ROW_DEMOLISH,
    PICKER_ROW_DRAGON_TEETH, PICKER_ROW_MINEFIELD, PICKER_ROW_TRENCH_LINE,
    PICKER_SECTION_DEPLOYABLES, PICKER_SECTION_FORTIFICATION, PICKER_SECTION_TRENCHES,
    PICKER_SECTION_WALLS, PICKER_TITLE_DEFENSE, PICKER_TITLE_INDUSTRY, PICKER_TITLE_ROADS,
    PICKER_TITLE_SHAPES, PICKER_TITLE_UTILITIES, PICKER_TITLE_ZONE,
};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const BUILD_PICKER_SHEET_W_PX: f32 = 320.0;
/// Gap between build rail and picker sheet — authority: [`super::simulation_shell_phase2::BUILD_PICKER_SHEET_GAP_PX`].
pub const BUILD_PICKER_RAIL_GAP_PX: f32 =
    super::simulation_shell_phase2::BUILD_PICKER_SHEET_GAP_PX;
pub const BUILD_PICKER_MAX_H_PX: f32 = 480.0;
pub const AD_HOC_SUBMENU_WINDOWS: u32 = 0;

#[must_use]
pub fn sim_build_picker_constants_green() -> bool {
    BUILD_PICKER_RAIL_GAP_PX == 8.0
        && BUILD_PICKER_SHEET_W_PX == 320.0
        && AD_HOC_SUBMENU_WINDOWS == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BuildPickerCategory {
    #[default]
    Zone,
    Roads,
    Industry,
    Utilities,
    Shapes,
    Defense,
}

impl BuildPickerCategory {
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Zone => PICKER_TITLE_ZONE,
            Self::Roads => PICKER_TITLE_ROADS,
            Self::Industry => PICKER_TITLE_INDUSTRY,
            Self::Utilities => PICKER_TITLE_UTILITIES,
            Self::Shapes => PICKER_TITLE_SHAPES,
            Self::Defense => PICKER_TITLE_DEFENSE,
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Zone,
            Self::Roads,
            Self::Industry,
            Self::Utilities,
            Self::Shapes,
            Self::Defense,
        ]
    }
}

#[must_use]
pub const fn tool_context_to_picker_category(ctx: ToolContext) -> BuildPickerCategory {
    match ctx {
        ToolContext::Civil => BuildPickerCategory::Zone,
        ToolContext::Roads | ToolContext::Rail => BuildPickerCategory::Roads,
        ToolContext::Industry => BuildPickerCategory::Industry,
        ToolContext::Utilities => BuildPickerCategory::Utilities,
        ToolContext::Ecology => BuildPickerCategory::Shapes,
        ToolContext::Military => BuildPickerCategory::Defense,
        ToolContext::None => BuildPickerCategory::Zone,
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SimBuildPickerState {
    pub open: bool,
    pub category: BuildPickerCategory,
    pub anchor_slot: ToolContext,
}

impl SimBuildPickerState {
    /// Sole category assignment used by the build rail. Does not toggle closed —
    /// a second call on the same slot stays open. The rail click system closes.
    pub fn open_for_slot(&mut self, slot: ToolContext) {
        self.open = true;
        self.anchor_slot = slot;
        self.category = tool_context_to_picker_category(slot);
    }

    /// Header word matches the rail button that opened this sheet.
    #[must_use]
    pub fn sheet_title(&self) -> &'static str {
        super::sim_hud_copy::rail_player_label(self.anchor_slot)
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[must_use]
pub fn build_rail_slot_anchor_y(slot: ToolContext) -> f32 {
    super::simulation_shell_phase2::build_rail_slot_anchor_y(slot)
}

#[must_use]
pub fn sim_build_picker_sheet_rect(
    state: &SimBuildPickerState,
    left_stack_collapsed: bool,
) -> bevy::prelude::Rect {
    let anchor = super::simulation_shell_phase2::build_rail_slot_anchor_xy(
        state.anchor_slot,
        left_stack_collapsed,
    );
    bevy::prelude::Rect::from_corners(anchor, anchor + bevy::prelude::Vec2::new(BUILD_PICKER_SHEET_W_PX, BUILD_PICKER_MAX_H_PX),)
}

#[must_use]
pub fn sim_build_picker_witness_green(state: &SimBuildPickerState) -> bool {
    !state.open || state.anchor_slot != ToolContext::None
}


fn draw_defense_picker_tab() {}

fn defense_row_clicked() {}

fn defense_row_with_caption_clicked() {}

fn draw_industry_picker_tab() {}

fn industry_chain_groups(
    registry: &BuildingDefinitionRegistry,
) -> (BTreeMap<String, Vec<&BuildingDefinition>>, Vec<&BuildingDefinition>) {
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
        defs.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    }
    unchained.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    (by_chain, unchained)
}

fn industry_card_clicked() {}

/// Legacy entry — delegates to picker (retires floating submenus).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_category_maps_civil_to_zone() {
        assert_eq!(
            tool_context_to_picker_category(ToolContext::Civil),
            BuildPickerCategory::Zone
        );
        assert_eq!(
            tool_context_to_picker_category(ToolContext::Industry),
            BuildPickerCategory::Industry
        );
        assert_eq!(
            tool_context_to_picker_category(ToolContext::Military),
            BuildPickerCategory::Defense
        );
    }

    #[test]
    fn military_opens_defense_picker() {
        let mut picker = SimBuildPickerState::default();
        picker.open_for_slot(ToolContext::Military);
        assert!(picker.open);
        assert_eq!(picker.anchor_slot, ToolContext::Military);
        assert_eq!(picker.category, BuildPickerCategory::Defense);
        assert_eq!(picker.sheet_title(), "Defense");
        assert_eq!(BuildPickerCategory::Defense.title(), "Defense");
        picker.open_for_slot(ToolContext::Military);
        assert!(picker.open, "open_for_slot sets the category; it does not toggle closed");
        picker.open_for_slot(ToolContext::Utilities);
        assert!(picker.open);
        assert_eq!(picker.category, BuildPickerCategory::Utilities);
        assert_eq!(picker.sheet_title(), "Power");
    }

    #[test]
    fn human_chain_label_portland() {
        assert_eq!(
            super::super::sim_hud_copy::human_chain_label("concrete_portland"),
            "Concrete (Portland)"
        );
    }
}
