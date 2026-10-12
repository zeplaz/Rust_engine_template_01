//! Bottom-left tool hint overlay (Round 2).

use bevy::prelude::*;

use crate::gui::InputBindings;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::path_feedback::ConstructionPathFeedback;

/// Sim session routes hints through tray Build peek — no LEFT_BOTTOM floater.
pub const TOOL_HINTS_DRAW_IN_SIM: bool = false;

