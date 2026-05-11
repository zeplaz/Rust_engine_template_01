//! Ghost placement preview entity marker (shared pipeline: roads, sites, corridors).

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles};

#[derive(Component, Debug, Clone)]
pub struct GhostBuildCursor {
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
}
