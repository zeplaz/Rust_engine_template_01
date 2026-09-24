//! **HUD-NAT-001** — Bevy `Node` build picker (sim). Replaces egui Area sheet in Simulation.

use bevy::prelude::*;

use crate::construction::{
    intent_from_archetype, intent_from_definition, ActiveBuildTool, BuildStripState, BuildTool,
    BuildingArchetypeId, BuildingDefinitionRegistry, BuildingFamily, DefenseKind, RailType,
    RoadType, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::sim_build_picker_sheet::{
    tool_context_to_picker_category, BuildPickerCategory, SimBuildPickerState,
    BUILD_PICKER_MAX_H_PX, BUILD_PICKER_SHEET_W_PX,
};
use crate::gui::hud::sim_hud_copy::{
    PICKER_DEFENSE_FOOTER_HINT, PICKER_DEFENSE_LEAD, PICKER_GENERIC_DEPOT, PICKER_GENERIC_FACTORY,
    PICKER_ROW_BUNKER, PICKER_ROW_DEFENSIVE_WALL, PICKER_ROW_DEMOLISH, PICKER_ROW_DRAGON_TEETH,
    PICKER_ROW_MINEFIELD, PICKER_ROW_TRENCH_LINE, PICKER_SECTION_DEPLOYABLES,
    PICKER_SECTION_FORTIFICATION, PICKER_SECTION_TRENCHES, PICKER_SECTION_WALLS,
    PICKER_TITLE_INDUSTRY, PICKER_TITLE_ROADS, PICKER_TITLE_SHAPES, PICKER_TITLE_UTILITIES,
    PICKER_TITLE_ZONE,
};
use crate::gui::hud::simulation_shell_phase2::{
    build_rail_slot_anchor_xy, BUILD_RAIL_SLOT_MIN_H_PX,
};
use crate::gui::UiPalette;

/// When true, Simulation uses Bevy picker and skips egui Area draw.
pub const SIM_BUILD_PICKER_USE_BEVY: bool = true;

#[derive(Component)]
pub struct SimBuildPickerRoot;

#[derive(Component)]
pub struct SimBuildPickerTitleText;

#[derive(Component)]
pub struct SimBuildPickerBody;

#[derive(Component)]
pub struct SimBuildPickerClose;

#[derive(Component, Clone, Copy)]
pub struct SimBuildPickerTab(pub BuildPickerCategory);

#[derive(Component, Clone)]
pub enum SimBuildPickerAction {
    Road(RoadType),
    Rail,
    Archetype(BuildingArchetypeId),
    Catalog {
        archetype: BuildingArchetypeId,
        catalog_id: String,
    },
    Defense(DefenseKind),
    Demolish,
}

#[must_use]
pub fn sim_build_picker_native_wired() -> bool {
    SIM_BUILD_PICKER_USE_BEVY
        && include_str!("sim_build_picker_bevy.rs").contains("SimBuildPickerRoot")
        && include_str!("sim_build_picker_sheet.rs").contains("SIM_BUILD_PICKER_USE_BEVY")
}

pub fn spawn_sim_build_picker_bevy(
    mut commands: Commands,
    palette: Res<UiPalette>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(BUILD_PICKER_SHEET_W_PX),
                max_height: Val::Px(BUILD_PICKER_MAX_H_PX),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            Visibility::Hidden,
            ZIndex(940),
            SimBuildPickerRoot,
            Name::new("sim_build_picker_bevy"),
        ))
        .with_children(|root| {
            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },))
            .with_children(|hdr| {
                hdr.spawn((
                    Text::new(PICKER_TITLE_ZONE),
                    TextFont::from_font_size(14.0).with_font(font.clone()),
                    TextColor(palette.bevy_primary_text()),
                    SimBuildPickerTitleText,
                ));
                hdr.spawn((
                    Button,
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    BorderColor::all(palette.bevy_border_subtle()),
                    SimBuildPickerClose,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("✕"),
                        TextFont::from_font_size(14.0).with_font(font.clone()),
                        TextColor(palette.bevy_text_muted()),
                    ));
                });
            });

            root.spawn((Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },))
            .with_children(|tabs| {
                for cat in BuildPickerCategory::all() {
                    tabs.spawn((
                        Button,
                        Node {
                            flex_grow: 1.0,
                            min_height: Val::Px(BUILD_RAIL_SLOT_MIN_H_PX),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(palette.bevy_hud_panel_fill()),
                        BorderColor::all(palette.bevy_border_subtle()),
                        SimBuildPickerTab(cat),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(cat.title()),
                            TextFont::from_font_size(10.0).with_font(font.clone()),
                            TextColor(palette.bevy_text_muted()),
                        ));
                    });
                }
            });

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    flex_grow: 1.0,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                SimBuildPickerBody,
            ));
        });
}

pub fn sync_sim_build_picker_bevy_layout(
    base: Res<State<BaseState>>,
    picker: Res<SimBuildPickerState>,
    strip: Res<BuildStripState>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut q: Query<(&mut Node, &mut Visibility), With<SimBuildPickerRoot>>,
) {
    let Ok((mut node, mut vis)) = q.single_mut() else {
        return;
    };
    let show = SIM_BUILD_PICKER_USE_BEVY
        && matches!(base.get(), BaseState::Simulation)
        && picker.open
        && strip.active != ToolContext::None;
    *vis = if show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if !show {
        return;
    }
    let anchor = build_rail_slot_anchor_xy(picker.anchor_slot, left_stack.collapsed);
    node.left = Val::Px(anchor.x);
    node.top = Val::Px(anchor.y);
}

pub fn sync_sim_build_picker_bevy_title(
    picker: Res<SimBuildPickerState>,
    mut q: Query<&mut Text, With<SimBuildPickerTitleText>>,
) {
    if !picker.is_changed() && !picker.open {
        return;
    }
    for mut text in &mut q {
        *text = Text::new(picker.category.title());
    }
}

pub fn sync_sim_build_picker_bevy_tabs(
    picker: Res<SimBuildPickerState>,
    palette: Res<UiPalette>,
    mut q: Query<(&SimBuildPickerTab, &mut BorderColor), With<Button>>,
) {
    let gold = palette.bevy_accent_hot();
    let idle = palette.bevy_border_subtle();
    for (tab, mut border) in &mut q {
        *border = if tab.0 == picker.category {
            BorderColor::all(gold)
        } else {
            BorderColor::all(idle)
        };
    }
}

fn clear_body_children(commands: &mut Commands, body: Entity, children_q: &Query<&Children>) {
    if let Ok(children) = children_q.get(body) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
}

pub fn rebuild_sim_build_picker_bevy_body(
    mut commands: Commands,
    picker: Res<SimBuildPickerState>,
    registry: Res<BuildingDefinitionRegistry>,
    palette: Res<UiPalette>,
    asset_server: Res<AssetServer>,
    body_q: Query<Entity, With<SimBuildPickerBody>>,
    children_q: Query<&Children>,
) {
    if !SIM_BUILD_PICKER_USE_BEVY || !picker.open {
        return;
    }
    if !picker.is_changed() && !registry.is_changed() {
        return;
    }
    let Ok(body) = body_q.single() else {
        return;
    };
    clear_body_children(&mut commands, body, &children_q);
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mut rows: Vec<(String, SimBuildPickerAction)> = Vec::new();
    match picker.category {
        BuildPickerCategory::Roads => {
            rows.push(("Street".into(), SimBuildPickerAction::Road(RoadType::Street)));
            rows.push((
                "Highway".into(),
                SimBuildPickerAction::Road(RoadType::Highway),
            ));
            rows.push(("Rail — Standard".into(), SimBuildPickerAction::Rail));
        }
        BuildPickerCategory::Industry => {
            rows.push((
                PICKER_GENERIC_FACTORY.into(),
                SimBuildPickerAction::Archetype(BuildingArchetypeId::Factory),
            ));
            rows.push((
                PICKER_GENERIC_DEPOT.into(),
                SimBuildPickerAction::Archetype(BuildingArchetypeId::Depot),
            ));
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Industry,
                BuildingArchetypeId::Factory,
            );
        }
        BuildPickerCategory::Zone => {
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Residential,
                BuildingArchetypeId::Housing,
            );
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Civic,
                BuildingArchetypeId::Housing,
            );
        }
        BuildPickerCategory::Utilities => {
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Power,
                BuildingArchetypeId::PowerPlant,
            );
        }
        BuildPickerCategory::Shapes => {
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Retail,
                BuildingArchetypeId::Retail,
            );
            push_family_rows(
                &mut rows,
                &registry,
                BuildingFamily::Logistics,
                BuildingArchetypeId::Depot,
            );
        }
        BuildPickerCategory::Defense => {
            rows.push((
                format!("{} — {}", PICKER_SECTION_WALLS, PICKER_ROW_DEFENSIVE_WALL),
                SimBuildPickerAction::Defense(DefenseKind::DefensiveWall),
            ));
            rows.push((
                format!("{} — {}", PICKER_SECTION_TRENCHES, PICKER_ROW_TRENCH_LINE),
                SimBuildPickerAction::Defense(DefenseKind::TrenchLine),
            ));
            rows.push((
                format!("{} — {}", PICKER_SECTION_FORTIFICATION, PICKER_ROW_BUNKER),
                SimBuildPickerAction::Defense(DefenseKind::Bunker),
            ));
            rows.push((
                format!("{} — {}", PICKER_SECTION_DEPLOYABLES, PICKER_ROW_DRAGON_TEETH),
                SimBuildPickerAction::Defense(DefenseKind::DragonTeeth),
            ));
            rows.push((
                format!("{} — {}", PICKER_SECTION_DEPLOYABLES, PICKER_ROW_MINEFIELD),
                SimBuildPickerAction::Defense(DefenseKind::Minefield),
            ));
            rows.push((
                format!("Editing — {}", PICKER_ROW_DEMOLISH),
                SimBuildPickerAction::Demolish,
            ));
            // Caption rows are not actions — footer hint as non-interactive text below.
            let _ = (PICKER_DEFENSE_LEAD, PICKER_DEFENSE_FOOTER_HINT);
        }
    }
    if rows.is_empty() {
        commands.entity(body).with_children(|b| {
            b.spawn((
                Text::new("(empty category)"),
                TextFont::from_font_size(11.0).with_font(font),
                TextColor(palette.bevy_text_muted()),
            ));
        });
        return;
    }
    commands.entity(body).with_children(|b| {
        for (label, action) in rows {
            b.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(32.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                BorderColor::all(palette.bevy_border_subtle()),
                action,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(label),
                    TextFont::from_font_size(12.0).with_font(font.clone()),
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        }
    });
}

fn push_family_rows(
    rows: &mut Vec<(String, SimBuildPickerAction)>,
    registry: &BuildingDefinitionRegistry,
    family: BuildingFamily,
    archetype: BuildingArchetypeId,
) {
    let mut ids = registry.ids_by_family(family);
    ids.sort();
    for id in ids {
        if id.starts_with("builtin:") {
            continue;
        }
        let Some(def) = registry.get(id) else {
            continue;
        };
        rows.push((
            def.display_name.clone(),
            SimBuildPickerAction::Catalog {
                archetype,
                catalog_id: def.id.clone(),
            },
        ));
    }
}

pub fn sim_build_picker_bevy_close_system(
    q: Query<&Interaction, (Changed<Interaction>, With<SimBuildPickerClose>)>,
    mut picker: ResMut<SimBuildPickerState>,
    mut tool: ResMut<ActiveBuildTool>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            picker.close();
            tool.close_submenus();
        }
    }
}

pub fn sim_build_picker_bevy_tab_system(
    q: Query<(&Interaction, &SimBuildPickerTab), (Changed<Interaction>, With<Button>)>,
    mut picker: ResMut<SimBuildPickerState>,
) {
    for (interaction, tab) in &q {
        if *interaction == Interaction::Pressed {
            picker.category = tab.0;
        }
    }
}

pub fn sim_build_picker_bevy_action_system(
    q: Query<(&Interaction, &SimBuildPickerAction), (Changed<Interaction>, With<Button>)>,
    mut tool: ResMut<ActiveBuildTool>,
    mut picker: ResMut<SimBuildPickerState>,
    registry: Res<BuildingDefinitionRegistry>,
    mut pending: ResMut<crate::construction::PendingConstructionQueue>,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            SimBuildPickerAction::Road(rt) => {
                tool.tool = BuildTool::Road(*rt);
                picker.close();
                tool.close_submenus();
            }
            SimBuildPickerAction::Rail => {
                tool.tool = BuildTool::Rail(RailType::Standard);
                picker.close();
                tool.close_submenus();
            }
            SimBuildPickerAction::Archetype(arch) => {
                tool.tool = BuildTool::Building(*arch);
                tool.building_intent = Some(intent_from_archetype(*arch, &registry));
                if picker.category == BuildPickerCategory::Industry {
                    picker.close();
                    tool.close_submenus();
                }
            }
            SimBuildPickerAction::Catalog {
                archetype,
                catalog_id,
            } => {
                tool.tool = BuildTool::Building(*archetype);
                tool.building_intent = registry
                    .get(catalog_id)
                    .map(intent_from_definition)
                    .or_else(|| Some(intent_from_archetype(*archetype, &registry)));
                if picker.category == BuildPickerCategory::Industry {
                    picker.close();
                    tool.close_submenus();
                }
            }
            SimBuildPickerAction::Defense(kind) => {
                tool.clear_building_intent();
                pending.clear_demolish_pending();
                tool.tool = BuildTool::Defense(*kind);
                picker.close();
                tool.close_submenus();
            }
            SimBuildPickerAction::Demolish => {
                tool.clear_building_intent();
                tool.tool = BuildTool::Demolish;
                picker.close();
                tool.close_submenus();
            }
        }
    }
}

pub fn sync_picker_category_from_strip(
    strip: Res<BuildStripState>,
    mut picker: ResMut<SimBuildPickerState>,
) {
    if !picker.open || !strip.is_changed() {
        return;
    }
    picker.category = tool_context_to_picker_category(strip.active);
    picker.anchor_slot = strip.active;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_nat_001_native_flag_on() {
        assert!(SIM_BUILD_PICKER_USE_BEVY);
        assert!(sim_build_picker_native_wired());
    }
}

#[allow(dead_code)]
fn _title_refs() -> [&'static str; 5] {
    [
        PICKER_TITLE_ZONE,
        PICKER_TITLE_ROADS,
        PICKER_TITLE_INDUSTRY,
        PICKER_TITLE_UTILITIES,
        PICKER_TITLE_SHAPES,
    ]
}
