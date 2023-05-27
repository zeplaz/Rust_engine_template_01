use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct BuildingTemplate {
    pub name: String,
    pub foundation_cost: FoundationCost,
    pub building_cost: u32,
    pub mask: Vec<Vec<u8>>,
    pub sprite_handle: Handle<ColorMaterial>,
    // other relevant fields
}



fn building_placement_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut build_mode: Res<BuildMode>,
    //mut map: Res<Map>,
    building_templates: Res<BuildingTemplates>,
    //resources: Res<Resources>,
    // other necessary resources/components
) {
    if build_mode.state != BuildModeState::Active || build_mode.mode_type != BuildModeType::Building {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let active_building = &building_templates.active_building;

        if let Some(position) = get_tile_under_cursor(/* necessary parameters */) {
            if is_valid_building_placement(position, active_building, &map) {
                if meets_building_requirements(active_building, &resources) {
                    let new_building = spawn_building(&mut commands, active_building);

                    if build_mode.simulation_mode {
                        update_resources(active_building, &mut resources);
                    }

                    add_building_to_map(new_building, position, &mut map);
                }
            }
        }
    }
}
