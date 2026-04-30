// Input/Output systems.
//
// `templates.rs` retired (2026-04-26): the file referenced removed modules
// (`crate::io::deserialzers`, `crate::road_vehicles`) and an undefined
// `RoadVehicleConfigResource`. Active loading paths now live in
// `crate::io::serialization::deserializers`.
mod mouse;
pub mod serialization;

pub use mouse::*;
