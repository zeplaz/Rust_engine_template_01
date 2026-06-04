//! Construction **phase tiles** on the GPU tile-debug path (RepresentationResult overlay slice).
//!
//! Publishes active-site footprint cells into [`TileDebugInstanceMap`] with
//! [`tile_flags::CONSTRUCTION_SITE`] + phase in `lod`. CPU egui labels remain in [`super::phase_visual`].

use bevy::prelude::*;

use crate::gui::{
    tile_flags, ScaffoldContract, TileDebugInstance, TileDebugInstanceMap, TileDebugViewId,
};
use crate::strategic::{
    ConstructionSite, PlannedSite, SiteConstructionPhase, SiteFootprint,
};

use super::tile_visual::ConstructionTileVisualSettings;

/// Transitional scaffold — same exit as footprint bridge.
pub const CONSTRUCTION_PHASE_TILE_SCAFFOLD: ScaffoldContract = ScaffoldContract {
    owner: "construction/site_phase_tile_instances",
    intended_replacement: "RepresentationResult overlay channel",
    exit_condition: "Phase tiles consumed from overlay_matrix.construction_phase only",
    removal_trigger: "duplicate TileDebug producer for site phase",
};

/// Drives [`RepresentationResult::overlay_matrix`](crate::gui::representation_policy::OverlayChannelMatrix).
#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionPhaseGpuChannel {
    pub active: bool,
    pub instance_count: u32,
}

#[inline]
fn phase_lod(phase: SiteConstructionPhase) -> u32 {
    match phase {
        SiteConstructionPhase::Planned => 0,
        SiteConstructionPhase::Surveying => 1,
        SiteConstructionPhase::Clearing => 2,
        SiteConstructionPhase::Foundation => 3,
        SiteConstructionPhase::UnderConstruction => 4,
        SiteConstructionPhase::Provisioning => 5,
        SiteConstructionPhase::Operational => 6,
        SiteConstructionPhase::Damaged => 7,
        SiteConstructionPhase::Offline => 8,
        SiteConstructionPhase::Abandoned => 9,
    }
}

pub fn push_site_phase_tile_instances(
    settings: Res<ConstructionTileVisualSettings>,
    sites: Query<(&ConstructionSite, &PlannedSite, &SiteFootprint)>,
    mut map: ResMut<TileDebugInstanceMap>,
    mut channel: ResMut<ConstructionPhaseGpuChannel>,
) {
    channel.active = false;
    channel.instance_count = 0;
    if !settings.show_site_phase_tiles {
        return;
    }
    let size = 0.46;
    let rows = map
        .per_view
        .entry(TileDebugViewId::WorldMain)
        .or_default();
    for (site, _planned, footprint) in &sites {
        if matches!(site.phase, SiteConstructionPhase::Abandoned) {
            continue;
        }
        let lod = phase_lod(site.phase);
        let flags = tile_flags::CONSTRUCTION_SITE;
        for tile in &footprint.tiles {
            rows.push(TileDebugInstance {
                world_pos: [tile.x as f32 + 0.5, tile.y as f32 + 0.5],
                size,
                lod,
                flags,
            });
        }
    }
    channel.instance_count = rows.len() as u32;
    channel.active = channel.instance_count > 0;
    let _ = CONSTRUCTION_PHASE_TILE_SCAFFOLD.is_declared();
}
