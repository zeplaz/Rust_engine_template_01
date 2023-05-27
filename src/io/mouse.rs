use bevy::prelude::*;



#[derive(Resource)]
pub struct MouseMovments{
pub sensitivity: f32,
pub speed: f32,
}


impl Default for MouseMovments {
    fn default() -> Self {
        Self {
            sensitivity: 0.001,
            speed: 3.0,
        }
}
}

#[derive(Component)]
pub struct MainCam; 