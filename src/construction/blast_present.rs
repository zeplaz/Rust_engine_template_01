//! Presentation / score consumer for [`BlastCraterBook`] (never writes sim beyond RO dump).

use bevy::prelude::*;

use super::blast::BlastCraterBook;

/// Lightweight dump for HUD / sim_review (sole writer: this system).
#[derive(Resource, Default, Debug, Clone)]
pub struct BlastCraterDump {
    pub tile_count: u32,
    pub blasts_applied: u64,
    pub sites_demolished: u64,
    pub sample_tiles: Vec<(i32, i32)>,
}

pub fn publish_blast_crater_dump(
    book: Res<BlastCraterBook>,
    mut dump: ResMut<BlastCraterDump>,
) {
    dump.tile_count = book.tiles.len() as u32;
    dump.blasts_applied = book.blasts_applied;
    dump.sites_demolished = book.sites_demolished;
    dump.sample_tiles = book.tiles.iter().copied().take(16).collect();
}
