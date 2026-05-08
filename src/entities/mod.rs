pub mod prelude;

// Entity basics
mod entity;
pub mod components;
pub mod damages;
pub mod types;
pub mod types_aliases;
pub mod types_of;

// Entity categories
pub mod production;
pub mod structure;
pub mod vehicles;

// Public exports (no `prelude::*` merge — avoids ambiguous `components` / `states` / `ConcreteType` paths)
pub use components::*;
pub use types::*;
pub use vehicles::config::{MilitaryCivilian, RoadVehicleConfig};