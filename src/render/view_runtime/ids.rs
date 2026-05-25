use bevy::prelude::*;

/// Stable surface identity (superset of legacy [`crate::gui::ViewId`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewSurfaceId {
    WorldMain,
    WorldPreview,
    Minimap,
    SimulationMap,
    DiagnosticsOverlay,
    ConstructionPreview,
}

impl ViewSurfaceId {
    #[must_use]
    pub fn from_view_id(id: crate::gui::ViewId) -> Self {
        match id {
            crate::gui::ViewId::WorldMain => Self::WorldMain,
            crate::gui::ViewId::WorldPreview => Self::WorldPreview,
            crate::gui::ViewId::Minimap => Self::Minimap,
            crate::gui::ViewId::SimulationMap => Self::SimulationMap,
        }
    }

    #[must_use]
    pub fn to_view_id(self) -> Option<crate::gui::ViewId> {
        match self {
            Self::WorldMain => Some(crate::gui::ViewId::WorldMain),
            Self::WorldPreview => Some(crate::gui::ViewId::WorldPreview),
            Self::Minimap => Some(crate::gui::ViewId::Minimap),
            Self::SimulationMap => Some(crate::gui::ViewId::SimulationMap),
            Self::DiagnosticsOverlay | Self::ConstructionPreview => None,
        }
    }
}

/// Hard isolation: systems declare which group they affect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewIsolationGroup {
    WorldSimulation,
    EditorPreview,
    MinimapPresentation,
    Diagnostics,
    ConstructionPresentation,
}

#[must_use]
pub const fn default_isolation_group(id: ViewSurfaceId) -> ViewIsolationGroup {
    match id {
        ViewSurfaceId::WorldMain | ViewSurfaceId::SimulationMap => ViewIsolationGroup::WorldSimulation,
        ViewSurfaceId::WorldPreview => ViewIsolationGroup::EditorPreview,
        ViewSurfaceId::Minimap => ViewIsolationGroup::MinimapPresentation,
        ViewSurfaceId::DiagnosticsOverlay => ViewIsolationGroup::Diagnostics,
        ViewSurfaceId::ConstructionPreview => ViewIsolationGroup::ConstructionPresentation,
    }
}

/// ECS marker: camera entity owned by one surface.
#[derive(Component, Debug, Clone, Copy)]
pub struct ViewEntity {
    pub surface: ViewSurfaceId,
    pub group: ViewIsolationGroup,
}
