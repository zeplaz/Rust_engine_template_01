//! **Operational build UX** — planning strip, ghost cursor, validation, overlay toggles (P2-F).
//!
//! See `prompts/guides/ui_boundary_guide_v1.md`: Bevy shell holds mode state; egui panels may mirror it.

mod build_commit;
mod build_ghost;
mod build_overlays;
mod build_strip;
mod build_validation;

pub use build_commit::queue_commit_construction_site;
pub use build_ghost::GhostBuildCursor;
pub use build_overlays::BuildOverlayVisibility;
pub use build_strip::{BuildStripState, ToolContext};
pub use build_validation::validate_planned_site_stubs;

use bevy::prelude::*;

use crate::gui::input_bindings::InputBindings;

pub fn cycle_build_planning_tool_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut strip: ResMut<BuildStripState>,
) {
    if !keyboard.just_pressed(bindings.cycle_build_planning_tool) {
        return;
    }
    strip.active = strip.active.next();
}

pub struct BuildPlanningPlugin;

impl Plugin for BuildPlanningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildStripState>()
            .init_resource::<BuildOverlayVisibility>()
            .add_systems(Update, cycle_build_planning_tool_system);
    }
}
