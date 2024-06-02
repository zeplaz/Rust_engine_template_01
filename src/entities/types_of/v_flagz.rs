#[derive(Debug)]
pub enum VehicleType {
    Road(RoadVehicleType),
    Ship(ShipType),
    Train,
    Military,
    Construction,
}



#[derive(Debug)]
pub enum ShipType{
    Passenger,
    Freight,
    Tanker,
}

#[derive(Debug)]
pub enum RoadVehicleType {
    Bus,
    Truck,
    Car,
    Cargo,
}

