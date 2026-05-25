//! ECS ghost cursor resources: last pick + placement preview (HUD / confirm).

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles, SiteArchetype, SitePlacementValidation};

use super::build_strip::ToolContext;

/// Last world pick while a build tool is active (`None` when tool is `None` or not yet clicked).
#[derive(Resource, Debug, Clone)]
pub struct BuildGhostState {
    pub origin: Option<BuildSiteTile>,
    pub footprint: FootprintTiles,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub drag_active: bool,
}

impl Default for BuildGhostState {
    fn default() -> Self {
        Self {
            origin: None,
            footprint: FootprintTiles {
                width: 1,
                depth: 1,
            },
            rotation_quarter_turns: 0,
            mirror_x: false,
            drag_active: false,
        }
    }
}

/// Latest [`SitePlacementValidation`] for the ghost origin (HUD + confirm gate).
#[derive(Resource, Debug, Clone)]
pub struct BuildPlacementPreview {
    pub report: SitePlacementValidation,
}

impl Default for BuildPlacementPreview {
    fn default() -> Self {
        Self {
            report: SitePlacementValidation::default(),
        }
    }
}

/// `Entity` used as `owner` on player [`CommitConstructionSiteEvent`](crate::strategic::CommitConstructionSiteEvent).
#[derive(Resource, Clone, Copy, Debug)]
pub struct BuildCommandActor(pub Entity);

impl ToolContext {
    /// Footprint hint per tool (stub — corridors / districts refine later).
    #[inline]
    pub fn footprint_for_tool(self) -> FootprintTiles {
        match self {
            ToolContext::Industry | ToolContext::Civil | ToolContext::Military => FootprintTiles {
                width: 2,
                depth: 2,
            },
            _ => FootprintTiles {
                width: 1,
                depth: 1,
            },
        }
    }

    /// Maps strip mode → provisional [`SiteArchetype`] for territorial site commits.
    #[inline]
    pub fn site_archetype(self) -> SiteArchetype {
        match self {
            ToolContext::Rail | ToolContext::Roads => SiteArchetype::RailDepot,
            ToolContext::Utilities => SiteArchetype::WaterPlant,
            ToolContext::Military => SiteArchetype::MilitaryBase,
            ToolContext::Industry => SiteArchetype::Factory,
            ToolContext::Ecology | ToolContext::Civil => SiteArchetype::CivilHousing,
            ToolContext::None => SiteArchetype::Factory,
        }
    }
}

/// Marker: singleton ghost entity for [`super::GhostBuildCursor`].
#[derive(Component)]
pub struct BuildGhostRoot;
