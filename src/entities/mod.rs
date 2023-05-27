pub mod buildings;
pub mod entity;
pub mod rails;
pub mod road_vehicles;
pub mod roads;
pub mod templates;

pub use entity::EntityInfo;
pub use entity::MilitaryCivilian;
pub use road_vehicles::{Bus, RoadVehicle, RoadVehicleConfig, Truck};
pub use templates::RoadVehiclesResource;
