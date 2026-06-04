//! Site orchestration systems: commit, validation pass, ordering via [`InfrastructureSiteSet`](crate::strategic::InfrastructureSiteSet).

use std::collections::HashSet;

use bevy::prelude::*;

use super::components::{
    BuildingScaleParams, ConstructionSite, PlannedSite, SiteConstructionRate, SiteFootprint,
    SiteNetworkAttachment, SiteOperationalStats, SiteResourceManifest, SiteStageProgress,
    SiteTerrainValidation, SiteWeightedFootprint,
};
use super::tile_occupation::TileOccupationBook;
use super::events::CommitConstructionSiteEvent;
use crate::economy::activation::BuildingDefinitionRef;

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
use crate::terrain::material::{invalidate_world, InvalidationReason, WorldPreviewState};

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
    mut occupation_book: Option<ResMut<TileOccupationBook>>,
    cfg: Option<Res<StrategicRasterConfig>>,
    mut overlays: Query<(&ChunkStrategicOverlay, &mut NetworkDirtyMask)>,
    mut preview_state: Option<ResMut<WorldPreviewState>>,
    mut hydro_queue: Option<ResMut<crate::substrate::hydrology::HydrologyEventQueue>>,
    mut hydro_coupling: Option<ResMut<crate::substrate::hydrology::HydrologyConstructionCouplingWitness>>,
    district_book: Option<Res<super::super::settlement::DistrictBook>>,
    mut block_book: Option<ResMut<super::super::settlement::BlockBook>>,
) {
    for ev in reader.read() {
        if let Some(placement) = ev.placement.as_ref() {
            let Some(occ) = occupation_book.as_mut() else {
                warn!(
                    "commit_construction_site: parametric placement without TileOccupationBook at {:?}",
                    placement.origin
                );
                continue;
            };
            if !occ.can_apply(&placement.weights) {
                warn!(
                    "commit_construction_site: weighted overlap rejected at {:?}",
                    placement.origin
                );
                continue;
            }
        }

        let id = if ev.site_id == SiteId::UNASSIGNED {
            issuer.next()
        } else {
            ev.site_id
        };

        if let Some(placement) = ev.placement.as_ref() {
            if let Some(occ) = occupation_book.as_mut() {
                occ.apply_site(id, &placement.weights);
            }
        }

        book.by_site.insert(
            id,
            SiteConstructionStatus {
                phase: SiteConstructionPhase::Planned,
                progress: 0.0,
            },
        );

        let tiles = footprint_tiles(ev.origin, ev.footprint);
        let block_tiles = tiles.clone();
        let emitter = zone_emitter_for_archetype(ev.archetype);
        let mut entity = commands.spawn((
            PlannedSite {
                site_id: id,
                origin: ev.origin,
                footprint: ev.footprint,
                archetype: ev.archetype,
                layer: ev.layer,
                catalog_id: ev.catalog_id.clone(),
                placement: ev.placement.clone(),
            },
            ConstructionSite {
                site_id: id.0,
                owner: ev.owner,
                archetype: ev.archetype,
                phase: SiteConstructionPhase::Planned,
                operational_readiness: 0.0,
            },
            SiteStageProgress::default(),
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
        if let Some(cid) = ev.catalog_id.clone() {
            entity.insert(BuildingDefinitionRef { catalog_id: cid });
        }
        if let Some(placement) = ev.placement.as_ref() {
            entity.insert((
                SiteWeightedFootprint {
                    weights: placement.weights.clone(),
                },
                BuildingScaleParams {
                    scale_factor: placement.scale_factor,
                    effective_scale: placement.effective_scale,
                },
            ));
        }
        if let Some(request) = crate::construction::procedural_building_request_from_commit(
            id,
            ev.archetype,
            ev.footprint,
            ev.placement.as_ref(),
        ) {
            entity.insert(super::components::ProceduralBuildingSpec(request));
        }

        if let Some(cfg) = cfg.as_ref() {
            let coords =
                footprint_affected_chunk_coords(ev.origin, ev.footprint, cfg.cells_per_chunk);
            let affected: HashSet<IVec2> = coords.iter().copied().collect();
            if !affected.is_empty() {
                for (ov, mut mask) in overlays.iter_mut() {
                    if affected.contains(&ov.chunk_coord) {
                        mask.mask |= NETWORK_DIRTY_FLOW;
                    }
                }
            }
            if let Some(mut preview) = preview_state.as_mut() {
                invalidate_world(
                    InvalidationReason::StrategicInfrastructure,
                    &mut preview,
                    coords.into_iter(),
                );
            }
        }

        if let (Some(districts), Some(blocks)) = (district_book.as_ref(), block_book.as_mut()) {
            crate::strategic::register_site_on_commit(districts, blocks, id, &block_tiles);
        }

        if let (Some(cfg), Some(hydro), Some(coupling)) = (
            cfg.as_ref(),
            hydro_queue.as_mut(),
            hydro_coupling.as_mut(),
        ) {
            crate::construction::emit_site_execute_hydro_dirty(
                hydro,
                coupling,
                id.0,
                ev.origin,
                ev.footprint,
                cfg.cells_per_chunk,
            );
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
    ), Without<SiteStageProgress>>,
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
