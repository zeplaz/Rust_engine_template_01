//! Growth actor layers + building usage (ECON-OG-1-A).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrowthActorLayer {
    State,
    Growth,
    LegacyCatalog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingUsage {
    Government,
    Industrial,
    Commercial,
    Residential,
    Office,
    Military,
    Civic,
    Logistics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrowthReasonCode {
    TransportHigh,
    EmploymentDemand,
    HousingDeficit,
    MarketSaturated,
    UtilityMissing,
    PolicyBlocked,
}

#[must_use]
pub fn growth_actor_may_enqueue(layer: GrowthActorLayer) -> bool {
    matches!(layer, GrowthActorLayer::Growth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_usage_serde_roundtrip() {
        assert!(building_usage_serde_witness_green());
    }

    #[test]
    fn growth_actor_layer_serde_roundtrip() {
        assert!(growth_actor_layer_serde_witness_green());
    }
}

#[must_use]
pub fn building_usage_serde_witness_green() -> bool {
    let usage = BuildingUsage::Commercial;
    let Ok(json) = serde_json::to_string(&usage) else {
        return false;
    };
    serde_json::from_str::<BuildingUsage>(&json).ok() == Some(usage)
}

#[must_use]
pub fn growth_actor_layer_serde_witness_green() -> bool {
    let layer = GrowthActorLayer::Growth;
    let Ok(json) = serde_json::to_string(&layer) else {
        return false;
    };
    serde_json::from_str::<GrowthActorLayer>(&json).ok() == Some(layer)
}
