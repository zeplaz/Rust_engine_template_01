//! **SET-P5-HIERARCHY-GAP** — playable books seed + placement→block + overlay hydrate.

use bevy::prelude::*;

use super::assign::{register_site_on_commit, three_sites_same_block_witness_green};
use super::block::BlockBook;
use super::district::{portland_fixture_district, DistrictBook};
use super::town::{portland_fixture_town, seed_settlement_books_if_empty, TownBook};
use super::{set_p5_hierarchy_books_seeded_witness_green, BLOCK_GRID_SIZE};
use crate::strategic::SiteId;

pub const SET_P5_HIERARCHY_GAP_LIVE_JSON: &str = "debug_runs/settlement_hierarchy_gap_live.json";

/// Empty books → Startup seed → default town + ≥1 district.
#[must_use]
pub fn set_p5_hierarchy_seed_lib_green() -> bool {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<TownBook>()
        .init_resource::<DistrictBook>()
        .add_systems(Startup, seed_settlement_books_if_empty);
    app.update();
    let towns = app.world().resource::<TownBook>();
    let districts = app.world().resource::<DistrictBook>();
    set_p5_hierarchy_books_seeded_witness_green(towns, districts)
}

/// Placement/commit path fills `tile_to_block` for footprint tiles inside a district.
#[must_use]
pub fn set_p5_hierarchy_placement_block_lib_green() -> (bool, usize) {
    let towns = portland_fixture_town();
    let districts = portland_fixture_district(&towns);
    if !set_p5_hierarchy_books_seeded_witness_green(&towns, &districts) {
        return (false, 0);
    }
    let mut blocks = BlockBook::default();
    let tiles = [
        IVec2::new(16, 16),
        IVec2::new(17, 16),
        IVec2::new(16 + BLOCK_GRID_SIZE, 16),
    ];
    register_site_on_commit(&districts, &mut blocks, SiteId(9001), &tiles);
    let mapped = blocks.tile_to_block.len();
    let ok = mapped == tiles.len()
        && tiles
            .iter()
            .all(|t| blocks.tile_to_block.contains_key(t))
        && blocks
            .blocks
            .values()
            .any(|b| b.site_ids.contains(&9001));
    (ok, mapped)
}

/// Exit rollup: seed + commit wiring + settlement overlay hydrate.
#[must_use]
pub fn set_p5_hierarchy_gap_lib_green() -> (bool, bool, bool, bool, usize) {
    let books_seeded = set_p5_hierarchy_seed_lib_green();
    let commit_wired = three_sites_same_block_witness_green();
    let (placement_to_block, tile_to_block_count) = set_p5_hierarchy_placement_block_lib_green();
    let overlay_hydrate = crate::io::save::econ_og_save_001_witness_green();
    (
        books_seeded,
        commit_wired,
        placement_to_block,
        overlay_hydrate,
        tile_to_block_count,
    )
}



#[cfg(test)]
mod tests {
    


}
