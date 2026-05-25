//! Supply-chain step roles — parsed from building JSON, used by economy activation.

/// Discrete step in a multi-building industrial chain (see `assets/configs/industrial_supply_chains.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndustrialSupplyChainRole {
    AggregateMine,
    CementKiln,
    ConcreteMixer,
    IntegratedPlant,
    BauxiteMine,
    AluminaRefinery,
    AluminumSmelter,
    AluminumFabrication,
}

impl IndustrialSupplyChainRole {
    pub fn from_catalog_id(catalog_id: &str) -> Option<Self> {
        match catalog_id {
            "concrete_aggregate_mine" => Some(Self::AggregateMine),
            "concrete_cement_kiln" | "concrete_cement_kiln_geopolymer" => Some(Self::CementKiln),
            "concrete_mixer_plant" | "concrete_mixer_geopolymer" => Some(Self::ConcreteMixer),
            "concrete_basic_production_plant" | "concrete_production_plant_copy" => {
                Some(Self::IntegratedPlant)
            }
            "aluminum_bauxite_mine" => Some(Self::BauxiteMine),
            "aluminum_alumina_refinery" => Some(Self::AluminaRefinery),
            "aluminum_smelter1" => Some(Self::AluminumSmelter),
            "aluminum_fabrication_plant" => Some(Self::AluminumFabrication),
            _ => None,
        }
    }

    pub fn from_json_role(s: &str) -> Option<Self> {
        match s {
            "aggregate_mine" => Some(Self::AggregateMine),
            "cement_kiln" => Some(Self::CementKiln),
            "concrete_mixer" => Some(Self::ConcreteMixer),
            "integrated_plant" => Some(Self::IntegratedPlant),
            "bauxite_mine" => Some(Self::BauxiteMine),
            "alumina_refinery" => Some(Self::AluminaRefinery),
            "aluminum_smelter" => Some(Self::AluminumSmelter),
            "aluminum_fabrication" => Some(Self::AluminumFabrication),
            _ => None,
        }
    }

    pub fn resolve(
        catalog_id: &str,
        json_role: Option<IndustrialSupplyChainRole>,
    ) -> Option<Self> {
        json_role.or_else(|| Self::from_catalog_id(catalog_id))
    }
}
