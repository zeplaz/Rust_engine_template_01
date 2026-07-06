//! Tactical map RTT + camera authority.
//! RGR-P5-002 mechanical move: sim_map_rtt (RTT pipeline) + map_camera (pose authority).
//! Old `crate::gui::sim_map_rtt::*` / `crate::gui::map_camera::*` paths kept alive as
//! re-export shims in `src/gui/mod.rs` — do not remove until all call sites are migrated.

pub mod map_camera;
pub mod sim_map_rtt;
