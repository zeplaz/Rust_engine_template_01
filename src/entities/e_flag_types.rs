use std::str::FromStr;
use serde::Deserialize;

#[derive(Debug)]
pub enum EntityType {
    Building(BuildingType),
    Tree,
    Tile,
    Vehicle(VehicleType),
    Train,
    Rail,
    Road(RoadSurfaceType),
    Resource(ResourceType)
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilitaryCivilian {
    Civilian,
    Military,
}

impl FromStr for MilitaryCivilian {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "civilian" | "Civilian" => Ok(MilitaryCivilian::Civilian),
            "military" | "Military" => Ok(MilitaryCivilian::Military),
            _ => Err(()),
        }
    }
}

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
pub enum BuildingType {
    Residencey(ResidenceType),
    Depanneur,
    Burocracy,
    FeildDepo,
    Warehouse,
    Factory(FactoryType),
    Mine(MineType),
    FuelStation,
    PowerPlant,
    Trasformer,
    PoweLine,
    ReserchCenter,
    Farm,
    RailDepot,
    TrainStation,
}

#[derive(Debug)]
pub enum PowerDistributionType {
    HeavyIndustrial,
    LightIndustrial,
    MediumIndustrial,
    Residential,
    LongDistance,
    Mixed, // for substations that can handle both industrial and residential
}

#[derive(Debug)]
pub enum ApartmentUnitType {
    Studio,
    Single,
    Double,
    ThreeBedrooms,
    Family,
    Luxury,
}

impl ApartmentUnitType {
    fn total_rooms(&self) -> f32 {
        match *self {
            ApartmentUnitType::Studio => 1.5,
            ApartmentUnitType::Single => 3.5,
            ApartmentUnitType::Double => 4.5,
            ApartmentUnitType::ThreeBedrooms => 7.5,
            ApartmentUnitType::Family => 9.0,
            ApartmentUnitType::Luxury => 10.0,
        }
    }
}

#[derive(Debug)]
pub enum ApartmentsType {
    Highrise { units_available: Vec<ApartmentUnitType> },
    Duplex { units_available: Vec<ApartmentUnitType> },
    Quadplex { units_available: Vec<ApartmentUnitType> },
}

impl ApartmentsType {
    fn total_rooms(&self) -> f32 {
        match *self {
            ApartmentsType::Highrise { ref units_available } => units_available
                .iter()
                .map(|x| x.total_rooms())
                .sum::<f32>(),
            ApartmentsType::Duplex { ref units_available } => units_available
                .iter()
                .map(|x| x.total_rooms())
                .sum::<f32>(),
            ApartmentsType::Quadplex { ref units_available } => units_available
                .iter()
                .map(|x| x.total_rooms())
                .sum::<f32>(),
        }
    }
}

#[derive(Debug)]
pub enum ResidenceType {
    SmallHouse,
    MediumHouse,
    LargeHouse,
    Estate,
    Apartments(ApartmentsType),
}

impl ResidenceType {
    fn total_rooms(&self) -> f32 {
        match *self {
            ResidenceType::SmallHouse => 6.5,
            ResidenceType::MediumHouse => 8.0,
            ResidenceType::LargeHouse => 14.0,
            ResidenceType::Estate => 21.0,
            ResidenceType::Apartments(ref apartment_type) => apartment_type.total_rooms(),
        }
    }
}



#[derive(Debug)]
pub enum MineType {
    Metal,
    RareEarth,
    Gravel,
    Oil,
}

#[derive(Debug)]
pub enum FactoryType { 
    Ammunition,
    Electronics,
    WarSupply,
    ProcessedMetals,
    Chemicals,
    Woood,
    Fertilizer,
    Refinery,
    PaperMill,
    Concreate(ConcreateType),
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


pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

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

#[derive(Debug)]
pub enum RoadSurfaceType
{
    Asphalt,
    Gravel,
    Dirt, 
}






