use bevy::prelude::*;

pub trait Spaceialization {
    type Position;

    // Method that must be implemented by the user.
    fn get_position(&self) -> &Self::Position;

    // Default methods that return a 'not supported' value.
    fn get_position2d(&self) -> Vec2 {
        Vec2::NEG_ONE
    }

    fn get_position3d(&self) -> Vec3 {
        Vec3::NEG_ONE
    }

    fn get_position4d(&self) -> Vec4 {
        Vec4::NEG_ONE
    }
}
