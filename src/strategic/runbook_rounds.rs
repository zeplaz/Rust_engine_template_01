//! Incremental **delivery gates** for strategic-field runbooks — small types + unit tests only.
//! Each submodule maps to one guide under `prompts/guides/` (Execution rounds §).

// --- Runbook: infrastructure_corridor_runbook_v1 -----------------------------------------------

pub mod corridor {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum CorridorType {
        Logistics,
        Rail,
        Highway,
        PowerTransmission,
        Pipeline,
        MilitarySupply,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct CorridorCost {
        pub construction: f32,
        pub maintenance: f32,
        pub vulnerability: f32,
        pub throughput: f32,
    }

    /// Lower is better — stub composite for ranking corridor sketches.
    pub fn corridor_total_cost(c: &CorridorCost) -> f32 {
        c.construction + c.maintenance + c.vulnerability * 2.0 - c.throughput * 0.5
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_rank_prefers_high_throughput_lower_vuln() {
            let a = CorridorCost {
                construction: 1.0,
                maintenance: 1.0,
                vulnerability: 0.5,
                throughput: 2.0,
            };
            let b = CorridorCost {
                construction: 1.0,
                maintenance: 1.0,
                vulnerability: 1.0,
                throughput: 1.0,
            };
            assert!(corridor_total_cost(&a) < corridor_total_cost(&b));
        }
    }
}

// --- Runbook: logistics_ai_runbook_v1 ----------------------------------------------------------

pub mod logistics_ai_policy {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum LogisticsPriority {
        Military,
        Civilian,
        Industrial,
        Emergency,
    }

    /// Stub: frontline crisis elevates military resupply weight.
    pub fn effective_priority_weight(p: LogisticsPriority, frontline_crisis: bool) -> f32 {
        match (p, frontline_crisis) {
            (LogisticsPriority::Military, true) => 2.0,
            (LogisticsPriority::Military, false) => 1.5,
            (LogisticsPriority::Emergency, _) => 1.8,
            (LogisticsPriority::Industrial, _) => 1.0,
            (LogisticsPriority::Civilian, _) => 1.2,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_military_weight_rises_under_crisis() {
            let w = effective_priority_weight(LogisticsPriority::Military, true);
            assert!(w > effective_priority_weight(LogisticsPriority::Military, false));
        }
    }
}

// --- Runbook: settlement_growth_runbook_v1 ----------------------------------------------------

pub mod settlement {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SettlementTier {
        Camp,
        Village,
        Town,
        City,
        Metropolis,
    }

    /// Stub population → tier curve (arbitrary thresholds for tests / placeholders).
    pub fn tier_from_population(pop: u32) -> SettlementTier {
        match pop {
            0..=99 => SettlementTier::Camp,
            100..=999 => SettlementTier::Village,
            1000..=9999 => SettlementTier::Town,
            10000..=99_999 => SettlementTier::City,
            _ => SettlementTier::Metropolis,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_lifecycle_thresholds_monotonic() {
            assert!(matches!(tier_from_population(50), SettlementTier::Camp));
            assert!(matches!(tier_from_population(500), SettlementTier::Village));
            assert!(matches!(tier_from_population(5000), SettlementTier::Town));
        }
    }
}

// --- Runbook: ai_city_planning_runbook_v1 -------------------------------------------------------

pub mod city_planning {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum SettlementArchetype {
        IndustrialHub,
        LogisticsJunction,
        MiningTown,
        AgriculturalRegion,
        MilitaryFortress,
        CoastalPort,
        ResearchCity,
        EnergyCluster,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_archetypes_are_stable_enum() {
            let _ = SettlementArchetype::ResearchCity;
        }
    }
}

// --- Runbook: ai_operational_warfare_runbook_v1 ------------------------------------------------

pub mod operational_warfare {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DroneDoctrine {
        ReconHeavy,
        SaturationStrike,
        EwSuppression,
        LogisticsInterdiction,
    }

    /// Stub: higher when ammo and rail throughput suffice (placeholders 0..1).
    pub fn offensive_commit_score(ammo_reserve: f32, rail_throughput: f32) -> f32 {
        (ammo_reserve * 0.5 + rail_throughput * 0.5).clamp(0.0, 1.0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_commit_needs_both_supply_signals() {
            assert!(offensive_commit_score(1.0, 0.0) < offensive_commit_score(1.0, 1.0));
        }
    }
}
