//! Ghost placement preview entity marker (shared pipeline: roads, sites, corridors).

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles};

#[derive(Component, Debug, Clone)]
pub struct GhostBuildCursor {
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
}

/// Shared ghost validity on the singleton [`crate::construction::BuildGhostRoot`]
/// (BUILD-P1-02) — mirrored from [`crate::construction::BuildPlacementPreview`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostValid(pub bool);
