// enties serlizers for configs.

#[derive(Debug, Deserialize)]
pub struct RoadVehicleConfig {
    pub name: String,
    pub vtype: RoadVehicleType,
}