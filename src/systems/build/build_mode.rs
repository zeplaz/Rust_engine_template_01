use crate::resources::ResourceType;


pub enum BuildModeType {
    Building,
    Rail,
    Road,
}

pub struct BuildMode {
    state: BuildModeState,
    mode_type: BuildModeType,
    simulation_mode: bool, // true if in simulation mode, false if in editor mode
}

#[derive(Debug, Clone)]
pub struct FoundationCost {
    pub resources: Vec<ResourceCost>,
    // other relevant fields
}

#[derive(Debug, Clone)]
pub struct ResourceCost {
    pub resource_type: ResourceType,
    pub amount: u32,
}
