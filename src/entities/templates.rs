use bevy::prelude::*;

use crate::io::deserialzers::derezers::{
    deserialize_road_vehicle_config, load_all_json_files_from_dir,
};
use crate::road_vehicles::*;

#[derive(Debug, Default)]
pub struct RoadVehiclesResource {
    truck_templates: HashMap<String, Truck>,
    bus_templates: HashMap<String, Bus>,
}

fn create_truck_template(config: RoadVehicleConfig) -> (String, Truck) {
    let name = config.name;
    let entityinfo = EntityInfo {
        id: EntityId::new(),
        owner_id: None,
        position: Vec2::new(0.0, 0.0),
        entity_type: EntityType::RoadVehicle(RoadVehicleType::Truck),
    };
    let vehicle = RoadVehicle::new(entityinfo, config);
    let template_truck = Truck::new(vehicle);
    (name, template_truck)
}

fn create_bus_template(config: RoadVehicleConfig) -> (String, Bus) {
    let name = config.name;
    let entityinfo = EntityInfo {
        id: EntityId::new(),
        owner_id: None,
        position: Vec2::new(0.0, 0.0),
        entity_type: EntityType::RoadVehicle(RoadVehicleType::Bus),
    };
    let vehicle = RoadVehicle::new(entityinfo, config);
    let template_bus = Bus::new(vehicle);
    (name, template_bus)
}

fn load_vehicle_configs() -> Result<RoadVehicleConfigResource, Box<dyn Error>> {
    let mut resource = RoadVehicleConfigResource {
        templates: HashMap::new(),
    };

    let config_files_contents = load_all_json_files_from_dir("src/data")?;

    // Check if there's a file named "vehicle_configs"
    if let Some(vehicle_configs_str) = config_files_contents.get("vehicle_configs") {
        let configs: Vec<RoadVehicleConfig> = deserialize_road_vehicle_config(vehicle_configs_str)?;
        for config in configs {
            // Create a template Truck or Bus based on the configuration
            match config.entity_type {
                EntityType::Truck => {
                    let (name, truck) = create_truck_template(config);
                    truck_templates.insert(name, truck);
                }
                EntityType::Bus => {
                    let (name, bus) = create_bus_template(config);
                    bus_templates.insert(name, bus);
                }
                _ => {}
            }
        }
    }

    let resource = RoadVehiclesResource {
        truck_templates,
        bus_templates,
    };

    Ok(resource)
}
