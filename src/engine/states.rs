//gui_main::Gui;

use bevy::prelude::*;
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum BaseState {
    Simulation,
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

/// High-level pipeline for **new** procedural worlds (preview → full → confirm enter).
/// Loading a save should use `LoadingSave` and must not run the procedural generator.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum WorldGenFlowState {
    /// No active world-build flow; procedural `GenerateWorldEvent` requests are ignored.
    #[default]
    Idle,
    /// User is editing parameters (sliders, seeds) before any commit.
    NewWorldSetup,
    /// Preview terrain exists; user may tweak and re-preview or request full generation.
    PreviewReady,
    /// Full-size world generated; user confirms before [`BaseState::Simulation`].
    FullReady,
    /// Reserved for save/load path (no procedural `GenerateWorldEvent`).
    LoadingSave,
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

pub struct EngineState {
    starting_up: bool,
    quit: bool,
    error: bool,
    running: bool,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState {
            starting_up: true,
            quit: false,
            error: false,
            running: false,
        }
    }
}
