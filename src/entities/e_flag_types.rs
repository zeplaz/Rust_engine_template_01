use std::str::FromStr;
use serde::Deserialize;

use crate::entities::strukturave::{BuildingType, RoadSurfaceType};
use crate::entities::vehicles::VehicleType;
use crate::entities::production::ResourceType;

//use crate::entities::
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


