//! ECS ghost cursor resources: last pick + placement preview (HUD / confirm).

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles, SiteArchetype, SitePlacementValidation};

use super::build_strip::ToolContext;

/// Building placement FSM (design Preview | Adjust | Place — TRIAGE-BUILD-CLICK-PLACE-001).
///
/// - [`Place`] — design **Preview**: ghost follows cursor; first LMB locks → [`Adjust`].
/// - [`Adjust`] — ghost fixed; second LMB commits (ephemeral Place) → back to [`Place`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildPlacementMode {
    /// Unlocked preview (design name: Preview).
    #[default]
    Place,
    /// Locked ghost — rotate/scale modifiers apply; second LMB places.
    Adjust,
}

/// Last world pick while a build tool is active (`None` when tool is `None` or not yet clicked).
#[derive(Resource, Debug, Clone)]
pub struct BuildGhostState {
    pub origin: Option<BuildSiteTile>,
    pub footprint: FootprintTiles,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub drag_active: bool,
    /// Parametric scale drag (Shift+vertical in product UX); clamped at raster time.
    pub scale_factor: f32,
    pub placement_mode: BuildPlacementMode,
    pub last_click_screen: Option<Vec2>,
    pub last_action_tile: Option<BuildSiteTile>,
    /// True only on the frame Preview→Adjust lock consumed LMB (blocks same-frame stage/commit).
    pub locked_this_frame: bool,
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
            scale_factor: 1.0,
            placement_mode: BuildPlacementMode::Place,
            last_click_screen: None,
            last_action_tile: None,
            locked_this_frame: false,
        }
    }
}

impl BuildGhostState {
    /// Drop Adjust lock → Preview ([`BuildPlacementMode::Place`]); tool stays armed.
    pub fn unlock_to_preview(&mut self) {
        self.placement_mode = BuildPlacementMode::Place;
        self.origin = None;
        self.drag_active = false;
        self.locked_this_frame = false;
        self.last_click_screen = None;
        self.last_action_tile = None;
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
