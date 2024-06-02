use bevy::prelude::*;


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum DamageState {
    #[default]
    FullyOperational,
    Damaged,
    Disabled,
    Wrecked,

}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum FireState{
    #[default]
    None,
    Smoldering,
    Burning,
    Burned,
}