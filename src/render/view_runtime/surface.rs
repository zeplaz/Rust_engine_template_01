use bevy::prelude::Entity;

use crate::gui::{ViewCameraState, ViewRenderPolicy};

use super::ids::{ViewIsolationGroup, ViewSurfaceId};
use super::layers::{
    InteractionViewportState, OverlayViewportPolicy, RenderViewportContract, SemanticViewportRect,
};

/// Committed per-surface snapshot (read model for extract + diagnostics).
#[derive(Clone, Debug)]
pub struct ViewSurface {
    pub id: ViewSurfaceId,
    pub group: ViewIsolationGroup,
    pub camera_entity: Entity,
    pub semantic: Option<SemanticViewportRect>,
    pub render: RenderViewportContract,
    pub interaction: InteractionViewportState,
    pub overlay: OverlayViewportPolicy,
    pub camera: ViewCameraState,
    pub render_policy: ViewRenderPolicy,
}
