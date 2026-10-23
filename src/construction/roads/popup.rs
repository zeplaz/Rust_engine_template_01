//! Road tool popup — Bevy UI (editor product shell). Segment summary, Build / Cancel / snap / upgrade (P2-05 + P9).
//!
//! Spawned once while a road/rail tool is armed; values, toggles and Build enablement are
//! updated in place. Commit intents only via [`commit_road_path_to_queue`] / [`enqueue_road_upgrade`].

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RailType, RoadType};
use crate::construction::construction_pipeline::{ConstructionPlanQueue, ExecutedRoadNetwork};
use crate::construction::sessions::ActiveToolSession;
use crate::construction::snap::RoadSnapSettings;
use crate::construction::upgrade::enqueue_road_upgrade;
use crate::gui::style::widgets::{
    panel_bundle, spawn_button, spawn_header, spawn_label_row, spawn_note, spawn_section_header,
    spawn_toggle, ui_button_fired, ToggleState, UiButton, UiButtonVariant, UiKit, UiTooltip, GAP,
};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{
    product_shell_active, MapCameraDesiredRes, PanelPresence, SimulationMapViewport,
};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::commit::commit_road_path_to_queue;
use super::input::cursor_world_on_map;
use super::placement::ActiveRoadPlacement;

#[derive(Resource, Default)]
pub struct RoadToolPopupState {
    pub cancel_requested: bool,
}

/// Editor road/rail tool popup root (Bevy UI).
#[derive(Component)]
pub struct RoadToolPopupBevyRoot;

#[derive(Component)]
pub(crate) struct RoadToolPopupTitle;

/// Right-hand value text of one stats row.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoadPopupStat {
    Points,
    Segments,
    Valid,
    Cost,
    Width,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoadPopupAction {
    Build,
    Cancel,
    Upgrade,
    ToggleGrid,
    ToggleNode,
    ToggleCurved,
}

fn road_tool_active(tool: &ActiveBuildTool) -> bool {
    matches!(tool.tool, BuildTool::Road(_) | BuildTool::Rail(_))
}

fn popup_title(tool: &ActiveBuildTool) -> &'static str {
    match tool.tool {
        BuildTool::Road(RoadType::Street) => "Road - Street",
        BuildTool::Road(RoadType::Highway) => "Road - Highway",
        BuildTool::Rail(RailType::Standard) => "Rail - Standard",
        _ => "Road",
    }
}

fn valid_segments(placement: &ActiveRoadPlacement) -> usize {
    placement.generated_segments.iter().filter(|s| s.valid).count()
}

fn stat_text(stat: RoadPopupStat, placement: &ActiveRoadPlacement) -> String {
    let valid = valid_segments(placement);
    match stat {
        RoadPopupStat::Points => placement.control_points.len().to_string(),
        RoadPopupStat::Segments => placement.generated_segments.len().to_string(),
        RoadPopupStat::Valid => valid.to_string(),
        RoadPopupStat::Cost => valid.saturating_mul(10).to_string(),
        RoadPopupStat::Width => format!("{:.1}", placement.width),
    }
}

/// Spawn the popup once when a road/rail tool is armed; despawn when it is not.
pub fn sync_road_tool_popup_bevy(
    mut commands: Commands,
    tool: Res<ActiveBuildTool>,
    snap: Res<RoadSnapSettings>,
    placement: Res<ActiveRoadPlacement>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<RoadToolPopupBevyRoot>>,
) {
    if !road_tool_active(tool.as_ref()) {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.road_popup_root = false;
        return;
    }
    if !roots.is_empty() {
        presence.road_popup_root = true;
        return;
    }
    let kit = UiKit::new(&palette, mono.0.clone());
    commands
        .spawn((
            RoadToolPopupBevyRoot,
            panel_bundle(
                &palette,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(12.0),
                    top: Val::Px(200.0),
                    width: Val::Px(248.0),
                    ..default()
                },
            ),
            ZIndex(950),
            Name::new("road_tool_popup_bevy"),
        ))
        .with_children(|root| {
            spawn_header(
                root,
                &kit,
                popup_title(tool.as_ref()),
                RoadToolPopupTitle,
                None::<()>,
            );
            for (label, stat) in [
                ("Control points", RoadPopupStat::Points),
                ("Segments", RoadPopupStat::Segments),
                ("Valid segments", RoadPopupStat::Valid),
                ("Est. cost", RoadPopupStat::Cost),
                ("Width", RoadPopupStat::Width),
            ] {
                spawn_label_row(root, &kit, label, &stat_text(stat, placement.as_ref()), stat);
            }
            spawn_section_header(root, &kit, "Snapping");
            spawn_toggle(root, &kit, "Grid snap", snap.grid_snap, RoadPopupAction::ToggleGrid);
            spawn_toggle(root, &kit, "Node snap", snap.node_snap, RoadPopupAction::ToggleNode);
            spawn_toggle(
                root,
                &kit,
                "Curved preview",
                placement.use_curved_preview,
                RoadPopupAction::ToggleCurved,
            );
            spawn_note(root, &kit, "LMB add - RMB undo - Shift+LMB commit", true, ());
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(GAP),
                margin: UiRect::top(Val::Px(GAP)),
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    &kit,
                    "Build",
                    UiButton::new(UiButtonVariant::Primary)
                        .disabled(valid_segments(placement.as_ref()) == 0),
                    RoadPopupAction::Build,
                )
                .insert(UiTooltip("Queue the valid segments for construction".into()));
                spawn_button(row, &kit, "Cancel", UiButton::default(), RoadPopupAction::Cancel)
                    .insert(UiTooltip("Clear the path being drawn".into()));
            });
            spawn_button(
                root,
                &kit,
                "Upgrade nearest segment",
                UiButton::default(),
                RoadPopupAction::Upgrade,
            )
            .insert(UiTooltip("Upgrade the existing segment under the cursor".into()));
        });
    presence.road_popup_root = true;
}

/// Refresh title, stat values, toggle state and Build enablement in place.
fn sync_road_tool_popup_values(
    tool: Res<ActiveBuildTool>,
    snap: Res<RoadSnapSettings>,
    placement: Res<ActiveRoadPlacement>,
    mut title_q: Query<&mut Text, (With<RoadToolPopupTitle>, Without<RoadPopupStat>)>,
    mut stats_q: Query<(&RoadPopupStat, &mut Text), Without<RoadToolPopupTitle>>,
    mut toggles: Query<(&RoadPopupAction, &mut ToggleState)>,
    mut buttons: Query<(&RoadPopupAction, &mut UiButton), Without<ToggleState>>,
) {
    let title = popup_title(tool.as_ref());
    for mut t in &mut title_q {
        if t.0 != title {
            t.0 = title.to_string();
        }
    }
    for (stat, mut t) in &mut stats_q {
        let next = stat_text(*stat, placement.as_ref());
        if t.0 != next {
            t.0 = next;
        }
    }
    for (action, mut state) in &mut toggles {
        let on = match action {
            RoadPopupAction::ToggleGrid => snap.grid_snap,
            RoadPopupAction::ToggleNode => snap.node_snap,
            RoadPopupAction::ToggleCurved => placement.use_curved_preview,
            _ => continue,
        };
        if state.0 != on {
            state.0 = on;
        }
    }
    let no_valid = valid_segments(placement.as_ref()) == 0;
    for (action, mut btn) in &mut buttons {
        if *action == RoadPopupAction::Build && btn.disabled != no_valid {
            btn.disabled = no_valid;
        }
    }
}

/// Commit / cancel / snap / upgrade — funnel only (no direct graph mutate).
pub fn road_tool_popup_bevy_action_system(
    q: Query<(&Interaction, &UiButton, &RoadPopupAction), Changed<Interaction>>,
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
    for (interaction, button, action) in &q {
        if !ui_button_fired(interaction, button) {
            continue;
        }
        match *action {
            RoadPopupAction::Build => {
                commit_road_path_to_queue(
                    placement.as_mut(),
                    queue.as_mut(),
                    params.as_ref(),
                    session.continuous_path,
                );
                session.record_commit();
            }
            RoadPopupAction::Cancel => {
                placement.control_points.clear();
                placement.generated_segments.clear();
                popup.cancel_requested = true;
            }
            RoadPopupAction::Upgrade => {
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
            RoadPopupAction::ToggleGrid => {
                snap.grid_snap = !snap.grid_snap;
            }
            RoadPopupAction::ToggleNode => {
                snap.node_snap = !snap.node_snap;
            }
            RoadPopupAction::ToggleCurved => {
                placement.use_curved_preview = !placement.use_curved_preview;
            }
        }
    }
}

fn despawn_road_tool_popup_bevy_when_inactive(
    mut commands: Commands,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<RoadToolPopupBevyRoot>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    presence.road_popup_root = false;
}

/// Systems for [`crate::construction::BuildPlanningPlugin`].
pub fn register_road_tool_popup_bevy_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_road_tool_popup_bevy,
            sync_road_tool_popup_values,
            road_tool_popup_bevy_action_system,
        )
            .chain()
            .run_if(product_shell_active),
    )
    .add_systems(
        Update,
        despawn_road_tool_popup_bevy_when_inactive.run_if(not(product_shell_active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::widgets::test_support::init_ok;

    #[test]
    fn popup_systems_have_disjoint_queries() {
        init_ok(sync_road_tool_popup_bevy);
        init_ok(sync_road_tool_popup_values);
        init_ok(road_tool_popup_bevy_action_system);
    }

    #[test]
    fn road_tool_popup_bevy_root_marker_present() {
        assert!(include_str!("popup.rs").contains("RoadToolPopupBevyRoot"));
        assert!(include_str!("popup.rs").contains("sync_road_tool_popup_bevy"));
        assert!(include_str!("popup.rs").contains("commit_road_path_to_queue"));
        assert!(include_str!("popup.rs").contains("enqueue_road_upgrade"));
    }
}
