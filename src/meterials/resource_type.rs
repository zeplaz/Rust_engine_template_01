

#[derive(Debug,PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    Labour,
    Water,
    Food,
    Ammunition,
    Fertilizer,
    Chemicals,
    RareEarth,
    Electronics,
    Metal,
    Oil,
    Wood,
    Energy,
    Coal,
    WarSupply,
    Knowledge,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceCategory {
    Flow,
    Storable,
}

impl ResourceType {
    pub fn category(&self) -> ResourceCategory {
        match *self {
            ResourceType::Labour | ResourceType::Energy | ResourceType::Knowledge => {
                ResourceCategory::Flow
            }
            _ => ResourceCategory::Storable,
        }
    }
}
