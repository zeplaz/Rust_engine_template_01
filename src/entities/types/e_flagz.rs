use serde::Deserialize;
use std::str::FromStr;

use super::p_enumz::ResourceType;
use super::s_flagz::{BuildingType, RoadSurfaceType};
use super::v_flagz::VehicleType;

use bevy::prelude::*;

//use crate::entities::
#[derive(Debug, Clone)]
pub enum EntityType {
    Building(BuildingType),
    Tree,
    Tile,
    Vehicle(VehicleType),
    Train,
    Rail,
    Road(RoadSurfaceType),
    Resource(ResourceType),
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ConstructionStates {
    #[default]
    NotStarted,
    Planning,
    InProgress,
    Paused,
    Completed,
    Maintenance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SegmentMembership {
    Civilian,
    Military,
    Police,
    NationalGuard,
    Praetorian,
    Intelligence,
}

impl FromStr for SegmentMembership {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "civilian" | "Civilian" => Ok(SegmentMembership::Civilian),
            "military" | "Military" => Ok(SegmentMembership::Military),
            "police" | "Police" => Ok(SegmentMembership::Police),
            "nationalGuard" | "NationalGuard" => Ok(SegmentMembership::NationalGuard),
            "praetorian" | "Praetorian" => Ok(SegmentMembership::Praetorian),
            "intelligence" | "Intelligence" => Ok(SegmentMembership::Intelligence),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EntityMenuState {
    #[default]
    None,
    RoadVehicle,
    Train,
    MiliaryUnit,
    Building,
    Resources,
    HQ,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CarryingState {
    Full,
    #[default]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MalfunctionType {
    OverCapacity,
    EquipmentFailure,
    ExternalInterference,
    GridInstability,
    FuelSupplyIssue,
    ThermalOverload,
    ElectricalOverload,
    TransformerFailure,
    SubstationFailure,
    StructuralDamage, // e.g., for buildings or vehicles
    EngineFailure,    // e.g., for vehicles
    CommunicationLoss,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EmergencyType {
    ReactorBreach,
    CoolingFailure,
    DamBreach,
    FireOutbreak,
    FuelLeak,
    StructuralCollapse,
    ElectricalShortCircuit,
    GridDisconnect,
    SubstationDisconnect,
    SubstationOverload,
    VehicleCrash,
    BuildingEvacuation,
    CommunicationBlackout,
    #[default]
    None,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum OperationalStatus {
    #[default]
    // The plant is operational but not currently generating electricity. It's ready to start when needed.
    Standby,

    // The plant is actively generating electricity.
    Operational,

    // The plant is temporarily shut down for routine maintenance or inspection.
    Maintenance,

    // The plant is temporarily shut down due to a lack of fuel or other resources.
    OutOfFuel,

    // The plant is in the process of starting up, which might take time for certain types of plants.
    StartingUp,

    // The plant is in the process of shutting down.
    ShuttingDown,

    // The plant is not operational and is awaiting decommissioning or demolition.
    Decommissioned,

    // The plant is shut down due to external factors, e.g., regulatory issues, labor strikes, etc.
    ExternalShutdown,

    // The plant is operating below its optimal capacity due to various reasons.
    ReducedCapacity,

    // The plant is operating but at an overcapacity, which might be risky.
    OverCapacity,

    // The plant is temporarily shut down due to environmental conditions, e.g., too little wind for a wind farm or overcast conditions for a solar plant.
    EnvironmentalShutdown,
}

enum EmergencySeverity {
    Extreme,
    High,
    Medium,
    Low,
}
