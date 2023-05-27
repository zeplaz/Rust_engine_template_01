use bevy::prelude::*;
use proc_A_dine01::entities::vehicles::{RoadVehicle, Train};
use proc_A_dine01::traits::Waypoints;
use proc_A_dine01::systems::motion_calculations::SpeedModifier;

use nav::find_tile_at_position;

use super idgens::EntityId

fn calculate_potential_field_influence(
    position: Vec2,
    waypoint: Vec2,
    tile: &Tile,
    requester__owner_id: EntityId
)->Vec2
{
    let distance = position.distance(waypoint);
    let direction = (waypoint - position).normalize();

       // Calculate influence based on the properties of the tile
/*
    if tile.owner_id == entity_info.get_owner() {

       let influence_strength = if ____{


       } else if  ___ {

       }

    } else {
       //should check if on allies, or enemies, or nutreal, noone

    }

*/


pub fn potential_field_navigation_system(
    mut query: Query<(&mut Transform, &Waypoints, &SpeedModifier)>,
    tile_query: Query<&Tile>,
) {
    for (mut transform, waypoints, speed_modifier) in query.iter_mut() {
        let waypoint = waypoints.points[waypoints.current_waypoint_index];
        let position = transform.translation.truncate();

        // Find the tile at the current position
        let tile = find_tile_at_position(position, &tile_query);

        if let Some(tile) = tile {
            let influence = calculate_potential_field_influence(position, waypoint, &tile);

            // Use the calculated influence to update the vehicle's position and orientation
            let movement_vector = influence * speed_modifier.value;
            transform.translation += movement_vector.extend(0.0);

            // Update vehicle orientation based on the movement direction
            let angle = movement_vector.angle_between(Vec2::new(1.0, 0.0));
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
//vs

pub fn potential_field_navigation_system(
    mut query: Query<(&mut Transform, &Waypoints, &SpeedModifier)>,
) {
    for (mut transform, waypoints, speed_modifier) in query.iter_mut() {
        let current_position = transform.translation.truncate();
        let current_waypoint = waypoints.points[waypoints.current_waypoint_index];

        // Calculate the potential field's influence on the vehicle's movement.
        let potential_field_influence = calculate_potential_field_influence(current_position, &waypoints.points);

        // Combine the influence with the global path's direction towards the next waypoint.
        let direction_to_next_waypoint = (current_waypoint - current_position).normalize();
        let combined_direction = (direction_to_next_waypoint + potential_field_influence).normalize();

        // Update the vehicle's position based on the combined direction and its speed modifier.
        let speed = speed_modifier.calculate_speed_modifier();
        transform.translation += Vec3::from((combined_direction * speed, 0.0));

        // Check if the vehicle has reached the current waypoint.
        if current_position.distance(current_waypoint) < REACHED_WAYPOINT_THRESHOLD {
            waypoints.current_waypoint_index += 1;
        }
    }

}
