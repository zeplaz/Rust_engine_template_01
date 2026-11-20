//! Road spline placement (construction stage P2).

mod ghost;
mod input;
mod intersections;
mod placement;
mod popup;
pub mod spline;

pub use input::{
    corridor_tool_kind, cursor_world_on_map, finish_corridor_path, finish_pending_corridor_path,
    road_path_input_system, sync_road_path_build_preview, sync_road_placement_width_from_tool,
    update_road_path_preview_system,
};
pub use intersections::{IntersectionId, IntersectionRegistry};
pub use placement::{ActiveRoadPlacement, RoadSegmentPreview};
pub use popup::{
    register_road_tool_popup_bevy_systems, RoadToolPopupBevyRoot, RoadToolPopupState,
};
