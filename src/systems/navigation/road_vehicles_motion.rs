//! Load-aware speed modifiers + NAV-06 path smoothing + MOVE-08 output filtering.
//!
//! Dual-cite: NEW **NAV-06** (aig3 4.4.6–4.4.7) · **MOVE-08** (aig3 output filtering /
//! capability-sensitive steering) · LEGACY `pathfinding` / `steering` (aig_e3).
//! Tile search stays on [`super::logistics_floodfill`]; motion consumers call
//! [`smooth_vehicle_tile_path`] then [`filter_vehicle_steering_output`] so trucks/buses
//! respect turn rate and accel limits (no jitter spikes).

use bevy::prelude::Vec2;

use crate::entities::vehicles::runtime::{Bus, Truck};
use crate::traits::LoadBasedSpeedModifier;

use super::logistics_floodfill::{smooth_logistics_tile_path, LogisticsTile};

impl LoadBasedSpeedModifier for Truck {
    fn speed_modifier(&self) -> f32 {
        let cap = self.vehicle.capacity.max(1) as f32;
        let load_ratio = self.current_load / cap;
        0.7 + (1.0 - load_ratio) * 0.7
    }
}

impl LoadBasedSpeedModifier for Bus {
    fn speed_modifier(&self) -> f32 {
        let cap = self.capacity.max(1) as f32;
        let passenger_ratio = self.passengers as f32 / cap;
        0.8 + (1.0 - passenger_ratio) * 0.8
    }
}

/// NAV-06: string-pull a logistics tile path for vehicle motion (open corridors → endpoints).
#[must_use]
pub fn smooth_vehicle_tile_path(
    path: &[(i32, i32)],
    width: i32,
    height: i32,
    tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile,
) -> Vec<(i32, i32)> {
    smooth_logistics_tile_path(path, width, height, tile_at)
}

/// MOVE-08 capability limits for heavy vehicles (world units / sec, rad / sec).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleMotionLimits {
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_turn_rate: f32,
}

impl VehicleMotionLimits {
    pub const TRUCK: Self = Self {
        max_speed: 8.0,
        max_accel: 3.0,
        max_turn_rate: 1.2,
    };
    pub const BUS: Self = Self {
        max_speed: 6.5,
        max_accel: 2.2,
        max_turn_rate: 0.9,
    };
}

/// MOVE-08: filter desired velocity by accel + turn-rate caps (capability-sensitive).
///
/// `heading` is the current facing unit vector (or zero → snap to desired direction).
/// Returns (filtered_velocity, new_heading_unit).
#[must_use]
pub fn filter_vehicle_steering_output(
    current_vel: Vec2,
    desired_vel: Vec2,
    heading: Vec2,
    limits: VehicleMotionLimits,
    dt: f32,
) -> (Vec2, Vec2) {
    let dt = dt.max(1e-4);
    let mut desired = desired_vel;
    let des_speed = desired.length();
    if des_speed > limits.max_speed {
        desired *= limits.max_speed / des_speed;
    }

    // Accel clamp toward desired.
    let delta = desired - current_vel;
    let max_delta = limits.max_accel * dt;
    let filtered = if delta.length_squared() > max_delta * max_delta {
        current_vel + delta.normalize_or_zero() * max_delta
    } else {
        desired
    };

    // Turn-rate clamp on heading.
    let want = if filtered.length_squared() > 1e-8 {
        filtered.normalize()
    } else if desired.length_squared() > 1e-8 {
        desired.normalize()
    } else {
        heading
    };
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        want
    };
    let cross = h.x * want.y - h.y * want.x;
    let dot = h.dot(want).clamp(-1.0, 1.0);
    let ang = cross.atan2(dot);
    let max_ang = limits.max_turn_rate * dt;
    let applied = ang.clamp(-max_ang, max_ang);
    let (s, c) = applied.sin_cos();
    let new_heading = Vec2::new(h.x * c - h.y * s, h.x * s + h.y * c).normalize_or_zero();

    // Project filtered speed along new heading (no sideways jitter).
    let speed = filtered.length().min(limits.max_speed);
    (new_heading * speed, new_heading)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav06_vehicle_smooth_matches_floodfill_pull() {
        let raw = [(0, 0), (1, 0), (2, 0), (3, 0)];
        let mut tile = |_x: i32, _y: i32| LogisticsTile::default();
        let s = smooth_vehicle_tile_path(&raw, 4, 1, &mut tile);
        assert_eq!(s, vec![(0, 0), (3, 0)]);
    }

    #[test]
    fn move08_accel_limits_speed_ramp() {
        let (v, _) = filter_vehicle_steering_output(
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            Vec2::X,
            VehicleMotionLimits::TRUCK,
            0.1,
        );
        assert!(v.x <= VehicleMotionLimits::TRUCK.max_accel * 0.1 + 1e-3);
        assert!(v.x > 0.0);
    }

    #[test]
    fn move08_turn_rate_limits_heading_change() {
        let (_, h) = filter_vehicle_steering_output(
            Vec2::X,
            Vec2::Y * 4.0,
            Vec2::X,
            VehicleMotionLimits::BUS,
            0.05,
        );
        // Cannot instantly face +Y in 0.05s at max_turn_rate 0.9.
        assert!(h.x > 0.5, "heading must still lean forward: {h:?}");
        assert!(h.y > 0.0);
    }
}
