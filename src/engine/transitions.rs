use crate::engine::states::*;
use bevy::prelude::*;

pub fn transition_to_simulation_state(mut next_state: ResMut<NextState<BaseState>>) {
    NextState::set_if_neq(&mut *next_state, BaseState::Simulation);
}

pub fn transition_to_main_menu_state(mut next_state: ResMut<NextState<BaseState>>) {
    NextState::set_if_neq(&mut *next_state, BaseState::MainMenu);
}

pub fn exit_game(mut app_exit: MessageWriter<bevy::app::AppExit>) {
    app_exit.write(bevy::app::AppExit::Success);
}
