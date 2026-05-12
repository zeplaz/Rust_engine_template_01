//! **Operational build UX** — planning strip, ghost cursor, validation, overlay toggles (P2-F).
//!
//! See `prompts/guides/ui_boundary_guide_v1.md`: Bevy shell holds mode state; egui panels may mirror it.

mod build_commit;
mod build_ghost;
mod build_interaction;
mod build_overlays;
mod build_state;
mod build_strip;
mod build_validation;

pub use build_commit::queue_commit_construction_site;
pub use build_ghost::GhostBuildCursor;
pub use build_interaction::{
    build_confirm_site_system, build_pick_ghost_tile_system, build_refresh_placement_validation_system,
    build_sync_ghost_cursor_entity_system,
};
pub use build_overlays::BuildOverlayVisibility;
pub use build_state::{
    BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementPreview,
};
pub use build_strip::{BuildStripState, ToolContext};
pub use build_validation::validate_planned_site_stubs;

use bevy::prelude::*;

use crate::gui::input_bindings::InputBindings;
use crate::gui::ui_gates::in_simulation_or_editor;

pub fn cycle_build_planning_tool_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut strip: ResMut<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
) {
    if !keyboard.just_pressed(bindings.cycle_build_planning_tool) {
        return;
    }
    strip.active = strip.active.next();
    ghost.footprint = strip.active.footprint_for_tool();
    if strip.active == ToolContext::None {
        ghost.origin = None;
    }
}

pub struct BuildPlanningPlugin;

impl Plugin for BuildPlanningPlugin {
    fn build(&self, app: &mut App) {
        let owner = app.world_mut().spawn_empty().id();

        app.insert_resource(BuildCommandActor(owner))
            .init_resource::<BuildStripState>()
            .init_resource::<BuildGhostState>()
            .init_resource::<BuildPlacementPreview>()
            .init_resource::<BuildOverlayVisibility>()
            .add_systems(Update, cycle_build_planning_tool_system.run_if(in_simulation_or_editor))
            .add_systems(
                Update,
                (
                    build_pick_ghost_tile_system,
                    build_refresh_placement_validation_system,
                    build_confirm_site_system,
                    build_sync_ghost_cursor_entity_system,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            );
    }
}
