//! **COD-SIM-HUD-BUILD-PICKER-001** — rail-anchored build picker sheet (sim only).

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::egui;

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
    pub fn open_for_slot(&mut self, slot: ToolContext) {
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
    super::simulation_shell_phase2::build_rail_slot_anchor_y(slot)
}

#[must_use]
pub fn sim_build_picker_sheet_rect(
    state: &SimBuildPickerState,
    left_stack_collapsed: bool,
) -> egui::Rect {
    let anchor = super::simulation_shell_phase2::build_rail_slot_anchor_xy(
        state.anchor_slot,
        left_stack_collapsed,
    );
    egui::Rect::from_min_size(
        anchor,
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
    left_stack: Res<crate::gui::CommandLeftStackState>,
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
    if crate::gui::hud::sim_build_picker_bevy::SIM_BUILD_PICKER_USE_BEVY {
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

    let anchor_bevy = sim_build_picker_sheet_rect(picker.as_ref(), left_stack.collapsed).min;
    let anchor = crate::gui::bevy_logical_to_egui_pos(ctx, anchor_bevy);
    let sheet_w = crate::gui::bevy_logical_vec_to_egui(
        ctx,
        egui::vec2(BUILD_PICKER_SHEET_W_PX, 0.0),
    )
    .x;
    let sheet_body_h = crate::gui::bevy_logical_vec_to_egui(
        ctx,
        egui::vec2(0.0, BUILD_PICKER_MAX_H_PX - 72.0),
    )
    .y;
    let mut close_requested = false;

    egui::Area::new(egui::Id::new("sim_build_picker_sheet"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_width(sheet_w);
            picker_sheet_frame(&palette).show(ui, |ui| {
                picker_header_frame(&palette).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(title_text(&palette, picker.category.title()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(body_text(&palette, "✕"))
                                        .min_size(egui::vec2(
                                            super::sim_build_picker_bevy::BUILD_PICKER_CLOSE_PX,
                                            super::sim_build_picker_bevy::BUILD_PICKER_CLOSE_PX,
                                        )),
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
                    .max_height(sheet_body_h)
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
                            BuildPickerCategory::Defense => {
                                draw_defense_picker_tab(ui, &palette, &mut tool);
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
    } else if matches!(tool.tool, BuildTool::Defense(_))
        && picker.category == BuildPickerCategory::Defense
    {
        picker.close();
        tool.close_submenus();
    }

    Ok(())
}

fn draw_defense_picker_tab(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tool: &mut ActiveBuildTool,
) {
    ui.label(caption_text(palette, PICKER_DEFENSE_LEAD));
    ui.add_space(4.0);

    ui.label(title_text(palette, PICKER_SECTION_WALLS));
    if defense_row_clicked(ui, palette, tool, DefenseKind::DefensiveWall, PICKER_ROW_DEFENSIVE_WALL)
    {
        return;
    }
    ui.separator();
    ui.label(title_text(palette, PICKER_SECTION_TRENCHES));
    if defense_row_clicked(ui, palette, tool, DefenseKind::TrenchLine, PICKER_ROW_TRENCH_LINE) {
        return;
    }
    ui.separator();
    ui.label(title_text(palette, PICKER_SECTION_FORTIFICATION));
    if defense_row_clicked(ui, palette, tool, DefenseKind::Bunker, PICKER_ROW_BUNKER) {
        return;
    }
    ui.separator();
    ui.label(title_text(palette, PICKER_SECTION_DEPLOYABLES));
    if defense_row_with_caption_clicked(
        ui,
        palette,
        tool,
        DefenseKind::DragonTeeth,
        PICKER_ROW_DRAGON_TEETH,
        PICKER_ROW_CAPTION_DRAGON_TEETH,
    ) {
        return;
    }
    if defense_row_with_caption_clicked(
        ui,
        palette,
        tool,
        DefenseKind::Minefield,
        PICKER_ROW_MINEFIELD,
        PICKER_ROW_CAPTION_MINEFIELD,
    ) {
        return;
    }
    ui.separator();
    ui.label(caption_text(palette, "Editing"));
    let demolish_selected = matches!(tool.tool, BuildTool::Demolish);
    let demolish_frame = egui::Frame::new()
        .fill(palette.bg_interactive)
        .stroke(egui::Stroke::new(
            if demolish_selected { 3.0 } else { 1.0 },
            if demolish_selected {
                palette.accent_gold
            } else {
                palette.fg_muted
            },
        ))
        .inner_margin(egui::Margin::symmetric(6, 4));
    demolish_frame.show(ui, |ui| {
        ui.set_min_height(40.0);
        if ui
            .button(body_text(palette, PICKER_ROW_DEMOLISH))
            .clicked()
        {
            tool.clear_building_intent();
            tool.tool = BuildTool::Demolish;
        }
    });
    ui.add_space(4.0);
    ui.label(caption_text(palette, PICKER_DEFENSE_FOOTER_HINT));
}

fn defense_row_clicked(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tool: &mut ActiveBuildTool,
    kind: DefenseKind,
    label: &str,
) -> bool {
    defense_row_with_caption_clicked(ui, palette, tool, kind, label, "")
}

fn defense_row_with_caption_clicked(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tool: &mut ActiveBuildTool,
    kind: DefenseKind,
    label: &str,
    caption: &str,
) -> bool {
    let selected = matches!(tool.tool, BuildTool::Defense(k) if k == kind);
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
        ui.set_min_height(if caption.is_empty() { 48.0 } else { 56.0 });
        ui.vertical(|ui| {
            clicked = ui.button(body_text(palette, label)).clicked();
            if !caption.is_empty() {
                ui.label(caption_text(palette, caption));
            }
        });
    });
    if clicked {
        tool.clear_building_intent();
        tool.tool = BuildTool::Defense(kind);
    }
    clicked
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
            tool.tool = BuildTool::Building(archetype);
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
    left_stack: Res<crate::gui::CommandLeftStackState>,
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
        left_stack,
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
        assert_eq!(BuildPickerCategory::Defense.title(), "Defense");
    }

    #[test]
    fn human_chain_label_portland() {
        assert_eq!(
            super::super::sim_hud_copy::human_chain_label("concrete_portland"),
            "Concrete (Portland)"
        );
    }
}
