use bevy::prelude::*;
use crate::idgen::EntityId;
use crate::entities::types_of::e_flagz::SegmentMembership;
use crate::entities::types_of::s_flagz::{RoadSurfaceType,BuildingType};

// Building component
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
}


// NearbyRoads component
#[derive(Component)]
pub struct NearbyRoads {
    pub roads: Vec<EntityId>,
}

// NearbyRails component
#[derive(Component)]
pub struct NearbyRails {
    pub rails: Vec<EntityId>,
}


#[derive(Component)]  
struct Road {
    lanes: u32,
    surface: RoadSurfaceType,

}


#[derive(Component)]  
struct RoadSegment {}

#[derive(Component)]  
struct RoadConnection {}


#[derive(Component)]  
pub struct Tree {}

#[derive(Component)]  
struct Rrails {

}

#[derive(Debug, PartialEq, Clone)]
pub enum GaugeType {
    Narrow,          // Example: Mountain railways
    Standard,        // Example: Most widespread global standard
    Broad,           // Example: Russia, Spain
    Metre,           // Example: Urban rail networks, trams
    TwoFoot,         // Example: Industrial railways
    Brunel,          // Example: Historically in the UK
    FiveFoot,        // Example: Historically in the USA
}

struct RailGauge {
    gauge_type: GaugeType,
    width_mm: f32,
    name: String,
    max_speed: f32,               // Maximum supported speed
    max_load_capacity: f32,      // Maximum load capacity
    stability_factor: f32,       // Factor for how stable the trains are on this gauge
    turn_radius: f32,            // Maximum supported turn radius
    infrastructure_cost_factor: f32, // Multiplier for infrastructure costs 
    rolling_stock_interchangeability: bool, // Whether rolling stock can be easily interchanged
}