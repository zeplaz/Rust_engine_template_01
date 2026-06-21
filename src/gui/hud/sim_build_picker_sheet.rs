//! **COD-SIM-HUD-BUILD-PICKER-001** — rail-anchored build picker sheet (sim only).

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::construction::{
    draw_commercial_submenu, draw_mock_shapes_submenu, draw_residential_submenu,
    draw_utilities_submenu, ActiveBuildTool, BuildStripState, BuildTool, BuildingArchetypeId,
    BuildingDefinition, BuildingDefinitionRegistry, BuildingFamily, RailType, RoadType,
    ToolContext, UtilitiesSubmenuIconUi,
};
use crate::construction::building_definitions::intent_from_archetype;
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{
    PowerHudEguiTextureCache, PowerHudIconAtlasManifest, PowerHudIconAtlasUi,
};
use crate::gui::hud::simulation_shell_phase2::{
    BUILD_RAIL_W_PX, COMMAND_LEFT_STACK_COLUMN_GAP_PX, CONTEXT_RAIL_W_PX,
};
use crate::gui::UiPalette;

use super::sim_hud_copy::{
    human_chain_label, power_tier_compact, PICKER_EMPTY_CATEGORY, PICKER_GENERIC_DEPOT,
    PICKER_GENERIC_FACTORY, PICKER_INDUSTRY_LEAD, PICKER_INDUSTRY_OTHER, PICKER_TITLE_INDUSTRY,
    PICKER_TITLE_ROADS, PICKER_TITLE_SHAPES, PICKER_TITLE_UTILITIES, PICKER_TITLE_ZONE,
};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const BUILD_PICKER_SHEET_W_PX: f32 = 320.0;
pub const BUILD_PICKER_RAIL_GAP_PX: f32 = 8.0;
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
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Zone,
            Self::Roads,
            Self::Industry,
            Self::Utilities,
            Self::Shapes,
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
        ToolContext::Military | ToolContext::None => BuildPickerCategory::Zone,
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SimBuildPickerState {
    pub open: bool,
    pub category: BuildPickerCategory,
    pub anchor_slot: ToolContext,
}

impl SimBuildPickerState {
    pub fn open_for_slot(&mut self, slot: ToolContext) {
        if slot == ToolContext::Military {
            self.open = false;
            return;
        }
        if self.open && self.anchor_slot == slot {
            self.open = false;
            return;
        }
        self.open = true;
        self.anchor_slot = slot;
        self.category = tool_context_to_picker_category(slot);
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[must_use]
pub fn build_rail_slot_anchor_y(slot: ToolContext) -> f32 {
    const SLOT_STEP: f32 = 36.0;
    const BASE_Y: f32 = 96.0;
    let idx = match slot {
        ToolContext::Roads => 0,
        ToolContext::Rail => 1,
        ToolContext::Utilities => 2,
        ToolContext::Military => 3,
        ToolContext::Industry => 4,
        ToolContext::Ecology => 5,
        ToolContext::Civil => 6,
        ToolContext::None => 0,
    };
    BASE_Y + idx as f32 * SLOT_STEP
}

#[must_use]
pub fn sim_build_picker_sheet_rect(state: &SimBuildPickerState) -> egui::Rect {
    let anchor_x =
        CONTEXT_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + BUILD_RAIL_W_PX + BUILD_PICKER_RAIL_GAP_PX;
    let anchor_y = build_rail_slot_anchor_y(state.anchor_slot);
    egui::Rect::from_min_size(
        egui::pos2(anchor_x, anchor_y),
        egui::vec2(BUILD_PICKER_SHEET_W_PX, BUILD_PICKER_MAX_H_PX),
    )
}

#[must_use]
pub fn sim_build_picker_witness_green(state: &SimBuildPickerState) -> bool {
    !state.open || state.anchor_slot != ToolContext::None
}

pub fn draw_sim_build_picker_sheet_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    strip: Res<BuildStripState>,
    palette: Res<UiPalette>,
    mut tool: ResMut<ActiveBuildTool>,
    mut picker: ResMut<SimBuildPickerState>,
    registry: Res<BuildingDefinitionRegistry>,
    atlas_ui: Option<Res<PowerHudIconAtlasUi>>,
    manifests: Res<Assets<PowerHudIconAtlasManifest>>,
    mut tex_cache: ResMut<PowerHudEguiTextureCache>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if strip.active == ToolContext::None || !picker.open {
        return Ok(());
    }

    let texture_id = atlas_ui
        .as_ref()
        .and_then(|atlas| tex_cache.resolve(&mut contexts, &atlas.atlas));
    let manifest = atlas_ui
        .as_ref()
        .and_then(|atlas| manifests.get(&atlas.manifest));
    let icon_ui = texture_id.zip(manifest).map(|(texture_id, manifest)| {
        UtilitiesSubmenuIconUi {
            texture_id,
            manifest,
            idle_tint: palette.accent_terminal,
            selected_tint: palette.accent_action,
        }
    });

    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &palette);

    let anchor = sim_build_picker_sheet_rect(picker.as_ref()).min;
    let mut close_requested = false;

    egui::Area::new(egui::Id::new("sim_build_picker_sheet"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_width(BUILD_PICKER_SHEET_W_PX);
            picker_sheet_frame(&palette).show(ui, |ui| {
                picker_header_frame(&palette).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(title_text(&palette, picker.category.title()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(body_text(&palette, "✕"))
                                        .min_size(egui::vec2(36.0, 36.0)),
                                )
                                .clicked()
                            {
                                close_requested = true;
                            }
                        });
                    });
                });
                ui.horizontal(|ui| {
                    for tab in BuildPickerCategory::all() {
                        let selected = picker.category == tab;
                        let resp = ui.selectable_label(selected, body_text(&palette, tab.title()));
                        if resp.clicked() {
                            picker.category = tab;
                        }
                        if selected {
                            ui.painter().hline(
                                resp.rect.x_range(),
                                resp.rect.bottom(),
                                egui::Stroke::new(2.0, palette.accent_terminal),
                            );
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(BUILD_PICKER_MAX_H_PX - 72.0)
                    .show(ui, |ui| {
                        match picker.category {
                            BuildPickerCategory::Zone => {
                                draw_residential_submenu(ui, &mut tool, &registry);
                            }
                            BuildPickerCategory::Roads => {
                                ui.label(caption_text(
                                    &palette,
                                    "Select Street or Highway from rail — path tools in road sheet.",
                                ));
                                if ui.button(body_text(&palette, "Street")).clicked() {
                                    tool.tool = BuildTool::Road(RoadType::Street);
                                    close_requested = true;
                                }
                                if ui.button(body_text(&palette, "Highway")).clicked() {
                                    tool.tool = BuildTool::Road(RoadType::Highway);
                                    close_requested = true;
                                }
                                if ui.button(body_text(&palette, "Rail — Standard")).clicked() {
                                    tool.tool = BuildTool::Rail(RailType::Standard);
                                    close_requested = true;
                                }
                            }
                            BuildPickerCategory::Industry => {
                                draw_industry_picker_tab(ui, &palette, &mut tool, &registry);
                            }
                            BuildPickerCategory::Utilities => {
                                draw_utilities_submenu(ui, &mut tool, &registry, icon_ui.as_ref());
                            }
                            BuildPickerCategory::Shapes => {
                                draw_mock_shapes_submenu(ui, &mut tool, &registry);
                                draw_commercial_submenu(ui, &mut tool, &registry);
                            }
                        }
                    });
            });
        });

    if close_requested {
        picker.close();
        tool.close_submenus();
    } else if tool.building_intent.is_some() && picker.category == BuildPickerCategory::Industry {
        picker.close();
        tool.close_submenus();
    }

    Ok(())
}

fn draw_industry_picker_tab(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
) {
    ui.label(caption_text(palette, PICKER_INDUSTRY_LEAD));
    ui.add_space(4.0);
    for archetype in [BuildingArchetypeId::Factory, BuildingArchetypeId::Depot] {
        let preview = intent_from_archetype(archetype, registry);
        let label = if archetype == BuildingArchetypeId::Factory {
            PICKER_GENERIC_FACTORY
        } else {
            PICKER_GENERIC_DEPOT
        };
        if ui.button(body_text(palette, label)).clicked() {
            tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
            tool.building_intent = Some(preview);
        }
    }
    let (chains, unchained) = industry_chain_groups(registry);
    let industry_empty = chains.is_empty() && unchained.is_empty();
    for (chain_id, defs) in chains {
        ui.separator();
        ui.label(title_text(palette, &human_chain_label(&chain_id)));
        ui.columns(2, |cols| {
            for (i, def) in defs.iter().enumerate() {
                let col = &mut cols[i % 2];
                if industry_card_clicked(col, palette, tool, registry, def) {
                    return;
                }
            }
        });
    }
    if !unchained.is_empty() {
        ui.separator();
        ui.label(title_text(palette, PICKER_INDUSTRY_OTHER));
        for def in unchained {
            let _ = industry_card_clicked(ui, palette, tool, registry, def);
        }
    }
    if industry_empty {
        ui.label(caption_text(palette, PICKER_EMPTY_CATEGORY));
    }
}

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

fn industry_card_clicked(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tool: &mut ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    def: &BuildingDefinition,
) -> bool {
    let selected = tool
        .building_intent
        .as_ref()
        .and_then(|i| i.catalog_id.as_deref())
        == Some(def.id.as_str());
    let power = power_tier_compact(def.power_consumption);
    let frame = egui::Frame::new()
        .fill(palette.bg_interactive)
        .stroke(egui::Stroke::new(
            if selected { 3.0 } else { 1.0 },
            if selected {
                palette.accent_gold
            } else {
                palette.fg_muted
            },
        ))
        .inner_margin(egui::Margin::symmetric(6, 4));
    let mut clicked = false;
    frame.show(ui, |ui| {
        ui.set_min_height(56.0);
        ui.label(body_text(palette, &def.display_name));
        ui.label(caption_text(palette, power));
        clicked = ui.interact(ui.max_rect(), ui.id(), egui::Sense::click()).clicked();
    });
    if clicked {
        tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
        tool.building_intent = registry.intent_preview(def.id.as_str());
    }
    clicked
}

/// Legacy entry — delegates to picker (retires floating submenus).
pub fn draw_sim_build_rail_submenus_egui(
    contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    strip: Res<BuildStripState>,
    palette: Res<UiPalette>,
    tool: ResMut<ActiveBuildTool>,
    picker: ResMut<SimBuildPickerState>,
    registry: Res<BuildingDefinitionRegistry>,
    atlas_ui: Option<Res<PowerHudIconAtlasUi>>,
    manifests: Res<Assets<PowerHudIconAtlasManifest>>,
    tex_cache: ResMut<PowerHudEguiTextureCache>,
) -> Result {
    draw_sim_build_picker_sheet_egui(
        contexts,
        base,
        strip,
        palette,
        tool,
        picker,
        registry,
        atlas_ui,
        manifests,
        tex_cache,
    )
}

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
    }

    #[test]
    fn human_chain_label_portland() {
        assert_eq!(
            super::super::sim_hud_copy::human_chain_label("concrete_portland"),
            "Concrete (Portland)"
        );
    }
}
