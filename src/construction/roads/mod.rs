//! Road spline placement (construction stage P2).

mod commit;
mod ghost;
mod input;
mod intersections;
mod pathing;
mod placement;
mod popup;
pub mod spline;

pub use intersections::{IntersectionId, IntersectionRegistry};
pub use ghost::draw_road_path_ghost_egui;
pub use input::{
    cursor_world_on_map, road_path_input_system, sync_road_path_build_preview,
    sync_road_placement_width_from_tool, update_road_path_preview_system,
};
#[cfg(test)]
pub use pathing::regenerate_road_segments;
pub use placement::{ActiveRoadPlacement, RoadSegmentPreview};
pub use commit::commit_road_path_to_queue;
pub use popup::{draw_road_tool_popup_egui, RoadToolPopupState};
