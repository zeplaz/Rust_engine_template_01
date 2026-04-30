use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleType {
    Road(RoadVehicleType),
    Ship(ShipType),
    Train,
    Military,
    Construction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipType {
    Passenger,
    Freight,
    Tanker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RoadVehicleType {
    Bus,
    Truck,
    Car,
    Cargo,
}

