//! Commit helper — emits the same message as AI / scenarios.

use bevy::prelude::*;

use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype, SiteId,
};

pub fn queue_commit_construction_site(
    writer: &mut MessageWriter<CommitConstructionSiteEvent>,
    owner: Entity,
    archetype: SiteArchetype,
    origin: BuildSiteTile,
    footprint: FootprintTiles,
    layer: LayerType,
    catalog_id: Option<String>,
) {
    writer.write(CommitConstructionSiteEvent {
        site_id: SiteId::UNASSIGNED,
        owner,
        archetype,
        origin,
        footprint,
        layer,
        catalog_id,
    });
}
