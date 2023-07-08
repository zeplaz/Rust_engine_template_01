

use crate::entities::production::ConcreateType;



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


//producer buildings

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


//trasporation road/rails etc 

#[derive(Debug)]
pub enum RoadSurfaceType
{
    Asphalt,
    Gravel,
    Dirt, 
}
