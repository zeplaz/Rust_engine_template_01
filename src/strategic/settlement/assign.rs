//! Site → block assignment on commit (SET-P5-002).

use std::collections::HashSet;

use bevy::prelude::*;

use super::block::{BlockBook, BlockRecord};
use super::district::DistrictBook;
use super::ids::{BlockId, DistrictId};
use crate::strategic::SiteId;

pub const BLOCK_GRID_SIZE: i32 = 8;

#[must_use]
pub fn block_cluster_key(district_id: &DistrictId, tile: IVec2) -> BlockId {
    BlockId(format!(
        "{}_{}_{}",
        district_id.0,
        tile.x.div_euclid(BLOCK_GRID_SIZE),
        tile.y.div_euclid(BLOCK_GRID_SIZE)
    ))
}

#[must_use]
pub fn district_for_tile(districts: &DistrictBook, tile: IVec2) -> Option<DistrictId> {
    districts
        .districts
        .values()
        .find(|d| d.contains_tile(tile))
        .map(|d| d.id.clone())
}

#[must_use]
pub fn assign_block_for_tile(
    districts: &DistrictBook,
    blocks: &mut BlockBook,
    tile: IVec2,
) -> BlockId {
    let district_id = district_for_tile(districts, tile)
        .or_else(|| districts.default_district.clone())
        .unwrap_or_else(|| DistrictId("default".into()));
    let block_id = block_cluster_key(&district_id, tile);
    blocks.tile_to_block.insert(tile, block_id.clone());
    blocks
        .blocks
        .entry(block_id.clone())
        .or_insert_with(|| BlockRecord {
            id: block_id.clone(),
            district_id,
            tiles: HashSet::new(),
            site_ids: Vec::new(),
            archetype: None,
        })
        .tiles
        .insert(tile);
    block_id
}

pub fn register_site_on_commit(
    districts: &DistrictBook,
    blocks: &mut BlockBook,
    site_id: SiteId,
    tiles: &[IVec2],
) {
    for &tile in tiles {
        let block_id = assign_block_for_tile(districts, blocks, tile);
        if let Some(block) = blocks.blocks.get_mut(&block_id) {
            if !block.site_ids.contains(&site_id.0) {
                block.site_ids.push(site_id.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    

}
