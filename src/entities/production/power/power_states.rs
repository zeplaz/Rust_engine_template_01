use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerPlantType {
    Coal,
    Nuclear,
    Solar,
    Wind,
    Oil,
    Gas,
    Geothermal,
    Hydro,
    Biomass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwitchState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationMechanism {
    Manual,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwitchCloseBehavior {
    NonReclosing,
    AutoReclosing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceMechanism {
    Flow,
    Storable,
}