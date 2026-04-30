//! Static **archetype** data per `PowerPlantType` — one table to extend new types without giant `match` arms
//! in every system. When `PowerPlant.definition_id` is set, **`PlantDefinition`** from
//! `assets/config/power/plant_definitions.json` overrides factors; this module is the **fallback**.
//!
//! **Design direction:** `plant_definition.rs` + `plant_registry.rs`; see
//! `prompts/designer_questions/production_economy/spec/06_power_plants_data_scripting_v1.md`.

use crate::entities::production::power::power_states::PowerPlantType;

#[derive(Clone, Copy, Debug)]
pub struct PlantArchetype {
    /// Part-load / technology modifier applied in `update_power_output_system` (was inline `match`).
    pub efficiency_factor: f32,
    /// Steam-water circuit: steam leaks, condenser limits, etc. (only these plants consult steam failure systems).
    pub is_steam_cycle: bool,
    /// Pressure vessel / containment: scram, containment breach, etc. (independent from steam-only plants).
    pub is_nuclear_containment: bool,
    /// Wind / solar style output variability (environmental derate hooks).
    pub is_variable_renewable: bool,
}

impl PlantArchetype {
    pub fn for_type(t: PowerPlantType) -> Self {
        use PowerPlantType::*;
        match t {
            Nuclear => Self {
                efficiency_factor: 0.95,
                is_steam_cycle: true,
                is_nuclear_containment: true,
                is_variable_renewable: false,
            },
            Coal | Oil | Gas | Biomass => Self {
                efficiency_factor: 0.85,
                is_steam_cycle: true,
                is_nuclear_containment: false,
                is_variable_renewable: false,
            },
            Geothermal => Self {
                efficiency_factor: 0.88,
                // Often flash / binary steam — treat as steam-capable; refine per-plant later.
                is_steam_cycle: true,
                is_nuclear_containment: false,
                is_variable_renewable: false,
            },
            Hydro => Self {
                efficiency_factor: 0.90,
                is_steam_cycle: false,
                is_nuclear_containment: false,
                is_variable_renewable: false,
            },
            Solar | Wind => Self {
                efficiency_factor: 0.75,
                is_steam_cycle: false,
                is_nuclear_containment: false,
                is_variable_renewable: true,
            },
        }
    }
}
