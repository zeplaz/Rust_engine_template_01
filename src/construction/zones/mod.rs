//! Zone paint tool (BUILD-P4-03): drag tiles → pending queue, no immediate site spawn.

mod commit;
mod ghost;

pub use ghost::zone_fill;
mod input;
mod placement;

#[cfg(test)]
pub use commit::commit_painted_zones_to_pending;
pub use commit::spawn_zone_at_tile;
pub use ghost::draw_zone_paint_ghost_egui;
pub use input::{sync_active_zone_from_tool, zone_paint_input_system};
pub use placement::ActiveZonePaint;
