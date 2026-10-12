//! Ghost footprint validity hint over the simulation map viewport.

use bevy::prelude::*;

use crate::construction::map_egui_projection::{tile_screen_extent, world_to_sim_map_screen};
use crate::construction::tile_visual::ConstructionTileVisualSettings;
use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;

use super::build_confidence::{confidence_from_validation, BuildConfidence};
use super::build_state::{BuildGhostState, BuildPlacementPreview};
use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::ActiveBuildTool;
use super::ghost_visual::{footprint_invalid_color, footprint_risky_color, footprint_valid_color};

pub fn build_footprint_validity_overlay_egui() -> Result { Ok(()) }
