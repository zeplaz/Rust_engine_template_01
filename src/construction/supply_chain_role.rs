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
    /// Construction-material producers and the import terminal (`construction_materials` chain).
    MaterialsProducer,
    /// Petroleum crude extraction (`petroleum` chain).
    OilWell,
    /// Petroleum refining (`petroleum` chain).
    FuelRefinery,
    /// UAS battery cells (`uas_assembly` chain).
    UasBattery,
    /// Fiber optic draw for FPV (`uas_assembly` chain).
    UasFiber,
    /// UAS final assembly (`uas_assembly` chain).
    UasFinalAssembly,
    /// Missile seeker kits (`missile_assembly` chain).
    MissileSeeker,
    /// Missile warheads (`missile_assembly` chain).
    MissileWarhead,
    /// Missile body / final assembly (`missile_assembly` chain).
    MissileBodyPlant,
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
            "industrial_oil_well" => Some(Self::OilWell),
            "industrial_fuel_refinery" => Some(Self::FuelRefinery),
            "uas_battery_plant" | "battery_cell_line" => Some(Self::UasBattery),
            "uas_fiber_plant" | "fiber_draw" => Some(Self::UasFiber),
            "uas_final_assembly" | "uas_assembly_plant" => Some(Self::UasFinalAssembly),
            "missile_seeker_plant" => Some(Self::MissileSeeker),
            "missile_warhead_plant" => Some(Self::MissileWarhead),
            "missile_body_plant" | "missile_assembly" => Some(Self::MissileBodyPlant),
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
            "materials_producer" => Some(Self::MaterialsProducer),
            "oil_well" => Some(Self::OilWell),
            "fuel_refinery" => Some(Self::FuelRefinery),
            "uas_battery" => Some(Self::UasBattery),
            "uas_fiber" => Some(Self::UasFiber),
            "uas_final_assembly" => Some(Self::UasFinalAssembly),
            "missile_seeker" => Some(Self::MissileSeeker),
            "missile_warhead" => Some(Self::MissileWarhead),
            "missile_body" => Some(Self::MissileBodyPlant),
            _ => None,
        }
    }

    pub fn resolve(catalog_id: &str, json_role: Option<IndustrialSupplyChainRole>) -> Option<Self> {
        json_role.or_else(|| Self::from_catalog_id(catalog_id))
    }
}
