//! Terrain-level **fire & fuel ontology** (material classes, structure profiles, scenario hazards).
//! Runtime ECS components live under [`crate::systems::fire`]; CPU fire ticks consume these definitions.

pub mod fuel;
pub mod fuel_layer;
pub mod scenario_hazard;
pub mod structure_fire;
pub mod vegetation_fuel;

pub use fuel::{fuel_material_def, FuelMaterialDef, FuelMaterialKind};
pub use fuel_layer::FuelLayer;
pub use scenario_hazard::ScenarioHazardLayer;
pub use structure_fire::{
    ammo_dump_profile, concrete_structure_profile, fuel_depot_profile, lithium_battery_warehouse,
    profile_for_site_archetype, steel_structure_profile, unit_fire_susceptibility,
    wood_structure_profile, StructureFireProfile, TRUCK_FIRE_SUSCEPTIBILITY,
};
pub use vegetation_fuel::{layer_fuel_mass, VegetationFuelLayer};
