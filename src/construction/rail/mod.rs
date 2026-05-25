//! Rail placement lane — spline, grade, and curve authority (not a road clone).

mod commit;
mod ghost;
mod input;
mod junction;
mod pathing;
mod placement;
mod validation;

pub use ghost::draw_rail_path_ghost_egui;
pub use input::{
    rail_path_input_system, sync_rail_path_build_preview, sync_rail_placement_from_tool,
    update_rail_path_preview_system,
};
pub use placement::ActiveRailPlacement;
pub use validation::validate_rail_segment;
pub use junction::RailJunctionAuthority;
