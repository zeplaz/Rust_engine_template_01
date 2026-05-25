//! Utilities infrastructure roles — transformers, substations, power plants (not supply-chain steps).

/// Placeable grid / generation utilities (`utility_role` in building JSON).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UtilityInfrastructureRole {
    Transformer,
    Substation,
    PowerPlant,
}

impl UtilityInfrastructureRole {
    pub fn from_json(s: &str) -> Option<Self> {
        match s {
            "transformer" => Some(Self::Transformer),
            "substation" => Some(Self::Substation),
            "power_plant" => Some(Self::PowerPlant),
            _ => None,
        }
    }

    pub fn from_catalog_id(catalog_id: &str) -> Option<Self> {
        match catalog_id {
            "grid_distribution_transformer" => Some(Self::Transformer),
            "grid_substation" => Some(Self::Substation),
            "utilities_coal_plant" => Some(Self::PowerPlant),
            _ => None,
        }
    }

    pub fn resolve(catalog_id: &str, json_role: Option<UtilityInfrastructureRole>) -> Option<Self> {
        json_role.or_else(|| Self::from_catalog_id(catalog_id))
    }
}
