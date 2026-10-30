//! **HUD-NAT-001** — rail-anchored Bevy build picker (sim). Built on the shared widget kit.
//!
//! The panel is spawned once on entering Simulation. The rail slot that opened it decides the
//! category (no second tab strip), the list is rebuilt only when the category or catalog
//! changes, and selection / hover / pressed states are painted in place. Every pick arms a
//! tool (intent only) and closes the sheet so the next click goes to the map.

use bevy::prelude::*;

use crate::construction::building_definitions::menu_rows::{
    affordability, player_depot_stock, MenuItem, MenuRow,
};
use crate::economy::logistics::orders::LogisticsOrdersSnapshotRes;
use crate::construction::{
    ActiveBuildTool, BuildStripState, BuildTool, BuildingDefinitionRegistry, BuildingFamily,
    DefenseKind, PendingConstructionQueue, RailType, RoadType, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::sim_build_picker_sheet::{
    BuildPickerCategory, SimBuildPickerState, BUILD_PICKER_MAX_H_PX, BUILD_PICKER_SHEET_W_PX,
};
use crate::gui::hud::sim_hud_copy::{
    PICKER_DEFENSE_FOOTER_HINT, PICKER_DEFENSE_LEAD, PICKER_EMPTY_CATEGORY,
    PICKER_ROW_BUNKER, PICKER_ROW_CAPTION_DEFENSIVE_WALL, PICKER_ROW_CAPTION_DRAGON_TEETH,
    PICKER_ROW_CAPTION_MINEFIELD, PICKER_ROW_DEFENSIVE_WALL, PICKER_ROW_DEMOLISH,
    PICKER_ROW_DRAGON_TEETH, PICKER_ROW_MINEFIELD, PICKER_ROW_TRENCH_LINE,
    PICKER_SECTION_DEPLOYABLES, PICKER_SECTION_FORTIFICATION, PICKER_SECTION_TRENCHES,
    PICKER_SECTION_WALLS,
};
use crate::gui::hud::simulation_shell_phase2::build_rail_slot_anchor_xy;
use crate::gui::style::widgets::{
    panel_bundle, scroll_column_bundle, spawn_header, spawn_list_button, spawn_note,
    spawn_section_header, ui_button_fired, UiButton, UiButtonLabel, UiKit, FONT_SMALL,
    GAP, SECTION_GAP,
};
use crate::gui::style::CmdUiMonoFont;
use crate::gui::UiPalette;

/// Catalog row hit target — `design_build_place_feel_v1.md`.
pub const BUILD_PICKER_ROW_MIN_H_PX: f32 = 40.0;

#[derive(Component)]
pub struct SimBuildPickerPanelRoot;

#[derive(Component)]
pub struct PickerTitle;

#[derive(Component)]
pub struct PickerList;

#[derive(Component)]
pub struct PickerClose;

#[derive(Component, Clone, Copy)]
pub struct PickerItem(pub usize);

/// Material requirement / shortfall line of a row (updated from depot stock).
#[derive(Component, Clone, Copy)]
pub struct PickerReq(pub usize);

/// What a picker row arms. Intent only — placement stays in the construction funnel.
#[derive(Clone, Debug)]
enum PickerAction {
    Menu(MenuItem),
    Road(RoadType),
    Rail,
    Defense(DefenseKind),
    Demolish,
}

impl PickerAction {
    fn apply(&self, tool: &mut ActiveBuildTool, pending: &mut PendingConstructionQueue) {
        match self {
            Self::Menu(item) => item.apply(tool),
            Self::Road(rt) => {
                tool.clear_building_intent();
                tool.tool = BuildTool::Road(*rt);
            }
            Self::Rail => {
                tool.clear_building_intent();
                tool.tool = BuildTool::Rail(RailType::Standard);
            }
            Self::Defense(kind) => {
                tool.clear_building_intent();
                pending.clear_demolish_pending();
                tool.tool = BuildTool::Defense(*kind);
            }
            Self::Demolish => {
                tool.clear_building_intent();
                tool.tool = BuildTool::Demolish;
            }
        }
    }

    fn is_selected(&self, tool: &ActiveBuildTool) -> bool {
        match self {
            Self::Menu(item) => item.is_selected(tool),
            Self::Road(rt) => tool.tool == BuildTool::Road(*rt),
            Self::Rail => matches!(tool.tool, BuildTool::Rail(_)),
            Self::Defense(kind) => tool.tool == BuildTool::Defense(*kind),
            Self::Demolish => tool.tool == BuildTool::Demolish,
        }
    }
}

#[derive(Clone, Debug)]
enum PickerRow {
    Section(String),
    Note(String),
    Item { label: String, detail: String, action: PickerAction },
}

fn item(label: &str, detail: &str, action: PickerAction) -> PickerRow {
    PickerRow::Item { label: label.to_string(), detail: detail.to_string(), action }
}

fn from_menu(rows: Vec<MenuRow>) -> Vec<PickerRow> {
    rows.into_iter()
        .map(|r| match r {
            MenuRow::Section(s) => PickerRow::Section(s),
            MenuRow::Note(n) => PickerRow::Note(n),
            MenuRow::Item(i) => PickerRow::Item {
                label: i.label.clone(),
                detail: i.detail.clone(),
                action: PickerAction::Menu(i),
            },
        })
        .collect()
}

fn family_rows(
    registry: &BuildingDefinitionRegistry,
    family: BuildingFamily,
    archetype: crate::construction::BuildingArchetypeId,
    section: &str,
) -> Vec<PickerRow> {
    let mut defs: Vec<_> = registry
        .ids_by_family(family)
        .into_iter()
        .filter(|id| !id.starts_with("builtin:") && !id.starts_with("mock:"))
        .filter_map(|id| registry.get(id))
        .collect();
    defs.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    if defs.is_empty() {
        return Vec::new();
    }
    let mut rows = vec![PickerRow::Section(section.to_string())];
    for def in defs {
        let menu = MenuItem::building(
            archetype,
            crate::construction::building_definitions::intent_from_definition(def),
            None,
        );
        rows.push(PickerRow::Item {
            label: menu.label.clone(),
            detail: menu.detail.clone(),
            action: PickerAction::Menu(menu),
        });
    }
    rows
}

fn category_rows(
    category: BuildPickerCategory,
    registry: &BuildingDefinitionRegistry,
) -> Vec<PickerRow> {
    use crate::construction::BuildingArchetypeId as Arch;
    match category {
        BuildPickerCategory::Roads => vec![
            PickerRow::Section("Roads".into()),
            item("Street", "", PickerAction::Road(RoadType::Street)),
            item("Highway", "", PickerAction::Road(RoadType::Highway)),
            PickerRow::Section("Rail".into()),
            item("Rail - Standard", "", PickerAction::Rail),
        ],
        BuildPickerCategory::Industry => {
            from_menu(crate::construction::building_definitions::menu_rows::industrial_rows(registry))
        }
        BuildPickerCategory::Zone => {
            let mut rows = from_menu(crate::construction::draw_residential_submenu(registry));
            rows.extend(family_rows(registry, BuildingFamily::Civic, Arch::Housing, "Civic"));
            rows
        }
        BuildPickerCategory::Utilities => {
            from_menu(crate::construction::draw_utilities_submenu(registry))
        }
        BuildPickerCategory::Shapes => {
            let mut rows = from_menu(crate::construction::draw_commercial_submenu(registry));
            rows.extend(family_rows(registry, BuildingFamily::Logistics, Arch::Depot, "Logistics"));
            rows
        }
        BuildPickerCategory::Defense => vec![
            PickerRow::Note(PICKER_DEFENSE_LEAD.into()),
            PickerRow::Section(PICKER_SECTION_WALLS.into()),
            item(
                PICKER_ROW_DEFENSIVE_WALL,
                PICKER_ROW_CAPTION_DEFENSIVE_WALL,
                PickerAction::Defense(DefenseKind::DefensiveWall),
            ),
            PickerRow::Section(PICKER_SECTION_TRENCHES.into()),
            item(PICKER_ROW_TRENCH_LINE, "", PickerAction::Defense(DefenseKind::TrenchLine)),
            PickerRow::Section(PICKER_SECTION_FORTIFICATION.into()),
            item(PICKER_ROW_BUNKER, "", PickerAction::Defense(DefenseKind::Bunker)),
            PickerRow::Section(PICKER_SECTION_DEPLOYABLES.into()),
            item(
                PICKER_ROW_DRAGON_TEETH,
                PICKER_ROW_CAPTION_DRAGON_TEETH,
                PickerAction::Defense(DefenseKind::DragonTeeth),
            ),
            item(
                PICKER_ROW_MINEFIELD,
                PICKER_ROW_CAPTION_MINEFIELD,
                PickerAction::Defense(DefenseKind::Minefield),
            ),
            PickerRow::Section("Editing".into()),
            item(PICKER_ROW_DEMOLISH, "", PickerAction::Demolish),
            PickerRow::Note(PICKER_DEFENSE_FOOTER_HINT.into()),
        ],
    }
}

/// Items of the built list, indexed by [`PickerItem`].
#[derive(Resource, Default)]
pub struct SimBuildPickerCache {
    actions: Vec<PickerAction>,
    built_for: Option<BuildPickerCategory>,
}

pub fn spawn_sim_build_picker_panel(
    mut commands: Commands,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
) {
    let kit = UiKit::new(&palette, mono.0.clone());
    commands
        .spawn((
            panel_bundle(
                &palette,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(BUILD_PICKER_SHEET_W_PX),
                    max_height: Val::Px(BUILD_PICKER_MAX_H_PX),
                    ..default()
                },
            ),
            Visibility::Hidden,
            ZIndex(940),
            SimBuildPickerPanelRoot,
            Name::new("sim_build_picker_bevy"),
        ))
        .with_children(|root| {
            spawn_header(root, &kit, "Build", PickerTitle, Some(PickerClose));
            root.spawn((
                scroll_column_bundle(Node {
                    width: Val::Percent(100.0),
                    row_gap: Val::Px(GAP),
                    min_height: Val::Px(0.0),
                    flex_shrink: 1.0,
                    ..default()
                }),
                PickerList,
            ));
        });
}

/// The build catalog stays hidden while the power sheet is the open sheet.
#[must_use]
pub fn picker_visible_beside_power(
    picker_open: bool,
    power_open: bool,
    tool_is_power: bool,
) -> bool {
    picker_open && !(power_open && tool_is_power)
}

pub fn sync_sim_build_picker_panel_layout(
    base: Res<State<BaseState>>,
    picker: Res<SimBuildPickerState>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    power_sheet: Res<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut root: Query<(&mut Node, &mut Visibility), With<SimBuildPickerPanelRoot>>,
    mut title: Query<&mut Text, With<PickerTitle>>,
) {
    let Ok((mut node, mut vis)) = root.single_mut() else {
        return;
    };
    let tool_is_power = matches!(tool.tool, BuildTool::PowerLine(_));
    let show = matches!(base.get(), BaseState::Simulation)
        && picker_visible_beside_power(picker.open, power_sheet.open, tool_is_power)
        && strip.active != ToolContext::None;
    let want = if show { Visibility::Visible } else { Visibility::Hidden };
    if *vis != want {
        *vis = want;
    }
    if !show {
        return;
    }
    let anchor = build_rail_slot_anchor_xy(picker.anchor_slot, left_stack.collapsed);
    if node.left != Val::Px(anchor.x) {
        node.left = Val::Px(anchor.x);
    }
    if node.top != Val::Px(anchor.y) {
        node.top = Val::Px(anchor.y);
    }
    let label = picker.sheet_title();
    for mut t in &mut title {
        if t.0 != label {
            t.0 = label.to_string();
        }
    }
}

/// Rebuild the list only when the category or catalog changes.
pub fn rebuild_sim_build_picker_panel_list(
    mut commands: Commands,
    picker: Res<SimBuildPickerState>,
    registry: Res<BuildingDefinitionRegistry>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut cache: ResMut<SimBuildPickerCache>,
    list: Query<Entity, With<PickerList>>,
) {
    if !picker.open {
        return;
    }
    let Ok(list) = list.single() else {
        cache.built_for = None;
        return;
    };
    if cache.built_for == Some(picker.category) && !registry.is_changed() {
        return;
    }
    cache.built_for = Some(picker.category);
    commands.entity(list).despawn_related::<Children>();
    let rows = category_rows(picker.category, &registry);
    let kit = UiKit::new(&palette, mono.0.clone());
    let mut actions: Vec<PickerAction> = Vec::new();
    commands.entity(list).with_children(|col| {
        if !rows.iter().any(|r| matches!(r, PickerRow::Item { .. })) {
            spawn_note(col, &kit, PICKER_EMPTY_CATEGORY, true, ());
        }
        for row in rows {
            match row {
                PickerRow::Section(s) => spawn_section_header(col, &kit, &s),
                PickerRow::Note(n) => spawn_note(col, &kit, &n, true, ()),
                PickerRow::Item { label, detail, action } => {
                    let idx = actions.len();
                    let mut ec = spawn_list_button(
                        col,
                        &kit,
                        &label,
                        UiButton::default(),
                        PickerItem(idx),
                    );
                    ec.insert(Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(BUILD_PICKER_ROW_MIN_H_PX),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    });
                    let bill = match &action {
                        PickerAction::Menu(m) => m.bill.clone(),
                        _ => Vec::new(),
                    };
                    ec.with_children(|b| {
                        if !detail.is_empty() {
                            b.spawn((
                                UiButtonLabel,
                                Text::new(detail.clone()),
                                kit.text_font(FONT_SMALL),
                                TextColor(palette.bevy_text_muted()),
                            ));
                        }
                        if !bill.is_empty() {
                            b.spawn((
                                UiButtonLabel,
                                PickerReq(idx),
                                Text::new(affordability(&bill, None).text),
                                kit.text_font(FONT_SMALL),
                                TextColor(palette.bevy_text_muted()),
                            ));
                        }
                    });
                    actions.push(action);
                }
            }
        }
    });
    cache.actions = actions;
}

/// Highlight the armed tool's row, and gate rows on the player's depot stock — in place.
pub fn sync_sim_build_picker_panel_selection(
    tool: Res<ActiveBuildTool>,
    cache: Res<SimBuildPickerCache>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut items: Query<(&PickerItem, &mut UiButton)>,
    mut reqs: Query<(&PickerReq, &mut Text)>,
) {
    let stock = snapshot.as_deref().and_then(|s| player_depot_stock(&s.0));
    let bill_of = |idx: usize| -> Vec<(&'static str, u32)> {
        match cache.actions.get(idx) {
            Some(PickerAction::Menu(m)) => m.bill.clone(),
            _ => Vec::new(),
        }
    };
    for (item, mut btn) in &mut items {
        let Some(action) = cache.actions.get(item.0) else { continue };
        let sel = action.is_selected(&tool);
        let disabled = !affordability(&bill_of(item.0), stock.as_ref()).affordable;
        if btn.selected != sel || btn.disabled != disabled {
            btn.selected = sel;
            btn.disabled = disabled;
        }
    }
    for (req, mut text) in &mut reqs {
        let next = affordability(&bill_of(req.0), stock.as_ref()).text;
        if text.0 != next {
            text.0 = next;
        }
    }
}

/// True while the click that opened the sheet is still down, so sheet buttons under that
/// pointer cannot immediately fire.
fn swallow_opening_press(
    open: bool,
    mouse_down: bool,
    was_open: &mut bool,
    block: &mut bool,
) -> bool {
    if open && !*was_open {
        *block = true;
    }
    *was_open = open;
    if !*block {
        return false;
    }
    if mouse_down {
        return true;
    }
    *block = false;
    true
}

pub fn sim_build_picker_panel_actions(
    close: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<PickerClose>)>,
    items: Query<(&Interaction, &UiButton, &PickerItem), Changed<Interaction>>,
    cache: Res<SimBuildPickerCache>,
    mut tool: ResMut<ActiveBuildTool>,
    mut picker: ResMut<SimBuildPickerState>,
    mut pending: ResMut<PendingConstructionQueue>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut was_open: Local<bool>,
    mut block: Local<bool>,
) {
    if swallow_opening_press(
        picker.open,
        mouse.pressed(MouseButton::Left),
        &mut was_open,
        &mut block,
    ) {
        return;
    }
    for (interaction, btn) in &close {
        if ui_button_fired(interaction, btn) {
            picker.close();
            tool.close_submenus();
        }
    }
    for (interaction, btn, item) in &items {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        if let Some(action) = cache.actions.get(item.0) {
            action.apply(&mut tool, &mut pending);
            picker.close();
            tool.close_submenus();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::widgets::test_support::init_ok;

    #[test]
    fn picker_systems_have_disjoint_queries() {
        init_ok(sync_sim_build_picker_panel_layout);
        init_ok(rebuild_sim_build_picker_panel_list);
        init_ok(sync_sim_build_picker_panel_selection);
        init_ok(sim_build_picker_panel_actions);
    }


    #[test]
    fn cityfix_one_power_sheet_visible() {
        assert!(!picker_visible_beside_power(true, true, true));
        assert!(picker_visible_beside_power(true, false, true));
        assert!(picker_visible_beside_power(true, true, false));
    }

    #[test]
    fn every_category_offers_a_pick_and_defense_rows_arm_defense() {
        let reg = BuildingDefinitionRegistry::default();
        for cat in BuildPickerCategory::all() {
            let rows = category_rows(cat, &reg);
            assert!(
                rows.iter().any(|r| matches!(r, PickerRow::Item { .. })),
                "{cat:?} must have a pick"
            );
        }
        let mut tool = ActiveBuildTool::default();
        let mut pending = PendingConstructionQueue::default();
        let rows = category_rows(BuildPickerCategory::Defense, &reg);
        let action = rows
            .iter()
            .find_map(|r| match r {
                PickerRow::Item { action: a @ PickerAction::Defense(_), .. } => Some(a.clone()),
                _ => None,
            })
            .expect("defense item");
        assert!(!action.is_selected(&tool));
        action.apply(&mut tool, &mut pending);
        assert!(action.is_selected(&tool));
    }
}
