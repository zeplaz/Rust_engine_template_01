pub mod states;

use crate::states::*;
use bevy::prelude::*;

pub fn transtion_to_simulation_state(mut commands: Commands, base_state: Res<BaseState>) {
    if base_state.0 != BaseState::Simulation {
        comands.insert_resource(NextState(Some(BaseState::Simulation)));
    }
}

pub fn transition_to_MainMenu_state(mut commands: Commands, base_state: Res<BaseState>) {
    if base_state.0 != BaseState::MainMenu {
        comands.insert_resource(NextState(Some(BaseState::MainMenu)));
    }
}

pub fn toggle_simulation_state(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    sim_state: Res<SimulationState>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        if sim_state.0 == SimulationState::Paused {
            commands.insert_resource(SimulationState(SimulationState::Running));
        }
        if sim_state.0 == SimulationState::Running {
            commands.insert_resource(SimulationState(SimulationState::Paused));
        }
    }
}

pub fn exit_game() {
    {
        app_exit_event_writer.send(AppExit);
    }
}
