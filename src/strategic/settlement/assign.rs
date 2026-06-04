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

/// Lib witness runner for SET-P5-002.
#[must_use]
pub fn three_sites_same_block_witness_green() -> bool {
    use crate::strategic::settlement::district::portland_fixture_district;
    use crate::strategic::settlement::town::portland_fixture_town;
    use crate::strategic::{
        BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, SiteArchetype,
    };
    use crate::strategic::site::{SiteConstructionBook, SiteIdIssuer};
    use bevy::app::App;
    use bevy::MinimalPlugins;

    fn commit_site(app: &mut App, origin: (u32, u32)) {
        let mut events = app.world_mut().resource_mut::<Messages<CommitConstructionSiteEvent>>();
        events.write(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner: Entity::PLACEHOLDER,
            origin: BuildSiteTile {
                x: origin.0,
                z: origin.1,
            },
            footprint: FootprintTiles {
                width: 1,
                depth: 1,
            },
            archetype: SiteArchetype::Factory,
            layer: crate::strategic::LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.update();
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .init_resource::<BlockBook>()
        .insert_resource(portland_fixture_town())
        .insert_resource(portland_fixture_district(&portland_fixture_town()))
        .add_message::<CommitConstructionSiteEvent>()
        .add_systems(Update, crate::strategic::commit_construction_site_system);

    commit_site(&mut app, (16, 16));
    commit_site(&mut app, (17, 16));
    commit_site(&mut app, (18, 16));

    let blocks = app.world().resource::<BlockBook>();
    if blocks.tile_to_block.len() != 3 {
        return false;
    }
    let block_ids: HashSet<_> = blocks.tile_to_block.values().cloned().collect();
    if block_ids.len() != 1 {
        return false;
    }
    let block = blocks
        .blocks
        .get(block_ids.iter().next().unwrap())
        .unwrap();
    block.site_ids.len() == 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_sites_same_block_after_portland_chain() {
        assert!(three_sites_same_block_witness_green());
    }
}
