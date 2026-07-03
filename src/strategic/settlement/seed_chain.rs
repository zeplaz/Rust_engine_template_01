//! **CITY-G1-C4-001** — formal seed chain: world → town → block → lot → building grammar.
//!
//! Deterministic SHA-256 mixing (idgen-adjacent). Python parity: `rust_engine_mcp.city_seed_chain`.

use sha2::{Digest, Sha256};

use super::ids::{BlockId, TownId};

pub const DEFAULT_WORLD_SEED: u64 = 99_001;
pub const DEFAULT_TOWN_ID: &str = "portland";
pub const CITY_G1_C4_WIT_BLOCK: &str = "industrial_west_b01";
pub const CITY_G1_C4_WIT_LOT_IDX: u32 = 7;
pub const CITY_G1_C4_WIT_ARCHETYPE: &str = "IndustrialWarehouse";
pub const CITY_G1_C4_WIT_DISTRICT: &str = "industrial_west";
pub const CITY_G1_C4_LIVE_JSON: &str = "debug_runs/city_g1_c4_001_live.json";

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

#[must_use]
pub fn city_g1_c4_witness_context() -> CitySeedContext {
    CitySeedContext {
        world_seed: DEFAULT_WORLD_SEED,
        town_id: TownId(DEFAULT_TOWN_ID.into()),
        block_id: BlockId(CITY_G1_C4_WIT_BLOCK.into()),
        lot_idx: CITY_G1_C4_WIT_LOT_IDX,
        archetype_id: CITY_G1_C4_WIT_ARCHETYPE.into(),
    }
}

#[must_use]
pub fn build_city_g1_c4_001_witness_body() -> serde_json::Value {
    use crate::construction::procedural::{
        assembly_snapshot_stable_hash, build_assembly_snapshot_from_grammar,
        city_g0_wit_001_determinism_witness_green, load_procedural_module_registry,
        load_style_pack_registry,
    };

    let ctx = city_g1_c4_witness_context();
    let grammar_seed = ctx.building_grammar_seed();
    let modules = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    let registry_ok = modules.load_errors.is_empty() && packs.load_errors.is_empty();

    let mut run_hashes = Vec::new();
    let mut contract_ok = false;
    for _ in 0..3 {
        match build_assembly_snapshot_from_grammar(
            &ctx.archetype_id,
            CITY_G1_C4_WIT_DISTRICT,
            grammar_seed,
            &modules,
            &packs,
        ) {
            Ok(snapshot) => {
                contract_ok =
                    crate::construction::procedural::snapshot_passes_auto_001_contract(&snapshot);
                run_hashes.push(assembly_snapshot_stable_hash(&snapshot));
            }
            Err(err) => {
                return serde_json::json!({
                    "gate": "CITY-G1-C4-001",
                    "green": false,
                    "error": err,
                });
            }
        }
    }

    let three_run_stable = run_hashes.len() == 3 && run_hashes.windows(2).all(|w| w[0] == w[1]);
    let chain_layers = {
        let ts = town_seed(ctx.world_seed, &ctx.town_id);
        let bs = block_seed(ts, &ctx.block_id);
        let ls = lot_seed(bs, ctx.lot_idx);
        serde_json::json!({
            "town_seed": format!("{ts:#018x}"),
            "block_seed": format!("{bs:#018x}"),
            "lot_seed": format!("{ls:#018x}"),
            "building_grammar_seed": format!("{grammar_seed:#018x}"),
        })
    };
    let g0_wit = city_g0_wit_001_determinism_witness_green();
    let green = registry_ok && three_run_stable && contract_ok && g0_wit;

    serde_json::json!({
        "gate": "CITY-G1-C4-001",
        "issue": "CITY-C4",
        "green": green,
        "registry_ok": registry_ok,
        "three_run_stable": three_run_stable,
        "auto_001_contract": contract_ok,
        "city_g0_wit_still_green": g0_wit,
        "chain_layers": chain_layers,
        "stable_hash": run_hashes.first(),
        "run_hashes": run_hashes,
        "witness_context": {
            "world_seed": ctx.world_seed,
            "town_id": ctx.town_id.0,
            "block_id": ctx.block_id.0,
            "lot_idx": ctx.lot_idx,
            "archetype_id": ctx.archetype_id,
            "district_style": CITY_G1_C4_WIT_DISTRICT,
        },
    })
}

#[must_use]
pub fn city_g1_c4_001_seed_chain_witness_green() -> bool {
    build_city_g1_c4_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_city_g1_c4_001_seed_chain_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g1_c4_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G1-C4-001",
        "refresh_city_g1_c4_001_seed_chain_witness",
        CITY_G1_C4_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G1_C4_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_chain_layers_are_deterministic() {
        let town = TownId("portland".into());
        let block = BlockId("industrial_west_b01".into());
        let ts = town_seed(99_001, &town);
        let bs = block_seed(ts, &block);
        let ls = lot_seed(bs, 7);
        let bg = building_grammar_seed(ls, "IndustrialWarehouse");
        assert_eq!(ts, town_seed(99_001, &town));
        assert_eq!(bg, building_grammar_seed_chain(99_001, &town, &block, 7, "IndustrialWarehouse"));
    }

    #[test]
    fn seed_chain_differs_by_lot_idx() {
        let town = TownId(DEFAULT_TOWN_ID.into());
        let block = BlockId(CITY_G1_C4_WIT_BLOCK.into());
        let a = building_grammar_seed_chain(DEFAULT_WORLD_SEED, &town, &block, 0, "IndustrialWarehouse");
        let b = building_grammar_seed_chain(DEFAULT_WORLD_SEED, &town, &block, 1, "IndustrialWarehouse");
        assert_ne!(a, b);
    }

    #[test]
    fn site_derived_seed_is_stable() {
        let s = building_grammar_seed_for_site(DEFAULT_WORLD_SEED, 42, "IndustrialWarehouse");
        assert_eq!(s, building_grammar_seed_for_site(DEFAULT_WORLD_SEED, 42, "IndustrialWarehouse"));
    }

    #[test]
    fn seed_chain_witness_golden_layers() {
        let ctx = city_g1_c4_witness_context();
        let ts = town_seed(ctx.world_seed, &ctx.town_id);
        let bs = block_seed(ts, &ctx.block_id);
        let ls = lot_seed(bs, ctx.lot_idx);
        let bg = ctx.building_grammar_seed();
        assert_eq!(ts, 0x4AC870FAAB87F9AD);
        assert_eq!(bs, 0x490697089FF6F2F5);
        assert_eq!(ls, 0x59DCFEF41AF9F0F9);
        assert_eq!(bg, 0x035111798AD871AA);
    }

    #[test]
    fn city_g1_c4_001_witness_green() {
        assert!(city_g1_c4_001_seed_chain_witness_green());
    }
}
