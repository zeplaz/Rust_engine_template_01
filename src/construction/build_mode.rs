//! Build mode state machine (presentation / input phase — parallel construction stage).

use bevy::prelude::*;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::build_state::BuildGhostState;
use super::build_strip::ToolContext;
use super::rail::ActiveRailPlacement;
use super::roads::ActiveRoadPlacement;
use super::sessions::ActiveToolSession;
use super::zones::ActiveZonePaint;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BuildMode {
    #[default]
    None,
    ZoneMenu,
    ZonePaint,
    PlaceBuilding,
    RoadPlacement,
    RailPlacement,
    GhostPreview,
    ConfirmPlacement,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BuildModeState {
    pub mode: BuildMode,
}

pub fn sync_build_mode_state(
    tool: Res<ActiveBuildTool>,
    strip: Res<super::BuildStripState>,
    ghost: Res<BuildGhostState>,
    path: Res<ActiveRoadPlacement>,
    zone: Res<ActiveZonePaint>,
    mut mode: ResMut<BuildModeState>,
) {
    mode.mode = if tool.residential_menu_open
        || tool.commercial_menu_open
        || tool.industrial_menu_open
        || tool.utilities_menu_open
    {
        BuildMode::ZoneMenu
    } else {
        match tool.tool {
            BuildTool::Zone(_) if !zone.painted.is_empty() => BuildMode::ZonePaint,
            BuildTool::None if strip.active == ToolContext::None => BuildMode::None,
            BuildTool::Road(_) | BuildTool::Rail(_) if !path.control_points.is_empty() => {
                BuildMode::GhostPreview
            }
            BuildTool::Road(_) => BuildMode::RoadPlacement,
            BuildTool::Rail(_) => BuildMode::RailPlacement,
            BuildTool::Building(_) => {
                if ghost.origin.is_some() {
                    BuildMode::GhostPreview
                } else {
                    BuildMode::PlaceBuilding
                }
            }
            BuildTool::Demolish => BuildMode::PlaceBuilding,
            BuildTool::None => BuildMode::None,
            BuildTool::Zone(_) => BuildMode::PlaceBuilding,
        }
    };
}

pub fn build_escape_cancel_system(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<ActiveToolSession>,
    mut tool: ResMut<ActiveBuildTool>,
    mut strip: ResMut<super::BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    mut path: ResMut<ActiveRoadPlacement>,
    mut rail: ResMut<ActiveRailPlacement>,
    mut zone: ResMut<ActiveZonePaint>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    ghost.origin = None;
    ghost.drag_active = false;
    path.control_points.clear();
    path.generated_segments.clear();
    rail.control_points.clear();
    rail.generated_segments.clear();
    zone.clear();
    if !session.keep_tool_after_commit {
        tool.tool = BuildTool::None;
        tool.close_submenus();
        tool.clear_building_intent();
        strip.active = ToolContext::None;
        zone.zone = None;
    }
}
