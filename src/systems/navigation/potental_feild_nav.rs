//! Potential-field steering toward waypoints with per-tile influence (faction / safety hooks).
//! Rename module to `potential_field_nav` when paths are updated (`implementation_questions_v1.md` §12–14).

use bevy::prelude::*;

use crate::entities::Waypoints;
use crate::idgen::EntityId;
use crate::terrain::Tile;

use super::motion::SpeedModifier;
use super::nav::find_tile_at_position;

/// How close an agent must be before advancing to the next waypoint.
const REACHED_WAYPOINT_THRESHOLD: f32 = 0.75;

fn calculate_potential_field_influence(
    position: Vec2,
    waypoint: Vec2,
    tile: &Tile,
    requester_owner_id: EntityId,
) -> Vec2 {
    let to_wp = waypoint - position;
    let distance = to_wp.length();
    let direction = if distance > f32::EPSILON {
        to_wp / distance
    } else {
        Vec2::ZERO
    };

    // Base falloff — stronger pull when far from waypoint.
    let mut influence = direction * (1.0 / distance.max(1.0));

    // Ownership / diplomacy bias (expand when faction graph + treaties exist).
    if let Some(owner) = tile.owner_id {
        if owner != requester_owner_id {
            influence *= 0.35;
        }
    }

    // Safety reduces effective crossing intent; clamp keeps field numerically stable.
    influence *= tile.safety_rating.clamp(0.1, 2.0);

    influence
}

pub fn potential_field_navigation_system(
    mut query: Query<(&mut Transform, &mut Waypoints, &SpeedModifier)>,
    tile_query: Query<&Tile>,
) {
    for (mut transform, mut wp, speed_modifier) in query.iter_mut() {
        if wp.points.is_empty() {
            continue;
        }

        let idx = wp.current_waypoint_index.min(wp.points.len() - 1);
        let waypoint = wp.points[idx];
        let position = transform.translation.truncate();

        if let Some(ref tile) = find_tile_at_position(position, &tile_query) {
            // TODO: replace default with entity-derived owner when AgentOwnable is on the vehicle entity.
            let requester = EntityId::default();
            let influence =
                calculate_potential_field_influence(position, waypoint, tile, requester);
            let movement_vector = influence * speed_modifier.value;
            transform.translation += movement_vector.extend(0.0);

            if movement_vector.length_squared() > f32::EPSILON {
                let angle = movement_vector.y.atan2(movement_vector.x);
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }

        let position = transform.translation.truncate();
        if position.distance(waypoint) < REACHED_WAYPOINT_THRESHOLD
            && wp.current_waypoint_index + 1 < wp.points.len()
        {
            wp.current_waypoint_index += 1;
        }
    }
}
