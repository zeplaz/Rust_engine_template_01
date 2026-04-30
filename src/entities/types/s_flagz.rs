use serde::{Deserialize, Serialize};

/// Concrete formulation subset for factory lines that produce concrete variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcreteType {
    Limecrete,
    Portland,
    Geopolymer,
    Gypsum,
}

/// Discrete factory production lines (paired with `resource_deserializer` text format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactoryType {
    ConcreteType(ConcreteType),
    Ammunition,
    Electronics,
    WarSupply,
    Chemical,
    Wood,
    Fertilizer,
    Refinery,
    MetalProcessing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MineType {
    Gravel,
    Metal,
    RareEarth,
    Oil,
}

/// Building classification for structure components and I/O parsers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    Generic,
    FactoryType(FactoryType),
    MineType(MineType),
    Farm,
    House,
    RaiLDepot,
    Burocracy,
    WareHouse,
    Depanneur,
    FeildDepot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoadSurfaceType {
    Asphalt,
    Dirt,
}
