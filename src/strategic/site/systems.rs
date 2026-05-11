//! Site orchestration systems: commit, validation pass, ordering via [`InfrastructureSiteSet`](crate::strategic::InfrastructureSiteSet).

use std::collections::HashSet;

use bevy::prelude::*;

use super::components::{
    ConstructionSite, PlannedSite, SiteConstructionRate, SiteFootprint, SiteNetworkAttachment,
    SiteOperationalStats, SiteResourceManifest, SiteTerrainValidation,
};
use super::events::CommitConstructionSiteEvent;
use super::overlays::zone_emitter_for_archetype;
use super::resources::{
    FootprintTiles, SiteConstructionBook, SiteConstructionPhase, SiteConstructionStatus, SiteId,
    SiteIdIssuer,
};
use super::validation::evaluate_site_placement_stubs;
use crate::strategic::build_order::BuildSiteTile;
use crate::strategic::network_flow::{NetworkDirtyMask, NETWORK_DIRTY_FLOW};
use crate::strategic::transport_bridge::StrategicRasterConfig;
use crate::strategic::ChunkStrategicOverlay;

fn footprint_tiles(origin: BuildSiteTile, fp: FootprintTiles) -> Vec<IVec2> {
    let ox = origin.x as i32;
    let oz = origin.z as i32;
    let mut tiles = Vec::with_capacity((fp.width * fp.depth) as usize);
    for dz in 0..fp.depth {
        for dx in 0..fp.width {
            tiles.push(IVec2::new(ox + dx as i32, oz + dz as i32));
        }
    }
    tiles
}

/// Chunk coordinates (strategic raster space) touched by a site footprint.
pub fn footprint_affected_chunk_coords(origin: BuildSiteTile, fp: FootprintTiles, cells_per_chunk: UVec2) -> Vec<IVec2> {
    let cw = cells_per_chunk.x.max(1) as i32;
    let ch = cells_per_chunk.y.max(1) as i32;
    let mut seen = HashSet::new();
    for dz in 0..fp.depth {
        for dx in 0..fp.width {
            let tx = origin.x as i32 + dx as i32;
            let tz = origin.z as i32 + dz as i32;
            let ccx = tx.div_euclid(cw);
            let ccz = tz.div_euclid(ch);
            seen.insert(IVec2::new(ccx, ccz));
        }
    }
    seen.into_iter().collect()
}

pub fn commit_construction_site_system(
    mut commands: Commands,
    mut reader: MessageReader<CommitConstructionSiteEvent>,
    mut book: ResMut<SiteConstructionBook>,
    mut issuer: ResMut<SiteIdIssuer>,
    cfg: Option<Res<StrategicRasterConfig>>,
    mut overlays: Query<(&ChunkStrategicOverlay, &mut NetworkDirtyMask)>,
) {
    for ev in reader.read() {
        let id = if ev.site_id == SiteId::UNASSIGNED {
            issuer.next()
        } else {
            ev.site_id
        };

        book.by_site.insert(
            id,
            SiteConstructionStatus {
                phase: SiteConstructionPhase::Planned,
                progress: 0.0,
            },
        );

        let tiles = footprint_tiles(ev.origin, ev.footprint);
        let emitter = zone_emitter_for_archetype(ev.archetype);
        commands.spawn((
            PlannedSite {
                site_id: id,
                origin: ev.origin,
                footprint: ev.footprint,
                archetype: ev.archetype,
                layer: ev.layer,
            },
            ConstructionSite {
                site_id: id.0,
                owner: ev.owner,
                archetype: ev.archetype,
                phase: SiteConstructionPhase::Planned,
                operational_readiness: 0.0,
            },
            SiteFootprint {
                tiles,
                layer: ev.layer,
            },
            SiteResourceManifest::default(),
            SiteNetworkAttachment::default(),
            SiteTerrainValidation::default(),
            SiteOperationalStats::default(),
            SiteConstructionRate::default(),
            emitter,
        ));

        if let Some(cfg) = cfg.as_ref() {
            let affected: HashSet<IVec2> =
                footprint_affected_chunk_coords(ev.origin, ev.footprint, cfg.cells_per_chunk)
                    .into_iter()
                    .collect();
            if !affected.is_empty() {
                for (ov, mut mask) in overlays.iter_mut() {
                    if affected.contains(&ov.chunk_coord) {
                        mask.mask |= NETWORK_DIRTY_FLOW;
                    }
                }
            }
        }
    }
}

pub fn validate_committed_site_terrain_system(mut q: Query<&mut SiteTerrainValidation, With<ConstructionSite>>) {
    let stub = evaluate_site_placement_stubs();
    for mut terrain in &mut q {
        terrain.slope_ok = stub.valid;
        terrain.hydrology_ok = stub.valid;
        terrain.geology_ok = stub.valid;
        terrain.flood_risk = (1.0 - stub.terrain_score).max(0.0);
    }
}

/// When terrain checks pass, move from **Planned** → **UnderConstruction** and seed manifest requirements (stub curve).
pub fn site_advance_planned_to_under_construction_system(
    mut q: Query<(
        &mut ConstructionSite,
        &SiteTerrainValidation,
        &PlannedSite,
        &mut SiteResourceManifest,
    )>,
    mut book: ResMut<SiteConstructionBook>,
) {
    for (mut site, terrain, planned, mut manifest) in &mut q {
        if site.phase != SiteConstructionPhase::Planned {
            continue;
        }
        if !(terrain.slope_ok && terrain.hydrology_ok && terrain.geology_ok) {
            continue;
        }
        site.phase = SiteConstructionPhase::UnderConstruction;
        site.operational_readiness = 0.0;
        if let Some(st) = book.by_site.get_mut(&planned.site_id) {
            st.phase = SiteConstructionPhase::UnderConstruction;
            st.progress = 0.0;
        }
        if manifest.concrete_required == 0.0
            && manifest.steel_required == 0.0
            && manifest.fuel_required == 0.0
            && manifest.machinery_required == 0.0
        {
            manifest.concrete_required = 100.0;
            manifest.steel_required = 40.0;
            manifest.fuel_required = 20.0;
            manifest.machinery_required = 10.0;
        }
    }
}
