//! Construction site events (P2-A).

use bevy::prelude::*;

use super::components::SiteArchetype;
use super::resources::{FootprintTiles, SiteId};
use crate::strategic::build_order::BuildSiteTile;
use crate::strategic::spatial_network::LayerType;

/// Commit a planned site: allocates [`SiteId`] when `site_id` is [`SiteId::UNASSIGNED`].
#[derive(Message, Clone, Debug)]
pub struct CommitConstructionSiteEvent {
    pub site_id: SiteId,
    pub owner: Entity,
    pub archetype: SiteArchetype,
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
    pub layer: LayerType,
}
