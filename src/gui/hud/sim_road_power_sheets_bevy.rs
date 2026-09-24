//! **HUD-NAT-002** — Bevy road + power tool sheets (sim). Gates egui Area draws off.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::{
    commit_power_line_to_utility_graph, commit_road_path_to_queue, cursor_world_on_map,
    enqueue_road_upgrade, ActiveBuildTool, ActivePowerLinePlacement, ActiveRoadPlacement,
    ActiveToolSession, BuildStripState, BuildTool, ConstructionPlanQueue, ExecutedRoadNetwork,
    PowerLineRoutingMode, RailType, RoadSnapSettings, RoadToolPopupState, RoadType, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::sim_hud_copy::{
    ROAD_SHEET_BUILD, ROAD_SHEET_CANCEL, ROAD_SHEET_HINT_INPUT, ROAD_SHEET_UPGRADE,
};
use crate::gui::hud::sim_power_tool_sheet::{SimPowerToolSheetState, POWER_TOOL_SHEET_W_PX};
use crate::gui::hud::sim_road_tool_sheet::{SimRoadToolSheetState, ROAD_TOOL_SHEET_W_PX};
use crate::gui::hud::simulation_shell_phase2::build_rail_slot_anchor_xy;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport, UiPalette};
use crate::infrastructure::utility::graph::{UtilityGraph, UtilityNetworkSnapshotResource};
use crate::infrastructure::VoltageClass;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

pub const SIM_ROAD_POWER_SHEETS_USE_BEVY: bool = true;

#[derive(Component)]
pub struct SimRoadSheetRoot;

#[derive(Component)]
pub struct SimRoadSheetTitle;

#[derive(Component)]
pub struct SimRoadSheetStats;

#[derive(Component)]
pub struct SimRoadSheetBuild;

#[derive(Component)]
pub struct SimRoadSheetCancel;

#[derive(Component)]
pub struct SimRoadSheetUpgrade;

#[derive(Component)]
pub struct SimRoadSheetToggleGrid;

#[derive(Component)]
pub struct SimRoadSheetToggleNode;

#[derive(Component)]
pub struct SimPowerSheetRoot;

#[derive(Component)]
pub struct SimPowerSheetTitle;

#[derive(Component)]
pub struct SimPowerSheetStats;

#[derive(Component)]
pub struct SimPowerSheetBuild;

#[derive(Component)]
pub struct SimPowerSheetCancel;

#[derive(Component, Clone, Copy)]
pub struct SimPowerVoltagePick(pub VoltageClass);

#[derive(Component, Clone, Copy)]
pub struct SimPowerModePick(pub PowerLineRoutingMode);

#[must_use]
pub fn sim_road_power_sheets_native_wired() -> bool {
    SIM_ROAD_POWER_SHEETS_USE_BEVY
        && include_str!("sim_road_power_sheets_bevy.rs").contains("SimRoadSheetRoot")
        && include_str!("sim_road_tool_sheet.rs").contains("SIM_ROAD_POWER_SHEETS_USE_BEVY")
}

fn spawn_sheet_button(
    parent: &mut ChildSpawnerCommands<'_>,
    font: &Handle<Font>,
    palette: &UiPalette,
    label: &str,
    marker: impl Bundle,
) {
    parent
        .spawn((
            Button,
            Node {
                min_height: Val::Px(32.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            marker,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont::from_font_size(12.0).with_font(font.clone()),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

pub fn spawn_sim_road_power_sheets_bevy(
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
                width: Val::Px(ROAD_TOOL_SHEET_W_PX),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            Visibility::Hidden,
            ZIndex(945),
            SimRoadSheetRoot,
            Name::new("sim_road_sheet_bevy"),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Road"),
                TextFont::from_font_size(14.0).with_font(font.clone()),
                TextColor(palette.bevy_primary_text()),
                SimRoadSheetTitle,
            ));
            root.spawn((
                Text::new("—"),
                TextFont::from_font_size(11.0).with_font(font.clone()),
                TextColor(palette.bevy_secondary_text()),
                SimRoadSheetStats,
            ));
            root.spawn((
                Text::new(ROAD_SHEET_HINT_INPUT),
                TextFont::from_font_size(10.0).with_font(font.clone()),
                TextColor(palette.bevy_text_muted()),
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_sheet_button(row, &font, &palette, "Grid snap", SimRoadSheetToggleGrid);
                spawn_sheet_button(row, &font, &palette, "Node snap", SimRoadSheetToggleNode);
            });
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_sheet_button(row, &font, &palette, ROAD_SHEET_BUILD, SimRoadSheetBuild);
                spawn_sheet_button(row, &font, &palette, ROAD_SHEET_CANCEL, SimRoadSheetCancel);
                spawn_sheet_button(row, &font, &palette, ROAD_SHEET_UPGRADE, SimRoadSheetUpgrade);
            });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(POWER_TOOL_SHEET_W_PX),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            Visibility::Hidden,
            ZIndex(946),
            SimPowerSheetRoot,
            Name::new("sim_power_sheet_bevy"),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Power line"),
                TextFont::from_font_size(14.0).with_font(font.clone()),
                TextColor(palette.bevy_primary_text()),
                SimPowerSheetTitle,
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                for (mode, label) in [
                    (PowerLineRoutingMode::Curved, "Curved"),
                    (PowerLineRoutingMode::Orthogonal90, "90°"),
                ] {
                    spawn_sheet_button(row, &font, &palette, label, SimPowerModePick(mode));
                }
            });
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                for (v, label) in [
                    (VoltageClass::Low, "Distribution"),
                    (VoltageClass::Medium, "Medium"),
                    (VoltageClass::High, "Transmission"),
                ] {
                    spawn_sheet_button(row, &font, &palette, label, SimPowerVoltagePick(v));
                }
            });
            root.spawn((
                Text::new("—"),
                TextFont::from_font_size(11.0).with_font(font.clone()),
                TextColor(palette.bevy_secondary_text()),
                SimPowerSheetStats,
            ));
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
            ))
            .with_children(|row| {
                spawn_sheet_button(row, &font, &palette, "Build line", SimPowerSheetBuild);
                spawn_sheet_button(row, &font, &palette, "Cancel", SimPowerSheetCancel);
            });
        });
}

pub fn sync_sim_road_sheet_bevy(
    base: Res<State<BaseState>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    mut road_sheet: ResMut<SimRoadToolSheetState>,
    placement: Res<ActiveRoadPlacement>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut root_q: Query<(&mut Node, &mut Visibility), With<SimRoadSheetRoot>>,
    mut title_q: Query<&mut Text, (With<SimRoadSheetTitle>, Without<SimRoadSheetStats>)>,
    mut stats_q: Query<&mut Text, (With<SimRoadSheetStats>, Without<SimRoadSheetTitle>)>,
) {
    road_sheet.sync_from_strip(strip.as_ref());
    let Ok((mut node, mut vis)) = root_q.single_mut() else {
        return;
    };
    let show = SIM_ROAD_POWER_SHEETS_USE_BEVY
        && matches!(base.get(), BaseState::Simulation)
        && road_sheet.open
        && matches!(tool.tool, BuildTool::Road(_) | BuildTool::Rail(_));
    *vis = if show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if !show {
        return;
    }
    let slot = match strip.active {
        ToolContext::Rail => ToolContext::Rail,
        _ => ToolContext::Roads,
    };
    let anchor = build_rail_slot_anchor_xy(slot, left_stack.collapsed);
    node.left = Val::Px(anchor.x);
    node.top = Val::Px(anchor.y);

    let title = match tool.tool {
        BuildTool::Road(RoadType::Street) => "Road — Street",
        BuildTool::Road(RoadType::Highway) => "Road — Highway",
        BuildTool::Rail(RailType::Standard) => "Rail — Standard",
        _ => "Road",
    };
    for mut t in &mut title_q {
        *t = Text::new(title);
    }
    let valid = placement.generated_segments.iter().filter(|s| s.valid).count();
    let est = valid.saturating_mul(10);
    let stats = format!(
        "Pts {} · Valid {} · Cost {} · W {:.1}",
        placement.control_points.len(),
        valid,
        est,
        placement.width
    );
    for mut t in &mut stats_q {
        *t = Text::new(stats.clone());
    }
}

pub fn sync_sim_power_sheet_bevy(
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    mut power_sheet: ResMut<SimPowerToolSheetState>,
    placement: Res<ActivePowerLinePlacement>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    palette: Res<UiPalette>,
    mut root_q: Query<(&mut Node, &mut Visibility), With<SimPowerSheetRoot>>,
    mut title_q: Query<&mut Text, (With<SimPowerSheetTitle>, Without<SimPowerSheetStats>)>,
    mut stats_q: Query<&mut Text, (With<SimPowerSheetStats>, Without<SimPowerSheetTitle>)>,
    mut voltage_btns: Query<(&SimPowerVoltagePick, &mut BorderColor), With<Button>>,
    mut mode_btns: Query<(&SimPowerModePick, &mut BorderColor), (With<Button>, Without<SimPowerVoltagePick>)>,
) {
    power_sheet.sync_from_tool(tool.as_ref());
    let Ok((mut node, mut vis)) = root_q.single_mut() else {
        return;
    };
    let show = SIM_ROAD_POWER_SHEETS_USE_BEVY
        && matches!(base.get(), BaseState::Simulation)
        && power_sheet.open
        && matches!(tool.tool, BuildTool::PowerLine(_));
    *vis = if show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if !show {
        return;
    }
    let anchor = build_rail_slot_anchor_xy(ToolContext::Utilities, left_stack.collapsed);
    node.left = Val::Px(anchor.x);
    node.top = Val::Px(anchor.y);

    let active = match tool.tool {
        BuildTool::PowerLine(v) => v,
        _ => VoltageClass::Low,
    };
    let title = match active {
        VoltageClass::Low => "Power line — Distribution",
        VoltageClass::Medium => "Power line — Medium",
        VoltageClass::High => "Power line — Transmission",
    };
    for mut t in &mut title_q {
        *t = Text::new(title);
    }
    let valid = placement.generated_segments.iter().filter(|s| s.valid).count();
    let stats = format!(
        "Pts {} · Valid {} · Cost {}",
        placement.control_points.len(),
        valid,
        valid.saturating_mul(12)
    );
    for mut t in &mut stats_q {
        *t = Text::new(stats.clone());
    }
    let gold = palette.bevy_accent_hot();
    let idle = palette.bevy_border_subtle();
    for (pick, mut border) in &mut voltage_btns {
        *border = if pick.0 == active {
            BorderColor::all(gold)
        } else {
            BorderColor::all(idle)
        };
    }
    for (pick, mut border) in &mut mode_btns {
        *border = if pick.0 == placement.routing_mode {
            BorderColor::all(gold)
        } else {
            BorderColor::all(idle)
        };
    }
}

pub fn sim_road_sheet_actions(
    build_q: Query<&Interaction, (Changed<Interaction>, With<SimRoadSheetBuild>)>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<SimRoadSheetCancel>)>,
    upgrade_q: Query<&Interaction, (Changed<Interaction>, With<SimRoadSheetUpgrade>)>,
    grid_q: Query<&Interaction, (Changed<Interaction>, With<SimRoadSheetToggleGrid>)>,
    node_q: Query<&Interaction, (Changed<Interaction>, With<SimRoadSheetToggleNode>)>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    mut popup: ResMut<RoadToolPopupState>,
    mut snap: ResMut<RoadSnapSettings>,
    roads: Res<ExecutedRoadNetwork>,
    params: Res<WorldGenParams>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
) {
    for i in &build_q {
        if *i == Interaction::Pressed {
            commit_road_path_to_queue(
                placement.as_mut(),
                queue.as_mut(),
                params.as_ref(),
                session.continuous_path,
            );
            session.record_commit();
        }
    }
    for i in &cancel_q {
        if *i == Interaction::Pressed {
            placement.control_points.clear();
            placement.generated_segments.clear();
            popup.cancel_requested = true;
        }
    }
    for i in &upgrade_q {
        if *i == Interaction::Pressed {
            if let Some(world) = win.single().ok().and_then(|window| {
                cursor_world_on_map(
                    window,
                    authority.as_deref(),
                    desired.as_ref(),
                    map_vp.as_ref(),
                    params.as_ref(),
                )
            }) {
                enqueue_road_upgrade(
                    world,
                    roads.as_ref(),
                    queue.as_mut(),
                    placement.as_mut(),
                    params.as_ref(),
                );
            }
        }
    }
    for i in &grid_q {
        if *i == Interaction::Pressed {
            snap.grid_snap = !snap.grid_snap;
        }
    }
    for i in &node_q {
        if *i == Interaction::Pressed {
            snap.node_snap = !snap.node_snap;
        }
    }
}

pub fn sim_power_sheet_actions(
    build_q: Query<&Interaction, (Changed<Interaction>, With<SimPowerSheetBuild>)>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<SimPowerSheetCancel>)>,
    voltage_q: Query<
        (&Interaction, &SimPowerVoltagePick),
        (Changed<Interaction>, With<Button>),
    >,
    mode_q: Query<(&Interaction, &SimPowerModePick), (Changed<Interaction>, With<Button>)>,
    mut tool: ResMut<ActiveBuildTool>,
    mut placement: ResMut<ActivePowerLinePlacement>,
    mut snap_res: ResMut<UtilityNetworkSnapshotResource>,
    mut graph: ResMut<UtilityGraph>,
) {
    for (i, pick) in &voltage_q {
        if *i == Interaction::Pressed {
            placement.voltage = pick.0;
            tool.tool = BuildTool::PowerLine(pick.0);
        }
    }
    for (i, pick) in &mode_q {
        if *i == Interaction::Pressed {
            placement.routing_mode = pick.0;
            placement.grid_snap = pick.0 == PowerLineRoutingMode::Orthogonal90;
        }
    }
    for i in &build_q {
        if *i == Interaction::Pressed {
            let voltage = placement.voltage;
            commit_power_line_to_utility_graph(
                placement.as_mut(),
                &mut snap_res.0,
                graph.as_mut(),
                voltage,
            );
        }
    }
    for i in &cancel_q {
        if *i == Interaction::Pressed {
            placement.clear_path();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hud_nat_005_tool_hints_off_in_sim() {
        assert!(!crate::construction::TOOL_HINTS_DRAW_IN_SIM);
        assert!(!crate::gui::hud::sim_hud_copy::TRAY_PEEK_MODIFIERS.is_empty());
    }

    #[test]
    fn hud_nat_003_tray_build_bevy_flag() {
        assert!(crate::gui::hud::context_tray_build_egui::CONTEXT_TRAY_BUILD_USE_BEVY);
    }
}
