use bevy::math::Vec2;

pub struct Waypoints{
    pub points:Vec<Vec2>,
    pub current_waypoint_index:usize,
}
