//! Corridor path placement state (roads, rail, firebreaks — one tool, PLAN-ROADS-v1 RD-6).

use bevy::prelude::*;

use crate::construction::corridor_tool::{CorridorPathSummary, StretchCheck};

#[derive(Debug, Clone)]
pub struct RoadSegmentPreview {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    /// Buildable (not blocked).
    pub valid: bool,
    /// Buildable only with works (bridge, cut/fill, crossing): drawn amber.
    pub works: bool,
}

/// The path being drawn. Sole writers: the corridor preview (cursor, checks, segments) and the
/// corridor input (control points, finish).
#[derive(Resource, Debug, Clone)]
pub struct ActiveRoadPlacement {
    pub control_points: Vec<Vec3>,
    /// Drawn stretches: placed path, then the live stretch to the cursor.
    pub generated_segments: Vec<RoadSegmentPreview>,
    pub width: f32,
    /// When true, preview/commit sample Catmull-Rom chain (PHASE2-BUILD-17).
    pub use_curved_preview: bool,
    /// Checks of the placed path (what finishing commits).
    pub checks: Vec<StretchCheck>,
    /// Checks including the live stretch to the cursor (what the readout shows).
    pub live_checks: Vec<StretchCheck>,
    pub summary: crate::construction::corridor_tool::CorridorPathSummary,
    /// Snapped (and angle-locked) cursor on the map, tile units.
    pub cursor: Option<Vec3>,
    /// The cursor is snapped onto an existing line: a click there finishes the path.
    pub cursor_joins_line: bool,
    /// Control point being dragged.
    pub dragging: Option<usize>,
    /// The last click ended on an existing line: finish once the checks include it.
    pub finish_requested: bool,
    /// Bridge type the player cycled to (T); None = cheapest that fits each crossing.
    pub bridge_choice: Option<crate::infrastructure::bridge_types::BridgeType>,
    /// G pressed: route-assist from the last point to the cursor on the next preview update.
    pub assist_requested: bool,
    /// Last route-assist result for the readout ("no way within 12% found").
    pub assist_note: Option<String>,
    /// Hash of what the preview checks were computed from (skip when unchanged).
    pub preview_key: u64,
}

impl Default for ActiveRoadPlacement {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            generated_segments: Vec::new(),
            width: 8.0,
            use_curved_preview: true,
            checks: Vec::new(),
            live_checks: Vec::new(),
            summary: CorridorPathSummary::default(),
            cursor: None,
            cursor_joins_line: false,
            dragging: None,
            finish_requested: false,
            bridge_choice: None,
            assist_requested: false,
            assist_note: None,
            preview_key: 0,
        }
    }
}

impl ActiveRoadPlacement {
    /// Drop the path being drawn.
    pub fn clear_path(&mut self) {
        self.control_points.clear();
        self.generated_segments.clear();
        self.checks.clear();
        self.live_checks.clear();
        self.summary = CorridorPathSummary::default();
        self.dragging = None;
        self.bridge_choice = None;
        self.assist_note = None;
        self.preview_key = 0;
    }
}
