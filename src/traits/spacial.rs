use bevy::prelude::*;

pub trait Spaceialization {
    type Position;

    // Method that must be implemented by the user.
    fn get_position(&self) -> &Self::Position;

    // Default methods that return a 'not supported' value.
    fn get_position2d(&self) -> Vec2 {
        println!("get_position2d: Not Supported");
        Vec2::NEG_ONE // Return default Vec2
    }

    fn get_position3d(&self) -> Vec3 {
        println!("get_position3d: Not Supported");
        Vec3::NEG_ONE // Return default Vec3
    }

    fn get_position4d(&self) -> Vec4 {
        println!("get_position4d: Not Supported");
        Vec4::NEG_ONE // Return default Vec4
    }
}
