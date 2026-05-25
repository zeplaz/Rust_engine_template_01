use bevy::math::{Rect, UVec2, Vec2};
use bevy::prelude::*;

/// HUD / layout expectation (semantic layer).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticViewportRect {
    pub rect: Rect,
    pub valid: bool,
}

/// Committed pixel contract (render layer).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderViewportContract {
    pub logical_size: Vec2,
    pub physical_extent: UVec2,
    pub valid: bool,
    pub target: ViewRenderTargetDesc,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ViewRenderTargetDesc {
    PrimaryWindowSubrect { min: Vec2, max: Vec2 },
    OffscreenImage { handle: Handle<Image> },
    None,
}

impl Default for ViewRenderTargetDesc {
    fn default() -> Self {
        Self::None
    }
}

/// Pointer capture / pan deltas (interaction layer).
#[derive(Clone, Debug, Default)]
pub struct InteractionViewportState {
    pub captured: bool,
    pub pan_delta: Vec2,
    pub zoom_factor: f32,
}

/// Debug / construction draw policy (overlay layer).
#[derive(Clone, Debug, Default)]
pub struct OverlayViewportPolicy {
    pub allow_debug_outline: bool,
    pub allow_construction_ghost: bool,
}
