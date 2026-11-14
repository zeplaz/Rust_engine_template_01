//! Blast / crater execute funnel (Ranking C3) — sole construction-side damage path.
//!
//! Combat ballistics **enqueue** [`BlastIntent`]; only [`execute_blast_intents_system`]
//! mutates sites / crater book (construction invariants).

use bevy::prelude::*;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, FootprintTiles, PlannedSite, SiteFootprint, TileOccupationBook,
};
use crate::substrate::hydrology::{HydrologyConstructionCouplingStats, HydrologyEventQueue};
use crate::systems::sim_control::SimControlSystemSet;

use super::demolish::execute_demolish_at_tile;
use super::hydro_coupling::emit_site_execute_hydro_dirty;

/// One blast to apply this tick (ingress from combat / tools).
#[derive(Clone, Debug)]
pub struct BlastIntent {
    pub center_xz: Vec2,
    /// World metres.
    pub radius_m: f32,
    /// ≥50 demolishes overlapping sites; lower = crater only.
    pub damage: f32,
    pub source_flight: u64,
}

#[derive(Resource, Default, Debug)]
pub struct BlastIntentQueue {
    pub pending: Vec<BlastIntent>,
}

impl BlastIntentQueue {
    pub fn enqueue(&mut self, intent: BlastIntent) {
        self.pending.push(intent);
    }
}

/// Crater / scorched tiles for presentation (sim truth for “blast happened here”).
#[derive(Resource, Default, Debug, Clone)]
pub struct BlastCraterBook {
    pub tiles: std::collections::BTreeSet<(i32, i32)>,
    pub blasts_applied: u64,
    pub sites_demolished: u64,
}

const METRES_PER_TILE: f32 = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

fn tile_of_xz(xz: Vec2) -> IVec2 {
    IVec2::new(
        (xz.x / METRES_PER_TILE).floor() as i32,
        (xz.y / METRES_PER_TILE).floor() as i32,
    )
}

fn tile_center_xz(tile: IVec2) -> Vec2 {
    Vec2::new(
        (tile.x as f32 + 0.5) * METRES_PER_TILE,
        (tile.y as f32 + 0.5) * METRES_PER_TILE,
    )
}

/// Sole writer of [`BlastCraterBook`] + blast demolish side-effects.
pub fn execute_blast_intents_system(
    mut commands: Commands,
    mut queue: ResMut<BlastIntentQueue>,
    mut craters: ResMut<BlastCraterBook>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut hydro_q: Option<ResMut<HydrologyEventQueue>>,
    mut hydro_stats: Option<ResMut<HydrologyConstructionCouplingStats>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
) {
    if queue.pending.is_empty() {
        return;
    }
    let intents = std::mem::take(&mut queue.pending);
    for intent in intents {
        let center = intent.center_xz;
        let r = intent.radius_m.max(0.5);
        let r2 = r * r;
        let center_tile = tile_of_xz(center);

        let tile_r = (r / METRES_PER_TILE).ceil() as i32;
        for dz in -tile_r..=tile_r {
            for dx in -tile_r..=tile_r {
                let t = IVec2::new(center_tile.x + dx, center_tile.y + dz);
                let c = tile_center_xz(t);
                if c.distance_squared(center) <= r2 {
                    craters.tiles.insert((t.x, t.y));
                }
            }
        }
        craters.blasts_applied = craters.blasts_applied.saturating_add(1);

        if intent.damage < 50.0 {
            continue;
        }

        let mut hit_origins: Vec<BuildSiteTile> = Vec::new();
        for (_e, _site, planned, footprint, _) in sites.iter() {
            let overlaps = footprint.tiles.iter().any(|t| {
                let c = tile_center_xz(*t);
                c.distance_squared(center) <= r2
            });
            if overlaps {
                hit_origins.push(planned.origin);
            }
        }
        hit_origins.sort_by_key(|t| (t.x, t.z));
        hit_origins.dedup();

        for origin in hit_origins {
            let (n, _events) =
                execute_demolish_at_tile(&mut commands, origin, &sites, occupation.as_deref_mut());
            if n > 0 {
                craters.sites_demolished = craters.sites_demolished.saturating_add(u64::from(n));
                if let (Some(q), Some(stats)) = (hydro_q.as_mut(), hydro_stats.as_mut()) {
                    emit_site_execute_hydro_dirty(
                        q,
                        stats,
                        intent.source_flight,
                        origin,
                        FootprintTiles {
                            width: 1,
                            depth: 1,
                        },
                        UVec2::new(16, 16),
                    );
                }
            }
        }
    }
}

pub struct ConstructionBlastPlugin;

impl Plugin for ConstructionBlastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlastIntentQueue>()
            .init_resource::<BlastCraterBook>()
            .init_resource::<super::blast_present::BlastCraterDump>()
            .add_systems(
                Update,
                (
                    execute_blast_intents_system
                        .after(SimControlSystemSet::AdvanceSimTick)
                        .after(crate::combat::ballistics::step_ballistic_flights),
                    super::blast_present::publish_blast_crater_dump
                        .after(execute_blast_intents_system),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crater_disk_marks_center_tile() {
        let mut book = BlastCraterBook::default();
        let center = Vec2::new(8.0, 8.0);
        let t = tile_of_xz(center);
        book.tiles.insert((t.x, t.y));
        assert!(book.tiles.contains(&(t.x, t.y)));
    }
}
