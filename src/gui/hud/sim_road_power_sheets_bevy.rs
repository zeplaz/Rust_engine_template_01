//! **HUD-NAT-002** — Bevy road + power tool sheets (sim). Gates egui Area draws off.
//!
//! Both sheets are spawned once on entering Simulation and updated in place: values,
//! toggle state, selection and Build enablement never respawn the panel. Sheet heights stay
//! inside `ROAD_TOOL_SHEET_BLOCK_H_PX` / `POWER_TOOL_SHEET_BLOCK_H_PX` (pointer-gate rects).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::{
    commit_power_line_to_utility_graph, commit_road_path_to_queue, cursor_world_on_map,
    enqueue_road_upgrade, ActiveBuildTool, ActivePowerLinePlacement, ActiveRoadPlacement,
    ActiveToolSession, BuildStripState, BuildTool, ConstructionPlanQueue, ExecutedRoadNetwork,
    PowerLineRoutingMode, RailType, RoadSnapSettings, RoadToolPopupState, RoadType, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{PowerHudDrawnIcon, PowerHudIconId};
use crate::gui::hud::sim_hud_copy::{
    ROAD_SHEET_BUILD, ROAD_SHEET_CANCEL, ROAD_SHEET_HINT_INPUT, ROAD_SHEET_UPGRADE,
};
use crate::gui::hud::sim_power_tool_sheet::{SimPowerToolSheetState, POWER_TOOL_SHEET_W_PX};
use crate::gui::hud::sim_road_tool_sheet::{SimRoadToolSheetState, ROAD_TOOL_SHEET_W_PX};
use crate::gui::hud::simulation_shell_phase2::build_rail_slot_anchor_xy;
use crate::gui::style::widgets::{
    panel_bundle, spawn_button, spawn_button_with, spawn_header, spawn_label_row, spawn_note,
    spawn_toggle, ui_button_fired, ToggleState, UiButton, UiButtonVariant, UiKit, UiTooltip, GAP,
    ROW_H, SECTION_GAP,
};
use crate::gui::style::CmdUiMonoFont;
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

/// Value text of one road-sheet stats row.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SimSheetStat {
    Points,
    Valid,
    Cost,
}

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
pub struct SimPowerSheetBuild;

#[derive(Component)]
pub struct SimPowerSheetCancel;

#[derive(Component, Clone, Copy)]
pub struct SimPowerVoltagePick(pub VoltageClass);

#[derive(Component, Clone, Copy)]
pub struct SimPowerModePick(pub PowerLineRoutingMode);

/// Value text of a power-sheet stats row (shares [`SimSheetStat`] kinds).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SimPowerStat(pub SimSheetStat);

#[must_use]
pub fn sim_road_power_sheets_native_wired() -> bool {
    SIM_ROAD_POWER_SHEETS_USE_BEVY
        && include_str!("sim_road_power_sheets_bevy.rs").contains("SimRoadSheetRoot")
        && include_str!("sim_road_tool_sheet.rs").contains("SIM_ROAD_POWER_SHEETS_USE_BEVY")
}

/// Icon + label button used by the power sheet (mode / voltage picks).
fn spawn_sheet_button_with_icon(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands<'_>,
    kit: &UiKit,
    icon: PowerHudIconId,
    label: &str,
    tooltip: &str,
    marker: impl Bundle,
) {
    spawn_button_with(parent, kit, label, UiButton::default(), marker, |b| {
        b.spawn((
            Node {
                width: Val::Px(16.0),
                height: Val::Px(16.0),
                flex_shrink: 0.0,
                ..default()
            },
            PowerHudDrawnIcon(icon),
            bevy::ui::widget::ImageNode::default(),
        ));
    })
    .insert((
        Node {
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            min_height: Val::Px(ROW_H + 4.0),
            padding: UiRect::axes(Val::Px(GAP), Val::Px(GAP)),
            border: UiRect::all(Val::Px(1.0)),
            column_gap: Val::Px(GAP),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        UiTooltip(tooltip.to_string()),
    ));
}

fn grow_node(min_h: f32) -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: Val::Px(0.0),
        min_height: Val::Px(min_h),
        padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
        border: UiRect::all(Val::Px(1.0)),
        column_gap: Val::Px(SECTION_GAP),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn row_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(GAP),
        ..default()
    }
}

pub fn spawn_sim_road_power_sheets_bevy(
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
                    width: Val::Px(ROAD_TOOL_SHEET_W_PX),
                    ..default()
                },
            ),
            Visibility::Hidden,
            ZIndex(945),
            SimRoadSheetRoot,
            Name::new("sim_road_sheet_bevy"),
        ))
        .with_children(|root| {
            spawn_header(root, &kit, "Road", SimRoadSheetTitle, None::<()>);
            spawn_label_row(root, &kit, "Control points", "0", SimSheetStat::Points);
            spawn_label_row(root, &kit, "Valid segments", "0 of 0", SimSheetStat::Valid);
            spawn_label_row(root, &kit, "Est. cost / width", "0", SimSheetStat::Cost);
            root.spawn(row_node()).with_children(|row| {
                spawn_toggle(row, &kit, "Grid snap", true, SimRoadSheetToggleGrid)
                    .insert(grow_node(ROW_H));
                spawn_toggle(row, &kit, "Node snap", true, SimRoadSheetToggleNode)
                    .insert(grow_node(ROW_H));
            });
            spawn_note(root, &kit, ROAD_SHEET_HINT_INPUT, true, ());
            root.spawn(row_node()).with_children(|row| {
                spawn_button(
                    row,
                    &kit,
                    ROAD_SHEET_BUILD,
                    UiButton::new(UiButtonVariant::Primary).disabled(true),
                    SimRoadSheetBuild,
                )
                .insert(grow_node(ROW_H))
                .insert(UiTooltip("Queue the valid segments for construction".into()));
                spawn_button(row, &kit, ROAD_SHEET_CANCEL, UiButton::default(), SimRoadSheetCancel)
                    .insert(grow_node(ROW_H))
                    .insert(UiTooltip("Clear the path being drawn".into()));
                spawn_button(row, &kit, "Upgrade", UiButton::default(), SimRoadSheetUpgrade)
                    .insert(grow_node(ROW_H))
                    .insert(UiTooltip(ROAD_SHEET_UPGRADE.into()));
            });
        });

    commands
        .spawn((
            panel_bundle(
                &palette,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(POWER_TOOL_SHEET_W_PX),
                    ..default()
                },
            ),
            Visibility::Hidden,
            ZIndex(946),
            SimPowerSheetRoot,
            Name::new("sim_power_sheet_bevy"),
        ))
        .with_children(|root| {
            spawn_header(root, &kit, "Power line", SimPowerSheetTitle, None::<()>);
            root.spawn(row_node()).with_children(|row| {
                for (mode, label, tip) in [
                    (PowerLineRoutingMode::Curved, "Curved", "Smooth curved routing"),
                    (PowerLineRoutingMode::Orthogonal90, "90°", "Orthogonal routing, grid snapped"),
                ] {
                    spawn_sheet_button_with_icon(
                        row,
                        &kit,
                        PowerHudIconId::for_routing_mode(mode),
                        label,
                        tip,
                        SimPowerModePick(mode),
                    );
                }
            });
            root.spawn(row_node()).with_children(|row| {
                for (v, label, tip) in [
                    (VoltageClass::Low, "Low", "Distribution"),
                    (VoltageClass::Medium, "Medium", "Medium voltage"),
                    (VoltageClass::High, "High", "Transmission"),
                ] {
                    spawn_sheet_button_with_icon(
                        row,
                        &kit,
                        PowerHudIconId::for_voltage(v),
                        label,
                        tip,
                        SimPowerVoltagePick(v),
                    );
                }
            });
            spawn_label_row(root, &kit, "Control points", "0", SimPowerStat(SimSheetStat::Points));
            spawn_label_row(root, &kit, "Valid segments", "0 of 0", SimPowerStat(SimSheetStat::Valid));
            spawn_label_row(root, &kit, "Est. cost", "0", SimPowerStat(SimSheetStat::Cost));
            root.spawn(row_node()).with_children(|row| {
                spawn_button(
                    row,
                    &kit,
                    "Build line",
                    UiButton::new(UiButtonVariant::Primary).disabled(true),
                    SimPowerSheetBuild,
                )
                .insert(grow_node(ROW_H));
                spawn_button(row, &kit, "Cancel", UiButton::default(), SimPowerSheetCancel)
                    .insert(grow_node(ROW_H));
            });
        });
}

fn set_text(text: &mut Text, next: &str) {
    if text.0 != next {
        text.0 = next.to_string();
    }
}

pub fn sync_sim_road_sheet_bevy(
    base: Res<State<BaseState>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    mut road_sheet: ResMut<SimRoadToolSheetState>,
    placement: Res<ActiveRoadPlacement>,
    snap: Res<RoadSnapSettings>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut root_q: Query<(&mut Node, &mut Visibility), With<SimRoadSheetRoot>>,
    mut title_q: Query<&mut Text, (With<SimRoadSheetTitle>, Without<SimSheetStat>)>,
    mut stats_q: Query<(&SimSheetStat, &mut Text), Without<SimRoadSheetTitle>>,
    mut grid_q: Query<&mut ToggleState, (With<SimRoadSheetToggleGrid>, Without<SimRoadSheetToggleNode>)>,
    mut node_q: Query<&mut ToggleState, (With<SimRoadSheetToggleNode>, Without<SimRoadSheetToggleGrid>)>,
    mut build_q: Query<&mut UiButton, With<SimRoadSheetBuild>>,
) {
    road_sheet.sync_from_strip(strip.as_ref());
    let Ok((mut node, mut vis)) = root_q.single_mut() else {
        return;
    };
    let show = SIM_ROAD_POWER_SHEETS_USE_BEVY
        && matches!(base.get(), BaseState::Simulation)
        && road_sheet.open
        && matches!(tool.tool, BuildTool::Road(_) | BuildTool::Rail(_));
    let want_vis = if show { Visibility::Visible } else { Visibility::Hidden };
    if *vis != want_vis {
        *vis = want_vis;
    }
    if !show {
        return;
    }
    let slot = match strip.active {
        ToolContext::Rail => ToolContext::Rail,
        _ => ToolContext::Roads,
    };
    let anchor = build_rail_slot_anchor_xy(slot, left_stack.collapsed);
    if node.left != Val::Px(anchor.x) {
        node.left = Val::Px(anchor.x);
    }
    if node.top != Val::Px(anchor.y) {
        node.top = Val::Px(anchor.y);
    }

    let title = match tool.tool {
        BuildTool::Road(RoadType::Street) => "Road - Street",
        BuildTool::Road(RoadType::Highway) => "Road - Highway",
        BuildTool::Rail(RailType::Standard) => "Rail - Standard",
        _ => "Road",
    };
    for mut t in &mut title_q {
        set_text(&mut t, title);
    }
    let valid = placement.generated_segments.iter().filter(|s| s.valid).count();
    let total = placement.generated_segments.len();
    for (stat, mut t) in &mut stats_q {
        let next = match stat {
            SimSheetStat::Points => placement.control_points.len().to_string(),
            SimSheetStat::Valid => format!("{valid} of {total}"),
            SimSheetStat::Cost => format!("{} / w {:.1}", valid.saturating_mul(10), placement.width),
        };
        set_text(&mut t, &next);
    }
    for mut s in &mut grid_q {
        if s.0 != snap.grid_snap {
            s.0 = snap.grid_snap;
        }
    }
    for mut s in &mut node_q {
        if s.0 != snap.node_snap {
            s.0 = snap.node_snap;
        }
    }
    for mut b in &mut build_q {
        if b.disabled != (valid == 0) {
            b.disabled = valid == 0;
        }
    }
}

pub fn sync_sim_power_sheet_bevy(
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    mut power_sheet: ResMut<SimPowerToolSheetState>,
    placement: Res<ActivePowerLinePlacement>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut root_q: Query<(&mut Node, &mut Visibility), With<SimPowerSheetRoot>>,
    mut title_q: Query<&mut Text, (With<SimPowerSheetTitle>, Without<SimPowerStat>)>,
    mut stats_q: Query<(&SimPowerStat, &mut Text), Without<SimPowerSheetTitle>>,
    mut voltage_btns: Query<(&SimPowerVoltagePick, &mut UiButton), Without<SimPowerModePick>>,
    mut mode_btns: Query<(&SimPowerModePick, &mut UiButton), Without<SimPowerVoltagePick>>,
    mut build_q: Query<&mut UiButton, (With<SimPowerSheetBuild>, Without<SimPowerVoltagePick>, Without<SimPowerModePick>)>,
) {
    power_sheet.sync_from_tool(tool.as_ref());
    let Ok((mut node, mut vis)) = root_q.single_mut() else {
        return;
    };
    let show = SIM_ROAD_POWER_SHEETS_USE_BEVY
        && matches!(base.get(), BaseState::Simulation)
        && power_sheet.open
        && matches!(tool.tool, BuildTool::PowerLine(_));
    let want_vis = if show { Visibility::Visible } else { Visibility::Hidden };
    if *vis != want_vis {
        *vis = want_vis;
    }
    if !show {
        return;
    }
    let anchor = build_rail_slot_anchor_xy(ToolContext::Utilities, left_stack.collapsed);
    if node.left != Val::Px(anchor.x) {
        node.left = Val::Px(anchor.x);
    }
    if node.top != Val::Px(anchor.y) {
        node.top = Val::Px(anchor.y);
    }

    let active = match tool.tool {
        BuildTool::PowerLine(v) => v,
        _ => VoltageClass::Low,
    };
    let title = match active {
        VoltageClass::Low => "Power line - Distribution",
        VoltageClass::Medium => "Power line - Medium",
        VoltageClass::High => "Power line - Transmission",
    };
    for mut t in &mut title_q {
        set_text(&mut t, title);
    }
    let valid = placement.generated_segments.iter().filter(|s| s.valid).count();
    let total = placement.generated_segments.len();
    for (stat, mut t) in &mut stats_q {
        let next = match stat.0 {
            SimSheetStat::Points => placement.control_points.len().to_string(),
            SimSheetStat::Valid => format!("{valid} of {total}"),
            SimSheetStat::Cost => valid.saturating_mul(12).to_string(),
        };
        set_text(&mut t, &next);
    }
    for (pick, mut b) in &mut voltage_btns {
        let sel = pick.0 == active;
        if b.selected != sel {
            b.selected = sel;
        }
    }
    for (pick, mut b) in &mut mode_btns {
        let sel = pick.0 == placement.routing_mode;
        if b.selected != sel {
            b.selected = sel;
        }
    }
    for mut b in &mut build_q {
        if b.disabled != (valid == 0) {
            b.disabled = valid == 0;
        }
    }
}

pub fn sim_road_sheet_actions(
    build_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimRoadSheetBuild>)>,
    cancel_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimRoadSheetCancel>)>,
    upgrade_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimRoadSheetUpgrade>)>,
    grid_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimRoadSheetToggleGrid>)>,
    node_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimRoadSheetToggleNode>)>,
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
    for (i, b) in &build_q {
        if ui_button_fired(i, b) {
            commit_road_path_to_queue(
                placement.as_mut(),
                queue.as_mut(),
                params.as_ref(),
                session.continuous_path,
            );
            session.record_commit();
        }
    }
    for (i, b) in &cancel_q {
        if ui_button_fired(i, b) {
            placement.control_points.clear();
            placement.generated_segments.clear();
            popup.cancel_requested = true;
        }
    }
    for (i, b) in &upgrade_q {
        if ui_button_fired(i, b) {
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
    for (i, b) in &grid_q {
        if ui_button_fired(i, b) {
            snap.grid_snap = !snap.grid_snap;
        }
    }
    for (i, b) in &node_q {
        if ui_button_fired(i, b) {
            snap.node_snap = !snap.node_snap;
        }
    }
}

pub fn sim_power_sheet_actions(
    build_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimPowerSheetBuild>)>,
    cancel_q: Query<(&Interaction, &UiButton), (Changed<Interaction>, With<SimPowerSheetCancel>)>,
    voltage_q: Query<(&Interaction, &UiButton, &SimPowerVoltagePick), Changed<Interaction>>,
    mode_q: Query<(&Interaction, &UiButton, &SimPowerModePick), Changed<Interaction>>,
    mut tool: ResMut<ActiveBuildTool>,
    mut placement: ResMut<ActivePowerLinePlacement>,
    mut snap_res: ResMut<UtilityNetworkSnapshotResource>,
    mut graph: ResMut<UtilityGraph>,
) {
    for (i, b, pick) in &voltage_q {
        if ui_button_fired(i, b) {
            placement.voltage = pick.0;
            tool.tool = BuildTool::PowerLine(pick.0);
        }
    }
    for (i, b, pick) in &mode_q {
        if ui_button_fired(i, b) {
            placement.routing_mode = pick.0;
            placement.grid_snap = pick.0 == PowerLineRoutingMode::Orthogonal90;
        }
    }
    for (i, b) in &build_q {
        if ui_button_fired(i, b) {
            let voltage = placement.voltage;
            commit_power_line_to_utility_graph(
                placement.as_mut(),
                &mut snap_res.0,
                graph.as_mut(),
                voltage,
            );
        }
    }
    for (i, b) in &cancel_q {
        if ui_button_fired(i, b) {
            placement.clear_path();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::widgets::test_support::init_ok;

    #[test]
    fn sheets_spawn_cleanly() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(crate::gui::UiPalette::default());
        world.insert_resource(crate::gui::style::CmdUiMonoFont(Handle::default()));
        world.run_system_once(spawn_sim_road_power_sheets_bevy).expect("spawn");
    }

    #[test]
    fn sheet_systems_have_disjoint_queries() {
        init_ok(sync_sim_road_sheet_bevy);
        init_ok(sync_sim_power_sheet_bevy);
        init_ok(sim_road_sheet_actions);
        init_ok(sim_power_sheet_actions);
    }

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
