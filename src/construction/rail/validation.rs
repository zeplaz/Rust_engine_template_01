//! Rail segment validation — slope + bounds (distinct from road).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::super::construction_pipeline::ConstructionValidation;

#[must_use]
pub fn validate_rail_segment(
    head: BuildSiteTile,
    tail: BuildSiteTile,
    start_y: f32,
    end_y: f32,
    max_slope: f32,
    params: &WorldGenParams,
) -> ConstructionValidation {
    let mut required_actions = Vec::new();
    if head == tail {
        return ConstructionValidation {
            valid: false,
            required_actions: vec!["rail segment needs distinct endpoints".into()],
        };
    }
    if params.width == 0 || params.height == 0 {
        return ConstructionValidation {
            valid: false,
            required_actions: vec!["world bounds unavailable".into()],
        };
    }
    for tile in [head, tail] {
        if tile.x >= params.width || tile.z >= params.height {
            required_actions.push(format!("tile ({},{}) outside world", tile.x, tile.z));
        }
    }
    let dx = (tail.x as f32 - head.x as f32).abs().max(0.5);
    let dz = (tail.z as f32 - head.z as f32).abs().max(0.5);
    let horiz = (dx * dx + dz * dz).sqrt();
    let rise = (end_y - start_y).abs();
    if horiz > 0.0 && rise / horiz > max_slope {
        required_actions.push(format!(
            "grade {:.0}% exceeds rail max {:.0}%",
            (rise / horiz) * 100.0,
            max_slope * 100.0
        ));
    }
    ConstructionValidation {
        valid: required_actions.is_empty(),
        required_actions,
    }
}

#[must_use]
pub fn segment_slope_ok(start: Vec3, end: Vec3, max_slope: f32) -> bool {
    let dx = (end.x - start.x).abs().max(0.5);
    let dz = (end.z - start.z).abs().max(0.5);
    let horiz = (dx * dx + dz * dz).sqrt();
    let rise = (end.y - start.y).abs();
    horiz <= 0.0 || rise / horiz <= max_slope
}

#[must_use]
pub fn turn_radius_ok(prev: Vec3, corner: Vec3, next: Vec3, min_radius: f32) -> bool {
    let v1 = Vec2::new(corner.x - prev.x, corner.z - prev.z);
    let v2 = Vec2::new(next.x - corner.x, next.z - corner.z);
    if v1.length_squared() < 0.01 || v2.length_squared() < 0.01 {
        return true;
    }
    let dot = v1.normalize().dot(v2.normalize()).clamp(-1.0, 1.0);
    let angle = dot.acos();
    if angle < 0.15 {
        return true;
    }
    let chord = (prev - next).xz().length();
    let radius = chord / (2.0 * (angle / 2.0).sin().max(0.01));
    radius >= min_radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steep_grade_fails_rail_validation() {
        let head = BuildSiteTile { x: 1, z: 1 };
        let tail = BuildSiteTile { x: 2, z: 1 };
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let v = validate_rail_segment(head, tail, 0.0, 5.0, 0.12, &params);
        assert!(!v.valid);
    }
}
