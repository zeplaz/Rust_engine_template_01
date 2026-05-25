//! Road path control points + segment previews (P2-01 foundation).

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct RoadSegmentPreview {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub valid: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct ActiveRoadPlacement {
    pub control_points: Vec<Vec3>,
    pub generated_segments: Vec<RoadSegmentPreview>,
    pub width: f32,
    /// When true, preview/commit sample Catmull-Rom chain (PHASE2-BUILD-17).
    pub use_curved_preview: bool,
}

impl Default for ActiveRoadPlacement {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            generated_segments: Vec::new(),
            width: 8.0,
            use_curved_preview: true,
        }
    }
}
