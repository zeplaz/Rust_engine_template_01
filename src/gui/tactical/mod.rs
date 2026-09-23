//! Tactical map RTT + camera authority.
//! RGR-P5-002 mechanical move: `sim_map_rtt` (RTT pipeline) + `map_camera` (pose authority).
//! Canonical paths: `crate::gui::tactical::sim_map_rtt` / `crate::gui::tactical::map_camera`.
//! Root re-exports (`crate::gui::MainWorldCamera`, etc.) remain on `gui` for callers that prefer flat imports.

pub mod map_camera;
pub mod sim_map_rtt;
