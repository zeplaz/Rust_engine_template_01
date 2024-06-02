use crate::engine::states::EnigneState;
//use crate::gui::main_menu::MainMenuPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EnginePlugin {}

impl Plugin for EnginePlugin {
    //pub fn setup(mut commands: Commands) {}

    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins).add_plugin(EguiPlugin);

        //app.add_plugin(MenuStatePlugin)
        //.add_state(AppState::MainMenu)
        //.add_system_set(SystemSet::on_enter(AppState::LoadEditor).with_system(setup_load_editor.system()))
        //.add_system_set(SystemSet::on_update(AppState::LoadEditor).with_system(handle_load_editor.system()))
        //.add_system(generate_terrain.in_base_set(TerrainGeneration))
        //.add_system(generate_regions.in_base_set(RegionGeneration))
        //.add_system(generate_agents.in_base_set(AgentGeneration))
        //.add_system(setup_agent_relationships.in_base_set(AgentRelationships))
        //.add_system(run_simulation.in_base_set(Simulation))
        /*.configure_sets(
            (
            //    TerrainGeneration,
            //    RegionGeneration,
        //        AgentGeneration,
            //    AgentRelationships,
        //        Simulation,
            )
                .chain(),
        );*/
    }
}

/*

fn main_loop()
{

// Event loop where you would update and draw your game or application.
loop {



    // Begin a new frame of ImGui and perform any setup you need to do for your frame.

    if let Some(text) = submitted_text {
        // Do something with the submitted text here.
        println!("Submitted text: {}", text);
    }

    // Render the UI and present it to your display.
    // You'll need to have your own implementation of the `Renderer` trait to handle this.
    // See the `examples/imgui_impl_*` modules in the imgui-rs repository for examples.
    let draw_data = ui.render();
    // ...
}

}
*/
