//gui_main::Gui;

use bevy::prelude::*;
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum BaseState {
    Simuation,
    #[default]
    MainMenu,
    Editor,
    Shutdown,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MainMenuState {
    #[default]
    MainMenu,
    Settings,
    Load,
    Editor,
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum BuildMenuState {
    #[default]
    None,
    BuildingsMenu,
    VehiclesMenu,
    RailMenu,
    RoadMenu,
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum OverlayWorldState {
    Production,
    Safty,
    #[default]
    Normal,
    Supply,
    Miliary,
    Transport,
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MiniMapState {
    #[default]
    Normal,
    Production,
    Transit,
    Poltical,
    Safty,
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
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum InGameMenuState {
    #[default]
    Normal,
    Pause,
    Settings,
    Map,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum InGameEditorState {
    #[default]
    None,
    Select,
    Create,
    Modify,
    Delete,
    Settings,
    Layers,
    EntityTypes,
    Terrain,
    Road,
    Rail,
}

pub struct EnigneState {
    starting_up: bool,
    quit: bool,
    error: bool,
    running: bool,
}

impl Default for EnigneState {
    fn default() -> Self {
        EnigneState {
            starting_up: true,
            quit: false,
            error: false,
            running: false,
        }
    }
}
