//! Rail path placement state (separate from road — Round 2 Wave C).

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct RailSegmentPreview {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub valid: bool,
    pub slope_ok: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct ActiveRailPlacement {
    pub control_points: Vec<Vec3>,
    pub generated_segments: Vec<RailSegmentPreview>,
    pub width: f32,
    /// Minimum horizontal turn radius (tiles) — rail rejects tight corners.
    pub min_curve_radius: f32,
    pub max_slope: f32,
    /// **INFRA-E2-004** — selected corridor profile id for execute.
    pub profile_id: String,
}

impl Default for ActiveRailPlacement {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            generated_segments: Vec::new(),
            width: 6.0,
            min_curve_radius: 3.0,
            max_slope: 0.12,
            profile_id: "default_rail".into(),
        }
    }
}
