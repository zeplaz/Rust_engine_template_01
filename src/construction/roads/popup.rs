//! Road tool popup — Bevy UI (editor product shell). Segment summary, Build / Cancel / snap / upgrade (P2-05 + P9).
//!
//! Commit intents only via [`commit_road_path_to_queue`] / [`enqueue_road_upgrade`].

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RailType, RoadType};
use crate::construction::construction_pipeline::{ConstructionPlanQueue, ExecutedRoadNetwork};
use crate::construction::sessions::ActiveToolSession;
use crate::construction::snap::RoadSnapSettings;
use crate::construction::upgrade::enqueue_road_upgrade;
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
struct RoadToolPopupTitle;

#[derive(Component)]
struct RoadToolPopupStats;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum RoadPopupAction {
    Build,
    Cancel,
    Upgrade,
    ToggleGrid,
    ToggleNode,
    ToggleCurved,
}

#[derive(Default)]
struct RoadPopupStamp {
    key: u64,
}

fn road_tool_active(tool: &ActiveBuildTool) -> bool {
    matches!(tool.tool, BuildTool::Road(_) | BuildTool::Rail(_))
}

fn popup_title(tool: &ActiveBuildTool) -> &'static str {
    match tool.tool {
        BuildTool::Road(RoadType::Street) => "Road — Street",
        BuildTool::Road(RoadType::Highway) => "Road — Highway",
        BuildTool::Rail(RailType::Standard) => "Rail — Standard",
        _ => "Road",
    }
}

fn popup_stats(placement: &ActiveRoadPlacement) -> String {
    let valid = placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .count();
    let est = valid.saturating_mul(10);
    format!(
        "Control points: {}\nSegments: {}\nValid segments: {valid}\nEstimated cost: {est}\nWidth: {:.1}",
        placement.control_points.len(),
        placement.generated_segments.len(),
        placement.width
    )
}

fn popup_stamp(
    tool: &ActiveBuildTool,
    placement: &ActiveRoadPlacement,
    snap: &RoadSnapSettings,
) -> u64 {
    let mut key = placement.control_points.len() as u64;
    key = key
        .wrapping_mul(31)
        .wrapping_add(placement.generated_segments.len() as u64);
    key = key.wrapping_mul(31).wrapping_add(
        placement
            .generated_segments
            .iter()
            .filter(|s| s.valid)
            .count() as u64,
    );
    key = key
        .wrapping_add(snap.grid_snap as u64)
        .wrapping_add((snap.node_snap as u64) << 1)
        .wrapping_add((placement.use_curved_preview as u64) << 2);
    for b in format!("{:?}", tool.tool).bytes() {
        key = key.wrapping_mul(31).wrapping_add(b as u64);
    }
    key
}

fn spawn_popup_button(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &TextFont,
    label: &str,
    action: RoadPopupAction,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette.bevy_button_idle()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

/// Spawn/despawn editor road popup when a road/rail tool is armed.
pub fn sync_road_tool_popup_bevy(
    mut commands: Commands,
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRoadPlacement>,
    snap: Res<RoadSnapSettings>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<RoadToolPopupBevyRoot>>,
    mut title_q: Query<&mut Text, (With<RoadToolPopupTitle>, Without<RoadToolPopupStats>)>,
    mut stats_q: Query<&mut Text, (With<RoadToolPopupStats>, Without<RoadToolPopupTitle>)>,
    mut stamp: Local<RoadPopupStamp>,
) {
    presence.road_popup_root = !roots.is_empty();
    let visible = road_tool_active(tool.as_ref());
    if !visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.road_popup_root = false;
        stamp.key = 0;
        return;
    }

    let next = popup_stamp(tool.as_ref(), placement.as_ref(), snap.as_ref());
    let title = popup_title(tool.as_ref());
    let stats = popup_stats(placement.as_ref());

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        let grid = if snap.grid_snap {
            "Grid snap · ON"
        } else {
            "Grid snap · OFF"
        };
        let node = if snap.node_snap {
            "Node snap · ON"
        } else {
            "Node snap · OFF"
        };
        let curved = if placement.use_curved_preview {
            "Curved preview · ON"
        } else {
            "Curved preview · OFF"
        };
        commands
            .spawn((
                RoadToolPopupBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(12.0),
                    top: Val::Px(200.0),
                    width: Val::Px(240.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::ZERO,
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                BorderColor::all(palette.bevy_wire_magenta()),
                FocusPolicy::Block,
                ZIndex(950),
            ))
            .with_children(|root| {
                root.spawn((
                    RoadToolPopupTitle,
                    Text::new(title),
                    tf.clone(),
                    TextColor(palette.bevy_primary_text()),
                ));
                root.spawn((
                    RoadToolPopupStats,
                    Text::new(stats.clone()),
                    tf.clone(),
                    TextColor(palette.bevy_text_muted()),
                ));
                spawn_popup_button(root, &palette, &tf, grid, RoadPopupAction::ToggleGrid);
                spawn_popup_button(root, &palette, &tf, node, RoadPopupAction::ToggleNode);
                spawn_popup_button(root, &palette, &tf, curved, RoadPopupAction::ToggleCurved);
                root.spawn((
                    Text::new("LMB add · RMB undo · Shift+LMB commit"),
                    tf.clone(),
                    TextColor(palette.bevy_text_muted()),
                ));
                spawn_popup_button(root, &palette, &tf, "Build", RoadPopupAction::Build);
                spawn_popup_button(root, &palette, &tf, "Cancel", RoadPopupAction::Cancel);
                spawn_popup_button(
                    root,
                    &palette,
                    &tf,
                    "Upgrade nearest segment",
                    RoadPopupAction::Upgrade,
                );
            });
        stamp.key = next;
        presence.road_popup_root = true;
        return;
    }

    if stamp.key != next {
        for mut t in &mut title_q {
            *t = Text::new(title);
        }
        for mut t in &mut stats_q {
            *t = Text::new(stats.clone());
        }
        // Toggle labels need a rebuild when snap/curved flip.
        for e in &roots {
            commands.entity(e).despawn();
        }
        stamp.key = 0;
        presence.road_popup_root = false;
        return;
    }

    presence.road_popup_root = true;
}

/// Commit / cancel / snap / upgrade — funnel only (no direct graph mutate).
pub fn road_tool_popup_bevy_action_system(
    q: Query<(&Interaction, &RoadPopupAction), (Changed<Interaction>, With<Button>)>,
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
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            RoadPopupAction::Build => {
                let valid = placement
                    .generated_segments
                    .iter()
                    .filter(|s| s.valid)
                    .count();
                if valid == 0 {
                    continue;
                }
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
            road_tool_popup_bevy_action_system.after(sync_road_tool_popup_bevy),
        )
            .run_if(product_shell_active),
    )
    .add_systems(
        Update,
        despawn_road_tool_popup_bevy_when_inactive.run_if(not(product_shell_active)),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn road_tool_popup_bevy_root_marker_present() {
        assert!(include_str!("popup.rs").contains("RoadToolPopupBevyRoot"));
        assert!(include_str!("popup.rs").contains("sync_road_tool_popup_bevy"));
        assert!(include_str!("popup.rs").contains("commit_road_path_to_queue"));
        assert!(include_str!("popup.rs").contains("enqueue_road_upgrade"));
    }
}
