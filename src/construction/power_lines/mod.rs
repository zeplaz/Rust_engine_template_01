//! Power line draw tool — path input, routing, commit to [`UtilityGraph`].

mod commit;
mod ghost;
mod input;
mod placement;
mod routing;

pub use commit::{
    commit_power_line_to_utility_graph, node_key_for_world, node_position_from_key,
    power_line_commit_witness_green,
};
pub use ghost::{draw_power_line_path_ghost_egui, power_line_ghost_preview_dashed_witness_green};
pub use input::{
    power_line_path_input_system, power_line_routing_mode_hotkey_system,
    sync_power_line_build_preview, sync_power_line_from_build_tool,
    sync_power_line_preview_overlay_system, update_power_line_path_preview_system,
};
pub use placement::{
    ActivePowerLinePlacement, PowerLineRoutingMode, PowerLineSegmentPreview,
};
pub use routing::{
    build_sample_chain, flatten_chain, orthogonal_chain_between, regenerate_power_line_segments,
    segment_preview_valid, snap_power_grid,
};

#[must_use]
pub fn power_line_draw_witness_green() -> bool {
    use bevy::prelude::Vec3;

    power_line_commit_witness_green()
        && power_line_ghost_preview_dashed_witness_green()
        && routing::segment_preview_valid(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            PowerLineRoutingMode::Orthogonal90,
        )
}
