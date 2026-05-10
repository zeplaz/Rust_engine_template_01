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

        /// **R2** — corridor *kind* scales nominal capacity (stands in for graph edge class until construction graph ships).
        #[test]
        fn round2_kind_scales_sketch_capacity() {
            assert!(corridor_capacity_weight(CorridorType::Rail) > corridor_capacity_weight(CorridorType::Logistics));
        }

        /// **R3** — expansion stub picks lower composite cost.
        #[test]
        fn round3_ai_prefers_lower_corridor_total_cost() {
            let cheap = CorridorCost {
                construction: 1.0,
                maintenance: 0.5,
                vulnerability: 0.2,
                throughput: 3.0,
            };
            let costly = CorridorCost {
                construction: 4.0,
                maintenance: 2.0,
                vulnerability: 0.9,
                throughput: 1.0,
            };
            assert_eq!(
                pick_cheaper_corridor_index(&cheap, &costly),
                0
            );
        }
    }

    /// Weight for nominal throughput by corridor class (R2 sketch → edge class).
    pub fn corridor_capacity_weight(ctype: CorridorType) -> f32 {
        match ctype {
            CorridorType::Rail => 1.35,
            CorridorType::Highway => 1.15,
            CorridorType::PowerTransmission => 1.0,
            CorridorType::Pipeline => 0.95,
            CorridorType::MilitarySupply => 1.1,
            CorridorType::Logistics => 1.0,
        }
    }

    /// **R3** — return index of cheaper corridor (`0` = `a`, `1` = `b`).
    pub fn pick_cheaper_corridor_index(a: &CorridorCost, b: &CorridorCost) -> usize {
        if corridor_total_cost(a) <= corridor_total_cost(b) {
            0
        } else {
            1
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

    /// **R2** — recommend reroute when congestion + structural damage pressure is high (0..1 each).
    pub fn reroute_recommended(route_congestion: f32, edge_damage: f32) -> bool {
        (route_congestion.clamp(0.0, 1.0) + edge_damage.clamp(0.0, 1.0)) > 1.0
    }

    /// **R3** — stub demand forecast from base load, offensive pressure, weather delay factor.
    pub fn demand_forecast(
        base_load: f32,
        offensive_pressure: f32,
        weather_delay: f32,
    ) -> f32 {
        (base_load * (1.0 + offensive_pressure * 0.4)) * (1.0 + weather_delay.clamp(0.0, 0.8))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_military_weight_rises_under_crisis() {
            let w = effective_priority_weight(LogisticsPriority::Military, true);
            assert!(w > effective_priority_weight(LogisticsPriority::Military, false));
        }

        #[test]
        fn round2_reroute_when_congestion_and_damage_stack() {
            assert!(!reroute_recommended(0.4, 0.5));
            assert!(reroute_recommended(0.6, 0.6));
        }

        #[test]
        fn round3_forecast_rises_with_offense_and_weather() {
            let calm = demand_forecast(10.0, 0.0, 0.0);
            let hot = demand_forecast(10.0, 1.0, 0.5);
            assert!(hot > calm);
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

    /// **R2** — migration pull from opportunity / quality signals (0..1 each).
    pub fn migration_pull(jobs: f32, safety: f32, housing: f32) -> f32 {
        (jobs * 0.45 + safety * 0.35 + housing * 0.2).clamp(0.0, 1.0)
    }

    /// **R3** — emigration / decline pressure from ecology hazards (overlay inputs, 0..1).
    pub fn ecology_hazard_pressure(fire_risk: f32, flood_risk: f32) -> f32 {
        (fire_risk.clamp(0.0, 1.0) * 0.5 + flood_risk.clamp(0.0, 1.0) * 0.5).clamp(0.0, 1.0)
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

        #[test]
        fn round2_migration_responds_to_jobs_and_safety() {
            let low = migration_pull(0.2, 0.2, 0.5);
            let high = migration_pull(0.9, 0.8, 0.5);
            assert!(high > low);
        }

        #[test]
        fn round3_ecology_pressure_combines_fire_and_flood() {
            assert!(ecology_hazard_pressure(0.0, 0.0) < ecology_hazard_pressure(1.0, 1.0));
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

    /// **R2** — site score from overlay-style scalars (logistics strength ↑, flood & threat ↓).
    pub fn site_score(logistics_strength: f32, flood_risk: f32, mean_threat: f32) -> f32 {
        (logistics_strength.clamp(0.0, 1.0) * 1.2
            - flood_risk.clamp(0.0, 1.0) * 0.6
            - mean_threat.clamp(0.0, 1.0) * 0.7)
            .clamp(-1.0, 2.0)
    }

    /// **R3** — redundant utility expectation weight (fortified / industrial hubs need more).
    pub fn utility_redundancy_weight(archetype: SettlementArchetype) -> f32 {
        match archetype {
            SettlementArchetype::MilitaryFortress | SettlementArchetype::IndustrialHub => 1.5,
            SettlementArchetype::ResearchCity | SettlementArchetype::EnergyCluster => 1.35,
            SettlementArchetype::LogisticsJunction => 1.25,
            _ => 1.0,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_archetypes_are_stable_enum() {
            let _ = SettlementArchetype::ResearchCity;
        }

        #[test]
        fn round2_site_score_prefers_logistics_over_flood() {
            let good = site_score(0.9, 0.1, 0.1);
            let bad = site_score(0.2, 0.9, 0.5);
            assert!(good > bad);
        }

        #[test]
        fn round3_fortress_demands_higher_redundancy_than_village_ag() {
            assert!(
                utility_redundancy_weight(SettlementArchetype::MilitaryFortress)
                    > utility_redundancy_weight(SettlementArchetype::AgriculturalRegion)
            );
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

    /// **R2** — strike commitment modifier by drone doctrine (recon-heavy reduces raw strike weight).
    pub fn doctrine_strike_weight(d: DroneDoctrine) -> f32 {
        match d {
            DroneDoctrine::ReconHeavy => 0.45,
            DroneDoctrine::EwSuppression => 0.65,
            DroneDoctrine::LogisticsInterdiction => 0.85,
            DroneDoctrine::SaturationStrike => 1.0,
        }
    }

    /// **R3** — default path couples strikes to infrastructure (see `doctrine_simulation_alignment_runbook_v1` §9).
    pub const INFRASTRUCTURE_COUPLED_STRIKES_DEFAULT: bool = true;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round1_commit_needs_both_supply_signals() {
            assert!(offensive_commit_score(1.0, 0.0) < offensive_commit_score(1.0, 1.0));
        }

        #[test]
        fn round2_recon_doctrine_lowers_strike_weight_vs_saturation() {
            assert!(
                doctrine_strike_weight(DroneDoctrine::ReconHeavy)
                    < doctrine_strike_weight(DroneDoctrine::SaturationStrike)
            );
        }

        #[test]
        fn round3_infrastructure_coupling_policy_enabled_by_default() {
            assert!(INFRASTRUCTURE_COUPLED_STRIKES_DEFAULT);
        }
    }
}
