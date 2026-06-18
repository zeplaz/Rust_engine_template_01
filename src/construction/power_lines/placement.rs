//! Power line path control points + segment previews (COD-POWER-LINE-DRAW-001).

use bevy::prelude::*;

use crate::infrastructure::VoltageClass;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PowerLineRoutingMode {
    #[default]
    Curved,
    Orthogonal90,
}

impl PowerLineRoutingMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Curved => "Curved",
            Self::Orthogonal90 => "90°",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Curved => Self::Orthogonal90,
            Self::Orthogonal90 => Self::Curved,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PowerLineSegmentPreview {
    pub start: Vec3,
    pub end: Vec3,
    pub valid: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct ActivePowerLinePlacement {
    pub control_points: Vec<Vec3>,
    pub generated_segments: Vec<PowerLineSegmentPreview>,
    pub routing_mode: PowerLineRoutingMode,
    pub grid_snap: bool,
    pub voltage: VoltageClass,
}

impl Default for ActivePowerLinePlacement {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            generated_segments: Vec::new(),
            routing_mode: PowerLineRoutingMode::Curved,
            grid_snap: true,
            voltage: VoltageClass::Medium,
        }
    }
}

impl ActivePowerLinePlacement {
    pub fn clear_path(&mut self) {
        self.control_points.clear();
        self.generated_segments.clear();
    }
}
