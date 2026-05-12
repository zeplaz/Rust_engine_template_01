//! Terrain-level **fire & fuel ontology** (material classes, structure profiles, scenario hazards).
//! Runtime ECS components live under [`crate::systems::fire`]; CPU fire ticks consume these definitions.

pub mod fuel;
pub mod scenario_hazard;
pub mod structure_fire;
pub mod vegetation_fuel;

pub use fuel::{fuel_material_def, FuelMaterialDef, FuelMaterialKind};
pub use scenario_hazard::ScenarioHazardLayer;
pub use structure_fire::{
    ammo_dump_profile, fuel_depot_profile, lithium_battery_warehouse, StructureFireProfile,
};
pub use vegetation_fuel::{layer_fuel_mass, VegetationFuelLayer};
