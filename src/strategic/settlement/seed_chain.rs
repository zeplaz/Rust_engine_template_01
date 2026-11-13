//! **CITY-G1-C4-001** — formal seed chain: world → town → block → lot → building grammar.
//!
//! Deterministic SHA-256 mixing (idgen-adjacent). Python parity: `rust_engine_mcp.city_seed_chain`.

use sha2::{Digest, Sha256};

use super::ids::{BlockId, TownId};

pub const DEFAULT_WORLD_SEED: u64 = 99_001;
pub const DEFAULT_TOWN_ID: &str = "portland";
#[must_use]
pub fn mix_u64(parent: u64, label: &str, key: &str) -> u64 {
    let raw = format!("{parent}:{label}:{key}");
    let digest = Sha256::digest(raw.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().expect("8 byte digest"))
}

#[must_use]
pub fn town_seed(world_seed: u64, town_id: &TownId) -> u64 {
    mix_u64(world_seed, "town", &town_id.0)
}

#[must_use]
pub fn block_seed(parent_town_seed: u64, block_id: &BlockId) -> u64 {
    mix_u64(parent_town_seed, "block", &block_id.0)
}

#[must_use]
pub fn lot_seed(parent_block_seed: u64, lot_idx: u32) -> u64 {
    mix_u64(parent_block_seed, "lot", &lot_idx.to_string())
}

/// Final seed fed to `building_grammar::generate` (lot tier).
#[must_use]
pub fn building_grammar_seed(parent_lot_seed: u64, archetype_id: &str) -> u64 {
    mix_u64(parent_lot_seed, "building_grammar", archetype_id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitySeedContext {
    pub world_seed: u64,
    pub town_id: TownId,
    pub block_id: BlockId,
    pub lot_idx: u32,
    pub archetype_id: String,
}

impl CitySeedContext {
    #[must_use]
    pub fn building_grammar_seed(&self) -> u64 {
        let ts = town_seed(self.world_seed, &self.town_id);
        let bs = block_seed(ts, &self.block_id);
        let ls = lot_seed(bs, self.lot_idx);
        building_grammar_seed(ls, &self.archetype_id)
    }
}

#[must_use]
pub fn building_grammar_seed_chain(
    world_seed: u64,
    town_id: &TownId,
    block_id: &BlockId,
    lot_idx: u32,
    archetype_id: &str,
) -> u64 {
    CitySeedContext {
        world_seed,
        town_id: town_id.clone(),
        block_id: block_id.clone(),
        lot_idx,
        archetype_id: archetype_id.to_owned(),
    }
    .building_grammar_seed()
}

#[must_use]
pub fn lot_idx_from_site_id(site_id: u64) -> u32 {
    (site_id & 0xFFFF_FFFF) as u32
}

#[must_use]
pub fn block_id_for_site(site_id: u64) -> BlockId {
    BlockId(format!("site_block_{site_id}"))
}

/// PG-3 commit path — default town + site-derived block/lot until block recipes land (C3).
#[must_use]
pub fn building_grammar_seed_for_site(world_seed: u64, site_id: u64, archetype_id: &str) -> u64 {
    building_grammar_seed_chain(
        world_seed,
        &TownId(DEFAULT_TOWN_ID.into()),
        &block_id_for_site(site_id),
        lot_idx_from_site_id(site_id),
        archetype_id,
    )
}



#[cfg(test)]
mod tests {
    

}
