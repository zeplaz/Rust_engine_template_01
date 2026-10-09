//gui_main::Gui;

use bevy::prelude::*;
use bevy_egui::{
    EguiContextSettings, EguiFullOutput, EguiGlobalSettings, EguiPostUpdateSet, EguiPreUpdateSet,
    PrimaryEguiContext,
};
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

// Legacy `EngineState` boot flags were removed: use [`BaseState`], [`WorldGenFlowState`], and
// [`crate::systems::sim_control::SimControlState`] for lifecycle + sim cadence.

/// While [`BaseState::Simulation`], skip `bevy_egui` play passes.
/// `EguiPreUpdateSet::ProcessInput` stays skipped. `InitContexts` stays scheduled.
/// `BeginPass`, `EndPass`, `ProcessOutput`, and `PostProcessOutput` run only outside Simulation.
/// Editor and WorldGen still run those sets. Play picks stay on `simulation_tool_pointer`.
pub fn omit_egui_pointer_preupdate_in_simulation(app: &mut App) {
    // The pointer-schedule unit test calls this without `EguiPlugin`. Production engines insert
    // the resource first; `init_resource` is a no-op when the plugin already did.
    app.init_resource::<EguiGlobalSettings>();
    app.configure_sets(
        PreUpdate,
        EguiPreUpdateSet::ProcessInput.run_if(not(in_state(BaseState::Simulation))),
    );
    app.configure_sets(
        PreUpdate,
        EguiPreUpdateSet::BeginPass.run_if(not(in_state(BaseState::Simulation))),
    );
    app.configure_sets(
        PostUpdate,
        EguiPostUpdateSet::EndPass.run_if(not(in_state(BaseState::Simulation))),
    );
    app.configure_sets(
        PostUpdate,
        EguiPostUpdateSet::ProcessOutput.run_if(not(in_state(BaseState::Simulation))),
    );
    app.configure_sets(
        PostUpdate,
        EguiPostUpdateSet::PostProcessOutput.run_if(not(in_state(BaseState::Simulation))),
    );
    app.add_systems(
        OnEnter(BaseState::Simulation),
        clear_primary_egui_output_on_simulation_enter,
    );
    app.add_systems(
        OnExit(BaseState::Simulation),
        restore_egui_cursor_and_capture_on_simulation_exit,
    );
}

fn clear_primary_egui_output_on_simulation_enter(
    mut global: ResMut<EguiGlobalSettings>,
    mut primary: Query<(&mut EguiFullOutput, &mut EguiContextSettings), With<PrimaryEguiContext>>,
) {
    global.enable_cursor_icon_updates = false;
    for (mut output, mut settings) in &mut primary {
        *output = EguiFullOutput(None);
        settings.capture_pointer_input = false;
    }
}

fn restore_egui_cursor_and_capture_on_simulation_exit(
    mut global: ResMut<EguiGlobalSettings>,
    mut primary: Query<&mut EguiContextSettings, With<PrimaryEguiContext>>,
) {
    global.enable_cursor_icon_updates = true;
    for mut settings in &mut primary {
        settings.capture_pointer_input = true;
    }
}
