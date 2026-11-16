//! Potential-field steering toward waypoints with per-tile influence (faction / safety hooks).
//!
//! **MOVE-03** (aig3 3.3.13 *Separation* · 3.3.14 *Collision Avoidance* · LEGACY
//! `steering` aig_e3): truck/agent queues at depots need separation + avoidance on top of
//! waypoint pull — otherwise agents stack on the same tile.
//!
//! **MOVE-04** (aig3 3.3.x blending vs arbitration): [`arbitrate_steering_pipeline`] picks
//! one dominant channel by priority (avoid → separate → seek) instead of summing forces that
//! cancel. Rename module to `potential_field_nav` when paths are updated
//! (`implementation_questions_v1.md` §12–14).

use bevy::prelude::*;

use crate::entities::Waypoints;
use crate::idgen::EntityId;
use crate::terrain::Tile;

use super::motion::SpeedModifier;
use super::nav::find_tile_at_position;

/// How close an agent must be before advancing to the next waypoint.
const REACHED_WAYPOINT_THRESHOLD: f32 = 0.75;

/// MOVE-03: agents closer than this contribute separation (world units / tiles).
pub const SEPARATION_RADIUS: f32 = 1.25;
/// Separation gain (scaled by inverse distance).
pub const SEPARATION_STRENGTH: f32 = 1.1;
/// Collision-avoidance look-ahead distance along the seek direction.
pub const AVOIDANCE_LOOKAHEAD: f32 = 2.0;
/// Lateral push when another agent sits in the look-ahead cone.
pub const AVOIDANCE_STRENGTH: f32 = 0.85;
/// Half-angle (radians) of the forward avoidance cone (~45°).
const AVOIDANCE_HALF_ANGLE_COS: f32 = 0.70710678; // cos(π/4)

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

/// MOVE-03 separation (aig3 3.3.13): sum of pushes away from neighbors inside radius.
#[must_use]
pub fn separation_steer(
    position: Vec2,
    neighbors: &[Vec2],
    radius: f32,
    strength: f32,
) -> Vec2 {
    let r2 = radius * radius;
    let mut push = Vec2::ZERO;
    for &other in neighbors {
        let delta = position - other;
        let d2 = delta.length_squared();
        if d2 < f32::EPSILON || d2 > r2 {
            continue;
        }
        let d = d2.sqrt();
        push += (delta / d) * (strength * (radius - d) / radius);
    }
    push
}

/// MOVE-03 collision avoidance (aig3 3.3.14): lateral steer when a neighbor sits ahead.
#[must_use]
pub fn collision_avoidance_steer(
    position: Vec2,
    forward: Vec2,
    neighbors: &[Vec2],
    lookahead: f32,
    strength: f32,
) -> Vec2 {
    let fwd_len = forward.length();
    if fwd_len < f32::EPSILON {
        return Vec2::ZERO;
    }
    let fwd = forward / fwd_len;
    let mut lateral = Vec2::ZERO;
    for &other in neighbors {
        let to = other - position;
        let along = to.dot(fwd);
        if along <= 0.0 || along > lookahead {
            continue;
        }
        let to_len = to.length();
        if to_len < f32::EPSILON {
            continue;
        }
        let cos = along / to_len;
        if cos < AVOIDANCE_HALF_ANGLE_COS {
            continue;
        }
        // Perpendicular to forward; pick side that increases separation.
        let side = Vec2::new(-fwd.y, fwd.x);
        let side_sign = if to.dot(side) >= 0.0 { -1.0 } else { 1.0 };
        let weight = (1.0 - along / lookahead).clamp(0.0, 1.0);
        lateral += side * side_sign * strength * weight;
    }
    lateral
}

/// MOVE-04 channel priority (higher wins arbitration).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SteeringChannel {
    Seek = 0,
    Separate = 1,
    Avoid = 2,
}

/// Minimum squared length to treat a channel as active.
const STEER_ACTIVE_EPS2: f32 = 1e-6;

/// MOVE-04 arbitration: pick the highest-priority non-zero channel (no force cancelling).
#[must_use]
pub fn arbitrate_steering_pipeline(seek: Vec2, separate: Vec2, avoid: Vec2) -> (Vec2, SteeringChannel) {
    if avoid.length_squared() > STEER_ACTIVE_EPS2 {
        return (avoid, SteeringChannel::Avoid);
    }
    if separate.length_squared() > STEER_ACTIVE_EPS2 {
        return (separate, SteeringChannel::Separate);
    }
    (seek, SteeringChannel::Seek)
}

/// Combine seek + separation + avoidance via MOVE-04 arbitration (not naive sum).
#[must_use]
pub fn combine_steering_for_depot_queue(
    position: Vec2,
    seek: Vec2,
    neighbors: &[Vec2],
) -> Vec2 {
    let sep = separation_steer(position, neighbors, SEPARATION_RADIUS, SEPARATION_STRENGTH);
    let avoid = collision_avoidance_steer(
        position,
        seek,
        neighbors,
        AVOIDANCE_LOOKAHEAD,
        AVOIDANCE_STRENGTH,
    );
    arbitrate_steering_pipeline(seek, sep, avoid).0
}

pub fn potential_field_navigation_system(
    mut query: Query<(Entity, &mut Transform, &mut Waypoints, &SpeedModifier)>,
    tile_query: Query<&Tile>,
) {
    // Pass 1: snapshot positions for MOVE-03 neighbor forces (no dual mut borrow).
    let positions: Vec<(Entity, Vec2)> = query
        .iter()
        .map(|(e, t, _, _)| (e, t.translation.truncate()))
        .collect();

    for (entity, mut transform, mut wp, speed_modifier) in query.iter_mut() {
        if wp.points.is_empty() {
            continue;
        }

        let idx = wp.current_waypoint_index.min(wp.points.len() - 1);
        let waypoint = wp.points[idx];
        let position = transform.translation.truncate();

        let neighbors: Vec<Vec2> = positions
            .iter()
            .filter(|(e, _)| *e != entity)
            .map(|(_, p)| *p)
            .collect();

        if let Some(ref tile) = find_tile_at_position(position, &tile_query) {
            // TODO: replace default with entity-derived owner when AgentOwnable is on the vehicle entity.
            let requester = EntityId::default();
            let seek = calculate_potential_field_influence(position, waypoint, tile, requester);
            let combined = combine_steering_for_depot_queue(position, seek, &neighbors);
            let movement_vector = combined * speed_modifier.value;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move03_separation_pushes_apart_when_stacked() {
        let a = Vec2::ZERO;
        let neighbors = [Vec2::new(0.2, 0.0)];
        let push = separation_steer(a, &neighbors, SEPARATION_RADIUS, SEPARATION_STRENGTH);
        assert!(push.x < 0.0, "must push left away from neighbor on +x: {push:?}");
    }

    #[test]
    fn move03_avoidance_steers_lateral_when_ahead() {
        let pos = Vec2::ZERO;
        let forward = Vec2::X;
        let neighbors = [Vec2::new(1.0, 0.1)];
        let lat = collision_avoidance_steer(
            pos,
            forward,
            &neighbors,
            AVOIDANCE_LOOKAHEAD,
            AVOIDANCE_STRENGTH,
        );
        assert!(lat.y.abs() > 0.0, "must add lateral component: {lat:?}");
        assert!(lat.x.abs() < lat.y.abs() + 0.5);
    }

    #[test]
    fn move03_combine_keeps_seek_when_alone() {
        let seek = Vec2::new(1.0, 0.0);
        let out = combine_steering_for_depot_queue(Vec2::ZERO, seek, &[]);
        assert_eq!(out, seek);
    }

    #[test]
    fn move04_arbitration_prefers_avoid_over_seek() {
        let seek = Vec2::new(1.0, 0.0);
        let sep = Vec2::new(-0.5, 0.0);
        let avoid = Vec2::new(0.0, 1.0);
        let (out, ch) = arbitrate_steering_pipeline(seek, sep, avoid);
        assert_eq!(ch, SteeringChannel::Avoid);
        assert_eq!(out, avoid);
    }

    #[test]
    fn move04_arbitration_falls_back_to_seek() {
        let seek = Vec2::new(2.0, 0.0);
        let (out, ch) = arbitrate_steering_pipeline(seek, Vec2::ZERO, Vec2::ZERO);
        assert_eq!(ch, SteeringChannel::Seek);
        assert_eq!(out, seek);
    }
}
