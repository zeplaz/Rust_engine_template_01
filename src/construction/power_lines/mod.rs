//! Power line draw tool — path input, routing, commit to [`UtilityGraph`].

mod cut_input;
mod damage;
mod commit;
mod ghost;
mod input;
mod placement;
mod repair;
mod routing;

pub use cut_input::{
    power_line_cut_input_wired, power_line_demolish_cut_system, PowerLineCutToast,
};
pub use damage::{
    cut_power_line_segment, damage_power_line_segment, power_damage_segment_witness_green,
    preview_island_offline_from_cut, register_power_segments_from_graph_system,
    sync_power_damage_to_presentation_system, PowerLineDamageBook, PowerLineSegmentHealth,
};
pub use repair::{
    power_repair_queue_witness_green, tick_power_repair_queue_system, PowerRepairJob,
    PowerRepairQueue, POWER_REPAIR_PARTS_PER_SEGMENT, POWER_REPAIR_TICKS_PER_JOB,
};

pub use commit::{
    commit_power_line_to_utility_graph, node_key_for_world, power_line_commit_witness_green,
};
pub use ghost::{draw_power_line_path_ghost_egui, power_line_ghost_preview_dashed_witness_green};
pub use input::{
    power_line_path_input_system, power_line_routing_mode_hotkey_system,
    sync_power_line_build_preview, sync_power_line_from_build_tool,
    sync_power_line_preview_overlay_system, update_power_line_path_preview_system,
};
pub use placement::{ActivePowerLinePlacement, PowerLineRoutingMode};

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
