//! LEGACY SHIM — prefer `crate::entities::types` and submodules.
//!
//! Kept so older `use crate::entities::types_of::...` paths keep compiling during migration.

pub use crate::entities::types::e_flagz::{
    CarryingState, ConstructionStates, EmergencyType, EntityMenuState, EntityType, MalfunctionType,
    OperationalStatus, SegmentMembership,
};
pub use crate::entities::types::p_enumz::{ResourceFilter, ResourceType};
pub use crate::entities::types::s_flagz::{
    BuildingType, ConcreteType, FactoryType, MineType, RoadSurfaceType,
};
pub use crate::entities::types::v_flagz::{RoadVehicleType, ShipType, VehicleType};
