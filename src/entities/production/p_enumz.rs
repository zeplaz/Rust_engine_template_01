

// power 

#[derive(Debug)]
pub enum PowerDistributionType {
    ThreePhaseHeavyIndustrial,
    ThreePhaseMediumIndustrial,
    OnePhaseLightIndustrial,
    ThreePhaseResidential,
    OnePhaseResidential,
    ThreePhaseLongDistance,
    OnephaseLongDistance,
    Mixed, // for substations that can handle both industrial and residential
}


//production



// resources

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceMechanism {
    Flow,
    Storable,
}

impl ResourceType {
    pub fn mechanism(&self) -> ResourceMechanism {
        match *self {
            ResourceType::Labour | ResourceType::Energy | ResourceType::Knowledge => {
                ResourceMechanism::Flow
            }
            _ => ResourceMechanism::Storable,
        }
    }
}


#[derive(Debug)]
pub enum ConcreateType { 
    Limecrete,
    Portland,
    Geopolymer, 
    Gypsum, 
}

#[derive(Debug)]
pub enum ResourceType {
    Labour,
    Water,
    Food,
    Wood,
    Steal,
    Concreate(ConcreateType),
    Ammunition,
    Fertilizer,
    Chemicals,
    RareEarth,
    Electronics,
    Metal,
    Oil,
    Paper,
    Electrcity,
    Coal,
    WarSupply,
    Knowledge,
    Fuel,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ResourceCategory {
    RawMaterial,   // For resources that are typically harvested or mined
    ProcessedMaterial,  // For resources that are typically crafted or manufactured
    Energy,  // For resources related to power or fuel
    Military,  // For resources with a military application
    Human,  // For resources related to labour or reserch etc
    Essentials, //food, water and wood i guess.
}

pub enum ResourceFilter {
    Type(ResourceType),
    Category(ResourceCategory),
}

#[derive(Debug)]
pub enum CargoType {
    Fluid,
    Gas,
    People,
    DryGoods,
}

#[derive(Debug)]
pub enum AddToCapacityStatus {
    Success,
    ResourceTypeNotAllowed,
    ResourceTypeMismatch,
}