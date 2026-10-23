//! Movable construction toolbox — Bevy UI (editor product shell).
//!
//! Tool-select intents only: mutates [`ActiveBuildTool`]; never executes placement.
//! The panel is spawned once and updated in place (selection, intent detail, collapse);
//! only the catalog list is rebuilt, and only when the open category or registry changes.

use bevy::prelude::*;

use crate::gui::hud::{HudDockRegistry, HudWidgetId};
use crate::gui::style::widgets::{
    panel_bundle, scroll_column_bundle, spawn_button, spawn_list_button, spawn_note,
    spawn_section_header, ui_button_fired, UiButton, UiButtonLabel, UiButtonVariant, UiKit,
    UiTooltip, FONT_BODY, FONT_SMALL, FONT_TITLE, GAP, ROW_H, SECTION_GAP,
};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{product_shell_active, PanelPresence};

use super::build_tool_authority::{
    ActiveBuildTool, BuildTool, DefenseKind, RailType, RoadType,
};
use super::building_definitions::menu_rows::{intent_detail_lines, MenuItem, MenuRow};
use super::building_definitions::BuildingDefinitionRegistry;
use super::commercial_menu::draw_commercial_submenu;
use super::industrial_menu::draw_industrial_submenu;
use super::mock_shapes_menu::draw_mock_shapes_submenu;
use super::residential_menu::draw_residential_submenu;
use super::utilities_menu::draw_utilities_submenu;

/// Editor construction toolbox root (Bevy UI).
#[derive(Component)]
pub struct BuildToolboxBevyRoot;

#[derive(Component)]
struct ToolboxBody;

#[derive(Component)]
struct ToolboxCollapse;

#[derive(Component)]
struct ToolboxActiveText;

#[derive(Component)]
struct ToolboxCatalogText;

#[derive(Component)]
struct ToolboxMenuList;

#[derive(Component)]
struct ToolboxIntentText;

#[derive(Component)]
struct ToolboxTitle;

/// Index into [`ToolboxMenuCache`].
#[derive(Component, Clone, Copy)]
struct ToolboxMenuItem(usize);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BuildToolboxAction {
    ZoneResidential,
    Commercial,
    Industrial,
    Utilities,
    MockShapes,
    Road,
    Rail,
    Demolish,
    Clear,
    Defense(DefenseKind),
}

impl BuildToolboxAction {
    fn is_category(self) -> bool {
        matches!(
            self,
            Self::ZoneResidential
                | Self::Commercial
                | Self::Industrial
                | Self::Utilities
                | Self::MockShapes
        )
    }
}

/// Items of the open category, indexed by [`ToolboxMenuItem`].
#[derive(Resource, Default)]
struct ToolboxMenuCache {
    items: Vec<MenuItem>,
    built_for: Option<BuildToolboxAction>,
}

#[derive(Resource, Default)]
struct ToolboxUiState {
    collapsed: bool,
}

fn open_category(tool: &ActiveBuildTool) -> Option<BuildToolboxAction> {
    if tool.residential_menu_open {
        Some(BuildToolboxAction::ZoneResidential)
    } else if tool.commercial_menu_open {
        Some(BuildToolboxAction::Commercial)
    } else if tool.industrial_menu_open {
        Some(BuildToolboxAction::Industrial)
    } else if tool.utilities_menu_open {
        Some(BuildToolboxAction::Utilities)
    } else if tool.mock_shapes_menu_open {
        Some(BuildToolboxAction::MockShapes)
    } else {
        None
    }
}

fn category_rows(
    category: BuildToolboxAction,
    registry: &BuildingDefinitionRegistry,
) -> Vec<MenuRow> {
    match category {
        BuildToolboxAction::ZoneResidential => draw_residential_submenu(registry),
        BuildToolboxAction::Commercial => draw_commercial_submenu(registry),
        BuildToolboxAction::Industrial => draw_industrial_submenu(registry),
        BuildToolboxAction::Utilities => draw_utilities_submenu(registry),
        BuildToolboxAction::MockShapes => draw_mock_shapes_submenu(registry),
        _ => Vec::new(),
    }
}

fn tool_summary(tool: &ActiveBuildTool) -> String {
    match tool.tool {
        BuildTool::None => "none".to_string(),
        BuildTool::Zone(z) => format!("Zone - {z:?}"),
        BuildTool::Building(a) => match &tool.building_intent {
            Some(i) => format!("Building - {}", i.label),
            None => format!("Building - {a:?}"),
        },
        BuildTool::Defense(k) => format!("Defense - {k:?}"),
        BuildTool::Road(r) => format!("Road - {r:?}"),
        BuildTool::Rail(r) => format!("Rail - {r:?}"),
        BuildTool::PowerLine(v) => format!("Power line - {v:?}"),
        BuildTool::Demolish => "Demolish".to_string(),
    }
}

fn action_selected(action: BuildToolboxAction, tool: &ActiveBuildTool) -> bool {
    match action {
        BuildToolboxAction::ZoneResidential => tool.residential_menu_open,
        BuildToolboxAction::Commercial => tool.commercial_menu_open,
        BuildToolboxAction::Industrial => tool.industrial_menu_open,
        BuildToolboxAction::Utilities => tool.utilities_menu_open,
        BuildToolboxAction::MockShapes => tool.mock_shapes_menu_open,
        BuildToolboxAction::Road => matches!(tool.tool, BuildTool::Road(_)),
        BuildToolboxAction::Rail => matches!(tool.tool, BuildTool::Rail(_)),
        BuildToolboxAction::Demolish => tool.tool == BuildTool::Demolish,
        BuildToolboxAction::Clear => false,
        BuildToolboxAction::Defense(k) => tool.tool == BuildTool::Defense(k),
    }
}

/// Spawn the toolbox once while the dock slot is visible; despawn when hidden.
pub fn sync_build_toolbox_bevy(
    mut commands: Commands,
    dock: Res<HudDockRegistry>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<BuildToolboxBevyRoot>>,
) {
    let visible = dock.slot(HudWidgetId::BuildToolbox).visible
        && !dock.slot(HudWidgetId::BuildToolbox).minimized;
    if !visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.build_toolbox_root = false;
        return;
    }
    if !roots.is_empty() {
        presence.build_toolbox_root = true;
        return;
    }

    let kit = UiKit::new(&palette, mono.0.clone());
    commands
        .spawn((
            BuildToolboxBevyRoot,
            panel_bundle(
                &palette,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(72.0),
                    right: Val::Px(16.0),
                    width: Val::Px(288.0),
                    max_height: Val::Vh(84.0),
                    ..default()
                },
            ),
            ZIndex(940),
            Name::new("build_toolbox_bevy"),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(GAP)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .insert(BorderColor::all(palette.bevy_border_subtle()))
            .with_children(|row| {
                row.spawn((
                    ToolboxTitle,
                    Text::new("Construction"),
                    kit.text_font(FONT_TITLE),
                    TextColor(palette.bevy_primary_text()),
                ));
                spawn_button(row, &kit, "Hide", UiButton::default(), ToolboxCollapse).insert(
                    UiTooltip("Collapse / expand the construction panel".into()),
                );
            });

            root.spawn(scroll_column_bundle(Node {
                width: Val::Percent(100.0),
                row_gap: Val::Px(GAP),
                min_height: Val::Px(0.0),
                flex_shrink: 1.0,
                ..default()
            }))
            .insert(ToolboxBody)
            .with_children(|body| {
                body.spawn((
                    ToolboxActiveText,
                    Text::new("Active: none"),
                    kit.text_font(FONT_BODY),
                    TextColor(palette.bevy_accent_gold()),
                ));
                body.spawn((
                    ToolboxCatalogText,
                    Text::new(""),
                    kit.text_font(FONT_SMALL),
                    TextColor(palette.bevy_text_muted()),
                ));

                spawn_section_header(body, &kit, "Buildings & zones");
                for (label, action) in [
                    ("Residential", BuildToolboxAction::ZoneResidential),
                    ("Commercial", BuildToolboxAction::Commercial),
                    ("Industrial", BuildToolboxAction::Industrial),
                    ("Utilities", BuildToolboxAction::Utilities),
                    ("Mock shapes", BuildToolboxAction::MockShapes),
                ] {
                    spawn_toolbox_button(body, &kit, label, action);
                }
                body.spawn((
                    ToolboxMenuList,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(GAP),
                        padding: UiRect::left(Val::Px(SECTION_GAP)),
                        ..default()
                    },
                ));
                body.spawn((
                    ToolboxIntentText,
                    Text::new(""),
                    kit.text_font(FONT_SMALL),
                    TextColor(palette.bevy_secondary_text()),
                    Node { display: Display::None, width: Val::Percent(100.0), ..default() },
                ));

                spawn_section_header(body, &kit, "Transport");
                spawn_toolbox_button(body, &kit, "Roads", BuildToolboxAction::Road);
                spawn_toolbox_button(body, &kit, "Rail", BuildToolboxAction::Rail);

                spawn_section_header(body, &kit, "Defense");
                for (kind, label) in [
                    (DefenseKind::DefensiveWall, "Wall"),
                    (DefenseKind::TrenchLine, "Trench"),
                    (DefenseKind::Bunker, "Bunker"),
                    (DefenseKind::DragonTeeth, "Dragon teeth"),
                    (DefenseKind::Minefield, "Minefield"),
                ] {
                    spawn_toolbox_button(body, &kit, label, BuildToolboxAction::Defense(kind));
                }

                spawn_section_header(body, &kit, "Edit");
                spawn_toolbox_button(body, &kit, "Demolish", BuildToolboxAction::Demolish);
                spawn_toolbox_button(body, &kit, "Clear tool", BuildToolboxAction::Clear)
                    .insert(UiTooltip("Disarm the active tool (Esc closes an open list)".into()));
                spawn_note(body, &kit, "Pick a category, then an entry. Click the map to place.", true, ());
            });
        });
    presence.build_toolbox_root = true;
}

fn spawn_toolbox_button<'a>(
    parent: &'a mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    action: BuildToolboxAction,
) -> EntityCommands<'a> {
    let variant = if action == BuildToolboxAction::Demolish {
        UiButtonVariant::Danger
    } else {
        UiButtonVariant::Normal
    };
    let mut ec = spawn_button(parent, kit, label, UiButton::new(variant), action);
    ec.insert(Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(ROW_H),
        padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: if action.is_category() {
            JustifyContent::SpaceBetween
        } else {
            JustifyContent::Center
        },
        align_items: AlignItems::Center,
        ..default()
    });
    ec
}

/// Rebuild the catalog list only when the open category or the registry changes.
fn rebuild_toolbox_menu_list(
    mut commands: Commands,
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut cache: ResMut<ToolboxMenuCache>,
    list: Query<Entity, With<ToolboxMenuList>>,
) {
    let Ok(list) = list.single() else {
        // Panel not spawned: force a rebuild once it exists.
        cache.built_for = None;
        return;
    };
    let want = open_category(&tool);
    if cache.built_for == want && !registry.is_changed() && !cache.is_added() {
        return;
    }
    cache.built_for = want;
    cache.items.clear();
    commands.entity(list).despawn_related::<Children>();
    let Some(category) = want else { return };
    let rows = category_rows(category, &registry);
    let kit = UiKit::new(&palette, mono.0.clone());
    let mut items: Vec<MenuItem> = Vec::new();
    commands.entity(list).with_children(|col| {
        for row in rows {
            match row {
                MenuRow::Section(title) => spawn_section_header(col, &kit, &title),
                MenuRow::Note(text) => spawn_note(col, &kit, &text, true, ()),
                MenuRow::Item(item) => {
                    let idx = items.len();
                    let detail = item.detail.clone();
                    let mut ec = spawn_list_button(
                        col,
                        &kit,
                        &item.label,
                        UiButton::default(),
                        ToolboxMenuItem(idx),
                    );
                    if !detail.is_empty() {
                        ec.insert(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        });
                        ec.with_children(|b| {
                            b.spawn((
                                UiButtonLabel,
                                Text::new(detail),
                                kit.text_font(FONT_SMALL),
                                TextColor(palette.bevy_text_muted()),
                            ));
                        });
                    }
                    items.push(item);
                }
            }
        }
    });
    cache.items = items;
}

/// Selection, active-tool text, intent detail and collapse state — all in place.
fn sync_toolbox_state(
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    ui: Res<ToolboxUiState>,
    cache: Res<ToolboxMenuCache>,
    mut actions: Query<(&BuildToolboxAction, &mut UiButton), Without<ToolboxMenuItem>>,
    mut items: Query<(&ToolboxMenuItem, &mut UiButton), Without<BuildToolboxAction>>,
    mut active: Query<&mut Text, (With<ToolboxActiveText>, Without<ToolboxCatalogText>, Without<ToolboxIntentText>, Without<UiButtonLabel>)>,
    mut catalog: Query<&mut Text, (With<ToolboxCatalogText>, Without<ToolboxActiveText>, Without<ToolboxIntentText>, Without<UiButtonLabel>)>,
    mut intent: Query<(&mut Text, &mut Node), (With<ToolboxIntentText>, Without<ToolboxActiveText>, Without<ToolboxCatalogText>, Without<UiButtonLabel>, Without<ToolboxBody>)>,
    mut body: Query<&mut Node, (With<ToolboxBody>, Without<ToolboxIntentText>)>,
) {
    for (action, mut btn) in &mut actions {
        let sel = action_selected(*action, &tool);
        if btn.selected != sel {
            btn.selected = sel;
        }
    }
    for (item, mut btn) in &mut items {
        let sel = cache
            .items
            .get(item.0)
            .is_some_and(|i| i.is_selected(&tool));
        if btn.selected != sel {
            btn.selected = sel;
        }
    }
    let summary = format!("Active: {}", tool_summary(&tool));
    for mut t in &mut active {
        if t.0 != summary {
            t.0 = summary.clone();
        }
    }
    let cat = format!("Catalog: {} definitions", registry.by_id.len());
    for mut t in &mut catalog {
        if t.0 != cat {
            t.0 = cat.clone();
        }
    }
    let detail = tool
        .building_intent
        .as_ref()
        .map(|i| format!("{}\n{}", i.label, intent_detail_lines(i).join("\n")));
    for (mut t, mut node) in &mut intent {
        match &detail {
            Some(d) => {
                if t.0 != *d {
                    t.0 = d.clone();
                }
                if node.display != Display::Flex {
                    node.display = Display::Flex;
                }
            }
            None => {
                if node.display != Display::None {
                    node.display = Display::None;
                }
            }
        }
    }
    for mut node in &mut body {
        let want = if ui.collapsed { Display::None } else { Display::Flex };
        if node.display != want {
            node.display = want;
        }
    }
}

/// Tool-select intents only — no execute / place / queue mutation.
fn build_toolbox_bevy_action_system(
    q: Query<
        (&Interaction, &UiButton, &BuildToolboxAction),
        (Changed<Interaction>, Without<ToolboxMenuItem>),
    >,
    items: Query<(&Interaction, &UiButton, &ToolboxMenuItem), Changed<Interaction>>,
    collapse: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<ToolboxCollapse>)>,
    cache: Res<ToolboxMenuCache>,
    mut ui: ResMut<ToolboxUiState>,
    mut tool: ResMut<ActiveBuildTool>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut labels: Query<(&ChildOf, &mut Text), With<UiButtonLabel>>,
    collapse_btn: Query<Entity, With<ToolboxCollapse>>,
) {
    if keys.is_some_and(|k| k.just_pressed(KeyCode::Escape)) && open_category(&tool).is_some() {
        tool.close_submenus();
    }
    for (interaction, btn) in &collapse {
        if ui_button_fired(interaction, btn) {
            ui.collapsed = !ui.collapsed;
            for (parent, mut text) in &mut labels {
                if collapse_btn.get(parent.parent()).is_ok() {
                    text.0 = if ui.collapsed { "Show".into() } else { "Hide".into() };
                }
            }
        }
    }
    for (interaction, btn, item) in &items {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        if let Some(entry) = cache.items.get(item.0) {
            entry.apply(&mut tool);
        }
    }
    for (interaction, btn, action) in &q {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        match *action {
            a if a.is_category() => {
                let was_open = open_category(&tool) == Some(a);
                tool.close_submenus();
                if !was_open {
                    match a {
                        BuildToolboxAction::ZoneResidential => tool.residential_menu_open = true,
                        BuildToolboxAction::Commercial => tool.commercial_menu_open = true,
                        BuildToolboxAction::Industrial => tool.industrial_menu_open = true,
                        BuildToolboxAction::Utilities => tool.utilities_menu_open = true,
                        _ => tool.mock_shapes_menu_open = true,
                    }
                }
            }
            BuildToolboxAction::Road => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Road(RoadType::Street);
            }
            BuildToolboxAction::Rail => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Rail(RailType::Standard);
            }
            BuildToolboxAction::Demolish => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Demolish;
            }
            BuildToolboxAction::Clear => {
                tool.tool = BuildTool::None;
                tool.close_submenus();
                tool.clear_building_intent();
            }
            BuildToolboxAction::Defense(kind) => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Defense(kind);
            }
            _ => {}
        }
    }
}

fn despawn_build_toolbox_bevy_when_inactive(
    mut commands: Commands,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<BuildToolboxBevyRoot>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    presence.build_toolbox_root = false;
}

/// Systems for [`crate::construction::BuildPlanningPlugin`].
pub fn register_build_toolbox_bevy_systems(app: &mut App) {
    app.init_resource::<ToolboxMenuCache>()
        .init_resource::<ToolboxUiState>()
        .add_systems(
            Update,
            (
                sync_build_toolbox_bevy,
                rebuild_toolbox_menu_list,
                build_toolbox_bevy_action_system,
                sync_toolbox_state,
            )
                .chain()
                .run_if(product_shell_active),
        )
        .add_systems(
            Update,
            despawn_build_toolbox_bevy_when_inactive.run_if(not(product_shell_active)),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::widgets::test_support::init_ok;

    #[test]
    fn toolbox_systems_have_disjoint_queries() {
        init_ok(sync_build_toolbox_bevy);
        init_ok(rebuild_toolbox_menu_list);
        init_ok(sync_toolbox_state);
        init_ok(build_toolbox_bevy_action_system);
    }


    #[test]
    fn menu_rows_have_selectable_items_for_every_category() {
        let reg = BuildingDefinitionRegistry::default();
        for cat in [
            BuildToolboxAction::ZoneResidential,
            BuildToolboxAction::Commercial,
            BuildToolboxAction::Industrial,
            BuildToolboxAction::Utilities,
        ] {
            let rows = category_rows(cat, &reg);
            assert!(
                rows.iter().any(|r| matches!(r, MenuRow::Item(_))),
                "{cat:?} must offer at least one entry"
            );
        }
    }

    #[test]
    fn picking_an_item_arms_the_tool_and_marks_it_selected() {
        let reg = BuildingDefinitionRegistry::default();
        let rows = category_rows(BuildToolboxAction::Industrial, &reg);
        let item = rows
            .into_iter()
            .find_map(|r| match r {
                MenuRow::Item(i) => Some(i),
                _ => None,
            })
            .expect("industrial item");
        let mut tool = ActiveBuildTool::default();
        assert!(!item.is_selected(&tool));
        item.apply(&mut tool);
        assert!(item.is_selected(&tool));
        assert!(tool.building_intent.is_some());
    }
}
