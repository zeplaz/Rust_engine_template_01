//! **COD-SIM-HUD-POPUP-MIGRATE-001** — rail-anchored road tool sheet (sim).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::{
    commit_road_path_to_queue, cursor_world_on_map, enqueue_road_upgrade, ActiveBuildTool,
    ActiveRoadPlacement, ActiveToolSession, BuildStripState, BuildTool, ConstructionPlanQueue,
    ExecutedRoadNetwork, RailType, RoadType, RoadSnapSettings, RoadToolPopupState, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport, UiPalette};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::sim_hud_copy::{ROAD_SHEET_BUILD, ROAD_SHEET_CANCEL, ROAD_SHEET_HINT_INPUT, ROAD_SHEET_UPGRADE};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const ROAD_TOOL_SHEET_W_PX: f32 = 280.0;
/// Logical height of the Bevy road/rail sheet, used by the pointer gate.
pub const ROAD_TOOL_SHEET_BLOCK_H_PX: f32 = 240.0;

/// Sim uses rail-anchored sheet; editor keeps legacy floating window.
pub const ROAD_POPUP_FLOATING_IN_SIM: bool = false;

#[derive(Resource, Debug, Clone, Default)]
pub struct SimRoadToolSheetState {
    pub open: bool,
}

impl SimRoadToolSheetState {
    pub fn sync_from_strip(&mut self, strip: &BuildStripState) {
        self.open = matches!(strip.active, ToolContext::Roads | ToolContext::Rail);
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[derive(SystemParam)]
pub struct SimRoadToolSheetDrawParams<'w> {
    pub strip: Res<'w, BuildStripState>,
    pub palette: Res<'w, UiPalette>,
    pub tool: Res<'w, ActiveBuildTool>,
    pub road_sheet: ResMut<'w, SimRoadToolSheetState>,
    pub placement: ResMut<'w, ActiveRoadPlacement>,
    pub popup: ResMut<'w, RoadToolPopupState>,
    pub queue: ResMut<'w, ConstructionPlanQueue>,
    pub session: ResMut<'w, ActiveToolSession>,
    pub snap: ResMut<'w, RoadSnapSettings>,
    pub roads: Res<'w, ExecutedRoadNetwork>,
    pub params: Res<'w, WorldGenParams>,
    pub desired: Res<'w, MapCameraDesiredRes>,
    pub map_vp: Res<'w, SimulationMapViewport>,
}

#[must_use]
pub fn road_tool_sheet_anchor(strip: &BuildStripState, left_stack_collapsed: bool) -> bevy::prelude::Vec2 {
    let slot = match strip.active {
        ToolContext::Rail => ToolContext::Rail,
        _ => ToolContext::Roads,
    };
    super::simulation_shell_phase2::build_rail_slot_anchor_xy(slot, left_stack_collapsed)
}

