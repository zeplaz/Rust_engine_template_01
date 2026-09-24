//! Build mode state machine (presentation / input phase — parallel construction stage).

use bevy::prelude::*;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::build_state::BuildGhostState;
use super::build_strip::ToolContext;
use super::power_lines::ActivePowerLinePlacement;
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
    power: Res<ActivePowerLinePlacement>,
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
            BuildTool::PowerLine(_) if !power.control_points.is_empty() => BuildMode::GhostPreview,
            BuildTool::Road(_) => BuildMode::RoadPlacement,
            BuildTool::Rail(_) => BuildMode::RailPlacement,
            BuildTool::PowerLine(_) => BuildMode::RoadPlacement,
            BuildTool::Building(_) | BuildTool::Defense(_) => {
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

/// GUI-ESC-001 — tracks whether the last Esc only cleared ghosts (tool still armed).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BuildEscCancelLatch {
    /// True after an Esc that cleared ghosts but left the tool armed (`keep_tool_after_commit`).
    pub awaiting_second_esc_to_drop_tool: bool,
}

fn clear_placement_ephemera(
    ghost: &mut BuildGhostState,
    path: &mut ActiveRoadPlacement,
    power: &mut ActivePowerLinePlacement,
    rail: &mut ActiveRailPlacement,
    zone: &mut ActiveZonePaint,
    pending: &mut super::pending_construction::PendingConstructionQueue,
    staged: &mut super::staged_ghost_panel::StagedPlacementBook,
    staging_mode: &mut super::staged_ghost_panel::StagedPlacementMode,
) {
    ghost.origin = None;
    ghost.drag_active = false;
    ghost.placement_mode = super::build_state::BuildPlacementMode::Place;
    ghost.locked_this_frame = false;
    path.control_points.clear();
    path.generated_segments.clear();
    power.clear_path();
    rail.control_points.clear();
    rail.generated_segments.clear();
    zone.clear();
    pending.clear();
    staged.rows.clear();
    staging_mode.enabled = false;
}

fn drop_active_build_tool(
    tool: &mut ActiveBuildTool,
    strip: &mut super::BuildStripState,
    zone: &mut ActiveZonePaint,
    latch: &mut BuildEscCancelLatch,
) {
    tool.tool = BuildTool::None;
    tool.close_submenus();
    tool.clear_building_intent();
    strip.active = ToolContext::None;
    zone.zone = None;
    latch.awaiting_second_esc_to_drop_tool = false;
}

/// Pure Esc cascade step (GUI-ESC-001) — shared by system + lib tests.
#[must_use]
pub fn apply_build_escape_cancel(
    session: &ActiveToolSession,
    latch: &mut BuildEscCancelLatch,
    tool: &mut ActiveBuildTool,
    strip: &mut super::BuildStripState,
    ghost: &mut BuildGhostState,
    path: &mut ActiveRoadPlacement,
    power: &mut ActivePowerLinePlacement,
    rail: &mut ActiveRailPlacement,
    zone: &mut ActiveZonePaint,
    pending: &mut super::pending_construction::PendingConstructionQueue,
    staged: &mut super::staged_ghost_panel::StagedPlacementBook,
    staging_mode: &mut super::staged_ghost_panel::StagedPlacementMode,
) {
    let building_armed = tool.tool.uses_two_click_place();
    // Design A4: Esc from Adjust → Preview (keep tool); further Esc uses cascade below.
    if building_armed && ghost.placement_mode == super::build_state::BuildPlacementMode::Adjust {
        ghost.unlock_to_preview();
        latch.awaiting_second_esc_to_drop_tool = true;
        return;
    }
    let tool_armed = tool.tool != BuildTool::None || strip.active != ToolContext::None;
    let had_ephemera = ghost.origin.is_some()
        || ghost.drag_active
        || !path.control_points.is_empty()
        || !power.control_points.is_empty()
        || !rail.control_points.is_empty()
        || !zone.painted.is_empty()
        || !pending.entries.is_empty()
        || !staged.rows.is_empty();

    clear_placement_ephemera(
        ghost, path, power, rail, zone, pending, staged, staging_mode,
    );

    let drop_tool = !tool_armed
        || building_armed
        || !session.keep_tool_after_commit
        || latch.awaiting_second_esc_to_drop_tool
        || !had_ephemera;

    if drop_tool && tool_armed {
        drop_active_build_tool(tool, strip, zone, latch);
    } else if tool_armed {
        latch.awaiting_second_esc_to_drop_tool = true;
    } else {
        latch.awaiting_second_esc_to_drop_tool = false;
    }
}

/// Esc cascade (GUI-ESC-001): clear ghosts/queue first; second Esc (or first when buildings /
/// `!keep_tool`) drops the tool to `None`.
pub fn build_escape_cancel_system(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<ActiveToolSession>,
    mut latch: ResMut<BuildEscCancelLatch>,
    mut tool: ResMut<ActiveBuildTool>,
    mut strip: ResMut<super::BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    mut path: ResMut<ActiveRoadPlacement>,
    mut power: ResMut<ActivePowerLinePlacement>,
    mut rail: ResMut<ActiveRailPlacement>,
    mut zone: ResMut<ActiveZonePaint>,
    mut pending: ResMut<super::pending_construction::PendingConstructionQueue>,
    mut staged: ResMut<super::staged_ghost_panel::StagedPlacementBook>,
    mut staging_mode: ResMut<super::staged_ghost_panel::StagedPlacementMode>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    let _ = apply_build_escape_cancel(
        session.as_ref(),
        latch.as_mut(),
        tool.as_mut(),
        strip.as_mut(),
        ghost.as_mut(),
        path.as_mut(),
        power.as_mut(),
        rail.as_mut(),
        zone.as_mut(),
        pending.as_mut(),
        staged.as_mut(),
        staging_mode.as_mut(),
    );
}

#[cfg(test)]
mod gui_esc_001_tests {
    use super::*;
    use crate::construction::build_strip::{BuildStripState, ToolContext};
    use crate::construction::build_tool_authority::{BuildingArchetypeId, BuildTool};
    use crate::construction::pending_construction::PendingConstructionQueue;
    use crate::construction::staged_ghost_panel::{StagedPlacementBook, StagedPlacementMode};

    #[test]
    fn second_esc_drops_keep_tool_road_session() {
        let session = ActiveToolSession {
            keep_tool_after_commit: true,
            ..Default::default()
        };
        let mut latch = BuildEscCancelLatch::default();
        let mut tool = ActiveBuildTool {
            tool: BuildTool::Road(crate::construction::RoadType::Street),
            ..Default::default()
        };
        let mut strip = BuildStripState {
            active: ToolContext::Roads,
            ..Default::default()
        };
        let mut ghost = BuildGhostState::default();
        let mut path = ActiveRoadPlacement {
            control_points: vec![bevy::math::Vec3::ZERO],
            ..Default::default()
        };
        let mut power = ActivePowerLinePlacement::default();
        let mut rail = ActiveRailPlacement::default();
        let mut zone = ActiveZonePaint::default();
        let mut pending = PendingConstructionQueue::default();
        let mut staged = StagedPlacementBook::default();
        let mut staging_mode = StagedPlacementMode::default();

        let _ = apply_build_escape_cancel(
            &session,
            &mut latch,
            &mut tool,
            &mut strip,
            &mut ghost,
            &mut path,
            &mut power,
            &mut rail,
            &mut zone,
            &mut pending,
            &mut staged,
            &mut staging_mode,
        );
        assert_ne!(tool.tool, BuildTool::None, "first Esc keeps road tool");
        assert!(latch.awaiting_second_esc_to_drop_tool);
        assert!(path.control_points.is_empty());

        let _ = apply_build_escape_cancel(
            &session,
            &mut latch,
            &mut tool,
            &mut strip,
            &mut ghost,
            &mut path,
            &mut power,
            &mut rail,
            &mut zone,
            &mut pending,
            &mut staged,
            &mut staging_mode,
        );
        assert_eq!(tool.tool, BuildTool::None, "second Esc must drop tool");
        assert!(!latch.awaiting_second_esc_to_drop_tool);
    }

    #[test]
    fn building_esc_drops_tool_first_press() {
        let session = ActiveToolSession::default();
        let mut latch = BuildEscCancelLatch::default();
        let mut tool = ActiveBuildTool {
            tool: BuildTool::Building(BuildingArchetypeId::Factory),
            ..Default::default()
        };
        let mut strip = BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        };
        let mut ghost = BuildGhostState {
            origin: Some(crate::strategic::BuildSiteTile { x: 1, z: 1 }),
            ..Default::default()
        };
        let mut path = ActiveRoadPlacement::default();
        let mut power = ActivePowerLinePlacement::default();
        let mut rail = ActiveRailPlacement::default();
        let mut zone = ActiveZonePaint::default();
        let mut pending = PendingConstructionQueue::default();
        let mut staged = StagedPlacementBook::default();
        let mut staging_mode = StagedPlacementMode::default();

        let _ = apply_build_escape_cancel(
            &session,
            &mut latch,
            &mut tool,
            &mut strip,
            &mut ghost,
            &mut path,
            &mut power,
            &mut rail,
            &mut zone,
            &mut pending,
            &mut staged,
            &mut staging_mode,
        );
        assert_eq!(tool.tool, BuildTool::None);
        assert!(ghost.origin.is_none());
    }

    #[test]
    fn building_esc_from_adjust_unlocks_preview() {
        use crate::construction::build_state::BuildPlacementMode;

        let session = ActiveToolSession::default();
        let mut latch = BuildEscCancelLatch::default();
        let mut tool = ActiveBuildTool {
            tool: BuildTool::Building(BuildingArchetypeId::Factory),
            ..Default::default()
        };
        let mut strip = BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        };
        let mut ghost = BuildGhostState {
            origin: Some(crate::strategic::BuildSiteTile { x: 1, z: 1 }),
            placement_mode: BuildPlacementMode::Adjust,
            ..Default::default()
        };
        let mut path = ActiveRoadPlacement::default();
        let mut power = ActivePowerLinePlacement::default();
        let mut rail = ActiveRailPlacement::default();
        let mut zone = ActiveZonePaint::default();
        let mut pending = PendingConstructionQueue::default();
        let mut staged = StagedPlacementBook::default();
        let mut staging_mode = StagedPlacementMode::default();

        let _ = apply_build_escape_cancel(
            &session,
            &mut latch,
            &mut tool,
            &mut strip,
            &mut ghost,
            &mut path,
            &mut power,
            &mut rail,
            &mut zone,
            &mut pending,
            &mut staged,
            &mut staging_mode,
        );
        assert_ne!(tool.tool, BuildTool::None, "first Esc unlocks Adjust only");
        assert_eq!(ghost.placement_mode, BuildPlacementMode::Place);
        assert!(ghost.origin.is_none());
        assert!(latch.awaiting_second_esc_to_drop_tool);
    }
}
