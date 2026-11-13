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
    /// A threat cell or shot target is within 16 tiles of the town.
    ThreatInRange,
}

#[must_use]
pub fn growth_actor_may_enqueue(layer: GrowthActorLayer) -> bool {
    matches!(layer, GrowthActorLayer::Growth)
}

#[cfg(test)]
mod tests {
    

}

