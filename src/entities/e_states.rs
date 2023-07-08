use bevy::prelude::*;

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
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum LightingState{
    Night,
    #[default]
    Midday,
}


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ConstructionStates{
    #[default]
    NotStarted,
    Planning,
    InProgress,
    Paused,
    Completed,
    Maintenance,
}
