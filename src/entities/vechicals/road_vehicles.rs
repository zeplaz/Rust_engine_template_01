use crate::entity::{EntityInfo, MilitaryCivilian, RoadVehicleType};
use crate::idgen::{EntityId, IdGenerator};
use crate::resource::ResourceType;
use crate::systems::damage::RoadVehicleDamageInfo;
use crate::traits::damage::{DamageInfoProvider, TakesDamage};

#[derive(Debug)]
pub enum RoadVehicleVisualStates {
    Full,
    Empty,
    Night,
    Midday,
}

#[derive(Debug, Deserialize)]
pub struct RoadVehicleConfig {
    pub name: String,
    pub vtype: RoadVehicleType,
    pub capacity: u32,
    pub mass: f32,
    pub max_speed: f32,
    pub military_civilian: Option<String>,
    pub textures: HashMap<String, HashMap<String, HashMap<String, TextureInfo>>>,
}

#[derive(Debug, Deserialize)]
pub struct TextureInfo {
    pub path: String,
    pub tiles: u32,
    pub emission: f32,
}

struct RoadVehicle {
    entityinfo: EntityInfo,
    pub name: String,
    pub vtype: RoadVehicleType,
    pub mass: f32,
    pub capacity: i32,
    pub max_speed: f32,
    pub velocity: Vec2,
    pub military_civilian: MilitaryCivilian,
    pub visual_state: vec<RoadVehicleVisualStates>,
    damage_info: RoadVehicleDamageInfo,
}

impl RoadVehicle {
    fn new(entityinfo: EntityInfo, rv_config: RoadVehicleConfig) -> RoadVehicle {
        // Extract visual states from the config
        let mut visual_state = vec![];
        for state in rv_config.textures.keys() {
            match state.as_str() {
                "full" => visual_state.push(RoadVehicleVisualStates::Full),
                "empty" => visual_state.push(RoadVehicleVisualStates::Empty),
                _ => {}
            }
        }

        // Iterate through the next level HashMap to find night and midday states
        for (_, inner_map) in rv_config.textures {
            for state in inner_map.keys() {
                match state.as_str() {
                    "night" => visual_state.push(RoadVehicleVisualStates::Night),
                    "midday" => visual_state.push(RoadVehicleVisualStates::Midday),
                    _ => {}
                }
            }
        }

        let military_civilian = match rv_config.military_civilian {
            Some(s) => MilitaryCivilian::from_str(&s).unwrap_or(MilitaryCivilian::Civilian),
            None => MilitaryCivilian::Civilian,
        };

        RoadVehicle {
            entityinfo,
            name: rv_config.name,
            vtype: rv_config.vtype,
            mass: rv_config.mass,
            capacity: rv_config.capacity,
            max_speed: rv_config.max_speed,
            velocity: Vec2::new(0.0, 0.0),
            military_civilian,
            visual_state,
            damage_info: RoadVehicleDamageInfo::default(),
        }
    }
}

pub struct Truck {
    vehicle: RoadVehicle,
    pub current_load: f32,
    pub current_resource_type: Option<ResourceType>,
    pub whitelist: HashSet<ResourceType>,
    pub blacklist: HashSet<ResourceType>,
    // other fields
}

#[derive(Debug)]
pub enum AddToCapacityStatus {
    Success,
    ResourceTypeNotAllowed,
    ResourceTypeMismatch,
}

impl Truck {
    pub fn new(veh: RoadVehicle) -> Truck {
        Truck {
            vehicle: veh,
            current_load: 0.0,
            current_resource_type: None,
            whitelist: HashSet::new(),
            blacklist: HashSet::new(),
        }
    }
    // Function to empty the truck
    pub fn empty(&mut self) {
        self.current_load = 0.0;
        self.current_resource_type = None;
    }

    pub fn is_empty(&self) -> bool {
        self.current_load < 0.01
    }

    // Function to set the current resource type
    pub fn set_current_resource_type(&mut self, resource_type: ResourceType) {
        self.current_resource_type = Some(resource_type);
    }

    // Function to add a resource type to the whitelist
    pub fn add_to_whitelist(&mut self, resource_type: ResourceType) {
        self.whitelist.insert(resource_type);
    }

    // Function to add a resource type to the blacklist
    pub fn add_to_blacklist(&mut self, resource_type: ResourceType) {
        self.blacklist.insert(resource_type);
    }

    pub fn add_to_blacklist(&mut self, resource_type: ResourceType) {
        self.blacklist.insert(resource_type);
    }

    pub fn remove_from_blacklist(&mut self, resource_type: &ResourceType) {
        self.blacklist.remove(resource_type);
    }

    // Function to check if the truck can accept the given resource type
    pub fn can_accept_resource(&self, resource_type: &ResourceType) -> bool {
        if self.blacklist.contains(resource_type) {
            return false;
        }
        if self.whitelist.is_empty() || self.whitelist.contains(resource_type) {
            return true;
        }
        false
    }

    pub fn add_to_capacity(
        &mut self,
        amount: f32,
        resource_type: ResourceType,
    ) -> AddToCapacityStatus {
        if !self.can_accept_resource(&resource_type) {
            return AddToCapacityStatus::ResourceTypeNotAllowed;
        }

        if self.is_empty() || self.current_resource_type.is_none() {
            self.current_resource_type = Some(resource_type);
        } else if let Some(current_res_type) = &self.current_resource_type {
            if *current_res_type != resource_type {
                return AddToCapacityStatus::ResourceTypeMismatch;
            }
        }

        self.current_load += amount;
        AddToCapacityStatus::Success
    }
}

struct Bus {
    vehicle: RoadVehicle,
    passengers: u32,
}

impl Bus {
    pub fn new(veh: RoadVehicle) -> Bus {
        Bus {
            vehicle: veh,
            passengers: 0,
        }
    }
}
